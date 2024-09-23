use crate::app::minner::Minner;
use crate::app::p2p::P2p;
use crate::blockchain::chain::Chain;
use crate::wallet::transaction_pool::Pool;
use crate::wallet::wallets::Wallet;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

lazy_static! {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub static ref CHAIN: Mutex<Chain> = Mutex::new(Chain::new());
    pub static ref WALLET: Mutex<Wallet> = Mutex::new(Wallet::new());
    pub static ref POOL: Mutex<Pool> = Mutex::new(Pool::new());
    pub static ref SERVER: P2p = P2p::new(CHAIN.lock().unwrap().clone(), POOL.lock().unwrap().clone());
    pub static ref MINER: Arc<Mutex<Minner>> = Arc::new(Mutex::new(Minner::new(
        CHAIN.lock().unwrap().clone(),
        POOL.lock().unwrap().clone(),
        WALLET.lock().unwrap().clone(),
        SERVER.clone()
    )));
}

// block struct to json
#[derive(Serialize, Deserialize)]
pub struct BlockJson {
    pub timestamp: u64,
    pub last_hash: String,
    pub hash: String,
    pub data: String,
}

// post
#[derive(Serialize, Deserialize)]
pub struct PostBlockJson {
    pub data: String,
}

#[derive(Serialize, Deserialize)]
pub struct PostPoolJson {
    pub receipient: String,
    pub amount: u64,
}

// p2p server
pub async fn start_p2p() {
    let server = SERVER.clone();
    tokio::spawn(async move {
        if let Err(e) = server.listen().await {
            eprintln!("Error listening: {}", e);
        }
    });
}
