//! Does the hierarchical rail actually derive concurrency?
//!
//! `plan_exact_cognitive_workflow_hierarchical` is the only entry point that
//! routes a witnessed plan through `PddlCausalAnalyzer` -> `wf_net_bridge` ->
//! Algorithm 3 -> `convert_and_verify`, so it is the only one that can return a
//! `Powl2Model::PartialOrder` derived from the domain's own causal structure
//! rather than from a schedule someone already committed to.
//!
//! It also had no caller anywhere in the workspace -- not in `src/`, not in a
//! test. Changing its return type broke nothing, which is how that was
//! discovered. So "this rail derives concurrency" was library surface, never a
//! checked claim. These tests check it.
//!
//! The domain is the minimal shape that makes the question decidable: two
//! actions that touch disjoint facts (so a correct derivation must leave them
//! unordered) and one that consumes both (so a correct derivation must order it
//! after each). A rail that simply preserved the witnessed sequence would order
//! `process-a` before `process-b`, and that is exactly what this distinguishes.

#![cfg(feature = "mfw-planner")]

use bcinr_pddl::cognitive::{
    plan_exact_cognitive_workflow_hierarchical, CognitiveProjectionStanding, HierarchicalProjection,
};

const DOMAIN: &str = r#"
(define (domain fan-in)
  (:requirements :strips)
  (:predicates (raw-a) (raw-b) (done-a) (done-b) (complete))
  (:action process-a
    :parameters ()
    :precondition (raw-a)
    :effect (and (done-a) (not (raw-a))))
  (:action process-b
    :parameters ()
    :precondition (raw-b)
    :effect (and (done-b) (not (raw-b))))
  (:action finish
    :parameters ()
    :precondition (and (done-a) (done-b))
    :effect (complete)))
"#;

const PROBLEM: &str = r#"
(define (problem fan-in-1)
  (:domain fan-in)
  (:init (raw-a) (raw-b))
  (:goal (complete)))
"#;

/// The load-bearing claim: independence is derived, not read off the plan order.
///
/// `process-a` and `process-b` share no fact. Nothing in the PDDL says they may
/// run together -- that is the answer the rail is supposed to compute.
#[test]
fn hierarchical_rail_derives_a_partial_order() {
    let projection = plan_exact_cognitive_workflow_hierarchical(DOMAIN, PROBLEM)
        .expect("fan-in domain is pure STRIPS and must plan");

    match &projection {
        HierarchicalProjection::Derived(_) => {}
        HierarchicalProjection::NotDerived(workflow) => panic!(
            "no partial order was derived from a domain whose whole point is that \
             two actions are independent. standing={:?} refusal={:?}",
            workflow.projection_standing, workflow.hierarchical_refusal
        ),
    }

    assert!(
        matches!(
            projection.standing(),
            CognitiveProjectionStanding::CausalHierarchical
        ),
        "Derived must mean CausalHierarchical, got {:?}",
        projection.standing()
    );

    // `CausalHierarchical` only says Algorithm 3 accepted the decomposition. It
    // does not say the two independent actions came out unordered -- asserting
    // the standing alone would be the same vacuous check this rail is meant to
    // replace. The structure is the claim, so read the structure.
    let workflow = projection.into_workflow_ignoring_standing();
    let waves = bcinr_powl::process_toolkit::dispatch_waves(&workflow.model)
        .expect("a derived partial order must yield antichains");

    // Assert by label, not by "some wave has more than one member" -- the
    // recomposition round-trip inserts tau nodes, so a shape-only check could
    // pass on scaffolding rather than on the two actions in question.
    let slot = |label: &str| -> usize {
        let bcinr_powl::powl2::Powl2Model::PartialOrder { children, .. } = &workflow.model else {
            panic!("expected a PartialOrder root, got {:?}", workflow.model);
        };
        children
            .iter()
            .position(|c| matches!(c, bcinr_powl::powl2::Powl2Model::Activity(a) if a == label))
            .unwrap_or_else(|| panic!("{label} missing from the projected model"))
    };
    let wave_of = |slot: usize| -> usize {
        waves
            .iter()
            .position(|w| w.contains(&slot))
            .unwrap_or_else(|| panic!("slot {slot} appears in no antichain: {waves:?}"))
    };

    let (a, b, finish) = (slot("process-a"), slot("process-b"), slot("finish"));

    assert_eq!(
        wave_of(a),
        wave_of(b),
        "process-a and process-b touch disjoint facts, so they must land in one \
         antichain -- the PDDL never says they may run together, that is the \
         answer this rail derives. waves={waves:?}"
    );
    assert!(
        wave_of(finish) > wave_of(a),
        "finish consumes both done-a and done-b, so it must be strictly ordered \
         after them. waves={waves:?}"
    );
}

