use crypto::sha2::Sha256;
use crypto::digest::Digest;
use merkle_cbt::merkle_tree::Merge;

pub struct MergeU8;

impl Merge for MergeU8 {
    type Item = Vec<u8>;
    fn merge(left: &Self::Item, right: &Self::Item) -> Self::Item {
        let mut hasher = Sha256::new();
        let mut data = left.clone();
        data.extend_from_slice(right);
        hasher.input(&data);
        let mut result = [0u8; 32];
        hasher.result(&mut result);
        result.to_vec()
    }
}