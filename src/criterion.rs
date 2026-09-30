//! Acceptance criterion — kernel invariants K-6/K-7 (D-131, Research Work Spine).
//!
//! An experiment declares *what would count as a result* before it is executed
//! (Р-3, INV-003, D-120's Protocol). This module is the machine half of that
//! declaration: a set of numeric predicates over the canonical result, and the
//! evaluator that turns them into one literal of the SVVRE vocabulary.
//!
//! It lives in the kernel and not in a route for the same reason `digest` does:
//! the CLI, the record path and any future federated node must reach the same
//! verdict from the same bytes, with zero drift.
//!
//! Normative choices, each of which is load-bearing:
//!
//! 1. **A missing metric is not a failure.** If the result simply does not
//!    carry the field the criterion names, the criterion could not be applied
//!    — `NOT_APPLICABLE`. Reporting `FAIL` there would manufacture a negative
//!    finding out of a contract mismatch.
//! 2. **An unmeasurable metric is not a failure.** A present but non-finite or
//!    non-numeric value yields `INCONCLUSIVE`. FAIL ≠ INVALID ≠ INCONCLUSIVE
//!    (D-129, D-120): "we looked and it did not hold" is a different statement
//!    from "we could not look".
//! 3. **A set is not silently collapsed into a conjunction.** When some
//!    predicates hold and others do not, the outcome is `PARTIAL`, not `FAIL`.
//!    The machine reports the *shape* of the result; whether a partial
//!    acceptance is good enough is a scientific judgement reserved for the
//!    author (K-4 — the kernel never rules on adequacy to the world).
//! 4. **Definite negatives survive uncertainty.** A predicate that definitely
//!    failed is not rescued by another predicate that could not be evaluated;
//!    `FAIL`/`PARTIAL` dominate `INCONCLUSIVE`.
//!
//! The criterion is also the object frozen by **K-7**: once an experiment has
//! its first execution, the declaration may no longer change. Freezing is
//! enforced in the schema (D-131 §6.4); this module only makes the frozen
//! object hashable, via [`CriterionSet::digest`].

use serde::{Deserialize, Serialize};

use crate::digest::{canonical_digest, DigestError};

/// The five SVVRE outcome literals (`ARCH-0001` §9), in the order the DB CHECK
/// constraint lists them. The DB and this array must never diverge.
pub const SVVRE_OUTCOMES: [&str; 5] = ["PASS", "PARTIAL", "FAIL", "INCONCLUSIVE", "NOT_APPLICABLE"];

/// Outcome of evaluating a criterion set against one execution's result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    /// Every applicable predicate held.
    #[serde(rename = "PASS")]
    Pass,
    /// Some predicates held, others definitely did not.
    #[serde(rename = "PARTIAL")]
    Partial,
    /// No predicate held, and at least one definitely did not.
    #[serde(rename = "FAIL")]
    Fail,
    /// Nothing definitely failed, but the declaration could not be fully
    /// evaluated against this result.
    #[serde(rename = "INCONCLUSIVE")]
    Inconclusive,
    /// There was no applicable criterion at all (an exploratory or archived
    /// experiment, or a declaration naming fields this engine never emits).
    #[serde(rename = "NOT_APPLICABLE")]
    NotApplicable,
}

impl Outcome {
    /// The DB / wire literal.
    pub fn as_str(self) -> &'static str {
        match self {
            Outcome::Pass => "PASS",
            Outcome::Partial => "PARTIAL",
            Outcome::Fail => "FAIL",
            Outcome::Inconclusive => "INCONCLUSIVE",
            Outcome::NotApplicable => "NOT_APPLICABLE",
        }
    }

    /// Parse a stored literal back. Case-sensitive by design, like the
    /// epistemic vocabulary.
    pub fn parse(literal: &str) -> Option<Self> {
        match literal {
            "PASS" => Some(Outcome::Pass),
            "PARTIAL" => Some(Outcome::Partial),
            "FAIL" => Some(Outcome::Fail),
            "INCONCLUSIVE" => Some(Outcome::Inconclusive),
            "NOT_APPLICABLE" => Some(Outcome::NotApplicable),
            _ => None,
        }
    }
}

impl std::fmt::Display for Outcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// How the observed value is compared against the declared threshold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Comparison {
    Lt,
    Le,
    Gt,
    Ge,
    /// `|observed − threshold| ≤ tolerance`. Requires `tolerance`; exact
    /// equality on floats is never what a scientist means.
    Within,
}

/// One numeric predicate over the canonical result.
///
/// `metric` is a dot-separated path into the result object, e.g.
/// `"fit.r_squared"` or `"peak.value"`. Only object traversal is supported:
/// indexing into arrays is deliberately absent, because a criterion that
/// depends on an element position is a criterion about the engine's output
/// layout rather than about the science.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Criterion {
    pub metric: String,
    pub op: Comparison,
    pub threshold: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tolerance: Option<f64>,
}

