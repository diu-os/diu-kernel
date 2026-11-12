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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    const INPUTS: &[InputPort] = &[InputPort {
        key: "r",
        symbol: "r",
        unit: "1/time",
        default: 1.0,
        min: Some(0.0),
        max: None,
        step: None,
        hint: None,
    }];
    const OUTPUTS: &[OutputPort] = &[OutputPort {
        key: "n",
        short: "N",
        label: "N (prey)",
    }];

    const SAMPLE: ModuleManifest = ModuleManifest {
        module_id: "sample",
        module_version: "1.0.0",
        execution_class: "native",
        discipline: DisciplineBinding {
            primary: "ecology",
            secondary: &[],
        },
        math_engine: MathEngineRef {
            engine_id: "E1",
            integrator: "rk4",
            dt: 0.01,
            steps: 100,
        },
        epistemic: EpistemicSpec {
            max_confidence: "hypothesis",
            default_type: "hypothesis",
        },
        ports: Ports {
            inputs: INPUTS,
            outputs: OUTPUTS,
        },
        equations: &["\\dot N = rN"],
        phase_portraits: &[],
        provenance: Provenance {
            doi: "",
            citation: "test",
        },
        standards: None,
        verification: None,
        tools: None,
    };

    fn keys(v: &Value) -> Vec<&str> {
        let mut k: Vec<&str> = v.as_object().unwrap().keys().map(String::as_str).collect();
        k.sort_unstable();
        k
    }

    /// The field names ARE the contract (D-113). A rename is a breaking change
    /// for every golden manifest and every frontend consumer, so it must fail here.
    #[test]
    fn top_level_keys_are_the_d113_contract() {
        let v = serde_json::to_value(SAMPLE).unwrap();
        assert_eq!(
            keys(&v),
            vec![
                "discipline",
                "epistemic",
                "equations",
                "execution_class",
                "math_engine",
                "module_id",
                "module_version",
                "phase_portraits",
                "ports",
                "provenance",
                "standards",
                "tools",
                "verification",
            ]
        );
    }

    /// Option slots always serialize (null in v1), so the shape never shifts
    /// when a contract lands later.
    #[test]
    fn empty_option_slots_serialize_as_null_not_absent() {
        let v = serde_json::to_value(SAMPLE).unwrap();
        assert_eq!(v["standards"], Value::Null);
        assert_eq!(v["verification"], Value::Null);
        assert_eq!(v["tools"], Value::Null);
        let port = &v["ports"]["inputs"][0];
        assert_eq!(port["max"], Value::Null);
        assert_eq!(port["step"], Value::Null);
        assert_eq!(port["hint"], Value::Null);
    }

    #[test]
    fn nested_shapes_are_stable() {
        let v = serde_json::to_value(SAMPLE).unwrap();
        assert_eq!(
            keys(&v["ports"]["inputs"][0]),
            vec!["default", "hint", "key", "max", "min", "step", "symbol", "unit"]
        );
        assert_eq!(
            keys(&v["ports"]["outputs"][0]),
            vec!["key", "label", "short"]
        );
        assert_eq!(
            keys(&v["math_engine"]),
            vec!["dt", "engine_id", "integrator", "steps"]
        );
        assert_eq!(
            v["epistemic"],
            json!({"max_confidence": "hypothesis", "default_type": "hypothesis"})
        );
        assert_eq!(v["provenance"], json!({"doi": "", "citation": "test"}));
    }

    /// The manifest feeds the digest path through serde; float defaults must
    /// survive serialization exactly, or K-5 (manifest ⇄ engine bit-for-bit)
    /// would be checked against a rounded number.
    #[test]
    fn numeric_defaults_round_trip_exactly() {
        let v = serde_json::to_value(SAMPLE).unwrap();
        assert_eq!(v["ports"]["inputs"][0]["default"].as_f64(), Some(1.0));
        assert_eq!(v["math_engine"]["dt"].as_f64(), Some(0.01));
        assert_eq!(v["math_engine"]["steps"].as_u64(), Some(100));
    }

    /// The epistemic slots carry the kernel's own vocabulary.
    #[test]
    fn sample_uses_canonical_epistemic_literals() {
        assert!(crate::epistemic::is_canonical(
            SAMPLE.epistemic.max_confidence
        ));
        assert!(crate::epistemic::is_canonical(
            SAMPLE.epistemic.default_type
        ));
    }
}
