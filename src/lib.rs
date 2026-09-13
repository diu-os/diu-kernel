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
//   - `epistemic` — the four-value epistemic vocabulary (D-042/D-107):
//                   fact | hypothesis | speculation | unknown. The ceiling never
//                   flows upward (no digest, attest, AI answer or review may
//                   raise an epistemic type — that is a type error, D-119).
//   - `syscall`   — the named syscall surface of the OS (D-123 §2.1): the six
//                   verbs userland uses to talk to the kernel, independent of
//                   transport (REST today, MCP planned).
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

pub mod digest;
pub mod epistemic;
pub mod manifest;
pub mod syscall;
