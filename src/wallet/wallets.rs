use std::fmt;
use secp256k1::rand::rngs::OsRng;
use secp256k1::{Secp256k1, Message};
use crate::blockchain::config::INITIAL_BALANCE;
use secp256k1::hashes::sha256;
use lazy_static::lazy_static;
use super::transaction::Transaction;
use super::transaction_pool::Pool;

lazy_static! {
    pub static ref SECP: Secp256k1<secp256k1::All> = Secp256k1::new();
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Wallet {
    pub balance: u64,
    pub keypair: secp256k1::Keypair,
    pub public_key: secp256k1::PublicKey,
}

impl Wallet {
    pub fn new() -> Wallet {
        let (secret_key, public_key) = SECP.generate_keypair(&mut OsRng);
        let keypair = secp256k1::Keypair::from_secret_key(&SECP, &secret_key);
        Wallet {
            balance: INITIAL_BALANCE,
            keypair,
            public_key,
        }
    }

    pub fn get_private_key(&self) -> secp256k1::SecretKey {
        self.keypair.secret_key()
    }

    pub fn sign(&self, message_hash: sha256::Hash) -> secp256k1::ecdsa::Signature {
        let message = Message::from_digest_slice(message_hash.as_ref()).unwrap();
        SECP.sign_ecdsa(&message, &self.keypair.secret_key())
    }

    pub fn verify(&self, message_hash: sha256::Hash, signature: secp256k1::ecdsa::Signature) -> bool {
        let message = Message::from_digest_slice(message_hash.as_ref()).unwrap();
        SECP.verify_ecdsa(&message, &signature, &self.public_key).is_ok()
    }

    pub fn create_transaction(&self, recipient: String, amount: u64, pool: &mut Pool) -> Result<Transaction, String> {
        if amount > self.balance {
            return Err("Amount exceeds balance".to_string());
        }
    
        let mut transaction = pool.exists(self.clone());
    
        if transaction.is_none() {
            transaction = Some(Transaction::new(self.clone(), recipient.clone(), amount)?);
        } else {
            let mut transaction = transaction.take().unwrap();
            transaction.output.push(super::transaction::Output {
                amount: amount,
                address: recipient,
            });
        }
        Ok(transaction.unwrap())
    }
}

impl fmt::Display for Wallet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Wallet {{ balance: {}, keypair: {:?}, public_key: {} }}",
            self.balance, self.keypair, self.public_key)
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

    #[test]
    fn test_wallet_create_transaction() {
        let mut pool = Pool::new();
        let wallet = Wallet::new();
        let recipient = "recipient".to_string();
        let amount = 10;
        let transaction = wallet.create_transaction(recipient.clone(), amount, &mut pool).unwrap();
        println!("{:?}", transaction);
    }
}