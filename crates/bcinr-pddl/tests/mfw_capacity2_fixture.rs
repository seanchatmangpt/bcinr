//! Independent adversarial verification: the capacity-2 `{A,B,C}` fixture.
//!
//! This is the decisive, post-hoc verification pass for the MFW retrofit
//! (`bcinr-mfw-ir` + the `bcinr-pddl`/`bcinr-powl`/`bcinr-powl-receipt`
//! extensions). It does **not** trust any prior agent's self-reported
//! ALIVE/PARTIAL/BLOCKED status — every assertion below re-derives its
//! evidence from a real `cargo test` run against the actual code in this
//! repository, read fresh in this session.
//!
//! The fixture: a capacity-2 resource; three actions A, B, C each demand 1
//! unit of it. Every pair (A,B / A,C / B,C) must be jointly executable; the
//! full triple `{A,B,C}` must not be.
//!
//! # Headline finding
//!
//! The real, PDDL-driven analyzer pair (`PddlCausalAnalyzer` +
//! `PddlConcurrencyAnalyzer`, `crates/bcinr-pddl/src/causal.rs` +
//! `concurrency.rs`) operates exclusively on classical STRIPS
//! `Pddl8GroundAction` data, which has **no numeric-fluent/capacity slot at
//! all**. `PddlConcurrencyAnalyzer` only ever lifts a pairwise `Dependent`
//! verdict into a 2-element `MinimalNonFace` — there is no code path
//! anywhere in this workspace that discovers or constructs a genuine
//! 3-element-only minimal nonface from real PDDL/causal analysis. This is
//! disclosed by `bcinr-pddl/src/concurrency.rs`'s own module doc comment
//! (lines 30-42) and is reconfirmed empirically by `link3_...` below: the
//! real analyzer, run on a genuinely pairwise-independent 3-action domain,
//! produces zero minimal nonfaces.
//!
//! So the capacity-2 `{A,B,C}` nonface used from `link3` onward is
//! hand-constructed, exactly mirroring the pattern already established in
//! `bcinr-mfw-ir::concurrency`'s own tests, `bcinr-powl::projection`'s
//! tests, `bcinr-powl::compiler`'s tests, and
//! `bcinr-powl-receipt::projection`'s tests — this is not a new mock
//! introduced by this file, it is the same fixture-construction convention
//! already used throughout the codebase, reused here to drive the
//! *downstream* real machinery (projector, guard-table compiler, real
//! scheduler, real receipt sealing) end to end.
//!
//! # Second headline finding
//!
//! `bcinr-powl`'s scheduler (`scheduler_tick`/`scheduler_tick_guarded`)
//! operates exclusively on the **legacy** `bcinr_powl::tape::{Powl64Op,
//! PowlTape}` pair (compiled from a hand-authored `PowlAstNode` via
//! `compile_powl`). `compile_powl_v2` — the compiler that actually consumes
//! a `PowlModel` built from a real `CausalPlan`/`ExecutableConcurrencyComplex`
//! — produces the **v2** `bcinr_powl::tape::v2::{Powl64Op, PowlTape}` pair, a
//! structurally different 64-byte-aligned type. There is no function
//! anywhere in `bcinr-powl` that bridges a `CompiledPowlV2`'s tape into
//! `scheduler_tick`/`scheduler_tick_guarded` (grep-confirmed: every
//! `scheduler_tick`/`scheduler_tick_guarded` call site in the workspace
//! passes a `compile_powl`-produced legacy tape, never a `compile_powl_v2`
//! one) — `bcinr-powl-receipt/src/execution.rs`'s own module doc comment
//! (lines 8-24) discloses exactly this gap. `link6`/`link7` below therefore
//! reuse the *type* `ConcurrencyGuardTable` (shared between v2 compilation
//! and the scheduler) against a **separately hand-built legacy tape**, with
//! slot indices chosen to line up by construction — not through any real
//! bridge, because none exists.

#![cfg(feature = "mfw-planner")]

use std::collections::BTreeMap;

