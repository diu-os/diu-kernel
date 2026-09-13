//! Epistemic vocabulary — kernel invariant K-3 (D-042 / D-105 / D-107).
//!
//! The four canonical claim types. Deliberately strings, not a Rust enum: the
//! whole platform (DB columns, manifests, frontend badges ✓◈◇✗) carries
//! `epistemic_type` as a string, and D-113 chose not to introduce an enum.
//!
//! Moved from `apps/api-gateway/src/routes/projects.rs` (D-123 stage 1) — the
//! projects API and the manifest Q-424 gate now validate against THIS const.
//!
//! K-3 (normative): the ceiling never rises through the chain
//! engine → gene → run → claim → export → attestation. A digest proves
//! integrity; an attestation anchors integrity on-chain; a Quantum answer is a
//! candidate, not evidence (D-119). None of them may promote `hypothesis` to
//! `fact` — that is a type error, not an unfortunate wording.

/// Canonical D-042/D-107 vocabulary, in decreasing epistemic strength.
pub const EPISTEMIC_TYPES: [&str; 4] = ["fact", "hypothesis", "speculation", "unknown"];

/// True if `candidate` is one of the four canonical values.
pub fn is_canonical(candidate: &str) -> bool {
    EPISTEMIC_TYPES.contains(&candidate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vocabulary_is_exactly_the_d042_four() {
        assert_eq!(
            EPISTEMIC_TYPES,
            ["fact", "hypothesis", "speculation", "unknown"]
        );
        assert!(is_canonical("hypothesis"));
        assert!(!is_canonical("verified")); // "verified" is a D-120 verification word, not an epistemic type
        assert!(!is_canonical("Fact")); // case-sensitive by design
    }
}
