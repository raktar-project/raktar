//! A lot of this is taken from the official crates.io code:
//! https://github.com/rust-lang/crates.io/blob/787630999d769287206f78861c3b33fca80c7fa6/src/util/token.rs

use rand::distributions::Uniform;
use rand::rngs::OsRng;
use rand::Rng;
use sha2::{Digest, Sha256};

const TOKEN_LENGTH: usize = 32;

pub type SecureToken = Vec<u8>;

pub fn generate_new_token() -> String {
    generate_secure_alphanumeric_string(TOKEN_LENGTH)
}

pub fn hash(token: &[u8]) -> Vec<u8> {
    Sha256::digest(token).as_slice().to_vec()
}

fn generate_secure_alphanumeric_string(len: usize) -> String {
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

    OsRng
        .sample_iter(Uniform::from(0..CHARS.len()))
        .map(|idx| CHARS[idx] as char)
        .take(len)
        .collect()
}
