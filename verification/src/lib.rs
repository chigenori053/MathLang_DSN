//! MLDSN-MVF-001: planning is untrusted; only audited kernel results are formal evidence.
pub mod counterexample;
pub mod dsn;
pub mod evidence;
pub mod fixtures;
pub mod ir;
pub mod lean;
pub mod numerical;
pub mod proof;
use serde::Serialize;
use sha2::{Digest, Sha256};
pub fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn hash<T: Serialize>(value: &T) -> String {
    hash_bytes(&serde_json::to_vec(value).expect("verification data serializes"))
}
