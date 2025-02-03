use serde::{Deserialize, Serialize};
use crate::domain::crypto::CryptographyAlgorithm;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Wallet<C: CryptographyAlgorithm> {
    pub secret_key: C::SecretKey,
    pub public_key: C::PublicKey,
}

impl <C: CryptographyAlgorithm> Wallet<C> {
    pub fn new() -> Self {
        let (secret_key, public_key) = C::generate_keypair();

        Self {
            secret_key,
            public_key,
        }
    }

    pub fn get_address(&self) -> String {
        C::get_address(&self.public_key)
    }
}