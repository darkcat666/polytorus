use bincode::{deserialize, serialize};
use bitcoincash_addr::*;
use crypto::digest::Digest;
use crypto::ripemd160::Ripemd160;
use crypto::sha2::Sha256;
use fn_dsa::{
    sign_key_size, vrfy_key_size, KeyPairGenerator, KeyPairGeneratorStandard,
    FN_DSA_LOGN_512,
};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Wallet {
    pub secret_key: Vec<u8>,
    pub public_key: Vec<u8>,
}

impl Wallet {
    /// 新規ウォレットを作成して返す
    pub fn new() -> Self {
        let mut kg = KeyPairGeneratorStandard::default();
        let mut sign_key = [0u8; sign_key_size(FN_DSA_LOGN_512)];
        let mut vrfy_key = [0u8; vrfy_key_size(FN_DSA_LOGN_512)];
        kg.keygen(FN_DSA_LOGN_512, &mut OsRng, &mut sign_key, &mut vrfy_key);

        Wallet {
            secret_key: sign_key.to_vec(),
            public_key: vrfy_key.to_vec(),
        }
    }

    /// ウォレットアドレスを返す
    pub fn get_address(&self) -> String {
        let mut pub_hash: Vec<u8> = self.public_key.clone();
        hash_pub_key(&mut pub_hash);
        let address = Address {
            body: pub_hash,
            scheme: Scheme::Base58,
            hash_type: HashType::Script,
            ..Default::default()
        };
        address.encode().unwrap()
    }
}

/// 公開鍵のハッシュを生成する関数
pub fn hash_pub_key(pub_key: &mut Vec<u8>) {
    let mut hasher1 = Sha256::new();
    hasher1.input(pub_key);
    hasher1.result(pub_key);
    let mut hasher2 = Ripemd160::new();
    hasher2.input(pub_key);
    pub_key.resize(20, 0);
    hasher2.result(pub_key);
}
