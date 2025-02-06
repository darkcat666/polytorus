use crate::domain::transaction::TXOutput;
use std::collections::HashMap;
use std::error::Error;
use std::hash::Hash;

pub struct UTXOSet<'a, R: crate::application::port::utxo::UTXORepository> {
    pub repo: &'a R,
}

impl<'a, R: crate::application::port::utxo::UTXORepository> UTXOSet<'a, R> {
    pub fn new(repo: &'a R) -> Self {
        UTXOSet { repo }
    }

    pub fn find_spendable_outputs(
        &self,
        pub_key_hash: &[u8],
        amount: i32,
    ) -> Result<(i32, HashMap<String, Vec<i32>>), Box<dyn Error>> {
        let mut unspent_outputs: HashMap<String, Vec<i32>> = HashMap::new();
        let mut accumulated = 0;

        let all_utxos = self.repo.get_all_utxos()?;
        for (txid, outs) in all_utxos.iter() {
            for (idx, output) in outs.outputs iter().enumerate() {
                if output.is_locked_with_key(pub_key_hash) && accumulated < amount {
                    accumulated += output.value;
                    unspent_outputs
                        .entry(txid.clone())
                        .or_insert_with(Vec::new)
                        .push(idx as i32);
                }
            }
        }
        Ok((accumulated, unspent_outputs))
    }
}