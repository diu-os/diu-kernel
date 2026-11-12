//! DOI normalization & format gate (D-105 Stage 2 / Q-429) — kernel-level
//! because Gate 1 (a kernel check, D-123) depends on it.
//!
//! Moved verbatim from `apps/api-gateway/src/routes/projects.rs` (D-123 stage 2).

/// Strip common DOI URL / `doi:` prefixes; return the bare `10.…/…` form.
/// Empty input stays empty.
pub fn normalize_doi(raw: &str) -> String {
    let s = raw.trim();
    let s = s
        .strip_prefix("https://doi.org/")
        .or_else(|| s.strip_prefix("http://doi.org/"))
        .or_else(|| s.strip_prefix("https://dx.doi.org/"))
        .or_else(|| s.strip_prefix("http://dx.doi.org/"))
        .unwrap_or(s);
    let s = s
        .strip_prefix("doi:")
        .or_else(|| s.strip_prefix("DOI:"))
        .unwrap_or(s);
    s.trim().to_string()
}

/// Format gate for `source_doi`: `10.<registrant>/<suffix>` (Crossref-ish).
/// Empty string is allowed (means "clear").
pub fn is_valid_doi_format(doi: &str) -> bool {
    if doi.is_empty() {
        return true;
    }
    let Some((reg, suffix)) = doi.split_once('/') else {
        return false;
    };
    reg.starts_with("10.")
        && reg.len() > 3
        && reg
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
        && !suffix.is_empty()
        && !suffix.chars().any(|c| c.is_whitespace())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_every_documented_prefix() {
        for raw in [
            "https://doi.org/10.1000/xyz",
            "http://doi.org/10.1000/xyz",
            "https://dx.doi.org/10.1000/xyz",
            "http://dx.doi.org/10.1000/xyz",
            "doi:10.1000/xyz",
            "DOI:10.1000/xyz",
            "  10.1000/xyz  ",
        ] {
            assert_eq!(normalize_doi(raw), "10.1000/xyz", "input {raw:?}");
        }
    }

    #[test]
    fn strips_one_prefix_then_trims_inside() {
        // URL prefix first, then `doi:` — the order the function documents.
        assert_eq!(normalize_doi("https://doi.org/doi:10.1/a"), "10.1/a");
        assert_eq!(normalize_doi("doi: 10.1/a"), "10.1/a");
    }

    #[test]
    fn empty_stays_empty_and_is_a_valid_clear() {
        assert_eq!(normalize_doi(""), "");
        assert_eq!(normalize_doi("   "), "");
        assert!(is_valid_doi_format(""));
    }

    #[test]
    fn accepts_real_crossref_shapes() {
        for doi in [
            "10.1038/nature12373",
            "10.1103/PhysRevLett.116.061102",
            "10.1016/0040-5809(73)90024-8", // brackets in the suffix are legal
            "10.48550/arXiv.2509.12345",
            "10.1000.10/abc", // dotted registrant
        ] {
            assert!(is_valid_doi_format(doi), "{doi} should pass");
        }
    }

    #[test]
    fn refuses_malformed_doi() {
        for doi in [
            "10.1038",                // no suffix separator
            "10.1038/",               // empty suffix
            "10./abc",                // empty registrant
            "11.1038/abc",            // not a 10. directory
            "10.1038/has space",      // whitespace in suffix
            "10.10_38/abc",           // illegal registrant char
            "https://doi.org/10.1/a", // format gate expects the normalized form
        ] {
            assert!(!is_valid_doi_format(doi), "{doi} should be refused");
        }
    }

    #[test]
    fn normalize_then_validate_is_the_gate_pipeline() {
        let n = normalize_doi("https://doi.org/10.1016/0040-5809(73)90024-8");
        assert!(is_valid_doi_format(&n));
    }
}
