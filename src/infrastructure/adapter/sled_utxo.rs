use crate::application::port::utxo::UTXORepository;
use crate::domain::transaction::TXOutput;
use bincode::{serialize, deserialize};
use sled;
use std::collections::HashMap;
use std::error::Error;
use std::str;

pub struct UTXO {
    pub db: sled::Db,
}

impl UTXO {
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let db = sled::open(path)?;
        Ok(UTXO { db })
    }
}

impl UTXORepository for UTXO {
    fn get_all_utxos(&self) -> Result<HashMap<String, TXOutput>, Box<dyn Error>> {
        let mut utxos = HashMap::new();
        for item in self.db.iter() {
            let (key, value) = item?;
            if key == b"LAST" {
                continue;
            }
            let txid = String::from_utf8(key.to_vec())?;
            let outs: TXOutput = deserialize(&value.to_vec())?;
            utxos.insert(txid, outs);
        }
        Ok(utxos)
    }

    fn store_utxos(&self, utxos: HashMap<String, TXOutput>) -> Result<(), Box<dyn Error>> {
        self.db.clear()?;
        for (txid, outs) in utxos {
            self.db.insert(txid.as_bytes(), serialize(&outs)?)?;
        }
        self.db.flush()?;
        Ok(())
    }
}