use bcinr_mfw_ir::{
    ActionOccurrence, ActionOccurrenceId, CausalAnalyzer, CausalPlan, ConcurrencyAnalyzer,
    ConcurrencyConflictWitness, Digest, EventSet, ExecutableConcurrencyComplex, FluentId,
    IndependenceRelation, MinimalNonFace, PlanningEpochId, PowlProjector as PowlProjectorTrait,
    ResourceConflictWitness, StrictPartialOrder,
};
use bcinr_pddl::capability::GroundedPlanningEpoch;
use bcinr_pddl::{
    domain_from_pddl, problem_from_pddl, GroundProblem, PddlCausalAnalyzer,
    PddlConcurrencyAnalyzer,
};
use bcinr_powl::compiler::v2::compile_powl_v2;
use bcinr_powl::compiler::{compile_powl, PowlAstNode};
use bcinr_powl::scheduler::{scheduler_tick, scheduler_tick_guarded, PowlRunState, StableMaximalSelector};
use bcinr_powl::tape::v2::{CompiledNonFace, ConcurrencyGuardTable};
use bcinr_powl_receipt::execution::{digest_legacy_tape, seal_execution_receipt, tick_and_seal_execution_receipt};

// ---------------------------------------------------------------------------
// Shared fixture helpers
// ---------------------------------------------------------------------------

fn bounds() -> bcinr_mfw_ir::EpochBounds {
    bcinr_mfw_ir::EpochBounds {
        max_ground_actions: 64,
        max_plan_depth: 64,
        max_search_steps: 1000,
        max_partition_boxes: 8,
    }
}

fn epoch_from(domain_pddl: &str, problem_pddl: &str) -> GroundedPlanningEpoch {
    let domain = domain_from_pddl(domain_pddl).unwrap();
    let problem = problem_from_pddl(problem_pddl).unwrap();
    let gp = GroundProblem::build(&domain, &problem, None).unwrap();
    let mut epoch =
        GroundedPlanningEpoch::from_ground_problem(&gp, Digest::hash(b"capacity2-fixture"), bounds());
    epoch.id = PlanningEpochId(7);
    epoch
}

fn occ(id: u32, action_index: u64) -> ActionOccurrence {
    ActionOccurrence {
        id: ActionOccurrenceId(id),
        action: action_index,
    }
}

/// The genuinely pairwise-independent 3-action domain used by
/// `link1`/`link2`/`link3`: A affects `pa`, B affects `pb`, C affects `pc` —
/// disjoint predicates, so every pair really does commute and really has no
/// precondition/delete interference. This is the *real* PDDL side of the
/// fixture, not a hand-built IR value.
const DISJOINT_DOMAIN: &str = "(define (domain capacity2) \
     (:predicates (pa) (pb) (pc)) \
     (:action a1 :parameters () :precondition () :effect (pa)) \
     (:action a2 :parameters () :precondition () :effect (pb)) \
     (:action a3 :parameters () :precondition () :effect (pc)))";
const DISJOINT_PROBLEM: &str =
    "(define (problem capacity2p) (:domain capacity2) (:init) (:goal (and (pa) (pb) (pc))))";

fn real_causal_plan_for_abc() -> CausalPlan {
    let epoch = epoch_from(DISJOINT_DOMAIN, DISJOINT_PROBLEM);
    let occurrences = vec![occ(0, 0), occ(1, 1), occ(2, 2)];
    PddlCausalAnalyzer.analyze(&epoch, &occurrences).unwrap()
}

