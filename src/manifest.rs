//! Module manifest contract (D-113) — kernel invariant K-5.
//!
//! A declarative description of an executable module: its identity, the numeric
//! engine behind it (D-107 binding), its input/output ports, the governing
//! equations (KaTeX), and provenance. Every field is `&'static` so a per-module
//! manifest is a plain `const`. Serialize drives the committed golden JSON that
//! both the Rust invariant tests and the frontend consume. JSON keys are stable —
//! the field names ARE the contract; `Option` slots always serialize (null in v1)
//! so the shape never shifts.
//!
//! Moved verbatim from `apps/api-gateway/src/routes/manifest.rs` (D-123 stage 1).
//! Serde output is unchanged, so the golden `module_manifests.json` (Q-421) and
//! the manifest⇄engine invariants (Q-418/Q-419/Q-420) prove the move neutral.
//!
//! In the D-123 vocabulary this file is the boundary between kernel and
//! userland: a gene is admitted to userland by presenting a manifest the kernel
//! can check against the engine's actual behaviour — Aristotle's four causes as
//! a struct (material: engine; formal: math_structure + ports; efficient:
//! integrator; final: provenance + epistemic ceiling).

use serde::Serialize;

/// One tunable model input, as emitted in the recorded `result.params` object.
#[derive(Debug, Clone, Serialize)]
pub struct InputPort {
    /// External param key, exactly as build_result emits it (case-sensitive).
    pub key: &'static str,
    /// KaTeX symbol for display (e.g. "N_0").
    pub symbol: &'static str,
    /// Physical unit (e.g. "1/time", "abundance", "dimensionless").
    pub unit: &'static str,
    /// Canonical default — must equal the engine's own default (invariant I-2).
    pub default: f64,
    /// Lower bound where the model requires it (e.g. non-negativity), else None.
    pub min: Option<f64>,
    /// Upper bound if the UI clamps one, else None.
    pub max: Option<f64>,
    /// Slider step for the UI (ergonomics only — NOT digest-relevant, never enters
    /// build_result). None → the frontend picks a sensible default.
    pub step: Option<f64>,
    /// Short human hint shown under the slider (ergonomics only, NOT digest-relevant).
    pub hint: Option<&'static str>,
}

/// One output series, keyed exactly as the trajectory sample field is named.
#[derive(Debug, Clone, Serialize)]
pub struct OutputPort {
    /// Series field key, exactly as build_result's series objects use it.
    pub key: &'static str,
    /// Short axis/CSV label (e.g. "N").
    pub short: &'static str,
    /// Full legend label (e.g. "N (prey)").
    pub label: &'static str,
}

/// A phase-portrait pairing of two output keys.
#[derive(Debug, Clone, Serialize)]
pub struct PhasePortrait {
    pub x_key: &'static str,
    pub y_key: &'static str,
    pub label: &'static str,
}

/// Which discipline(s) the module belongs to.
#[derive(Debug, Clone, Serialize)]
pub struct DisciplineBinding {
    pub primary: &'static str,
    pub secondary: &'static [&'static str],
}

/// The numeric/formal engine this module executes on (D-107 math-engine library).
#[derive(Debug, Clone, Serialize)]
pub struct MathEngineRef {
    /// Engine identifier from the D-107 catalogue (e.g. "E1").
    pub engine_id: &'static str,
    /// Integration method (e.g. "rk4").
    pub integrator: &'static str,
    /// Fixed integration step.
    pub dt: f64,
    /// Number of integration steps.
    pub steps: usize,
}

/// Epistemic ceiling + default marker (D-102 vocabulary: fact | hypothesis |
/// speculation | unknown). Typed as `&'static str` deliberately — the codebase
/// carries epistemic_type as a string (see `crate::epistemic::EPISTEMIC_TYPES`),
/// not a Rust enum, and D-113 does not introduce one.
#[derive(Debug, Clone, Serialize)]
pub struct EpistemicSpec {
    pub max_confidence: &'static str,
    pub default_type: &'static str,
}

/// Scientific provenance of the model form.
#[derive(Debug, Clone, Serialize)]
pub struct Provenance {
    /// DOI of the source, or "" if none is on file (never fabricated).
    pub doi: &'static str,
    pub citation: &'static str,
}

/// Input + output port sets.
#[derive(Debug, Clone, Serialize)]
pub struct Ports {
    pub inputs: &'static [InputPort],
    pub outputs: &'static [OutputPort],
}

/// The full declarative manifest for one executable module.
#[derive(Debug, Clone, Serialize)]
pub struct ModuleManifest {
    pub module_id: &'static str,
    pub module_version: &'static str,
    pub execution_class: &'static str,
    pub discipline: DisciplineBinding,
    pub math_engine: MathEngineRef,
    pub epistemic: EpistemicSpec,
    pub ports: Ports,
    /// Governing equations as KaTeX strings, in display order.
    pub equations: &'static [&'static str],
    pub phase_portraits: &'static [PhasePortrait],
    pub provenance: Provenance,
    // v1 placeholder slots — always None until their contracts land (D-107/D-042).
    pub standards: Option<&'static [&'static str]>,
    pub verification: Option<&'static [&'static str]>,
    pub tools: Option<&'static [&'static str]>,
}
