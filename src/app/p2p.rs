use crate::blockchain::chain::Chain;
use crate::wallet::{transaction::Transaction, transaction_pool::Pool};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{accept_async, connect_async, tungstenite::Message, WebSocketStream};
use url::Url;

use tokio_tungstenite::MaybeTlsStream;

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
enum MessageType {
    Chain,
    Transaction,
    ClearTransaction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct P2pMessage {
    type_: MessageType,
    chain: Option<Chain>,
    transaction: Option<Transaction>,
    clear_transaction: Option<bool>,
}

#[derive(Clone)]
pub struct P2p {
    blockchain: Arc<Mutex<Chain>>,
    transaction_pool: Arc<Mutex<Pool>>,
    sockets: Arc<Mutex<Vec<Arc<Mutex<WsStream>>>>>,
}

impl P2p {
    pub fn new(blockchain: Arc<Mutex<Chain>>, transaction_pool: Arc<Mutex<Pool>>) -> Self {
        P2p {
            blockchain,
            transaction_pool,
            sockets: Arc::new(Mutex::new(vec![])),
        }
    }

    pub async fn listen(&self) -> Result<(), Box<dyn std::error::Error>> {
        let p2p_port = std::env::var("P2P_PORT").unwrap_or_else(|_| "5001".to_string());
        let addr = format!("127.0.0.1:{}", p2p_port);
        let listener = TcpListener::bind(&addr).await?;
        println!("Listening on: {}", addr);

        self.connect_peers().await;

        while let Ok((stream, _)) = listener.accept().await {
            let ws_stream = accept_async(MaybeTlsStream::Plain(stream)).await?; // ws_streamの型はWebSocketStream<MaybeTlsStream<TcpStream>>
            self.connect_socket(ws_stream).await; // connect_socketメソッドに渡す
        }
        Ok(())
    }

    pub async fn connect_socket(&self, socket: WsStream) {
        let socket = Arc::new(Mutex::new(socket));
        self.sockets.lock().await.push(socket.clone());

        println!(
            "Socket connected. Total sockets: {}",
            self.sockets.lock().await.len()
        );
        self.message_handler(socket.clone()).await;
        self.send_chain(socket.clone()).await;
    }

    pub async fn connect_peers(&self) {
        if let Ok(peers) = std::env::var("PEERS") {
            for peer in peers.split(',') {
                if let Ok(url) = Url::parse(peer) {
                    loop {
                        println!("Attempting to connect to peer: {}", peer);
                        match connect_async(url.as_str()).await {
                            Ok((ws_stream, _)) => {
                                self.connect_socket(ws_stream).await;
                                println!("Connected to peer: {}", peer);
                                break;
                            }
                            Err(e) => {
                                println!("Failed to connect to peer {}: {}. Retrying...", peer, e);
                                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                            }
                        }
                    }
                }
            }
        }
    }

    async fn send_chain(&self, socket: Arc<Mutex<WsStream>>) {
        let blockchain = self.blockchain.lock().await.clone();
        let message = P2pMessage {
            type_: MessageType::Chain,
            chain: Some(blockchain),
            transaction: None,
            clear_transaction: None,
        };
        let json_message = match serde_json::to_string(&message) {
            Ok(msg) => msg,
            Err(e) => {
                println!("Failed to serialize chain message: {}", e);
                return;
            }
        };

        let mut socket = socket.lock().await;
        if let Err(e) = socket.send(Message::Text(json_message)).await {
            println!("Failed to send chain message: {}", e);
        }
    }

    pub async fn sync_chain(&self) {
        let sockets = self.sockets.lock().await;
        for socket in sockets.iter() {
            self.send_chain(socket.clone()).await;
        }
    }

    async fn send_transaction(&self, socket: Arc<Mutex<WsStream>>, transaction: Transaction) {
        let message = P2pMessage {
            type_: MessageType::Transaction,
            chain: None,
            transaction: Some(transaction),
            clear_transaction: None,
        };
        let json_message = match serde_json::to_string(&message) {
            Ok(msg) => msg,
            Err(e) => {
                println!("Failed to serialize transaction: {}", e);
                return;
            }
        };

        let mut socket = socket.lock().await;
        if let Err(e) = socket.send(Message::Text(json_message)).await {
            println!("Failed to send transaction: {}", e);
        }
    }

    pub async fn broadcast_transaction(&self, transaction: Transaction) {
        let sockets = self.sockets.lock().await;

        for socket in sockets.iter() {
            println!("Broadcasting transaction with ID: {}", transaction.id);
            self.send_transaction(socket.clone(), transaction.clone())
                .await;
        }
    }

    async fn message_handler(&self, socket: Arc<Mutex<WsStream>>) {
        let mut socket = socket.lock().await;
        while let Some(msg) = socket.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    let data: P2pMessage = serde_json::from_str(&text).unwrap();
                    match data.type_ {
                        MessageType::Chain => {
                            let mut blockchain = self.blockchain.lock().await;
                            // this.blockchain.replaceChain(data.chain);
                            if let Some(chain) = &data.chain {
                                blockchain.replace_chain(chain);
                            }
                        }
                        MessageType::Transaction => {
                            println!(
                                "Received transaction with ID: {}",
                                data.transaction.as_ref().unwrap().id
                            );
                            // トランザクションをプールに追加
                            let mut transaction_pool = self.transaction_pool.lock().await;
                            transaction_pool.update_or_add_transaction(data.transaction.unwrap());
                        }
                        MessageType::ClearTransaction => {
                            let mut transaction_pool = self.transaction_pool.lock().await;
                            transaction_pool.clear();
                        }
                    }
                }
                _ => (),
            }
        }
    }

    pub async fn broadcast_clear_transactions(&self) {
        let sockets = self.sockets.lock().await;
        for socket in sockets.iter() {
            let message = P2pMessage {
                type_: MessageType::ClearTransaction,
                chain: None,
                transaction: None,
                clear_transaction: Some(true),
            };
            let json_message = serde_json::to_string(&message).unwrap();
            let mut ws_stream = socket.lock().await;
            ws_stream.send(Message::Text(json_message)).await.unwrap();
        }
    }
}
