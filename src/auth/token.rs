// src/auth/token.rs
use base64::Engine;
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::{Digest, Sha256};

pub const DEFAULT_TOKEN_BYTES: usize = 32;

/// Generate a secure random token using the OS RNG.
/// This is what the web app should call.
pub fn generate_token_default() -> String {
    let mut rng = OsRng;
    generate_token(&mut rng, DEFAULT_TOKEN_BYTES)
}

/// Generate a URL-safe token from random bytes.
/// - Uses Base64 URL-safe, no padding.
/// - Typically 32 bytes -> ~43 char token.
pub fn generate_token<R: RngCore>(rng: &mut R, nbytes: usize) -> String {
    let mut buf = vec![0u8; nbytes];
    rng.fill_bytes(&mut buf);
    base64_url_nopad(&buf)
}

/// Hash a token using SHA-256.
/// Store this output in DB (BLOB).
pub fn hash_token(token: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let out = hasher.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&out);
    arr
}

/// Constant-time-ish compare for hashes (simple and sufficient here).
pub fn hashes_equal(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

fn base64_url_nopad(bytes: &[u8]) -> String {
    // URL_SAFE_NO_PAD makes tokens safe for query params without encoding.
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn token_is_url_safe_no_pad() {
        let mut rng = StdRng::seed_from_u64(123);
        let t = generate_token(&mut rng, 32);

        // URL-safe base64 characters: A-Z a-z 0-9 - _
        assert!(!t.contains('+'));
        assert!(!t.contains('/'));
        assert!(!t.contains('='));
        assert!(t
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
        assert!(t.len() >= 40); // 32 bytes => usually 43 chars
    }

    #[test]
    fn hash_is_deterministic() {
        let h1 = hash_token("hello");
        let h2 = hash_token("hello");
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_changes_with_input() {
        let h1 = hash_token("hello");
        let h2 = hash_token("hello!");
        assert_ne!(h1, h2);
    }

    #[test]
    fn hashes_equal_constant_time_style() {
        let a = hash_token("abc");
        let b = hash_token("abc");
        let c = hash_token("abd");

        assert!(hashes_equal(&a, &b));
        assert!(!hashes_equal(&a, &c));
    }

    #[test]
    fn generate_token_changes() {
        let mut rng = StdRng::seed_from_u64(1);
        let t1 = generate_token(&mut rng, 32);
        let t2 = generate_token(&mut rng, 32);
        assert_ne!(t1, t2);
    }
}