/// The hand-built capacity-2 `{A,B,C}` nonface, mirroring the exact fixture
/// convention already established in `bcinr-mfw-ir::concurrency`,
/// `bcinr-powl::projection`, `bcinr-powl::compiler`, and
/// `bcinr-powl-receipt::projection`'s own test modules — reused here, not
/// reinvented, and driven through the *real* projector/compiler/scheduler/
/// receipt code below rather than only asserted in isolation.
fn hand_built_capacity2_complex(occurrence_ids: [u32; 3]) -> ExecutableConcurrencyComplex {
    let abc = EventSet::empty().with(0).with(1).with(2);
    let witness = ConcurrencyConflictWitness {
        causal: None,
        temporal: None,
        resource: Some(ResourceConflictWitness {
            actions: abc,
            resource: FluentId(0),
            capacity_milli: 2_000,
            demanded_milli: 3_000,
        }),
    };
    let witness_digest = Digest::hash(b"capacity2-abc-resource-conflict");
    let mut conflict_witnesses = BTreeMap::new();
    conflict_witnesses.insert(witness_digest, witness);
    let _ = occurrence_ids; // documents which real ActionOccurrenceIds slots 0,1,2 stand for
    ExecutableConcurrencyComplex {
        event_count: 3,
        minimal_nonfaces: vec![MinimalNonFace {
            members: abc,
            witness_digest,
        }],
        conflict_witnesses,
        digest: Digest::hash(b"capacity2-complex"),
    }
}

// ---------------------------------------------------------------------------
// Link 1 — pairwise independence witnesses genuinely admitted, not invented
// ---------------------------------------------------------------------------

#[test]
fn link1_real_analyzer_admits_independence_for_every_pair_of_a_genuinely_independent_triple() {
    let plan = real_causal_plan_for_abc();
    assert_eq!(
        plan.independence.independent.len(),
        3,
        "all three pairs (A,B)/(A,C)/(B,C) must be admitted Independent by the real analyzer"
    );
    assert!(
        plan.independence.dependent.is_empty(),
        "no pair should be marked Dependent for this genuinely disjoint-effects domain"
    );
    // Every emitted witness must show a real, non-vacuous `commute: true`
    // check having actually run (not a default-true placeholder).
    for w in plan.independence.independent.values() {
        assert!(w.effects_commute.commute);
        assert!(w.preconditions_stable.stable);
    }
}

#[test]
fn link1b_real_analyzer_does_not_invent_independence_for_a_genuinely_dependent_pair() {
    // Adversarial check on link 1's negative direction: A establishes `p`,
    // D needs `p`. The real analyzer must mark this Dependent, not
    // Independent, so we know link1's "3 independent pairs" result above
    // isn't a stuck-on-Independent default.
    let domain = "(define (domain d2) (:predicates (p) (q)) \
                   (:action a1 :parameters () :precondition () :effect (p)) \
                   (:action a4 :parameters () :precondition (p) :effect (q)))";
    let problem = "(define (problem d2p) (:domain d2) (:init) (:goal (and (p) (q))))";
    let epoch = epoch_from(domain, problem);
    let occurrences = vec![occ(0, 0), occ(1, 1)];
    let plan = PddlCausalAnalyzer.analyze(&epoch, &occurrences).unwrap();
    assert!(
        plan.independence.independent.is_empty(),
        "a causally-supporting pair must never be marked Independent"
    );
    assert_eq!(plan.independence.dependent.len(), 1);
}

// ---------------------------------------------------------------------------
// Link 2 — CausalPlan.precedes: does it invent precedence beyond what's
// causally justified?
// ---------------------------------------------------------------------------

