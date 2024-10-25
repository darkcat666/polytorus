extern crate bellman;
extern crate pairing;
extern crate rand;
use sha2::Sha256;
use sha3::Digest;
use bellman::{Circuit, ConstraintSystem, SynthesisError};
use pairing::{Engine, Field, PrimeField};
use pairing::bls12_381::{Bls12, Fr};

use super::falcon512::{self, PublicKey, Signature};

struct FalconCircuit {
    pub msg: Option<Vec<u8>>,
    pub signature: Option<Signature>,
    pub public_key: Option<PublicKey>,
}

// メッセージをハッシュ化し、bls12_381::Scalar型に変換
fn hash_message(msg: &[u8]) -> bls12_381::Scalar {
    let mut hasher = Sha256::new();
    hasher.update(msg);
    let hash = hasher.finalize();

    let mut hash_bytes = [0u8; 32];
    hash_bytes.copy_from_slice(&hash);
    bls12_381::Scalar::from_bytes(&hash_bytes).unwrap_or_else(bls12_381::Scalar::zero)
}

// 動的な長さのバイト列をScalar型に変換
fn bytes_to_scalar(bytes: &[u8]) -> bls12_381::Scalar {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let hash = hasher.finalize();

    let mut hash_bytes = [0u8; 32];
    hash_bytes.copy_from_slice(&hash);
    bls12_381::Scalar::from_bytes(&hash_bytes).unwrap_or_else(bls12_381::Scalar::zero)
}

impl Circuit<bls12_381::Scalar> for FalconCircuit {
    fn synthesize<CS: ConstraintSystem<bls12_381::Scalar>>(self, cs: &mut CS) -> Result<(), SynthesisError> {
        let message_hash_var = cs.alloc(|| "message hash", || {
            let msg_hash = hash_message(self.msg.as_ref().ok_or(SynthesisError::AssignmentMissing)?);
            Ok(msg_hash)
        })?;
        
        let signature_var = cs.alloc(|| "signature", || {
            let signature = self.signature.as_ref().ok_or(SynthesisError::AssignmentMissing)?;
            let signature_scalar = bytes_to_scalar(&signature.to_bytes());
            Ok(signature_scalar)
        })?;
        
        let public_key_var = cs.alloc(|| "hidden public key", || {
            let public_key = self.public_key.as_ref().ok_or(SynthesisError::AssignmentMissing)?;
            let public_key_scalar = bytes_to_scalar(&public_key.to_bytes());
            Ok(public_key_scalar)
        })?;

        // 署名の正当性を検証
        let msg = self.msg.as_ref().ok_or(SynthesisError::AssignmentMissing)?;
        let signature = self.signature.as_ref().ok_or(SynthesisError::AssignmentMissing)?;
        let public_key = self.public_key.as_ref().ok_or(SynthesisError::AssignmentMissing)?;
        let is_valid_signature = falcon512::verify(msg, signature, public_key);

        // デバッグ出力で確認
        println!("is_valid_signature: {}", is_valid_signature);

        // 結果をスカラーに変換
        let is_valid_scalar = if is_valid_signature {
            bls12_381::Scalar::one()
        } else {
            bls12_381::Scalar::zero()
        };

        // is_valid_scalar を制約変数に割り当て
        let is_valid_var = cs.alloc(|| "is_valid_scalar", || Ok(is_valid_scalar))?;

        // 制約設定の再確認
        cs.enforce(
            || "is_valid_scalar constraint",
            |lc| lc + is_valid_var,
            |lc| lc + CS::one(),
            |lc| lc + (is_valid_scalar, CS::one()),
        );

        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use bellman::gadgets::test::TestConstraintSystem;
    use pairing::bls12_381::Bls12;
    use pairing::Engine;

    #[test]
    fn test_falcon_circuit() {
        let msg = b"hello world";
        let (sk, pk) = falcon512::keygen([0u8; 32]);
        let signature = falcon512::sign(msg, &sk);
        let public_key = pk;

        let circuit = FalconCircuit {
            msg: Some(msg.to_vec()),
            signature: Some(signature),
            public_key: Some(public_key),
        };

        let mut cs = TestConstraintSystem::<bls12_381::Scalar>::new();
        circuit.synthesize(&mut cs).unwrap();

        assert!(cs.is_satisfied());
    }
}