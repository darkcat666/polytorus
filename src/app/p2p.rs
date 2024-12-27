use crate::blockchain::chain::Chain;
use crate::wallet::{transaction::Transaction, transaction_pool::Pool};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::Duration as TokioDuration;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

#[derive(Debug, Clone, PartialEq, Eq)]
enum MessageType {
    Chain,
    Transaction,
    ClearTransaction,
}

impl MessageType {
    fn from_str(s: &str) -> Option<MessageType> {
        match s {
            "CHAIN" => Some(MessageType::Chain),
            "TRANSACTION" => Some(MessageType::Transaction),
            "CLEAR_TRANSACTION" => Some(MessageType::ClearTransaction),
            _ => None,
        }
    }

    fn to_str(&self) -> &'static str {
        match self {
            MessageType::Chain => "CHAIN",
            MessageType::Transaction => "TRANSACTION",
            MessageType::ClearTransaction => "CLEAR_TRANSACTION",
        }
    }
}

#[derive(Clone)]
pub struct P2p {
    pub blockchain: Arc<Mutex<Chain>>,
    pub transaction_pool: Arc<Mutex<Pool>>,
    pub sesstion: Arc<Mutex<Vec<Addr<WsSession>>>>,

    pub port: u16,
    pub peers: Vec<String>,
}

