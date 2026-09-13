//! The syscall surface of DIU OS (D-123 §2.1).
//!
//! Six verbs. Userland (genes, SDK/CLI, frontends, third-party tools) talks to
//! the kernel only through these; everything else is transport. Today the
//! transport is REST on api-gateway; MCP is the planned second transport
//! (`DIU_OS_MODULE_ARCHITECTURE.md` §2.3: syscalls = MCP API). The mapping
//! below is the contract a second transport (or a second instance — stage 4
//! federation) must reproduce.

/// One kernel syscall.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Syscall {
    /// Submit a module (gene) for registration: manifest + gate checks (D-062/D-064/D-068).
    /// Gates are kernel checks, not manual platform moderation.
    SubmitModule,
    /// Execute a recorded run of a registered module (D-102). Server computes the digest (K-1).
    Run,
    /// Recompute/verify the digest of a recorded result (D-112 verify-or-refuse, level R1).
    Digest,
    /// Attach a claim to runs: supports/refutes + falsification criterion (D-109, Q-519).
    Claim,
    /// Produce the sealed reproducibility package (manifest v1 + CSV, D-112).
    Export,
    /// Anchor a content hash on-chain (D-087). Anchors integrity, never raises epistemic type (K-3).
    Attest,
}

/// REST binding of each syscall as served by api-gateway today.
/// (method, path template) — paths are the live production routes.
pub const REST_BINDING: [(Syscall, &str, &str); 6] = [
    (
        Syscall::SubmitModule,
        "POST",
        "/api/v1/projects/:id/drafts", // then :draft_id/publish — admission via Gate 1
    ),
    (Syscall::Run, "POST", "/api/v1/projects/:id/runs"),
    (
        Syscall::Digest,
        "GET",
        "/api/v1/exports/:id/timeseries.csv", // verify-or-refuse re-execution path
    ),
    (Syscall::Claim, "POST", "/api/v1/projects/:id/claims"),
    (Syscall::Export, "POST", "/api/v1/projects/:id/exports"),
    (
        Syscall::Attest,
        "POST",
        "/api/v1/exports/:id/attestation",
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_syscall_has_exactly_one_rest_binding() {
        let all = [
            Syscall::SubmitModule,
            Syscall::Run,
            Syscall::Digest,
            Syscall::Claim,
            Syscall::Export,
            Syscall::Attest,
        ];
        for s in all {
            assert_eq!(
                REST_BINDING.iter().filter(|(b, _, _)| *b == s).count(),
                1,
                "{s:?} must have exactly one REST binding"
            );
        }
    }
}
