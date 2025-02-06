use crate::domain::block::Block;
use crate::application::port::chain::ChainRepository;
use bincode::{serialize, deserialize};
use sled;
use std::error::Error;
use std::str;

pub struct SledBlockchainRepository {
    db: sled::Db,
}

impl SledBlockchainRepository {
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let db = sled::open(path)?;
        Ok(SledBlockchainRepository { db })
    }
}

impl ChainRepository for SledBlockchainRepository {
    fn get_block(&self, block_hash: &str) -> Result<Block, Box<dyn Error>> {
        let data = self.db.get(block_hash)?.ok_or(format!("Block {} not found", block_hash))?;
        let block: Block = deserialize(&data.to_vec())?;
        Ok(block)
    }

    fn iter(&self) -> Box<dyn Iterator<Item = Block> + '_> {
        let tip_hash = self.db.get("LAST").unwrap().unwrap();
        let tip_str = str::from_utf8(&tip_hash).unwrap().to_string();

        let iter = self.db.iter().filter_map(|item| {
            if let Ok((key, value)) = item {
                if key != b"LAST" {
                    if let Ok(block) = deserialize::<Block>(&value.to_vec()) {
                        return Some(block);
                    }
                }
            }
            None
        });
        Box::new(iter)
    }

    fn add_block(&mut self, block: &Block) -> Result<(), Box<dyn Error>> {
        let data = serialize(block)?;
        self.db.insert(block.get_hash().as_bytes(), data)?;
        self.db.insert("LAST", block.get_hash().as_bytes())?;
        self.db.flush()?;
        Ok(())
    }

    fn get_tip(&self) -> Result<String, Box<dyn Error>> {
        let tip = self.db.get("LAST")?.ok_or("No tip found")?;
        let tip_str = str::from_utf8(&tip)?.to_string();
        Ok(tip_str)
    }
}