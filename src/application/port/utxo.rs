use crate::domain::transaction::TXOutput;
use std::collections::HashMap;
use std::error::Error;

pub trait UTXORepository {
    fn get_all_utxos(&self) -> Result<HashMap<String, TXOutput>, Box<dyn Error>>;
    fn store_utxos(&self, utxos: HashMap<String, TXOutput>) -> Result<(), Box<dyn Error>>;
}