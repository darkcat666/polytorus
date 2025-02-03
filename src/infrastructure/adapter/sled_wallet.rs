use crate::domain::crypto::CryptographyAlgorithm;
use crate::domain::wallet::Wallet;
use crate::application::port::wallet::WalletRepository;
use anyhow::Result;
use bincode::{serialize, deserialize};
use sled;
use std::collections::HashMap;

pub struct SledWalletRepository {
    wallets: HashMap<String, Vec<u8>>, // address -> Serialized data
    db: sled::Db,
}

impl<C> WalletRepository<C> for SledWalletRepository
where
    C: CryptographyAlgorithm,
    Wallet<C>: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    fn save_wallet(&mut self, wallet: &Wallet<C>) -> Result<()> {
        let address = wallet.get_address();
        let data = serialize(wallet)?;
        self.db.insert(address.as_bytes(), data.clone())?;
        self.wallets.insert(address, data);
        Ok(())
    }

    fn get_wallet(&self, address: &str) -> Result<Wallet<C>> {
        if let Some(data) = self.wallets.get(address) {
            let wallet: Wallet<C> = deserialize(data)?;
            Ok(wallet)
        } else {
            anyhow::bail!("wallet not found");
        }
    }

    fn get_all_addresses(&self) -> Result<Vec<String>> {
        Ok(self.wallets.keys().cloned().collect())
    }

    fn save_all(&self) -> Result<()> {
        for (address, data) in &self.wallets {
            self.db.insert(address.as_bytes(), data.clone())?;
        }
        self.db.flush()?;
        Ok(())
    }
}