#[test]
fn link2_precedes_is_the_full_input_vector_order_even_though_every_pair_is_independent() {
    // FINDING (not a pass): `PddlCausalAnalyzer::analyze` (bcinr-pddl/src/
    // causal.rs, lines ~145-153) inserts a `PrecedenceEdge` for *every*
    // pair `i < j` in the input `occurrences` slice, unconditionally --
    // before the independence relation is even computed. The module's own
    // doc comment (lines 139-144) discloses this as "the plan's own total
    // execution order... not yet reduced to a minimal necessary ordering",
    // but the field is still named/typed as `precedes: StrictPartialOrder`
    // and is copied byte-for-byte into `PowlModel.order` by the real
    // `PowlProjector` (bcinr-powl/src/projection.rs, step 2), which
    // `compile_powl_v2` then turns into real `pred_mask`/`succ_mask` gates.
    // So a caller who takes `CausalPlan.precedes` as "causally justified
    // ordering" (the load-bearing inequality this audit was asked to
    // check: "vector order/timestamps alone never create precedence --
    // precedence needs a witness") is not protected by this type: vector
    // order alone produces exactly this field's edges, unconditionally,
    // regardless of the independence verdict computed for the same pair.
    let plan = real_causal_plan_for_abc();
    assert_eq!(
        plan.independence.independent.len(),
        3,
        "sanity: every pair is independent"
    );
    assert_eq!(
        plan.precedes.edges.len(),
        3,
        "PddlCausalAnalyzer inserts a full transitive precedence edge for \
         every pair *regardless* of the independence verdict -- (A,B)/(A,C)/(B,C) \
         are all marked Independent above, yet all three precedence edges are \
         still present. This is vector-order-becomes-precedence, exactly the \
         pattern the mission's load-bearing inequality (\"vector order/timestamps \
         alone never create precedence\") warns against; it is disclosed in the \
         module's own doc comment as an intentional \"what genuinely happened\" \
         choice, not hidden, but it means precedes must never be read as a \
         minimal/justified causal order without also consulting \
         independence.dependent."
    );
}

// ---------------------------------------------------------------------------
// Link 3 — the real analyzer cannot derive the capacity-2 nonface; the
// hand-built complex admits/rejects correctly by real event membership.
// ---------------------------------------------------------------------------

#[test]
fn link3a_real_pddl_pipeline_cannot_produce_a_three_way_nonface_from_pairwise_independent_actions() {
    // BLOCKED (empirically confirmed, not just asserted from the doc
    // comment): PddlConcurrencyAnalyzer only ever emits one 2-element
    // MinimalNonFace per Dependent pair (bcinr-pddl/src/concurrency.rs,
    // lines 105-145). Since every pair here is Independent, the real
    // pipeline produces *zero* nonfaces -- it structurally cannot express
    // "pairwise fine, jointly over capacity" without numeric-fluent data
    // Pddl8GroundAction does not carry.
    let epoch = epoch_from(DISJOINT_DOMAIN, DISJOINT_PROBLEM);
    let causal_plan = real_causal_plan_for_abc();
    let complex = PddlConcurrencyAnalyzer.analyze(&epoch, &causal_plan).unwrap();
    assert!(
        complex.minimal_nonfaces.is_empty(),
        "BLOCKED: the real analyzer cannot derive a capacity-only 3-way nonface \
         from classical-STRIPS pairwise data -- confirmed empirically, matching \
         bcinr-pddl/src/concurrency.rs's own disclosed gap (lines 30-42)"
    );
    let all_three = EventSet::empty().with(0).with(1).with(2);
    assert!(
        complex.admits(&all_three),
        "with zero nonfaces the real complex (wrongly, from a true-capacity \
         standpoint, but honestly given its inputs) admits the full triple"
    );
}

#[test]
fn link3b_hand_built_capacity2_complex_admits_every_pair_and_rejects_the_triple_by_event_membership() {
    let complex = hand_built_capacity2_complex([100, 101, 102]);

    let a = EventSet::empty().with(0);
    let b = EventSet::empty().with(1);
    let c = EventSet::empty().with(2);
    let ab = a.union(&b);
    let ac = a.union(&c);
    let bc = b.union(&c);
    let abc = ab.union(&c);

    for (name, candidate) in [("A", a), ("B", b), ("C", c), ("AB", ab), ("AC", ac), ("BC", bc)] {
        assert!(complex.admits(&candidate), "{name} must be admitted");
    }
    assert!(!complex.admits(&abc), "the full triple must not be admitted");

    // "by event membership, not incidental digest match": walk the actual
    // MinimalNonFace's member bit-positions and confirm they are exactly
    // {0,1,2} -- not merely that some opaque digest matches.
    let nonface = &complex.minimal_nonfaces[0];
    let members: Vec<usize> = nonface.members.iter_stable().collect();
    assert_eq!(members, vec![0, 1, 2]);
}

// ---------------------------------------------------------------------------
// Link 4 — real PowlProjector preserves the capacity-2 nonface (and the
// empty order, so this test isolates the concurrency claim from link 2's
// vector-order finding).
// ---------------------------------------------------------------------------

