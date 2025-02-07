use crate::domain::block::Block;
use crate::domain::transaction::Transaction;
use crate::domain::utxo::UTXOSet;
use crate::infrastructure::protocol::messages::*;
use bincode::{serialize, deserialize};
use std::collections::{HashMap, HashSet};
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct Server {
    pub node_addr: String,
    pub mining_addr: String,
    pub inner: Arc<Mutex<ServerInner>>,
}

pub struct ServerInner {
    pub known_nodes: HashSet<String>,
    pub utxo: UTXOSet<'static, crate::infrastructure::adapter::sled_utxo::UTXO>,
    pub blocks_in_transit: Vec<String>,
    pub mempool: HashMap<String, Transaction>,
}

const VERSION: i32 = 1;

impl Server {
    pub fn new(
        node_addr: &str,
        mining_addr: &str,
        bootstrap: Option<&str>,
        utxo: UTXOSet<'static, crate::infrastructure::adapter::sled_utxo::UTXO>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut nodes = HashSet::new();
        if let Some(addr) = bootstrap {
            nodes.insert(addr.to_string());
        }
        Ok(Self {
            node_addr: node_addr.to_string(),
            mining_addr: mining_addr.to_string(),
            inner: Arc::new(Mutex::new(ServerInner {
                known_nodes: nodes,
                utxo,
                blocks_in_transit: Vec::new(),
                mempool: HashMap::new(),
            })),
        })
    }

    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        // サーバースレッドの起動（初期ブロック要求など）
        let server_clone = self.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(1));
            // 例：ブロックチェーンの最新状態を取得し、必要ならバージョン情報送信
            if server_clone.get_best_height()? == -1 {
                server_clone.request_blocks()?;
            } else {
                let nodes = server_clone.get_known_nodes();
                if !nodes.is_empty() {
                    let first = nodes.iter().next().unwrap();
                    server_clone.send_version(first)?;
                }
            }
            Ok::<(), Box<dyn std::error::Error>>(())
        });

        let listener = TcpListener::bind(&self.node_address)?;
        println!("Server listening on {}", &self.node_address);

        for stream in listener.incoming() {
            let stream = stream?;
            let server_clone = self.clone();
            thread::spawn(move || {
                if let Err(e) = server_clone.handle_connection(stream) {
                    eprintln!("Error handling connection: {}", e);
                }
            });
        }
        Ok(())
    }

    fn handle_connection(&self, mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer)?;
        let (cmd, data) = bytes_to_cmd(&buffer)?;
        match cmd.as_str() {
            "addr" => {
                let nodes: Vec<String> = deserialize(data)?;
                self.handle_addr(nodes)?;
            },
            "block" => {
                let msg: Blockmsg = deserialize(data)?;
                self.handle_block(msg)?;
            },
            "inv" => {
                let msg: Invmsg = deserialize(data)?;
                self.handle_inv(msg)?;
            },
            "getblocks" => {
                let msg: GetBlocksmsg = deserialize(data)?;
                self.handle_get_blocks(msg)?;
            },
            "getdata" => {
                let msg: GetDatamsg = deserialize(data)?;
                self.handle_get_data(msg)?;
            },
            "tx" => {
                let msg: Txmsg = deserialize(data)?;
                self.handle_tx(msg)?;
            },
            "version" => {
                let msg: Versionmsg = deserialize(data)?;
                self.handle_version(msg)?;
            },
            _ => eprintln!("Unknown command: {}", cmd),
        }
        Ok(())
    }

    // --- 以下、メッセージハンドリング関数群 ---
    fn handle_version(&self, msg: Versionmsg) -> Result<(), Box<dyn std::error::Error>> {
        println!("Received version message: {:?}", msg);
        // ここで、自ノードと相手ノードのブロック高さを比較し、必要なら version や getblocks 送信を行う
        Ok(())
    }

    fn handle_addr(&self, nodes: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        println!("Received addr message: {:?}", nodes);
        self.inner.lock().unwrap().known_nodes.extend(nodes);
        Ok(())
    }

    fn handle_block(&self, msg: Blockmsg) -> Result<(), Box<dyn std::error::Error>> {
        println!("Received block message from {}: hash={}", msg.addr_from, msg.block.get_hash());
        // ブロック追加・UTXO更新など
        Ok(())
    }

    fn handle_inv(&self, msg: Invmsg) -> Result<(), Box<dyn std::error::Error>> {
        println!("Received inv message: {:?}", msg);
        Ok(())
    }

    fn handle_get_blocks(&self, msg: GetBlocksmsg) -> Result<(), Box<dyn std::error::Error>> {
        println!("Received getblocks message from {}", msg.addr_from);
        Ok(())
    }

    fn handle_get_data(&self, msg: GetDatamsg) -> Result<(), Box<dyn std::error::Error>> {
        println!("Received getdata message: {:?}", msg);
        Ok(())
    }

    fn handle_tx(&self, msg: Txmsg) -> Result<(), Box<dyn std::error::Error>> {
        println!("Received tx message from {}: txid={}", msg.addr_from, msg.transaction.id);
        Ok(())
    }

    // --- 補助関数群 ---
    fn get_known_nodes(&self) -> HashSet<String> {
        self.inner.lock().unwrap().known_nodes.clone()
    }

    fn get_best_height(&self) -> Result<i32, Box<dyn std::error::Error>> {
        self.inner.lock().unwrap().utxo..get_best_height()
    }

    fn request_blocks(&self) -> Result<(), Box<dyn std::error::Error>> {
        for node in self.get_known_nodes() {
            self.send_get_blocks(&node)?;
        }
        Ok(())
    }

    fn send_get_blocks(&self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let msg = GetBlockmsg {
            addr_from: self.node_addr.clone(),
        };
        let data = serialize(&(cmd_to_bytes("getblocks"), msg))?;
        self.send_data(addr, &data)
    }

    fn send_version(&self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let version_msg = Versionmsg {
            addr_from: self.node_addr.clone(),
            version: VERSION,
            best_height: self.get_best_height()?,
        };
        let data = serialize(&(cmd_to_bytes("version"), version_msg))?;
        self.send_data(addr, &data)
    }

    fn send_data(&self, addr: &str, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        if addr == self.node_addr {
            return Ok(());
        }
        let mut stream = TcpStream::connect(addr)?;
        stream.write_all(data)?;
        println!("Data sent to {}", addr);
        Ok(())
    }
}