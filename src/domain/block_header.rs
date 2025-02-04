use serde::{Deserialize, Serialize};
use bincode::serialize;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use std::error::Error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockHeader {
    pub timestamp: u128,
    pub prev_block_hash: String,
    pub merkle_root: Vec<u8>,
    pub nonce: i32,
    pub height: i32,
}

impl BlockHeader {
    pub fn hash(&self) -> Result<String, Box<dyn Error>> {
        let data = serialize(self)?;
        let mut hasher = Sha256::new();
        hasher.input(&data);
        Ok(hasher.result_str())
    }
}