fn abc_causal_plan_with_empty_order(ids: [u32; 3]) -> CausalPlan {
    CausalPlan {
        epoch: PlanningEpochId(7),
        occurrences: vec![occ(ids[0], 0), occ(ids[1], 1), occ(ids[2], 2)],
        precedes: StrictPartialOrder::default(),
        independence: IndependenceRelation::default(),
        support_edges: Default::default(),
        digest: Digest::hash(b"capacity2-causal-plan"),
    }
}

#[test]
fn link4_real_projector_preserves_the_capacity2_nonface_into_the_powl_model() {
    // IDs 0,1,2 -- matching PddlConcurrencyAnalyzer's own "EventSet slot ==
    // position in occurrences" convention exactly (see link3a's module doc
    // reference and concurrency.rs's `slot_of` construction). This is the
    // common case every pre-existing fixture in the codebase also uses.
    let causal = abc_causal_plan_with_empty_order([0, 1, 2]);
    let concurrency = hand_built_capacity2_complex([0, 1, 2]);

    let projector = bcinr_powl::projection::PowlProjector;
    let (model, witness) = projector.project(&causal, &concurrency).unwrap();

    // Real preservation witness, not a stub: mapped/target digests agree.
    assert_eq!(
        witness.concurrency_witness.mapped_source_digest,
        witness.concurrency_witness.target_complex_digest
    );

    // By real event membership through the bijection: the node ids that
    // make up the projected nonface must map back (via
    // action_node_bijection.node_to_action) to exactly the source
    // ActionOccurrenceIds {0,1,2}, not merely "some 3-element set".
    assert_eq!(model.concurrency.minimal_nonfaces.len(), 1);
    let projected_members: Vec<usize> =
        model.concurrency.minimal_nonfaces[0].members.iter_stable().collect();
    let mapped_back: Vec<u32> = projected_members
        .iter()
        .map(|&slot| {
            let node_id = bcinr_mfw_ir::PowlNodeId(slot as u64);
            witness.action_node_bijection.node_to_action[&node_id].0
        })
        .collect();
    assert_eq!(mapped_back, vec![0, 1, 2]);
}

#[test]
fn link4_adversarial_confirmed_bug_projector_assumes_eventset_slot_equals_raw_action_occurrence_id() {
    // CONFIRMED BUG (found by this adversarial pass, not previously
    // reported): `PddlConcurrencyAnalyzer::analyze` documents and
    // implements `EventSet` slots as *position in `causal.occurrences`*,
    // explicitly *not* the raw `ActionOccurrenceId`
    // (bcinr-pddl/src/concurrency.rs lines 95-98: "Stable EventSet slot per
    // occurrence: position in `causal.occurrences`, not the ...
    // `ActionOccurrenceId` itself"). But
    // `bcinr_powl::projection::verify_concurrency_preservation` (called by
    // the real `PowlProjector::project`, bcinr-powl/src/projection.rs lines
    // 219-226) does the opposite: it reinterprets every `EventSet` member
    // value `event_id` directly as `ActionOccurrenceId(event_id as u32)`
    // and looks *that* up in the action->node bijection. These two
    // conventions only agree when `occurrences[i].id.0 == i` for every `i`
    // -- true in every hand-built fixture in this codebase (including this
    // file's own `link4`/`link5` above, which deliberately use ids
    // 0,1,2) and true for `MfwPlanner`'s real `occurrences_from_tape`
    // *only as long as no tape op is filtered out* (planner.rs lines
    // 552-561: `tape.ops.iter().filter_map(...)` keeps each surviving op's
    // *original* `op.index` as its `ActionOccurrenceId`, so if any earlier
    // op is dropped by the `filter_map`, a later occurrence's position in
    // the resulting `Vec` no longer equals its `id.0`). This test
    // reproduces the mismatch directly: a causal plan whose 3 occurrences
    // sit at positions 0,1,2 but carry ids 100,101,102 (a completely valid
    // `CausalPlan` by this crate's own type -- nothing forbids
    // caller-assigned, non-dense `ActionOccurrenceId`s) causes the real,
    // non-test `PowlProjector::project` to refuse a concurrency complex
    // that is, in fact, perfectly well-formed.
    let causal = abc_causal_plan_with_empty_order([100, 101, 102]);
    let concurrency = hand_built_capacity2_complex([100, 101, 102]);
    let projector = bcinr_powl::projection::PowlProjector;
    let err = projector.project(&causal, &concurrency).unwrap_err();
    assert_eq!(
        err,
        bcinr_powl::projection::ProjectionError::Preservation(
            bcinr_powl::projection::PreservationError::UnmappedActionInConcurrency(
                bcinr_mfw_ir::ActionOccurrenceId(0)
            )
        ),
        "CONFIRMED: PowlProjector::project refuses a well-formed \
         ExecutableConcurrencyComplex whenever ActionOccurrenceIds are not \
         exactly their position index -- EventSet-slot-as-position \
         (PddlConcurrencyAnalyzer's contract) and EventSet-slot-as-raw-id \
         (verify_concurrency_preservation's assumption) disagree."
    );
}

