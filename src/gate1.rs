//! ADR D-070 §2.5 — Gate 1 completeness validator (server mirror of validateGate1.ts).
//!
//! D-123 stage 2: moved verbatim from `apps/api-gateway/src/routes/module_doc_gate1.rs`
//! into the kernel — Gate 1 IS a kernel check (admission of a gene into userland),
//! not platform moderation. The validator stays pure and synchronous (the Q-96
//! retraction fact is injected via `ValidateGate1Options` by the caller), which is
//! exactly what lets the `diu` CLI run the SAME gate locally with zero drift.
//! Mirror parity with the TypeScript twin (Q-460/Q-461) is unaffected: bytes of
//! every message are unchanged.

use serde::Serialize;
use serde_json::Value;

use crate::doi::{is_valid_doi_format, normalize_doi};

const MIN_PROTECTED_LEN: usize = 40;
const MIN_VERIFICATION_LEN: usize = 20;
const NUMERIC_PORT_TYPES: [&str; 3] = ["scalar", "series", "matrix"];

#[derive(Debug, Clone, Serialize)]
pub struct Gate1Failure {
    pub pass: bool,
    pub block_id: String,
    pub message: String,
}

fn fail(block_id: &str, message: &str) -> Gate1Failure {
    Gate1Failure {
        pass: false,
        block_id: block_id.to_string(),
        message: message.to_string(),
    }
}

fn trimmed_len(v: Option<&str>) -> usize {
    v.map(str::trim).map(str::len).unwrap_or(0)
}

fn epistemic_strength(t: &str) -> Option<u8> {
    match t {
        "fact" => Some(0),
        "hypothesis" => Some(1),
        "speculation" => Some(2),
        "unknown" => Some(3),
        _ => None,
    }
}

