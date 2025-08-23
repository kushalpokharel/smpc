use std::marker::PhantomData;

use kzen_paillier::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EncodedCiphertextRepr<T> {
    #[serde(with = "kzen_paillier::serialize::bigint")]
    pub raw: BigInt,
    pub components: usize,
    _phantom: PhantomData<T>,
}

pub fn get_bigint_from_encoded_ciphertext<T>(encoded: &EncodedCiphertext<T>) -> BigInt {
    let encrypted_string = serde_json::to_string(&encoded).unwrap();
    let deserialized: EncodedCiphertextRepr<u64> = serde_json::from_str(&encrypted_string).unwrap();
    return deserialized.raw;
}