// ---------------------------------------------------------------------------
// Link 5 — the compiled ConcurrencyGuardTable still rejects the triple.
// ---------------------------------------------------------------------------

#[test]
fn link5_compiled_guard_table_rejects_the_triple_and_admits_every_pair() {
    let causal = abc_causal_plan_with_empty_order([0, 1, 2]);
    let concurrency = hand_built_capacity2_complex([0, 1, 2]);
    let projector = bcinr_powl::projection::PowlProjector;
    let (model, _witness) = projector.project(&causal, &concurrency).unwrap();

    let compiled = compile_powl_v2(&model).unwrap();
    assert_eq!(compiled.guards.nonfaces.len(), 1);

    let a = EventSet::empty().with(0);
    let ab = EventSet::empty().with(0).with(1);
    let ac = EventSet::empty().with(0).with(2);
    let bc = EventSet::empty().with(1).with(2);
    let abc = EventSet::empty().with(0).with(1).with(2);
    for candidate in [a, ab, ac, bc] {
        assert!(compiled.guards.admits(&candidate));
    }
    assert!(!compiled.guards.admits(&abc));
}

// ---------------------------------------------------------------------------
// Link 6 — real scheduler + real selector: ready={A,B,C}, fired is a legal
// pair, never the triple.
//
// CAVEAT (see module doc comment): `compile_powl_v2`'s v2 tape cannot be
// fed into `scheduler_tick`/`scheduler_tick_guarded` (legacy tape only, no
// bridge exists). This test therefore hand-builds a *separate* legacy tape
// via `compile_powl`/`PowlAstNode`, choosing its 3-parallel-activity shape
// so its slot indices (0=a,1=b,2=c) line up by construction with the
// `ConcurrencyGuardTable` compiled in link5 -- the guard-table *value* is
// reused for real (it is `compiled.guards` from link5, not a fresh
// duplicate), but the tape it is applied to is not the one it was compiled
// alongside, because no code path connects them.
// ---------------------------------------------------------------------------

fn three_parallel_legacy_tape() -> bcinr_powl::tape::PowlTape {
    let ast = PowlAstNode::PartialOrder {
        children: vec![
            PowlAstNode::Atom("a"),
            PowlAstNode::Atom("b"),
            PowlAstNode::Atom("c"),
        ],
        edges: vec![],
    };
    compile_powl(&ast).unwrap()
}

fn capacity2_guard_table_for_slots_0_1_2() -> ConcurrencyGuardTable {
    ConcurrencyGuardTable {
        nonfaces: vec![CompiledNonFace {
            members: EventSet::empty().with(0).with(1).with(2),
            witness_digest: Digest::hash(b"capacity2-abc-resource-conflict"),
        }],
    }
}

