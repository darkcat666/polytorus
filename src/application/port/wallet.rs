use anyhow::Result;
use crate::domain::crypto::CryptographyAlgorithm;
use crate::domain::wallet::Wallet;

pub trait WalletRepository<C: CryptographyAlgorithm> {
    fn save_wallet(&mut self, wallet: &Wallet<C>) ->  Result<()>;
    fn get_wallet(&self, address: &str) -> Result<Wallet<C>>;
    fn get_all_addresses(&self) -> Result<Vec<String>>;
    fn save_all(&self) -> Result<()>;
}