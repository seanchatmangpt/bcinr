//! M2: PDDL8 projection — lifecycle facts → domain.pddl + problem.pddl.
//!
//! The generated domain includes lifecycle actions AND build-slot coordination.
//! publish_release has exactly 8 preconditions — at the Need9 boundary.
//! Any additional condition would require splitting into prepare + finalize.

use crate::lifecycle::{LifecycleStage, ProjectLifecycle};

/// Generated PDDL8 domain and problem pair.
#[derive(Debug, Clone)]
pub struct Pddl8Projection {
    pub domain_text: String,
    pub problem_text: String,
}

/// Generate the lifecycle + build-slot domain.
///
/// Actions: 11 lifecycle + 6 build coordination = 17 total.
/// Every action: ≤1 parameter, ≤8 preconditions, ≤1 add effect. All bounds satisfied.
/// publish_release has exactly 8 preconditions (at the Need9 limit).
pub fn emit_domain() -> String {
    r#"(define (domain bcinr-lifecycle)
  (:requirements :strips)
  (:predicates
    (intent_captured ?p)
    (prd_exists ?p)
    (prd_admitted ?p)
    (ard_exists ?p)
    (ard_admitted ?p)
    (adr_recorded ?p)
    (work_units_generated ?p)
    (build_slot_available ?p)
    (build_slot_acquired ?p)
    (implementation_complete ?p)
    (heavy_build_complete ?p)
    (tests_passed ?p)
    (ocel_present ?p)
    (docs_projected ?p)
    (release_ready ?p)
    (receipt_present ?p)
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

  (:action record_adr
    :parameters (?p)
    :precondition (ard_admitted ?p)
    :effect (adr_recorded ?p))

  (:action generate_work_units
    :parameters (?p)
    :precondition (and
      (ard_admitted ?p)
      (adr_recorded ?p))
    :effect (work_units_generated ?p))

  (:action request_build_slot
    :parameters (?p)
    :precondition (work_units_generated ?p)
    :effect (build_slot_available ?p))

  (:action acquire_build_slot
    :parameters (?p)
    :precondition (build_slot_available ?p)
    :effect (build_slot_acquired ?p))

  (:action implement_work_units
    :parameters (?p)
    :precondition (and
      (work_units_generated ?p)
      (build_slot_acquired ?p))
    :effect (implementation_complete ?p))

  (:action run_tests
    :parameters (?p)
    :precondition (and
      (implementation_complete ?p)
      (build_slot_acquired ?p))
    :effect (tests_passed ?p))

  (:action record_build_ocel
    :parameters (?p)
    :precondition (tests_passed ?p)
    :effect (ocel_present ?p))

  (:action project_docs
    :parameters (?p)
    :precondition (and
      (tests_passed ?p)
      (implementation_complete ?p))
    :effect (docs_projected ?p))

  (:action prepare_release
    :parameters (?p)
    :precondition (and
      (prd_admitted ?p)
      (ard_admitted ?p)
      (adr_recorded ?p)
      (tests_passed ?p)
      (docs_projected ?p)
      (ocel_present ?p))
    :effect (release_ready ?p))

  (:action emit_receipt
    :parameters (?p)
    :precondition (release_ready ?p)
    :effect (receipt_present ?p))

  (:action publish_release
    :parameters (?p)
    :precondition (and
      (prd_admitted ?p)
      (ard_admitted ?p)
      (implementation_complete ?p)
      (tests_passed ?p)
      (docs_projected ?p)
      (release_ready ?p)
      (receipt_present ?p)
      (ocel_present ?p))
    :effect (published ?p))
)
"#
    .to_string()
}

/// Generate the problem from current lifecycle state.
pub fn emit_problem(lifecycle: &ProjectLifecycle) -> String {
    let pddl_name = sanitize_pddl_identifier(&lifecycle.project_name);

    let mut init_atoms: Vec<String> = lifecycle
        .true_stages
        .iter()
        .map(|s| format!("    ({} {})", s.predicate_name(), pddl_name))
        .collect();

    // intent_captured is always injected if missing (needed for BFS start)
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

/// Sanitize a string to a PDDL-safe identifier:
/// lowercase, hyphens only, must start with ASCII letter.
pub fn sanitize_pddl_identifier(name: &str) -> String {
    let raw = name
        .to_lowercase()
        .replace('_', "-")
        .replace(' ', "-")
        .replace('.', "")
        .replace('/', "-");
    if raw.starts_with(|c: char| c.is_ascii_alphabetic()) {
        raw
    } else {
        format!("p-{raw}")
    }
}

/// Full projection from lifecycle state.
pub fn project(lifecycle: &ProjectLifecycle) -> Pddl8Projection {
    Pddl8Projection {
        domain_text: emit_domain(),
        problem_text: emit_problem(lifecycle),
    }
}
