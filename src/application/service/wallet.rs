use anyhow::Result;
use crate::domain::crypto::CryptographyAlgorithm;
use crate::domain::wallet::Wallet;
use crate::application::port::wallet::WalletRepository;
use std::marker::PhantomData;

pub struct WalletService<R, C>
where
    R: WalletRepository<C>,
    C: CryptographyAlgorithm,
{
    repository: R,
    marker: PhantomData<C>,
}

impl<R, C> WalletService<R, C>
where
    R: WalletRepository<C>,
    C: CryptographyAlgorithm,
{
    pub fn new(repository: R) -> Self {
        Self {
            repository,
            marker: PhantomData,
        }
    }

    pub fn create_wallet(&mut self) -> Result<String> {
        let wallet = Wallet::new();
        let address = wallet.get_address();
        self.repository.save_wallet(&wallet)?;
        Ok(address)
    }

    pub fn get_all_addresses(&self) -> Result<Vec<String>> {
        self.repository.get_all_addresses()
    }

    pub fn save_all(&self) -> Result<()> {
        self.repository.save_all()
    }
}