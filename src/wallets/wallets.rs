use super::wallet::Wallet;
use bincode::{deserialize, serialize};
use sled;
use std::collections::HashMap;
use anyhow::Result;

pub struct Wallets {
    wallets: HashMap<String, Wallet>,
}

impl Wallets {
    /// Wallets を作成し、ファイルが存在すればそこから読み込む
    pub fn new() -> Result<Wallets> {
        let mut wlt = Wallets {
            wallets: HashMap::<String, Wallet>::new(),
        };
        let db = sled::open("data/wallets")?;

        for item in db.into_iter() {
            let i = item?;
            let address = String::from_utf8(i.0.to_vec())?;
            let wallet: Wallet = deserialize(&i.1.to_vec())?;
            wlt.wallets.insert(address, wallet);
        }
        drop(db);
        Ok(wlt)
    }

    /// 新しいウォレットを作成して Wallets に追加し、アドレスを返す
    pub fn create_wallet(&mut self) -> String {
        let wallet = Wallet::new();
        let address = wallet.get_address();
        self.wallets.insert(address.clone(), wallet);
        log::info!("create wallet: {}", address);
        address
    }

    /// 全ウォレットのアドレスの配列を返す
    pub fn get_all_addresses(&self) -> Vec<String> {
        self.wallets.keys().cloned().collect()
    }

    /// 指定したアドレスのウォレットを返す
    pub fn get_wallet(&self, address: &str) -> Option<&Wallet> {
        self.wallets.get(address)
    }

    /// 全ウォレットをファイルに保存する
    pub fn save_all(&self) -> Result<()> {
        let db = sled::open("data/wallets")?;

        for (address, wallet) in &self.wallets {
            let data = serialize(wallet)?;
            db.insert(address, data)?;
        }

        db.flush()?;
        drop(db);
        Ok(())
    }
}
