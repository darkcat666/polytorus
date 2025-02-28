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

pub struct ContractStage {
    db: sled::Db,
}

impl ContractStage {
    pub fn new() -> Result<Self> {
        let db = sled::open("data/contracts")?;
        Ok(ContractStage { db })
    }

    pub fn get_contract(&self, address: &str) -> Result<Option<ContractAccount>> {
        match self.db.get(address)? {
            Some(v) => {
                let constact: ContractAccount = deserialize(address)?;
                Ok(Some(constact))
            }
            None => Ok(None),
        }
    }

    pub fn save_contact(&self, contract: &ContractAccount) -> Result<()> {
        let data = serialize(contract)?;
        self.db.insert(&contract.address, data)?;
        self.db.flush()?;

        Ok(())
    }

    pub fn delete_contract(&self, address: &str) -> Result<()> {
        self.db.remove(address)?;
        self.db.flush()?;

        Ok(())
    }

    pub fn list_contracts(&self) -> Result<Vec<String>> {
        let mut contracts = Vec::new();
        
        for result in self.db.iter() {
            let (key, _) = result?;
            contracts.push(String::from_utf8(key.to_vec())?);
        }

        Ok(contracts)
    }
}