/// The declaration attached to an experiment: every predicate is part of the
/// same acceptance statement.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CriterionSet {
    #[serde(default)]
    pub all: Vec<Criterion>,
}

/// A declaration the kernel refuses to accept. Surfaced by the route as a 422
/// at *declaration* time, never at execution time — a criterion that cannot be
/// evaluated must not reach the point where it silently degrades an outcome.
#[derive(Debug, PartialEq, Eq)]
pub enum CriterionError {
    /// `metric` is empty or has an empty path segment.
    EmptyMetric,
    /// `threshold` is NaN or infinite.
    NonFiniteThreshold { metric: String },
    /// `Within` without a tolerance, or with a negative / non-finite one.
    InvalidTolerance { metric: String },
    /// The same metric is constrained twice; the author must express the
    /// intent as one predicate or rename the metrics.
    DuplicateMetric { metric: String },
}

impl std::fmt::Display for CriterionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CriterionError::EmptyMetric => f.write_str("criterion metric path is empty"),
            CriterionError::NonFiniteThreshold { metric } => {
                write!(f, "criterion threshold for `{metric}` is not finite")
            }
            CriterionError::InvalidTolerance { metric } => write!(
                f,
                "criterion `{metric}` uses `within` without a finite, non-negative tolerance"
            ),
            CriterionError::DuplicateMetric { metric } => {
                write!(f, "metric `{metric}` is constrained more than once")
            }
        }
    }
}

impl std::error::Error for CriterionError {}

/// Verdict for a single predicate, before set-level aggregation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Verdict {
    Held,
    Violated,
    /// Present, but nothing comparable came back (non-finite or non-numeric).
    Unmeasurable,
    /// The result does not carry this metric at all.
    Absent,
}

impl Criterion {
    /// Evaluate this predicate against a canonical result.
    fn verdict(&self, result: &serde_json::Value) -> Verdict {
        let Some(node) = lookup(result, &self.metric) else {
            return Verdict::Absent;
        };
        if node.is_null() {
            return Verdict::Absent;
        }
        let Some(observed) = node.as_f64() else {
            return Verdict::Unmeasurable;
        };
        if !observed.is_finite() {
            return Verdict::Unmeasurable;
        }

        let held = match self.op {
            Comparison::Lt => observed < self.threshold,
            Comparison::Le => observed <= self.threshold,
            Comparison::Gt => observed > self.threshold,
            Comparison::Ge => observed >= self.threshold,
            // `validate` guarantees the tolerance is present and sane; a set
            // that skipped validation degrades to Unmeasurable rather than
            // silently comparing against zero.
            Comparison::Within => match self.tolerance {
                Some(tol) if tol.is_finite() && tol >= 0.0 => {
                    (observed - self.threshold).abs() <= tol
                }
                _ => return Verdict::Unmeasurable,
            },
        };

        if held {
            Verdict::Held
        } else {
            Verdict::Violated
        }
    }
}

impl CriterionSet {
    /// An experiment may legitimately carry no criterion (exploratory work,
    /// `legacy_archive`). Such a set evaluates to `NOT_APPLICABLE`, never to
    /// `PASS` — absence of a bar is not a cleared bar.
    pub fn is_empty(&self) -> bool {
        self.all.is_empty()
    }

    /// Reject declarations that could not be honestly evaluated later.
    pub fn validate(&self) -> Result<(), CriterionError> {
        let mut seen: Vec<&str> = Vec::with_capacity(self.all.len());
        for c in &self.all {
            if c.metric.is_empty() || c.metric.split('.').any(str::is_empty) {
                return Err(CriterionError::EmptyMetric);
            }
            if !c.threshold.is_finite() {
                return Err(CriterionError::NonFiniteThreshold {
                    metric: c.metric.clone(),
                });
            }
            if matches!(c.op, Comparison::Within)
                && !matches!(c.tolerance, Some(t) if t.is_finite() && t >= 0.0)
            {
                return Err(CriterionError::InvalidTolerance {
                    metric: c.metric.clone(),
                });
            }
            if seen.contains(&c.metric.as_str()) {
                return Err(CriterionError::DuplicateMetric {
                    metric: c.metric.clone(),
                });
            }
            seen.push(&c.metric);
        }
        Ok(())
    }

    /// Stable hash of the frozen declaration (K-7 evidence). Same JCS→SHA-256
    /// definition as every other digest in the system, so field order in the
    /// stored JSON cannot change it.
    pub fn digest(&self) -> Result<String, DigestError> {
        let value = serde_json::to_value(self).map_err(|_| DigestError)?;
        canonical_digest(&value)
    }
}

