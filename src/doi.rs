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
