pub mod wallet;
pub mod wallets;

#[cfg(test)]
mod tests {
    use super::wallet::Wallet;
    use super::wallet::hash_pub_key;
    use super::wallets::Wallets;
    use fn_dsa::SigningKey;
    use fn_dsa::VerifyingKey;
    use fn_dsa::{
        signature_size,
        SigningKeyStandard, VerifyingKeyStandard, DOMAIN_NONE,
        HASH_ID_RAW,
    };
    use rand_core::OsRng;

    #[test]
    fn test_create_wallet_and_hash() {
        let w1 = Wallet::new();
        let w2 = Wallet::new();
        assert_ne!(w1, w2);
        assert_ne!(w1.get_address(), w2.get_address());

        let mut p2 = w2.public_key.clone();
        hash_pub_key(&mut p2);
        assert_eq!(p2.len(), 20);
        let pub_key_hash = bitcoincash_addr::Address::decode(&w2.get_address()).unwrap().body;
        assert_eq!(pub_key_hash, p2);
    }

    #[test]
    fn test_wallets() {
        let mut ws = Wallets::new().unwrap();
        let wa1 = ws.create_wallet();
        let w1 = ws.get_wallet(&wa1).unwrap().clone();
        ws.save_all().unwrap();

        let ws2 = Wallets::new().unwrap();
        let w2 = ws2.get_wallet(&wa1).unwrap();
        assert_eq!(&w1, w2);
    }

    #[test]
    #[should_panic]
    fn test_wallets_not_exist() {
        let w3 = Wallet::new();
        let ws2 = Wallets::new().unwrap();
        ws2.get_wallet(&w3.get_address()).unwrap();
    }

    #[test]
    fn test_signature() {
        let w = Wallet::new();
        let mut sk = SigningKeyStandard::decode(&w.secret_key).unwrap();
        let mut sig = vec![0u8; signature_size(sk.get_logn())];
        sk.sign(&mut OsRng, &DOMAIN_NONE, &HASH_ID_RAW, b"message", &mut sig);

        match VerifyingKeyStandard::decode(&w.public_key) {
            Some(vk) => {
                assert!(vk.verify(&sig, &DOMAIN_NONE, &HASH_ID_RAW, b"message"));
            }
            None => {
                panic!("failed to decode verifying key");
            }
        }
    }
}
