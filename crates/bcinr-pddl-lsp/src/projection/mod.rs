//! M2: PDDL8 projection — lifecycle facts → domain.pddl + problem.pddl.
//!
//! The generated PDDL8 lifecycle domain encodes the PRD→Publish lifecycle
//! as STRIPS actions. The problem encodes the current lifecycle state as init
//! atoms and `published(project)` as the goal.

use crate::lifecycle::{LifecycleStage, ProjectLifecycle};

/// Generated PDDL8 domain and problem pair.
#[derive(Debug, Clone)]
pub struct Pddl8Projection {
    pub domain_text: String,
    pub problem_text: String,
}

/// Generate the lifecycle domain — fixed set of lifecycle actions.
/// All actions respect PDDL8 bounds (≤8 params, ≤8 preconditions, ≤8 effects).
pub fn emit_domain() -> String {
    r#"(define (domain bcinr-lifecycle)
  (:requirements :strips)
  (:predicates
    (intent_captured ?p)
    (prd_exists ?p)
    (prd_admitted ?p)
    (ard_exists ?p)
    (ard_admitted ?p)
    (work_units_generated ?p)
    (implementation_complete ?p)
    (tests_passed ?p)
    (docs_projected ?p)
    (release_ready ?p)
    (published ?p))

  (:action create_prd
    :parameters (?p)
    :precondition (intent_captured ?p)
    :effect (prd_exists ?p))

  (:action admit_prd
    :parameters (?p)
    :precondition (prd_exists ?p)
    :effect (prd_admitted ?p))

  (:action derive_ard
    :parameters (?p)
    :precondition (prd_admitted ?p)
    :effect (ard_exists ?p))

  (:action admit_ard
    :parameters (?p)
    :precondition (ard_exists ?p)
    :effect (ard_admitted ?p))

  (:action generate_work_units
    :parameters (?p)
    :precondition (ard_admitted ?p)
    :effect (work_units_generated ?p))

  (:action implement_work_units
    :parameters (?p)
    :precondition (work_units_generated ?p)
    :effect (implementation_complete ?p))

  (:action run_tests
    :parameters (?p)
    :precondition (implementation_complete ?p)
    :effect (tests_passed ?p))

  (:action project_docs
    :parameters (?p)
    :precondition (tests_passed ?p)
    :effect (docs_projected ?p))

  (:action prepare_release
    :parameters (?p)
    :precondition (and
      (docs_projected ?p)
      (tests_passed ?p)
      (prd_admitted ?p)
      (ard_admitted ?p))
    :effect (release_ready ?p))

  (:action publish_release
    :parameters (?p)
    :precondition (and
      (prd_admitted ?p)
      (ard_admitted ?p)
      (implementation_complete ?p)
      (tests_passed ?p)
      (docs_projected ?p)
      (release_ready ?p))
    :effect (published ?p))
)
"#.to_string()
}

/// Generate the problem from current lifecycle state.
pub fn emit_problem(lifecycle: &ProjectLifecycle) -> String {
    let name = &lifecycle.project_name;
    // Escape to a PDDL-safe identifier: lowercase, hyphens, must start with letter
    let raw = name.to_lowercase().replace('_', "-").replace(' ', "-").replace('.', "");
    let pddl_name = if raw.starts_with(|c: char| c.is_ascii_alphabetic()) {
        raw
    } else {
        format!("p-{raw}")
    };

    let mut init_atoms: Vec<String> = lifecycle
        .true_stages
        .iter()
        .map(|s| format!("    ({} {})", s.predicate_name(), pddl_name))
        .collect();

    // intent_captured is always true if we're generating a problem
    if !lifecycle.has(&LifecycleStage::IntentCaptured) {
        init_atoms.insert(0, format!("    (intent_captured {})", pddl_name));
    }

    let init_block = if init_atoms.is_empty() {
        format!("    (intent_captured {})", pddl_name)
    } else {
        init_atoms.join("\n")
    };

    format!(
        r#"(define (problem lifecycle-{pddl_name})
  (:domain bcinr-lifecycle)
  (:objects {pddl_name})
  (:init
{init_block})
  (:goal (published {pddl_name}))
)
"#
    )
}

/// Full projection from lifecycle state.
pub fn project(lifecycle: &ProjectLifecycle) -> Pddl8Projection {
    Pddl8Projection {
        domain_text: emit_domain(),
        problem_text: emit_problem(lifecycle),
    }
}
