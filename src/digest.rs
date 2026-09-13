//! Canonical result digest — kernel invariant K-1/K-2 (D-086 / D-102 / D-112).
//!
//! RFC 8785 (JSON Canonicalization Scheme) over the result value, then SHA-256,
//! hex-encoded. This is the platform's attestation of *what was computed* — an
//! integrity statement, never a truth-about-the-world statement (K-4).
//!
//! Moved verbatim from `apps/api-gateway/src/routes/runs.rs::server_digest`
//! (D-123 stage 1). Byte-neutrality is proven by the frozen digest baselines
//! (Q-417 RM, Q-531 SEIR, Q-537 Dst): the fixtures are immutable, so any drift
//! in this function fails those gates, never the fixtures.

use sha2::{Digest, Sha256};

/// Canonicalization failed (RFC 8785 rejects the value, e.g. a non-finite float).
#[derive(Debug)]
pub struct DigestError;

impl std::fmt::Display for DigestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RFC 8785 canonicalization failed")
    }
}

impl std::error::Error for DigestError {}

/// RFC 8785 (JCS) canonicalization of the result, then SHA-256 → hex.
pub fn canonical_digest(result: &serde_json::Value) -> Result<String, DigestError> {
    let canonical = serde_jcs::to_vec(result).map_err(|_| DigestError)?;
    let mut hasher = Sha256::new();
    hasher.update(&canonical);
    Ok(hex::encode(hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::canonical_digest;
    use serde_json::json;

    // Key order must not matter (JCS sorts object members) — the heart of K-2.
    #[test]
    fn digest_is_key_order_independent() {
        let a = json!({ "b": 1.5, "a": [1, 2, 3] });
        let b = json!({ "a": [1, 2, 3], "b": 1.5 });
        assert_eq!(
            canonical_digest(&a).unwrap(),
            canonical_digest(&b).unwrap()
        );
    }

    // Known-answer stability: SHA-256 over JCS bytes of a fixed value. Guards the
    // algorithm choice itself (swap JCS for plain serde_json and this fails).
    #[test]
    fn digest_known_answer() {
        let v = json!({ "params": { "K": 3.0, "r": 1.0 }, "series": [{ "t": 0.0, "n": 1.0 }] });
        let d = canonical_digest(&v).unwrap();
        assert_eq!(d.len(), 64, "SHA-256 hex length");
        // Recompute independently: JCS bytes are deterministic for this value.
        let jcs = serde_jcs::to_vec(&v).unwrap();
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(&jcs);
        assert_eq!(d, hex::encode(h.finalize()));
    }
}
