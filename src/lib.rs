// diu-kernel — the kernel of DIU OS (ADR D-123, Kernel & Userland Architecture).
//
// The kernel is the set of verification invariants every module (gene), run,
// claim and export must satisfy — the scientific analogue of memory protection:
// mandatory, invisible, never the product itself.
//
// What lives here (the contract, not the engines):
//   - `digest`    — canonical result digest: RFC 8785 (JCS) → SHA-256 → hex.
//                   The single definition the record path (D-102), the export
//                   verify-or-refuse path (D-112) and on-chain attestation
//                   (D-087) all agree on. The client NEVER supplies a digest.
//   - `manifest`  — the declarative module manifest contract (D-113): identity,
//                   math engine binding (D-107), typed ports, epistemic ceiling,
//                   provenance. A gene without a manifest is a widget, not a
//                   scientific module.
//   - `criterion` — the acceptance criterion an experiment declares BEFORE it
//                   is executed (D-131), plus the evaluator that turns one
//                   result into a single SVVRE literal. Kernel-resident so the
//                   CLI, the record path and any federated node reach the same
//                   verdict from the same bytes.
//   - `epistemic` — the four-value epistemic vocabulary (D-042/D-107):
//                   fact | hypothesis | speculation | unknown. The ceiling never
//                   flows upward (no digest, attest, AI answer or review may
//                   raise an epistemic type — that is a type error, D-119).
//   - `syscall`   — the named syscall surface of the OS (D-123 §2.1): the six
//                   verbs userland uses to talk to the kernel, independent of
//                   transport (REST today, MCP planned).
//   - `gate1`     — the Gate 1 completeness validator (D-070 §2.5): admission
//                   of a gene into userland. Pure and synchronous, so the SDK
//                   (`diu` CLI) runs the SAME gate locally with zero drift.
//   - `doi`       — DOI normalization + format gate (D-105), a gate1 dependency.
//
// What deliberately does NOT live here:
//   - gene engines (RM/SIR/SEIR/Dst live in userland dispatch; WASM engines in
//     simulation/engine) — the kernel constrains them, it does not contain them;
//   - HTTP/Axum/SQLx — transports and storage are distro concerns;
//   - AI — Quantum is an init-service constrained by the kernel, never part of it.
//
// Kernel invariants (normative, D-123):
//   K-1  A recorded result's digest is computed server-side from the canonical
//        form of the result. No request body is ever trusted for it.
//   K-2  Same engine + same params + same horizon ⇒ same digest (determinism).
//   K-3  Epistemic ceilings do not rise through the chain
//        engine → gene → run → claim → export → attestation.
//   K-4  Verification ≠ validation (D-120): the kernel proves "computed
//        correctly", never "adequate to the world".
//   K-5  Every executable module carries a manifest whose declared ports,
//        defaults and horizon match its engine bit-for-bit (Q-418/Q-419/Q-420).
//   K-6  No execution is recorded without a previously declared experiment
//        (D-131): intent precedes execution, and is never reconstructed from
//        it afterwards (INV-003, INV-019).
//   K-7  An experiment's acceptance criterion is immutable once its first
//        execution exists (D-131). Moving the bar after seeing the result is
//        the failure mode the criterion exists to prevent.

/// Crate version of `diu-kernel`. The federation instance card reports this,
/// not the gateway binary's `/health` version (D-130 slice 3).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod criterion;
pub mod digest;
pub mod doi;
pub mod epistemic;
pub mod gate1;
pub mod manifest;
pub mod syscall;
