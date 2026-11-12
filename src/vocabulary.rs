//! Status vocabularies — which word belongs to which axis (ADR D-137).
//!
//! DIU OS speaks three status vocabularies. They answer three different
//! questions about three different objects, and none is a coarser or finer
//! version of another:
//!
//! | Axis | Question | Carried by | Literals |
//! |---|---|---|---|
//! | [`Axis::DeclaredCeiling`] | How strong may a statement built on this be, at most? | gene manifest, run snapshot, project (`epistemic_type`) | 4, D-042: `fact` `hypothesis` `speculation` `unknown` |
//! | [`Axis::ExecutionOutcome`] | Did this execution clear the bar its experiment declared first? | `module_runs.criterion_outcome` | 5, D-131 SVVRE: `PASS` `PARTIAL` `FAIL` `INCONCLUSIVE` `NOT_APPLICABLE` |
//! | [`Axis::ClaimState`] | What has a human assessment of the evidence established about this claim version, in this scope? | `epistemic_commits` (D-135) | 10, ARCH-0001 v1.2.0 §11 |
//!
//! Normative rules (D-137):
//!
//! * **R-1 Disjoint literals.** No literal belongs to two axes. Enforced below,
//!   case-sensitively, so a word on screen names exactly one axis.
//! * **R-2 No conversion.** There is deliberately no function from one axis to
//!   another. `hypothesis` does not become `UNTESTED`; `PASS` does not become
//!   `SUPPORTED`. Crossing an axis is a human assessment (D-135), never a
//!   lookup. Adding such a function requires a new ADR.
//! * **R-3 Absence is not a value.** A claim without a commit has *no* claim
//!   state. It is not `UNTESTED` (ARCH-0001 §12.2). A run without a declared bar
//!   is `NOT_APPLICABLE`, which is an outcome literal, not a claim state.
//! * **R-4 Ceilings bound, they do not grade.** The declared ceiling caps what a
//!   claim may assert (K-3); it is not evidence that the cap was reached.

use crate::criterion::SVVRE_OUTCOMES;
use crate::epistemic::EPISTEMIC_TYPES;

/// The ten canonical claim epistemic states, ARCH-0001 v1.2.0 §11 (book
/// package BASELINE / FROZEN). Historical states (`UNASSESSED`, `REPRODUCED`,
/// `FALSIFIED` as a claim state, …) are not canonical and are refused.
///
/// The production gate (D-135) writes only `SUPPORTED` and `CONTESTED` today;
/// the other eight are vocabulary, not reachable states.
pub const CLAIM_STATES: [&str; 10] = [
    "PROPOSED",
    "UNTESTED",
    "SUPPORTED",
    "WELL_SUPPORTED",
    "REPLICATED",
    "CONTESTED",
    "WEAKENED",
    "REFUTED",
    "SUPERSEDED",
    "INVALID",
];

/// The claim states the production gate can write (D-135). Everything else in
/// [`CLAIM_STATES`] is refused by the gate until its edge has its own ADR.
pub const CLAIM_STATES_WRITABLE: [&str; 2] = ["SUPPORTED", "CONTESTED"];

/// Which status axis a literal belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    /// D-042 epistemic type: the upper bound a statement may claim.
    DeclaredCeiling,
    /// D-131 SVVRE: one execution against the bar declared before it.
    ExecutionOutcome,
    /// ARCH-0001 §11: the assessed state of one claim version in one scope.
    ClaimState,
}

impl Axis {
    /// Every literal of this axis, in canonical order.
    pub fn literals(self) -> &'static [&'static str] {
        match self {
            Axis::DeclaredCeiling => &EPISTEMIC_TYPES,
            Axis::ExecutionOutcome => &SVVRE_OUTCOMES,
            Axis::ClaimState => &CLAIM_STATES,
        }
    }

    /// The object that carries this axis. A literal shown next to any other
    /// object is a display error.
    pub fn carried_by(self) -> &'static str {
        match self {
            Axis::DeclaredCeiling => "gene manifest, run snapshot, project",
            Axis::ExecutionOutcome => "one execution (module_runs.criterion_outcome)",
            Axis::ClaimState => "one claim version in one scope (epistemic_commits)",
        }
    }

    pub const ALL: [Axis; 3] = [
        Axis::DeclaredCeiling,
        Axis::ExecutionOutcome,
        Axis::ClaimState,
    ];
}

/// The axis a literal belongs to, or `None` if it is not canonical on any axis.
/// Case-sensitive by design: `Fact` and `pass` are not words of this system.
pub fn axis_of(literal: &str) -> Option<Axis> {
    Axis::ALL
        .into_iter()
        .find(|axis| axis.literals().contains(&literal))
}

/// True if `literal` is one of the ten canonical claim states.
pub fn is_claim_state(literal: &str) -> bool {
    CLAIM_STATES.contains(&literal)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn r1_axes_are_disjoint() {
        let mut seen: HashSet<&str> = HashSet::new();
        for axis in Axis::ALL {
            for lit in axis.literals() {
                assert!(seen.insert(lit), "`{lit}` appears on two axes (D-137 R-1)");
            }
        }
        assert_eq!(seen.len(), 4 + 5 + 10);
    }

    #[test]
    fn r1_disjoint_even_ignoring_case() {
        // `unknown` (ceiling) vs a hypothetical `UNKNOWN` state is the classic
        // trap; the book keeps UNKNOWN as a representability condition, not a
        // state (D-126). Guard the whole vocabulary case-insensitively.
        let mut seen: HashSet<String> = HashSet::new();
        for axis in Axis::ALL {
            for lit in axis.literals() {
                assert!(
                    seen.insert(lit.to_ascii_lowercase()),
                    "`{lit}` collides case-insensitively"
                );
            }
        }
    }

    #[test]
    fn every_literal_resolves_to_exactly_its_axis() {
        for axis in Axis::ALL {
            for lit in axis.literals() {
                assert_eq!(axis_of(lit), Some(axis));
            }
        }
        assert_eq!(axis_of("Fact"), None);
        assert_eq!(axis_of("pass"), None);
    }

    #[test]
    fn claim_states_are_the_ten_of_arch_0001_v1_2_0() {
        assert_eq!(CLAIM_STATES.len(), 10);
        for historical in [
            "UNASSESSED",
            "PARTIALLY_SUPPORTED",
            "REPRODUCED",
            "FALSIFIED",
            "TESTED",
            "VALIDATED",
            "UNKNOWN",
        ] {
            assert!(!is_claim_state(historical), "{historical} is not canonical");
            assert_eq!(axis_of(historical), None);
        }
    }

    #[test]
    fn writable_states_are_a_subset_and_match_d135() {
        for s in CLAIM_STATES_WRITABLE {
            assert!(is_claim_state(s));
        }
        assert_eq!(CLAIM_STATES_WRITABLE, ["SUPPORTED", "CONTESTED"]);
    }

    #[test]
    fn r3_absence_is_not_untested() {
        // There is no `Option<&str> -> &str` defaulting helper on purpose; the
        // only way to get a claim state is to read one that was written.
        assert_eq!(axis_of(""), None);
    }
}