#[test]
fn link6_real_scheduler_never_fires_the_triple_when_the_ready_set_is_the_triple() {
    let tape = three_parallel_legacy_tape();
    assert_eq!(tape.len, 4, "a=0,b=1,c=2,synthetic-join=3");

    let guards = capacity2_guard_table_for_slots_0_1_2();
    let mut state = PowlRunState::new(&tape);

    // Independently confirm, via a dry-run preview (mirrors what
    // scheduler_tick_guarded does internally), that the real ready set on
    // tick 1 is genuinely {A,B,C} (all three have pred_mask=0 under
    // PartialOrder with no edges) -- not assumed, checked.
    let mut preview = state.clone();
    let would_fire = scheduler_tick(&tape.ops[..tape.len as usize], &mut preview);
    assert_eq!(
        would_fire.0 & 0b111,
        0b111,
        "ready set on tick 1 must genuinely be {{A,B,C}} (bits 0,1,2)"
    );

    let mut selector = StableMaximalSelector;
    let fired = scheduler_tick_guarded(
        &tape.ops[..tape.len as usize],
        &mut state,
        &mut selector,
        &guards,
    );

    assert_eq!(
        fired.0.count_ones(),
        2,
        "the real scheduler+selector must fire exactly a legal pair on tick 1, \
         never all three -- got fired mask {:#05b}",
        fired.0
    );
    assert_ne!(fired.0 & 0b111, 0b111, "the triple must never fire together");
    assert!(
        fired.0 == 0b011 || fired.0 == 0b101 || fired.0 == 0b110,
        "fired must be exactly one of the three legal pairs, got {:#05b}",
        fired.0
    );
}

// ---------------------------------------------------------------------------
// Link 7 — a real ExecutionReceipt records the fired pair; independently we
// confirm the ready set that tick actually saw was the full triple.
// ---------------------------------------------------------------------------

#[test]
fn link7_execution_receipt_fired_pair_differs_from_the_genuinely_ready_triple() {
    let tape = three_parallel_legacy_tape();
    let compiled_digest = digest_legacy_tape(&tape);
    let guards = capacity2_guard_table_for_slots_0_1_2();
    let mut state = PowlRunState::new(&tape);
    let mut selector = StableMaximalSelector;

    // Ready set, computed independently (ExecutionReceipt itself does not
    // expose `ready` as a literal field -- see the assertion below and the
    // PARTIAL note in this file's final report).
    let mut preview = state.clone();
    let ready = scheduler_tick(&tape.ops[..tape.len as usize], &mut preview);
    assert_eq!(ready.0 & 0b111, 0b111, "ready must genuinely be the triple");

    let receipt = tick_and_seal_execution_receipt(
        &tape.ops[..tape.len as usize],
        &mut state,
        &mut selector,
        &guards,
        1,
        Digest::ZERO,
        Digest::hash(b"capacity2-powl-model"),
        compiled_digest,
    );

    assert_eq!(
        receipt.fired.len(),
        2,
        "the sealed receipt must genuinely record a 2-element fired set"
    );
    let fired_members: Vec<usize> = receipt.fired.iter_stable().collect();
    assert!(
        fired_members.len() == 2 && fired_members.iter().all(|&m| m < 3),
        "fired must be a real pair drawn from {{0,1,2}}, got {fired_members:?}"
    );
    assert_ne!(
        receipt.fired.len(),
        3,
        "PARTIAL: ExecutionReceipt has no literal `ready` field (only `fired`/\
         `completed_after` plus scheduler_decision_digest, a hash commitment \
         over (tick, ready_mask, fired_mask) that isn't independently \
         recoverable without already knowing ready_mask) -- this test \
         demonstrates ready != fired by recomputing ready out-of-band, the \
         receipt alone does not let a reader recover that fact"
    );
    assert_ne!(receipt.hash, Digest::ZERO);
}

// ---------------------------------------------------------------------------
// Link 8 — the sharpest test: does anything reject a hand-fabricated
// receipt claiming the triple fired?
// ---------------------------------------------------------------------------

