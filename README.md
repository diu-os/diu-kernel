# diu-kernel

**Independent verification of computational and AI-generated scientific claims.**

`diu-kernel` is the open core of **DIU OS** (Decentralized Intelligent Universe
Operating System). It holds the invariants every model (gene), run, claim and
export must satisfy. Anyone can recheck a DIU OS result with this crate alone,
without our service.

**License:** Apache-2.0 (ADR D-124). **Norm:** ADR D-123 (Kernel & Userland), D-131
(criterion before execution), D-137 (status vocabularies).
**Digest:** RFC 8785 (JCS) → SHA-256 → lowercase hex.

## What the kernel proves, and what it does not

It proves that a result was **computed correctly** from a declared model, declared
inputs and a bar declared **before** the run. It does **not** prove that the model
is adequate to the world (K-4: verification ≠ validation). A `PASS` is relative
to the stated bar, never a verdict on nature.

## Modules

| Module | Contract |
|---|---|
| `digest` | `canonical_digest()` — RFC 8785 JCS → SHA-256 → hex. The client never supplies a digest. |
| `manifest` | Module manifest (D-113): identity, engine binding, typed ports, epistemic ceiling, provenance. Field names are the contract. |
| `criterion` | The acceptance criterion an experiment declares before it runs (D-131) and `evaluate()`, which yields one SVVRE outcome: `PASS` `PARTIAL` `FAIL` `INCONCLUSIVE` `NOT_APPLICABLE`. A missing bar is never a passed bar. |
| `vocabulary` | The three status axes (D-137), with disjoint literals and no conversion between them. |
| `epistemic` | Declared ceiling: `fact` / `hypothesis` / `speculation` / `unknown`. Ceilings never rise through the chain. |
| `gate1` | Gate 1 completeness validator for a gene passport (`gene.json`). |
| `doi` | DOI normalisation and format gate (a Gate 1 dependency). |
| `syscall` | Six verbs: `submit_module`, `run`, `digest`, `claim`, `export`, `attest`. REST today; MCP planned. |

Engines, HTTP, databases, AI and production endpoints **do not** belong here.

## Three status axes (D-137)

| Axis | Question | Carried by | Literals |
|---|---|---|---|
| Declared ceiling | How strong may a statement built on this be, at most? | gene manifest, run, project | 4 (D-042) |
| Execution outcome | Did this run clear the bar declared before it? | one run | 5 (SVVRE) |
| Claim state | What has a human assessment of the evidence established for this claim version, in this scope? | one claim version + scope | 10 (ARCH-0001 §11) |

`hypothesis` never becomes `UNTESTED`, and `PASS` never becomes `SUPPORTED`.
Crossing an axis is a human assessment, never a lookup. A claim with no
assessment has no state; that is not `UNTESTED`.

## Kernel invariants

- **K-1** The digest of a recorded result is computed from the canonical form. Request bodies are never trusted for it.
- **K-2** Same engine + same params + same horizon ⇒ same digest (determinism).
- **K-3** The epistemic ceiling does not rise: engine → gene → run → claim → export → attestation.
- **K-4** Verification ≠ validation: the kernel proves “computed correctly”, not “adequate to the world”.
- **K-5** An executable module carries a manifest whose ports/defaults/horizon match the engine bit-for-bit.
- **K-6** No execution is recorded without a previously declared experiment.
- **K-7** An experiment's acceptance criterion is immutable once its first execution exists.

A change to the digest algorithm is a major semver bump and requires a new ADR. Never silently.

## Use it

```toml
[dependencies]
diu-kernel = "0.2"
```

```rust
use diu_kernel::digest::canonical_digest;
use serde_json::json;

let d = canonical_digest(&json!({"b": 1, "a": [1.0, 2.5]})).unwrap();
assert_eq!(d.len(), 64); // key order does not matter: JCS canonicalises first
```

Or from source:

```bash
git clone https://github.com/diu-os/diu-kernel.git
cd diu-kernel
cargo test
```

## Boundary

- **Kernel (this crate)** — Apache-2.0, inspectable verifier.
- **Distro** — `diu-os.com` is the commercial service that runs genes and seals results.
- **Platform source** — genes, API gateway, frontend and chain code stay private.

This crate is mirrored at [`github.com/diu-os/diu-kernel`](https://github.com/diu-os/diu-kernel)
as an Apache-2.0 subtree of `kernel/` only. It is **not** `diu-os.com` and **not**
the private platform repository.
