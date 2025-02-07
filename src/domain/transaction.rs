use std::result::Result::Ok;
use serde::{Deserialize, Serialize};
use bincode::serialize;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use std::collections::HashMap;
use bitcoincash_addr::Address;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXInput {
    pub id: String,
    pub out: i32,
    pub signature: Vec<u8>,
    pub pub_key: Vec<u8>,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXOutput {
    pub value: i32,
    pub pub_key_hash: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXOutputs {
    pub outputs: Vec<TXOutput>,
}

impl TXOutput {
    pub fn lock(&mut self, address: &str) -> Result<(), Box<dyn std::error::Error>> {
        let addr = Address::decode(address).map_err(|e| format!("{:?}", e))?;

        self.pub_key_hash = addr.into_body();
        Ok(())
    }

    pub fn new(value: i32, address: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut txo = TXOutput {
            value,
            pub_key_hash: vec![],
        };
        txo.lock(address)?;
        Ok(txo)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub id: String,
    pub input: Vec<TXInput>,
    pub output: Vec<TXOutput>,
}

impl Transaction {
    pub fn is_coinbase(&self) -> bool {
        self.input.len() == 1 && self.input[0].id.is_empty() && self.input[0].out == -1
    }

    pub fn hash(&self) -> Result<String, Box<dyn std::error::Error>> {
        let encoded = serialize(&self)?;
        let mut hasher = Sha256::new();
        hasher.input(&encoded);
        Ok(hasher.result_str())
    }

    pub fn trim_copy(&self) -> Self {
        let vin = self.input.iter().map(|input| TXInput {
            id: input.id.clone(),
            out: input.out,
            signature: Vec::new(),
            pub_key: Vec::new(),
        }).collect();

        Transaction {
            id: self.id.clone(),
            input: vin,
            output: self.output.clone(),
        }
    }

    pub fn new_coinbase(to: String, mut data: String) -> Result<Self, Box<dyn std::error::Error>> {
        if data.is_empty() {
            data = format!("Reward to '{}'", to);
        }

        let txin = TXInput {
            id: String::new(),
            out: -1,
            signature: vec![],
            pub_key: data.as_bytes().to_vec(),
        };
        let txout = TXOutput::new(10, &to)?;
        let mut tx = Transaction {
            id: String::new(),
            input: vec![txin],
            output: vec![txout],
        };
        tx.id = tx.hash()?;
        Ok(tx)
    }

    pub fn sign<C: crate::domain::crypto::CryptographyAlgorithm>(
        &mut self, 
        private_key: &C::SecretKey, 
        prev_txs: &HashMap<String, Transaction>
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_coinbase() {
            return Ok(());
        }

        for vin in &self.input {
            if !prev_txs.contains_key(&vin.id) {
                return Err(format!("Previous transaction {} not found", vin.id).into());
            }
        }

        let mut tx: Transaction = self.trim_copy();
        let mut signatures = Vec::new();
        for (i, input) in self.input.iter().enumerate() {
            let prev_tx = prev_txs.get(&input.id).ok_or_else(|| format!("Previous transaction {} not found", input.id))?;
            tx.input[i].pub_key = prev_tx.output[input.out as usize].pub_key_hash.clone();
            tx.id = tx.hash()?;
            tx.input[i].pub_key.clear();

            let signature = C::sign(&private_key, tx.id.as_bytes());
            signatures.push((i, signature));
        }

        for (i, signature) in signatures {
            self.input[i].signature = signature;
        }

        Ok(())
    }

    pub fn verify<C: crate::domain::crypto::CryptographyAlgorithm<PublicKey = Vec<u8>>>(
        &self,
        prev_txs: &HashMap<String, Transaction>,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        if self.is_coinbase() {
            return Ok(true);
        }

        for vin in &self.input {
            if !prev_txs.contains_key(&vin.id) {
                return Err(format!("Previous transaction {} not found", vin.id).into())
            }
        }

        let mut tx = self.trim_copy();
        for (i, input) in self.input.iter().enumerate() {
            let prev_tx = prev_txs.get(&input.id)
                .ok_or_else(|| format!("Previous transaction {} not found", input.id))?;
            tx.input[i].signature.clear();
            tx.input[i].pub_key = prev_tx.output[input.out as usize].pub_key_hash.clone();
            tx.id = tx.hash()?;
            tx.input[i].pub_key.clear();

            if !C::verify(&input.pub_key, tx.id.as_bytes(), &input.signature) {
                return Ok(false);
            }
        }

        Ok(true)
    }

}