fn valid_module_id(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.len() < 3 || bytes.len() > 64 {
        return false;
    }
    if !bytes[0].is_ascii_lowercase() {
        return false;
    }
    bytes[1..]
        .iter()
        .all(|&c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
}

fn valid_semver(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

fn port_needs_unit(data_type: &str) -> bool {
    NUMERIC_PORT_TYPES.contains(&data_type.trim().to_ascii_lowercase().as_str())
}

/// D-070 / D-062 Q-97 — canonical Gate 1 port-unit message (mirror validateGate1.ts).
fn port_unit_required_message(block_id: &str, key: &str) -> String {
    format!("{block_id}.{key}: numeric port must declare unit (Q-97)")
}

/// D-070 N6 / Q-96 — retracted `source_doi`. Deliberately free of interpolation: the message
/// must stay byte-identical to the TypeScript mirror, so retraction details travel in the
/// `evidence` field of the 422 body instead.
pub const GATE1_Q96_RETRACTED_DOI: &str = "source_doi is listed as retracted (Q-96)";

fn validate_ports(ports: Option<&Vec<Value>>, block_id: &str, failures: &mut Vec<Gate1Failure>) {
    let Some(arr) = ports else {
        failures.push(fail(
            block_id,
            &format!("{block_id} must declare at least one port"),
        ));
        return;
    };
    if arr.is_empty() {
        failures.push(fail(
            block_id,
            &format!("{block_id} must declare at least one port"),
        ));
        return;
    }
    for port in arr {
        let key = port
            .get("key")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .unwrap_or("");
        if key.is_empty() {
            failures.push(fail(block_id, &format!("{block_id} port is missing key")));
            continue;
        }
        let data_type = port
            .get("data_type")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .unwrap_or("scalar");
        if port_needs_unit(data_type) {
            let unit = port
                .get("unit")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .unwrap_or("");
            if unit.is_empty() {
                failures.push(fail(block_id, &port_unit_required_message(block_id, key)));
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct ValidateGate1Options {
    pub platform_epistemic_ceiling: &'static str,
    pub subscription_tier: i32,
    /// Q-96 (D-070 N6) — resolved by the caller against the retraction list. The validator
    /// stays pure and synchronous; an async DOI lookup inside it would break the mirror
    /// parity pinned by Q-460/Q-461.
    pub source_doi_retracted: bool,
}

impl Default for ValidateGate1Options {
    fn default() -> Self {
        Self {
            platform_epistemic_ceiling: "hypothesis",
            subscription_tier: 2,
            source_doi_retracted: false,
        }
    }
}

/// Pure Gate 1 validator. Returns empty vec when all checks pass.
pub fn validate_gate1(doc: &Value, options: ValidateGate1Options) -> Vec<Gate1Failure> {
    let mut failures = Vec::new();

    let module_id = doc
        .get("identity")
        .and_then(|v| v.get("module_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if !valid_module_id(module_id) {
        failures.push(fail(
            "identity.module_id",
            "module_id must match ^[a-z][a-z0-9_]{2,63}$",
        ));
    }

    let version = doc
        .get("identity")
        .and_then(|v| v.get("version"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if !valid_semver(version) {
        failures.push(fail(
            "identity.version",
            "version must be semver MAJOR.MINOR.PATCH",
        ));
    }

    let title = doc
        .get("identity")
        .and_then(|v| v.get("title"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or("");
    if title.is_empty() {
        failures.push(fail("identity.title", "title is required"));
    } else if title.len() > 120 {
        failures.push(fail("identity.title", "title must be ≤ 120 characters"));
    }

    if options.subscription_tier >= 2 {
        let waived = doc
            .get("identity")
            .and_then(|v| v.get("author_orcid_waived"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if !waived {
            let orcid = doc
                .get("identity")
                .and_then(|v| v.get("author_orcid"))
                .and_then(|v| v.as_str())
                .map(str::trim)
                .unwrap_or("");
            if orcid.is_empty() {
                failures.push(fail(
                    "identity.author_orcid",
                    "author ORCID required for Research tier",
                ));
            } else if !regex_lite_orcid(orcid) {
                failures.push(fail(
                    "identity.author_orcid",
                    "author_orcid must be a valid ORCID format",
                ));
            }
        }
    }

    if trimmed_len(doc.get("scientific_purpose").and_then(|v| v.as_str())) < MIN_PROTECTED_LEN {
        failures.push(fail(
            "scientific_purpose",
            &format!(
                "scientific_purpose must be at least {MIN_PROTECTED_LEN} characters (protected block)"
            ),
        ));
    }

    let equations = doc
        .get("mathematical_model")
        .and_then(|v| v.get("equations"))
        .and_then(|v| v.as_array());
    let has_equation = equations.is_some_and(|a| {
        a.iter()
            .any(|e| e.as_str().is_some_and(|s| !s.trim().is_empty()))
    });
    if !has_equation {
        failures.push(fail(
            "mathematical_model",
            "mathematical_model must include at least one equation",
        ));
    }

    if has_equation {
        let raw_doi = doc
            .get("mathematical_model")
            .and_then(|v| v.get("source_doi"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let doi = normalize_doi(raw_doi);
        if doi.is_empty() {
            failures.push(fail(
                "mathematical_model.source_doi",
                "source_doi is required when equations are declared (Q-95)",
            ));
        } else if !is_valid_doi_format(&doi) {
            failures.push(fail(
                "mathematical_model.source_doi",
                "source_doi must be a valid DOI (10.<registrant>/<suffix>)",
            ));
        } else if options.source_doi_retracted {
            failures.push(fail(
                "mathematical_model.source_doi",
                GATE1_Q96_RETRACTED_DOI,
            ));
        }
    }

    let inputs = doc.get("inputs").and_then(|v| v.as_array()).cloned();
    let outputs = doc.get("outputs").and_then(|v| v.as_array()).cloned();
    validate_ports(inputs.as_ref(), "inputs", &mut failures);
    validate_ports(outputs.as_ref(), "outputs", &mut failures);

    let default_type = doc
        .get("epistemic")
        .and_then(|v| v.get("default_type"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let max_conf = doc
        .get("epistemic")
        .and_then(|v| v.get("max_confidence"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if epistemic_strength(default_type).is_none() {
        failures.push(fail(
            "epistemic.default_type",
            "default_type must be one of: fact, hypothesis, speculation, unknown",
        ));
    }
    if epistemic_strength(max_conf).is_none() {
        failures.push(fail(
            "epistemic.max_confidence",
            "max_confidence must be one of: fact, hypothesis, speculation, unknown",
        ));
    }
    if let (Some(dt), Some(mc)) = (
        epistemic_strength(default_type),
        epistemic_strength(max_conf),
    ) {
        if dt < mc {
            failures.push(fail(
                "epistemic.default_type",
                "default_type cannot be stronger than max_confidence ceiling",
            ));
        }
        if let Some(ceiling) = epistemic_strength(options.platform_epistemic_ceiling) {
            if mc < ceiling {
                failures.push(fail(
                    "epistemic.max_confidence",
                    &format!(
                        "max_confidence cannot exceed platform ceiling ({})",
                        options.platform_epistemic_ceiling
                    ),
                ));
            }
        }
    }

    if trimmed_len(doc.get("known_limitations").and_then(|v| v.as_str())) < MIN_PROTECTED_LEN {
        failures.push(fail(
            "known_limitations",
            &format!(
                "known_limitations must be at least {MIN_PROTECTED_LEN} characters (protected block)"
            ),
        ));
    }

    if trimmed_len(doc.get("verification_test").and_then(|v| v.as_str())) < MIN_VERIFICATION_LEN {
        failures.push(fail(
            "verification_test",
            &format!(
                "verification_test must describe at least one reproducible check (min {MIN_VERIFICATION_LEN} chars)"
            ),
        ));
    }

    failures
}

pub fn gate1_passes(doc: &Value, options: ValidateGate1Options) -> bool {
    validate_gate1(doc, options).is_empty()
}

fn regex_lite_orcid(orcid: &str) -> bool {
    let parts: Vec<&str> = orcid.split('-').collect();
    if parts.len() != 4 {
        return false;
    }
    for (i, part) in parts.iter().enumerate() {
        if part.len() != 4 {
            return false;
        }
        for (j, c) in part.chars().enumerate() {
            if !c.is_ascii_digit() && !(i == 3 && j == 3 && (c == 'X' || c == 'x')) {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn minimal_doc(source_doi: Value) -> Value {
        json!({
            "identity": {
                "module_id": "test_module_x",
                "version": "0.1.0",
                "title": "Test module",
                "author_orcid": "0000-0002-1825-0097"
            },
            "scientific_purpose": "A".repeat(40),
            "mathematical_model": {
                "equations": ["x = 1"],
                "source_doi": source_doi
            },
            "inputs": [{ "key": "r", "data_type": "scalar", "unit": "1/time" }],
            "outputs": [{ "key": "n", "data_type": "series", "unit": "count" }],
            "epistemic": { "default_type": "hypothesis", "max_confidence": "hypothesis" },
            "known_limitations": "B".repeat(40),
            "verification_test": "Check output at defaults."
        })
    }

    fn research_opts() -> ValidateGate1Options {
        ValidateGate1Options {
            subscription_tier: 2,
            ..Default::default()
        }
    }

    #[test]
    fn gate1_passes_minimal_fixture() {
        let doc = minimal_doc(json!("10.1126/science.171.3969.385"));
        assert!(gate1_passes(&doc, research_opts()));
    }

    #[test]
    fn q95_missing_source_doi_blocks_with_description() {
        let doc = minimal_doc(json!(""));
        let failures = validate_gate1(&doc, research_opts());
        let hit = failures
            .iter()
            .find(|f| f.block_id == "mathematical_model.source_doi")
            .expect("Q-95 DOI failure");
        assert!(!hit.pass);
        assert!(hit.message.contains("Q-95"));
        assert!(hit.message.contains("source_doi"));
        assert!(!gate1_passes(&doc, research_opts()));
    }

    #[test]
    fn q95_null_source_doi_blocks_with_description() {
        let doc = minimal_doc(Value::Null);
        let failures = validate_gate1(&doc, research_opts());
        assert!(failures.iter().any(|f| {
            f.block_id == "mathematical_model.source_doi" && f.message.contains("Q-95")
        }));
    }

    #[test]
    fn q95_invalid_source_doi_blocks_with_format_message() {
        let doc = minimal_doc(json!("not-a-doi"));
        let failures = validate_gate1(&doc, research_opts());
        let hit = failures
            .iter()
            .find(|f| f.block_id == "mathematical_model.source_doi")
            .expect("invalid DOI failure");
        assert!(!hit.pass);
        assert!(hit.message.contains("valid DOI"));
    }

    // Q-491 / Q-494 — retracted DOI blocks with the canonical, non-interpolated message.
    #[test]
    fn q96_retracted_source_doi_blocks_with_canonical_message() {
        let doc = minimal_doc(json!("10.1021/am300292v"));
        let opts = ValidateGate1Options {
            source_doi_retracted: true,
            ..research_opts()
        };
        let failures = validate_gate1(&doc, opts);
        let hit = failures
            .iter()
            .find(|f| f.block_id == "mathematical_model.source_doi")
            .expect("Q-96 retraction failure");
        assert!(!hit.pass);
        // Q-494 — the literal is pinned on both sides of the mirror; the TypeScript twin
        // asserts the same string in validateGate1.test.ts.
        assert_eq!(hit.message, "source_doi is listed as retracted (Q-96)");
        assert_eq!(hit.message, GATE1_Q96_RETRACTED_DOI);
        assert!(!gate1_passes(&doc, opts));
        assert!(
            gate1_passes(&doc, research_opts()),
            "the same doc passes when the DOI is not listed"
        );
    }

    // Q-495 — Q-95 wins while the DOI is unusable; Q-96 only applies to a well-formed DOI.
    #[test]
    fn q495_missing_or_malformed_doi_reports_q95_not_q96() {
        let opts = ValidateGate1Options {
            source_doi_retracted: true,
            ..research_opts()
        };
        for raw in [json!(""), json!("not-a-doi")] {
            let doc = minimal_doc(raw);
            let failures = validate_gate1(&doc, opts);
            let messages: Vec<&str> = failures
                .iter()
                .filter(|f| f.block_id == "mathematical_model.source_doi")
                .map(|f| f.message.as_str())
                .collect();
            assert_eq!(messages.len(), 1, "one DOI failure at a time");
            assert_ne!(messages[0], GATE1_Q96_RETRACTED_DOI);
        }
    }

    #[test]
    fn q97_numeric_port_without_unit_blocks_with_description() {
        let mut doc = minimal_doc(json!("10.1126/science.171.3969.385"));
        doc["outputs"] = json!([{ "key": "n", "data_type": "series" }]);
        let failures = validate_gate1(&doc, research_opts());
        let hit = failures
            .iter()
            .find(|f| f.block_id == "outputs")
            .expect("Q-97 unit failure");
        assert!(!hit.pass);
        assert_eq!(hit.message, port_unit_required_message("outputs", "n"));
        assert!(!gate1_passes(&doc, research_opts()));
    }
}