/// Evaluate a whole declaration against one execution's result.
///
/// Aggregation, in order:
///
/// | Situation | Outcome |
/// |---|---|
/// | no predicates, or every predicate absent | `NOT_APPLICABLE` |
/// | some held, some violated | `PARTIAL` |
/// | none held, at least one violated | `FAIL` |
/// | nothing violated, something unmeasurable or absent | `INCONCLUSIVE` |
/// | every predicate held | `PASS` |
pub fn evaluate(set: &CriterionSet, result: &serde_json::Value) -> Outcome {
    if set.all.is_empty() {
        return Outcome::NotApplicable;
    }

    let (mut held, mut violated, mut unmeasurable, mut absent) = (0usize, 0usize, 0usize, 0usize);
    for c in &set.all {
        match c.verdict(result) {
            Verdict::Held => held += 1,
            Verdict::Violated => violated += 1,
            Verdict::Unmeasurable => unmeasurable += 1,
            Verdict::Absent => absent += 1,
        }
    }

    if absent == set.all.len() {
        return Outcome::NotApplicable;
    }
    if violated > 0 {
        return if held > 0 {
            Outcome::Partial
        } else {
            Outcome::Fail
        };
    }
    if unmeasurable > 0 || absent > 0 {
        return Outcome::Inconclusive;
    }
    Outcome::Pass
}

