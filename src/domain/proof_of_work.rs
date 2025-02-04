use crate::domain::block_header::BlockHeader;
use bincode::serialize;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use std::error::Error;

pub struct ProofOfWork<'a> {
    pub header: &'a mut BlockHeader,
    pub target_prefix: usize,
}

impl<'a> ProofOfWork<'a> {
    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        while !self.validate()? {
            self.header.nonce += 1;
        }

        Ok(())
    }

    fn prepare_data(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let data = serialize(self.header)?;
        Ok(data)
    }

    fn validate(&self) -> Result<bool, Box<dyn Error>> {
        let data = self.prepare_data()?;
        let mut hasher = Sha256::new();
        hasher.input(&data);
        let hash_str = hasher.result_str();
        Ok(hash_str.chars().take(self.target_prefix).all(|c| c == '0'))
    }
}