/// Same fan-in domain, plus one conditional-effect action the plan never uses.
///
/// The concurrency of `process-a` and `process-b` is unchanged by the existence
/// of an action nobody schedules, so the derived partial order must be
/// unchanged too. A domain-level lossiness gate cannot express that: it sees
/// one `when` anywhere and withholds concurrency from every plan over the
/// domain, including plans in which no lossy action appears.
///
/// The per-path signal already exists -- `ground_v2::legacy_action` records the
/// lossy kind per action and `path_to_tape` refuses only when such an action
/// lands on the witnessed path, which `plan_exact_cognitive_workflow_hierarchical`
/// turns into `plan_is_label_only`. So the finer fact is available at exactly
/// the point the coarser one is consulted.
#[test]
fn an_unused_conditional_effect_action_does_not_erase_concurrency() {
    const DOMAIN_WITH_UNUSED_WHEN: &str = r#"
(define (domain fan-in-plus-unused)
  (:requirements :strips :conditional-effects)
  (:predicates (raw-a) (raw-b) (done-a) (done-b) (complete) (armed) (side))
  (:action process-a
    :parameters ()
    :precondition (raw-a)
    :effect (and (done-a) (not (raw-a))))
  (:action process-b
    :parameters ()
    :precondition (raw-b)
    :effect (and (done-b) (not (raw-b))))
  (:action finish
    :parameters ()
    :precondition (and (done-a) (done-b))
    :effect (complete))
  (:action never-scheduled
    :parameters ()
    :precondition (armed)
    :effect (when (armed) (side))))
"#;
    // `armed` is never true, so `never-scheduled` is not applicable and cannot
    // appear on any plan for this goal.
    const PROBLEM_WITH_UNUSED_WHEN: &str = r#"
(define (problem fan-in-plus-unused-1)
  (:domain fan-in-plus-unused)
  (:init (raw-a) (raw-b))
  (:goal (complete)))
"#;

    let projection = plan_exact_cognitive_workflow_hierarchical(
        DOMAIN_WITH_UNUSED_WHEN,
        PROBLEM_WITH_UNUSED_WHEN,
    )
    .expect("the reachable part of the domain is pure STRIPS and must plan");

    let workflow = match projection {
        HierarchicalProjection::Derived(w) => w,
        HierarchicalProjection::NotDerived(w) => panic!(
            "concurrency was withheld because of an action the plan never uses. \
             standing={:?} refusal={:?}",
            w.projection_standing, w.hierarchical_refusal
        ),
    };

    let waves = bcinr_powl::process_toolkit::dispatch_waves(&workflow.model)
        .expect("a derived partial order must yield antichains");
    assert!(
        waves.iter().any(|w| w.len() > 1),
        "process-a and process-b are still independent; waves={waves:?}"
    );
}

/// The guard that must never regress: two actions whose only conflict is
/// through a `when` effect must NOT be reported independent.
///
/// `causal::simulate_two` replays over `preconditions`/`add_effects`/
/// `del_effects` only, so a conditional effect is invisible to it -- both
/// orderings commute and the pair looks independent. Independence claimed on
/// that basis reaches `dispatch_waves` and co-schedules genuinely conflicting
/// actions: correct per the model, wrong per reality.
///
/// Narrowing the lossiness gate to the witnessed path must not weaken this,
/// because both actions here *are* on the path and are therefore still caught.
#[test]
fn actions_conflicting_only_through_a_conditional_effect_are_not_called_independent() {
    const CONFLICT_VIA_WHEN: &str = r#"
(define (domain when-conflict)
  (:requirements :strips :conditional-effects)
  (:predicates (go) (flag) (x-done) (y-done) (both))
  (:action x
    :parameters ()
    :precondition (go)
    :effect (and (x-done) (when (flag) (not (flag)))))
  (:action y
    :parameters ()
    :precondition (go)
    :effect (and (y-done) (when (flag) (both))))
  (:action seal
    :parameters ()
    :precondition (and (x-done) (y-done))
    :effect (not (go))))
"#;
    const CONFLICT_PROBLEM: &str = r#"
(define (problem when-conflict-1)
  (:domain when-conflict)
  (:init (go) (flag))
  (:goal (and (x-done) (y-done))))
"#;

    let projection =
        plan_exact_cognitive_workflow_hierarchical(CONFLICT_VIA_WHEN, CONFLICT_PROBLEM)
            .expect("domain must still plan; the search lowers `when` exactly");

    match projection {
        HierarchicalProjection::NotDerived(w) => assert!(
            matches!(
                w.projection_standing,
                CognitiveProjectionStanding::RefusedLossyEffectModel
            ),
            "expected the lossy-effect refusal, got {:?}",
            w.projection_standing
        ),
        HierarchicalProjection::Derived(w) => {
            let waves = bcinr_powl::process_toolkit::dispatch_waves(&w.model).unwrap();
            panic!(
                "x and y conflict only through a `when` effect the independence \
                 test cannot see, so no partial order may be derived over them. \
                 waves={waves:?}"
            )
        }
    }
}

/// The erasure guard, from the other side: a flat result must not be reachable
/// without naming the arm it came from.
///
/// This is a compile-shaped assertion as much as a runtime one -- the point is
/// that `.powl` is not reachable off the return value directly.
#[test]
fn a_flat_projection_cannot_be_consumed_as_an_ordinary_success() {
    let projection = plan_exact_cognitive_workflow_hierarchical(DOMAIN, PROBLEM)
        .expect("fan-in domain must plan");

    // Reaching the tape at all requires either matching `Derived`, or naming
    // the opt-in. There is no third route.
    let workflow = projection.into_workflow_ignoring_standing();
    assert!(
        !workflow.plan_is_label_only,
        "pure STRIPS must lower exactly, so the exact tape must be the one used"
    );
}
