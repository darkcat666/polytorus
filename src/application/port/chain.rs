use crate::domain::block::Block;
use std::error::Error;

pub trait ChainRepository {
    fn get_block(&self, hash: &str) -> Result<Block, Box<dyn Error>>;
    fn iter(&self) -> Box<dyn Iterator<Item = Block> + '_>;
    fn add_block(&mut self, block: &Block) -> Result<(), Box<dyn Error>>;
    fn get_tip(&self) -> Result<String, Box<dyn Error>>;
}