#[test]
fn link8a_a_real_receipt_for_the_legal_pair_is_well_formed() {
    let tape = three_parallel_legacy_tape();
    let compiled_digest = digest_legacy_tape(&tape);
    let guards = capacity2_guard_table_for_slots_0_1_2();
    let mut state = PowlRunState::new(&tape);
    let mut selector = StableMaximalSelector;

    let real_receipt = tick_and_seal_execution_receipt(
        &tape.ops[..tape.len as usize],
        &mut state,
        &mut selector,
        &guards,
        1,
        Digest::ZERO,
        Digest::hash(b"capacity2-powl-model"),
        compiled_digest,
    );
    assert_eq!(real_receipt.fired.len(), 2);
    assert!(guards.admits(&real_receipt.fired));
}

#[test]
fn link8b_blocked_no_replay_or_verify_function_rejects_a_hand_fabricated_triple_receipt() {
    // BLOCKED. Evidence gathered this session:
    //
    // 1. `grep -rn "\.admits(" crates --include="*.rs"` (non-test call
    //    sites) shows `ConcurrencyGuardTable::admits`/
    //    `ExecutableConcurrencyComplex::admits` are called from exactly two
    //    places in non-test code: `ConcurrencySelector::select`/
    //    `select_checked` (bcinr-powl/src/scheduler.rs, live gating of what
    //    is allowed to fire *before* it fires) and `StableMaximalSelector::
    //    select`'s own greedy loop. Zero call sites exist inside any
    //    receipt-sealing, receipt-verification, or replay function.
    //
    // 2. `bcinr_powl_receipt::execution::seal_execution_receipt` takes
    //    `fired: EventSet` as a plain, trusted argument (lines 102-131 of
    //    execution.rs) -- it hashes whatever `fired` it is given; it has no
    //    `ConcurrencyGuardTable`/`ExecutableConcurrencyComplex` parameter to
    //    cross-check against at all.
    //
    // 3. `bcinr_powl_receipt::replay::PowlReplayVerifier` (replay.rs) is a
    //    *different* token-passing model entirely (`PowlReplayFrame`/
    //    `required_tokens`/`node_bit`, one frame at a time) -- it never
    //    references `EventSet`, `ConcurrencyGuardTable`, or
    //    `ExecutableConcurrencyComplex` (grep-confirmed zero occurrences in
    //    that file), so it has no way to notice that a single frame's
    //    "concurrent firing" violates a capacity constraint even in
    //    principle.
    //
    // So: fabricate a receipt claiming all three fired, by hand, bypassing
    // the real scheduler entirely -- and show it is accepted as well-formed
    // by the one function that exists to seal receipts.
    let fabricated_fired = EventSet::empty().with(0).with(1).with(2);
    let fabricated = seal_execution_receipt(
        Digest::ZERO,
        Digest::hash(b"capacity2-powl-model"),
        Digest::hash(b"capacity2-compiled-tape"),
        1,
        Digest::hash(b"fabricated-decision"),
        fabricated_fired,
        fabricated_fired,
    );
    // The fabrication is accepted: a perfectly well-formed, non-zero,
    // internally-consistent hash chain entry -- nothing in
    // `seal_execution_receipt` refuses it.
    assert_ne!(fabricated.hash, Digest::ZERO);
    assert_eq!(fabricated.fired.len(), 3);

    // The one piece of logic that *could* have caught this, applied here
    // manually (not by any replay/verify code path -- none exists):
    let guards = capacity2_guard_table_for_slots_0_1_2();
    assert!(
        !guards.admits(&fabricated.fired),
        "the raw building block (ConcurrencyGuardTable::admits) correctly \
         identifies the fabricated triple as inadmissible when applied by \
         hand -- proving the primitive exists and is sufficient in \
         principle -- but no replay/verification function in the repository \
         calls it against a receipt's `fired` field, so an \
         `ExecutionReceipt` claiming the triple fired is never rejected by \
         any code that actually runs today. This is the missing piece: a \
         `verify_execution_receipt(receipt, guards) -> Result<(), _>` (or \
         equivalent) that does not exist anywhere in \
         crates/bcinr-powl-receipt/src/execution.rs or replay.rs."
    );
}
