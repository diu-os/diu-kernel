# diu-kernel

The kernel of **DIU OS** (Decentralized Intelligent Universe Operating System).

Verification invariants every gene, run, claim and export must satisfy — the
scientific analogue of memory protection: mandatory, invisible, never the product.

**License:** Apache-2.0 (ADR D-124, ACCEPTED 14 Sep 2026).
**Norm:** ADR D-123 (Kernel & Userland). Digest: RFC 8785 (JCS) → SHA-256 → hex.

This crate is the portable contract. The commercial distro (`diu-os.com`) and the
private platform (`github.com/diu-platform/diu-workspace`) are not this crate.

## What is in the crate

| Module | Contract |
|---|---|
| `digest` | `canonical_digest()` — RFC 8785 JCS → SHA-256 → hex. The client never supplies a digest. |
| `manifest` | Module manifest (D-113): identity, engine binding, typed ports, epistemic ceiling, provenance. |
| `epistemic` | Vocabulary: `fact` / `hypothesis` / `speculation` / `unknown`. Ceilings never rise through the chain. |
| `gate1` | Gate 1 completeness validator — the same bytes the `diu` CLI and the server run. |
| `doi` | DOI normalisation + format gate (dependency of Gate 1). |
| `syscall` | Six verbs: `submit_module`, `run`, `digest`, `claim`, `export`, `attest`. REST today; MCP planned. |

Engines, HTTP, databases, AI and production endpoints **do not** belong here.

## Kernel invariants

- **K-1** Digest of a recorded result is computed from the canonical form. Request bodies are never trusted for it.
- **K-2** Same engine + same params + same horizon ⇒ same digest (determinism).
- **K-3** Epistemic ceiling does not rise: engine → gene → run → claim → export → attestation.
- **K-4** Verification ≠ validation: the kernel proves “computed correctly”, not “adequate to the world”.
- **K-5** An executable module carries a manifest whose ports/defaults/horizon match the engine bit-for-bit.

## Boundary

- **Kernel (this crate)** — Apache-2.0, inspectable verifier.
- **Distro** — `diu-os.com` remains a commercial distribution.
- **Platform source** — genes, API gateway, frontend and chain code stay private.

A change to the digest algorithm is a major semver bump and requires a new ADR. Never silently.

## Public tree

This crate is mirrored at [`github.com/diu-os/diu-kernel`](https://github.com/diu-os/diu-kernel)
(Apache-2.0 subtree of `kernel/` only). First public tag: **`v0.1.0`**.

It is **not** `diu-os.com` (commercial distro) and **not**
`github.com/diu-platform/diu-workspace` (private platform). Genes, the `diu`
CLI, the API gateway, the frontend and chain code are outside this tree.

```bash
git clone https://github.com/diu-os/diu-kernel.git
cd diu-kernel
git checkout v0.1.0
cargo test
```
