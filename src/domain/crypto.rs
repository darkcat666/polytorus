pub trait CryptographyAlgorithm {
    type SecretKey: serde::Serialize + serde::de::DeserializeOwned;
    type PublicKey: serde::Serialize + serde::de::DeserializeOwned;
    fn generate_keypair() -> (Self::SecretKey, Self::PublicKey);
    fn get_address(public_key: &Self::PublicKey) -> String;
    fn sign(secret_key: &Self::SecretKey, message: &[u8]) -> Vec<u8>;
    fn verify(public_key: &Self::PublicKey, message: &[u8], signature: &[u8]) -> bool;
}