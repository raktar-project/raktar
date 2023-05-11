//! A lot of this is taken from the official crates.io code:
//! https://github.com/rust-lang/crates.io/blob/787630999d769287206f78861c3b33fca80c7fa6/src/util/token.rs

use rand::distributions::Uniform;
use rand::rngs::OsRng;
use rand::Rng;
use sha2::{Digest, Sha256};

const TOKEN_LENGTH: usize = 32;

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

#[cfg(test)]
mod tests {
    use crate::auth::token::TOKEN_LENGTH;
    use crate::auth::{generate_new_token, hash};

    #[test]
    fn test_tokens_are_random() {
        let token1 = generate_new_token();
        let token2 = generate_new_token();

        assert_ne!(token1, token2);
    }

    #[test]
    fn test_tokens_have_expected_length() {
        let token = generate_new_token();

        assert_eq!(token.len(), TOKEN_LENGTH);
    }

    #[test]
    fn test_hash() {
        let token = "MZyrH7L0MgsKQTLjEHP72YMvAqC9nEXM";

        let actual = hash(token.as_bytes());
        let expected = vec![
            43, 19, 38, 114, 88, 177, 213, 58, 138, 123, 58, 88, 71, 10, 175, 140, 210, 99, 150,
            234, 15, 186, 163, 122, 113, 180, 38, 44, 66, 97, 204, 212,
        ];

        assert_eq!(actual, expected);
    }
}
