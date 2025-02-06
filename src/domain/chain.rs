use crate::application::port::chain::ChainRepository;
use crate::domain::block::Block;
use crate::domain::transaction::Transaction;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct Chain {
    pub tip: String,
}

impl Chain {
    pub fn new(tip: String) -> Self {
        Chain { tip }
    }

    pub fn iter<'a>(&'a self, repo: &'a dyn ChainRepository) -> ChainIterator<'a> {
        ChainIterator {
            current_hash: self.tip.clone(),
            repo,
        }
    }
}

pub struct ChainIterator<'a> {
    current_hash: String,
    repo: &'a dyn ChainRepository,
}

impl<'a> Iterator for ChainIterator<'a> {
    type Item = Block;
    fn next(&mut self) -> Option<Self::Item> {
        match self.repo.get_block(&self.current_hash) {
            Ok(block) => {
                self.current_hash = block.header.prev_block_hash.clone();
                Some(block)
            },
            Err(_) => None,
        }
    }
}