/// Walk a dot-separated path through nested objects.
fn lookup<'a>(root: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    let mut node = root;
    for segment in path.split('.') {
        node = node.get(segment)?;
    }
    Some(node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ge(metric: &str, threshold: f64) -> Criterion {
        Criterion {
            metric: metric.to_string(),
            op: Comparison::Ge,
            threshold,
            tolerance: None,
        }
    }

    fn set(all: Vec<Criterion>) -> CriterionSet {
        CriterionSet { all }
    }

    #[test]
    fn svvre_vocabulary_is_exactly_five_literals() {
        assert_eq!(SVVRE_OUTCOMES.len(), 5);
        for literal in SVVRE_OUTCOMES {
            let parsed = Outcome::parse(literal).expect("literal round-trips");
            assert_eq!(parsed.as_str(), literal);
        }
        assert_eq!(Outcome::parse("pass"), None, "case-sensitive by design");
        assert_eq!(
            Outcome::parse("VERIFIED"),
            None,
            "verification words are not outcomes"
        );
    }

    #[test]
    fn all_predicates_hold_is_pass() {
        let s = set(vec![ge("fit.r_squared", 0.8), ge("fit.iterations", 1.0)]);
        let r = json!({ "fit": { "r_squared": 0.93, "iterations": 12 } });
        assert_eq!(evaluate(&s, &r), Outcome::Pass);
    }

    #[test]
    fn every_predicate_violated_is_fail() {
        let s = set(vec![ge("fit.r_squared", 0.8)]);
        let r = json!({ "fit": { "r_squared": 0.12 } });
        assert_eq!(evaluate(&s, &r), Outcome::Fail);
    }

    #[test]
    fn mixed_hold_and_violation_is_partial_not_fail() {
        let s = set(vec![ge("fit.r_squared", 0.8), ge("fit.iterations", 1.0)]);
        let r = json!({ "fit": { "r_squared": 0.12, "iterations": 12 } });
        assert_eq!(evaluate(&s, &r), Outcome::Partial);
    }

    // Rule 1: a declaration naming a field this engine never emits is a
    // contract mismatch, not a negative finding.
    #[test]
    fn absent_metric_is_not_applicable_never_fail() {
        let s = set(vec![ge("fit.r_squared", 0.8)]);
        assert_eq!(
            evaluate(&s, &json!({ "series": [] })),
            Outcome::NotApplicable
        );
        assert_eq!(
            evaluate(&s, &json!({ "fit": { "r_squared": null } })),
            Outcome::NotApplicable,
            "an explicit null is absence, not a zero"
        );
    }

    // Rule 2: "we could not look" is not "it did not hold".
    #[test]
    fn unmeasurable_metric_is_inconclusive_never_fail() {
        let s = set(vec![ge("fit.r_squared", 0.8)]);
        assert_eq!(
            evaluate(&s, &json!({ "fit": { "r_squared": "diverged" } })),
            Outcome::Inconclusive
        );

        let nan = serde_json::Number::from_f64(f64::NAN);
        assert!(
            nan.is_none(),
            "JSON cannot carry NaN — guarded upstream too"
        );
    }

    // Rule 4: a definite negative is not rescued by an unknown.
    #[test]
    fn violation_dominates_uncertainty() {
        let s = set(vec![ge("a", 1.0), ge("b", 1.0)]);
        assert_eq!(
            evaluate(&s, &json!({ "a": 0.0 })),
            Outcome::Fail,
            "b absent, a definitely violated"
        );
        assert_eq!(
            evaluate(&s, &json!({ "a": 0.0, "b": "?" })),
            Outcome::Fail,
            "b unmeasurable, a definitely violated"
        );
    }

    #[test]
    fn partially_evaluable_success_is_inconclusive_not_pass() {
        let s = set(vec![ge("a", 1.0), ge("b", 1.0)]);
        assert_eq!(
            evaluate(&s, &json!({ "a": 5.0 })),
            Outcome::Inconclusive,
            "half the bar was never checked, so the bar was not cleared"
        );
    }

    // An experiment without a declared criterion must never read as success.
    #[test]
    fn empty_set_is_not_applicable_not_pass() {
        assert_eq!(
            evaluate(&CriterionSet::default(), &json!({ "anything": 1 })),
            Outcome::NotApplicable
        );
        assert!(CriterionSet::default().is_empty());
    }

    #[test]
    fn comparison_operators_behave_at_the_boundary() {
        let cases = [
            (Comparison::Lt, 1.0, Verdict::Violated),
            (Comparison::Le, 1.0, Verdict::Held),
            (Comparison::Gt, 1.0, Verdict::Violated),
            (Comparison::Ge, 1.0, Verdict::Held),
        ];
        for (op, threshold, expected) in cases {
            let c = Criterion {
                metric: "x".into(),
                op,
                threshold,
                tolerance: None,
            };
            assert_eq!(
                c.verdict(&json!({ "x": 1.0 })),
                expected,
                "{op:?} at equality"
            );
        }
    }

    #[test]
    fn within_uses_tolerance_and_refuses_to_guess_without_one() {
        let ok = Criterion {
            metric: "x".into(),
            op: Comparison::Within,
            threshold: 10.0,
            tolerance: Some(0.5),
        };
        assert_eq!(ok.verdict(&json!({ "x": 10.4 })), Verdict::Held);
        assert_eq!(ok.verdict(&json!({ "x": 11.0 })), Verdict::Violated);

        let missing_tolerance = Criterion {
            tolerance: None,
            ..ok.clone()
        };
        assert_eq!(
            missing_tolerance.verdict(&json!({ "x": 10.0 })),
            Verdict::Unmeasurable,
            "never silently compares against tolerance 0"
        );
    }

    #[test]
    fn validate_rejects_declarations_that_could_not_be_honoured() {
        assert_eq!(
            set(vec![ge("", 1.0)]).validate(),
            Err(CriterionError::EmptyMetric)
        );
        assert_eq!(
            set(vec![ge("fit..r", 1.0)]).validate(),
            Err(CriterionError::EmptyMetric)
        );
        assert_eq!(
            set(vec![ge("x", f64::INFINITY)]).validate(),
            Err(CriterionError::NonFiniteThreshold { metric: "x".into() })
        );
        assert_eq!(
            set(vec![Criterion {
                metric: "x".into(),
                op: Comparison::Within,
                threshold: 1.0,
                tolerance: Some(-1.0),
            }])
            .validate(),
            Err(CriterionError::InvalidTolerance { metric: "x".into() })
        );
        assert_eq!(
            set(vec![ge("x", 1.0), ge("x", 2.0)]).validate(),
            Err(CriterionError::DuplicateMetric { metric: "x".into() })
        );
        assert!(set(vec![ge("x", 1.0), ge("y", 2.0)]).validate().is_ok());
    }

    // K-7 evidence: the frozen declaration hashes stably regardless of how the
    // JSON happened to be ordered on the way in.
    #[test]
    fn digest_is_stable_and_order_independent() {
        let s = set(vec![ge("fit.r_squared", 0.8), ge("fit.iterations", 1.0)]);
        let d = s.digest().expect("hashable");
        assert_eq!(d.len(), 64);
        assert_eq!(d, s.clone().digest().unwrap());

        let from_unordered: CriterionSet = serde_json::from_value(json!({
            "all": [
                { "threshold": 0.8, "metric": "fit.r_squared", "op": "ge" },
                { "op": "ge", "metric": "fit.iterations", "threshold": 1.0 }
            ]
        }))
        .unwrap();
        assert_eq!(from_unordered.digest().unwrap(), d);
    }

    #[test]
    fn serde_uses_the_wire_literals() {
        let s = set(vec![Criterion {
            metric: "x".into(),
            op: Comparison::Within,
            threshold: 1.0,
            tolerance: Some(0.1),
        }]);
        let j = serde_json::to_value(&s).unwrap();
        assert_eq!(j["all"][0]["op"], "within");
        assert_eq!(
            serde_json::to_value(Outcome::NotApplicable).unwrap(),
            "NOT_APPLICABLE"
        );

        // `tolerance` is omitted when unset, so an op that does not use it
        // cannot change the declaration's digest by carrying a null.
        let plain = serde_json::to_value(set(vec![ge("x", 1.0)])).unwrap();
        assert!(plain["all"][0].get("tolerance").is_none());
    }
}
