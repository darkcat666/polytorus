use super::transaction::Transaction;
use super::transaction_pool::Pool;
use crate::blockchain::chain::Chain;
use crate::blockchain::config::INITIAL_BALANCE;
use lazy_static::lazy_static;
use rand::{thread_rng, Rng};
use secp256k1::hashes::sha256;
use secp256k1::{Message, Secp256k1};
use serde::{Deserialize, Serialize};
use std::fmt;
use tokio::sync::MutexGuard;
use crate::cryptography::falcon::falcon512;

lazy_static! {
    pub static ref SECP: Secp256k1<secp256k1::All> = Secp256k1::new();
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Wallet {
    pub balance: u64,
    pub keypair: (falcon512::SecretKey, falcon512::PublicKey),
    pub public_key: falcon512::PublicKey,
}

impl Wallet {
    pub fn new() -> Wallet {
        let mut rng = thread_rng();
        let (secret_key, public_key) = falcon512::keygen(rng.gen());
        Wallet {
            balance: INITIAL_BALANCE,
            keypair: (secret_key, public_key.clone()),
            public_key,
        }
    }

    fn get_private_key(&self) -> falcon512::SecretKey {
        self.keypair.0.clone()
    }

    pub fn sign(&self, message_hash: sha256::Hash) -> falcon512::Signature {
        let message = Message::from_digest_slice(message_hash.as_ref()).unwrap();
        falcon512::sign(message.as_ref(), &self.get_private_key())
    }

    pub fn verify(
        &self,
        message_hash: sha256::Hash,
        signature: falcon512::Signature,
    ) -> bool {
        let message = Message::from_digest_slice(message_hash.as_ref()).unwrap();
        falcon512::verify(message.as_ref(), &signature, &self.public_key)
    }

    pub fn create_transaction(
        &mut self,
        recipient: String,
        amount: u64,
        chain: &MutexGuard<'_, Chain>,
        pool: &mut MutexGuard<'_, Pool>,
    ) -> Result<Transaction, String> {
        self.balance = self.calc_balance(&chain);
        if amount > self.balance {
            return Err("Amount exceeds balance".to_string());
        }

        let transaction = pool.exists(self.clone().public_key);

        let transaction = if let Some(mut tx) = transaction {
            tx.update(self.clone(), recipient, amount)?
        } else {
            let new_tx = Transaction::new(self.clone(), recipient, amount)?;
            pool.update_or_add_transaction(new_tx.clone());
            new_tx
        };
        Ok(transaction)
    }

    pub fn blockchain_wallet() -> Wallet {
        let (secret_key, public_key) = falcon512::keygen([0; 32]);
        Wallet {
            balance: 0,
            keypair: (secret_key, public_key.clone()),
            public_key,
        }
    }

    pub fn calc_balance(&self, chain: &Chain) -> u64 {
        let mut balance = self.balance;
        let mut transactions = Vec::new();

        for block in &chain.chain {
            if let Ok(trans) = serde_json::from_str::<Vec<Transaction>>(&block.data) {
                transactions.extend(trans);
            }
        }

        let wallet_input_ts: Vec<&Transaction> = transactions
            .iter()
            .filter(|t| {
                t.input
                    .iter()
                    .any(|i| i.address.public_key == self.public_key)
            })
            .collect();

        if !wallet_input_ts.is_empty() {
            let recent_input_t = wallet_input_ts
                .into_iter()
                .max_by(|a, b| {
                    a.input
                        .iter()
                        .map(|i| i.timestamp)
                        .max()
                        .cmp(&b.input.iter().map(|i| i.timestamp).max())
                })
                .unwrap();
            balance = recent_input_t
                .output
                .iter()
                .find(|o| o.address == format!("{:?}", self.public_key))
                .map(|o| o.amount)
                .unwrap_or(0);

            let start_time = recent_input_t
                .input
                .iter()
                .map(|i| i.timestamp)
                .max()
                .unwrap();
            for t in transactions {
                if t.input.iter().any(|input| input.timestamp > start_time) {
                    if let Some(output) = t
                        .output
                        .iter()
                        .find(|o| o.address == format!("{:?}", self.public_key))
                    {
                        balance += output.amount;
                    }
                }
            }
        }
        balance
    }
}

impl fmt::Display for Wallet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Wallet {{ balance: {}, keypair: {:?}, public_key: {:?} }}",
            self.balance, self.keypair, self.public_key
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secp256k1::hashes::Hash;

    #[test]
    fn test_wallet_new() {
        let wallet = Wallet::new();
        println!("{}", wallet);
    }

    #[test]
    fn test_wallet_sign() {
        let wallet = Wallet::new();
        let message = "Hello, world!";
        let message_hash = sha256::Hash::hash(message.as_bytes());
        let signature = wallet.sign(message_hash);
        println!("{:?}", signature);
    }

    #[test]
    fn test_wallet_verify() {
        let wallet = Wallet::new();
        let message = "Hello, world!";
        let message_hash = sha256::Hash::hash(message.as_bytes());
        let signature = wallet.sign(message_hash);
        assert!(wallet.verify(message_hash, signature));
    }

    #[tokio::test]
    async fn test_wallet_create_transaction() {
        let pool = Pool::new();
        let mut wallet = Wallet::new();
        let recipient = "recipient".to_string();
        let amount = 10;
        let chain = tokio::sync::Mutex::new(Chain::new());
        let chain_guard = chain.lock().await;
        let binding = tokio::sync::Mutex::new(pool);
        let mut pool_guard = binding.lock().await;

        let transaction = wallet
            .create_transaction(recipient, amount, &chain_guard, &mut pool_guard)
            .unwrap();
        println!("{:?}", transaction);
    }

    #[test]
    fn test_update_wallet() {
        let wallet = Wallet::new();
        let recipient = "recipient".to_string();
        let amount = 10;
        let mut transaction = Transaction::new(wallet.clone(), recipient.clone(), amount).unwrap();
        let updated_transaction = transaction
            .update(wallet.clone(), recipient.clone(), amount)
            .unwrap();
        println!("{:?}", updated_transaction);
    }

    #[test]
    fn test_wallet_blockchain_wallet() {
        let wallet = Wallet::blockchain_wallet();
        println!("{}", wallet);
    }
}
