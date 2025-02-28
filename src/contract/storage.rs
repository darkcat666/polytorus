use crate::Result;
use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
use sled;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContractAccount {
    pub address: String,
    pub code: Vec<u8>,
    pub storage: HashMap<String, Vec<u8>>,
    pub balance: i32,
}

impl ContractAccount {
    pub fn new(address: String, code: Vec<u8>) -> Self {
        ContractAccount {
            address,
            code,
            storage: HashMap::new(),
            balance: 0,
        }
    }

    pub fn get_strage(&self, key: &str) -> Option<Vec<u8>> {
        self.storage.get(key).cloned()
    }

    pub fn set_storage(&mut self, key: String, value: Vec<u8>) {
        self.storage.insert(key, value);
    }
}
