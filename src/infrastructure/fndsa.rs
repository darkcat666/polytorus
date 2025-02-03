use crate::domain::crypto::CryptographyAlgorithm;
use crypto::digest::Digest;
use crypto::ripemd160::Ripemd160;
use crypto::sha2::Sha256;
use fn_dsa::{
    sign_key_size, signature_size, vrfy_key_size, KeyPairGenerator, KeyPairGeneratorStandard, SigningKey, SigningKeyStandard, VerifyingKey, VerifyingKeyStandard, DOMAIN_NONE, FN_DSA_LOGN_512, HASH_ID_RAW
};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FNDSASecretKey(pub Vec<u8>);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FNDSAPublicKey(pub Vec<u8>);

pub struct FNDSAAlgorithm;

impl CryptographyAlgorithm for FNDSAAlgorithm {
    type SecretKey = FNDSASecretKey;
    type PublicKey = FNDSAPublicKey;

    fn generate_keypair() -> (Self::SecretKey, Self::PublicKey) {
        let mut kg = KeyPairGeneratorStandard::default();
        let mut sign_key = [0u8; sign_key_size(FN_DSA_LOGN_512)];
        let mut vrfy_key = [0u8; vrfy_key_size(FN_DSA_LOGN_512)];
        kg.keygen(FN_DSA_LOGN_512, &mut OsRng, &mut sign_key, &mut vrfy_key);

        (
            FNDSASecretKey(sign_key.to_vec()),
            FNDSAPublicKey(vrfy_key.to_vec()),
        )
    }

    fn get_address(public_key: &Self::PublicKey) -> String {
        let pub_hash = hash_pub_key(&public_key.0);
        use bitcoincash_addr::{Address, Scheme, HashType};
        let address = Address {
            body: pub_hash,
            scheme: Scheme::Base58,
            hash_type: HashType::Script,
            ..Default::default()
        };
        address.encode().expect("Failed to encode address")
    }

    fn sign(secret_key: &Self::SecretKey, message: &[u8]) -> Vec<u8> {
        let mut sk = SigningKeyStandard::decode(&secret_key.0).expect("Failed to decode secret key");
        let mut signature = vec![0u8; signature_size(sk.get_logn())];
    
        sk.sign(&mut OsRng, &DOMAIN_NONE, &HASH_ID_RAW, message, &mut signature);
        signature
    }

    fn verify(public_key: &Self::PublicKey, message: &[u8], signature: &[u8]) -> bool {
        match VerifyingKeyStandard::decode(&public_key.0) {
            Some(vk) => vk.verify(signature, &DOMAIN_NONE, &HASH_ID_RAW, message),
            None => false,
        }
    }


}

fn hash_pub_key(pub_key: &[u8]) -> Vec<u8> {
    // SHA256
    let mut sha256 = Sha256::new();
    sha256.input(pub_key);
    let mut sha256_result = vec![0u8; sha256.output_bytes()];
    sha256.result(&mut sha256_result);

    // RIPEMD160
    let mut ripemd160 = Ripemd160::new();
    ripemd160.input(&sha256_result);
    let mut ripemd_result = vec![0u8; ripemd160.output_bytes()];
    ripemd160.result(&mut ripemd_result);

    ripemd_result
}