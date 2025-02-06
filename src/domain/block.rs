use crate::domain::block_header::BlockHeader;
use crate::domain::transaction::Transaction;
use crate::domain::merkle_tree::MergeU8;
use crate::domain::proof_of_work::ProofOfWork;
use merkle_cbt::merkle_tree::CBMT;
use bincode::serialize;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub hash: String,
}

impl Block {
    pub fn new(
        transaction: Vec<Transaction>,
        prev_block_hash: String,
        height: i32,
    ) -> Result<Self, Box<dyn Error>> {
        let timestamp: u128 = {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis()
        };

        let merkle_root = Self::calculate_merkle_root(&transaction)?;
        let header = BlockHeader {
            timestamp,
            prev_block_hash,
            merkle_root,
            nonce: 0,
            height,
        };

        let mut block = Block {
            header,
            transactions: transaction,
            hash: String::new(),
        };
        block.run_proof_of_work()?;
        Ok(block)
    }

    fn calculate_merkle_root(transactions: &[Transaction]) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut tx_hashes = Vec::new();
        for tx in transactions {
            let hash_str = tx.hash()?; // Transaction::hash() が文字列を返す前提
            tx_hashes.push(hash_str.as_bytes().to_vec());
        }
        let tree = CBMT::<Vec<u8>, MergeU8>::build_merkle_tree(tx_hashes);
        Ok(tree.root())
    }

    fn run_proof_of_work(&mut self) -> Result<(), Box<dyn Error>> {
        let target_prefix = 4;
        {
            let mut pow = ProofOfWork {
                header: &mut self.header,
                target_prefix,
            };
            pow.run()?;
        }
        let data = self.prepare_hash_data()?;
        let mut hasher = Sha256::new();
        hasher.input(&data);
        self.hash = hasher.result_str();
        Ok(())
    }

    fn prepare_hash_data(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let data = serialize(&self.header)?;
        Ok(data)
    }

    pub fn get_hash(&self) -> &str {
        &self.hash
    }
}
