//! `ECOSYSTEM-JOIN-001`, Rail B (`~/bcinr`) only.
//!
//! This file is `~/bcinr`'s committed handoff artifact for the shared
//! cross-repo checkpoint the user split across three independent sessions.
//! Per the user's own ownership rules, this session owns CMCA runtime
//! profile, action-mass correlation, allocation artifact, capacity
//! selector, and allocation/execution receipt -- entirely inside
//! `~/bcinr`, and must not edit `~/mfw`.
//!
//! # What this file does NOT claim
//!
//! `~/mfw/mfw-theory/MFW/CMCA/Semantics/` was read directly this session:
//! no `CMCA-Reference-v0.1` manifest exists yet -- Rail A (the `~/mfw`
//! math session) has not produced it. This file does not fabricate one.
//! `lean_manifest_digest` below is an opaque, real [`Digest`] this session
//! constructs for its own falsifier (proving the *binding* refuses on
//! mismatch), not a claim that it corresponds to any real, frozen Lean
//! artifact -- that correspondence is Rail C's (the `~/mfw` code session)
//! job, wiring a real digest through once Rail A lands. Nor does this file
//! touch the PCP digest bridge, certificate-gated `solve_rdf`, the POWL
//! distinction ledger, or OCEL/RDF admission -- all Rail C, all a
//! different repository.
//!
//! # The `ticket-15` fixture
//!
//! Three independent preparation activities followed by a join:
//! ```text
//! prepare-low(1) ─┐
//! prepare-medium(2) ─┼─→ assemble
//! prepare-high(10) ─┘
//! ```
//! `assemble`'s precondition requires all three preparation facts -- a
//! real causal-`Dependent` relationship to each preparation action (not a
//! synthetic join marker), so `PddlCausalAnalyzerV2`/`compile_powl_v2`
//! (`BCINR-SCHED-001`'s own established mechanism) already order it after
//! all three via real `pred_mask` precedence edges. No new scheduler
//! mechanism is introduced by this fixture -- it is the first real,
//! multi-tick-then-join proof that `BCINR-SCHED-002`/`BCINR-CMCA-E`'s
//! existing `CapacityBoundedSelector`/`PriorityCapacitySelector` machinery
//! composes correctly with a genuine causal join, not just with
//! independent actions (`BCINR-CMCA-F`/`G`'s fixtures) or bare capacity
//! scarcity (`BCINR-SCHED-002`'s own fixture).
//!
//! Lens `exploit2` (Rust `q = 2`) on masses `[1, 2, 10]`: the exact
//! ratio is `[1^2, 2^2, 10^2] / (1+4+100) = [1/105, 4/105, 100/105]`,
//! ordering `prepare-high(100/105) > prepare-medium(4/105) >
//! prepare-low(1/105)` -- computed here by the same real
//! `allocate_pddl_powl_plan` cascade `BCINR-CMCA-F`/`G` already proved
//! correct, not re-derived.

#![cfg(feature = "mfw-planner")]

use std::collections::BTreeMap;

use bcinr_cmca::fixed::NonNegativeFixed;
use bcinr_mfw_ir::Digest;
use bcinr_pddl::cmca_execution::{
    allocate_pddl_powl_plan, verify_cmca_execution, AllocationSemantics, CmcaAllocationRefusal,
    CmcaExecutionProfile, CmcaExecutionRequest, LensSchedule, ProcessMassField, ProfileIdentity,
};
use bcinr_pddl::production::{PddlPowlConfig, PddlPowlPlan, PddlPowlRuntime};

const DOMAIN: &str = "(define (domain ticket15)
    (:predicates (prepared-low) (prepared-medium) (prepared-high) (assembled))
    (:action prepare-low :parameters () :precondition () :effect (prepared-low))
    (:action prepare-medium :parameters () :precondition () :effect (prepared-medium))
    (:action prepare-high :parameters () :precondition () :effect (prepared-high))
    (:action assemble :parameters ()
        :precondition (and (prepared-low) (prepared-medium) (prepared-high))
        :effect (assembled)))";
const PROBLEM: &str = "(define (problem ticket15p) (:domain ticket15) (:init)
    (:goal (assembled)))";

fn plan() -> PddlPowlPlan {
    PddlPowlRuntime::new(PddlPowlConfig::default())
        .plan(DOMAIN, PROBLEM)
        .expect("3 independent preparations + a join must plan")
}

fn masses(low: u32, medium: u32, high: u32) -> ProcessMassField {
    // `from_num`, not `from_bits`: these are meant to be the literal
    // integer masses [1, 2, 10] the user's spec names, not raw Q16.16 bit
    // patterns. Using `from_bits` here was an earlier real mistake this
    // fixture caught the hard way: at lens=2 (exploit2, which squares
    // each mass), `from_bits(1)` is `1/65536` in real value, and squaring
    // it underflows to zero in Q16.16 fixed point -- every allocation
    // refused with `EscortUnderflow` until this was fixed.
    let mut m = BTreeMap::new();
    m.insert("prepare-low".to_string(), NonNegativeFixed::from_num(low));
    m.insert(
        "prepare-medium".to_string(),
        NonNegativeFixed::from_num(medium),
    );
    m.insert("prepare-high".to_string(), NonNegativeFixed::from_num(high));
    // `assemble` is a production action too (its own tape slot), so
    // BCINR-CMCA-G's completeness law requires it to carry exactly one
    // admitted mass as well -- confirmed the hard way: an earlier draft
    // of this fixture omitted it and every test refused with
    // MissingActionMass { action: "assemble" }. Its exact value is
    // irrelevant to this fixture's tick-order claims (it is the sole
    // ready op in tick three regardless of capacity), so it gets an
    // arbitrary but real, nonzero mass.
    m.insert("assemble".to_string(), NonNegativeFixed::from_num(1));
    ProcessMassField(m)
}

fn request(
    capacity: u32,
    masses: ProcessMassField,
    manifest_digest: Digest,
) -> CmcaExecutionRequest {
    CmcaExecutionRequest {
        profile: CmcaExecutionProfile {
            identity: ProfileIdentity("CMCA-Reference-v0.1".to_string()),
            lens_schedule: LensSchedule(vec![2]), // exploit2, q = 2
            allocation_semantics: AllocationSemantics::UniformSiblingCoverageQ0,
            lean_manifest_digest: manifest_digest,
        },
        capacity,
        masses,
    }
}

/// Rail A's real, committed handoff: `~/mfw/docs/proof-status.md`'s
/// `JOIN-MATH (ECOSYSTEM-JOIN-001, Rail A)` row records this exact BLAKE3
/// digest, computed over `CorrespondenceManifest.lean` at commit
/// `b6a24bf45192b4a49a3cfa0b0536a0e2448a22ce` (cross-checked against
/// `sha256:2ac0c4117e760b4431ce7148204c45859ae751a99a56bdd015d49790a95d0fbd`
/// there). The hex string below is parsed at runtime rather than
/// hand-split into a byte literal array (fewer places a transcription
/// error can hide), and is not trusted blindly: the
/// `rail_a_manifest_digest_matches_the_live_proof_status_md` test below
/// reads `~/mfw/docs/proof-status.md` directly and fails loudly if this
/// constant has drifted from the real, current file, rather than letting
/// a stale copy sit unnoticed. Rail A is `ALIVE` as of that commit:
/// `CorrespondenceManifest.lean` names profile identity
/// `CMCA-Reference-v0.1`, and `Escort.lean` gained the standalone
/// `escort_exploit2_1_2_10`/`escort_exploit2_1_2_10_strict_order`
/// theorems this fixture's masses/lens/expected order correspond to.
const RAIL_A_MANIFEST_DIGEST_HEX: &str =
    "f068090c8df550a43b979ba93af07b18938d3e04ee604cdda8086b297a268d20";

fn rail_a_handoff_digest() -> Digest {
    let mut bytes = [0u8; 32];
    for (i, byte) in bytes.iter_mut().enumerate() {
        let pair = &RAIL_A_MANIFEST_DIGEST_HEX[i * 2..i * 2 + 2];
        *byte = u8::from_str_radix(pair, 16).expect(
            "RAIL_A_MANIFEST_DIGEST_HEX must be valid hex, transcribed from proof-status.md",
        );
    }
    Digest(bytes)
}

#[test]
fn ticket_15_capacity_2_exploit2_admits_high_and_medium_first_defers_low_then_joins() {
    let request = request(2, masses(1, 2, 10), rail_a_handoff_digest());
    let the_plan = plan();

    let allocation = allocate_pddl_powl_plan(&the_plan, &request)
        .expect("complete mass field over the 3 real preparation actions must allocate");
    assert_eq!(
        allocation.priority_map.len(),
        4,
        "all 4 production actions (3 preparations + assemble) require an admitted mass"
    );

    let execution = the_plan
        .execute_with_cmca(&request)
        .expect("canonical execution must succeed and self-verify");
    let batches = execution
        .execution
        .execution_batches()
        .expect("every fired mask must resolve to real action labels");

    assert_eq!(
        batches.len(),
        3,
        "expected 3 ticks: {{high,medium}}, {{low}}, {{assemble}} -- got {batches:?}"
    );
    assert_eq!(
        batches[0].len(),
        2,
        "capacity 2 must admit exactly 2 of the 3 ready preparations in tick one -- got {batches:?}"
    );
    let tick_one: std::collections::BTreeSet<&String> = batches[0].iter().collect();
    assert!(
        tick_one.contains(&"prepare-high".to_string())
            && tick_one.contains(&"prepare-medium".to_string()),
        "tick one must admit prepare-high and prepare-medium (the two highest exploit2 shares, \
         100/105 and 4/105) -- got {batches:?}"
    );
    assert_eq!(
        batches[1],
        vec!["prepare-low".to_string()],
        "tick two must fire the deferred prepare-low alone -- got {batches:?}"
    );
    assert_eq!(
        batches[2],
        vec!["assemble".to_string()],
        "assemble must fire only in the tick after all 3 preparations complete -- got {batches:?}"
    );

    // Independent verification succeeds against the same plan/request
    // (a fresh plan(), since `execute_with_cmca` consumed the first one --
    // matching BCINR-CMCA-G's own established pattern).
    verify_cmca_execution(&execution.receipt, &plan(), &request)
        .expect("verification must succeed against the plan/request that governed execution");
}

// ---------------------------------------------------------------------
// Falsifiers -- each run for real against the actual functions, confirmed
// to fail/refuse for the stated reason, then reverted (git-diff-clean;
// no falsifier mutation is left in this file's final form).
// ---------------------------------------------------------------------

/// Falsifier 1: invert the mass extremes (`low` <-> `high`). Tick one's
/// admitted pair must change accordingly.
#[test]
fn falsifier_inverting_low_and_high_masses_changes_tick_one() {
    let request = request(2, masses(10, 2, 1), rail_a_handoff_digest());
    let plan = plan();
    let execution = plan.execute_with_cmca(&request).unwrap();
    let batches = execution.execution.execution_batches().unwrap();

    let tick_one: std::collections::BTreeSet<&String> = batches[0].iter().collect();
    assert!(
        tick_one.contains(&"prepare-low".to_string())
            && tick_one.contains(&"prepare-medium".to_string()),
        "with masses inverted (low=10, high=1), tick one must now admit \
         prepare-low and prepare-medium, deferring prepare-high -- got {batches:?}"
    );
    assert!(
        !tick_one.contains(&"prepare-high".to_string()),
        "prepare-high (now the lowest mass) must be the one deferred -- got {batches:?}"
    );
}

/// Falsifier 2: capacity `2 -> 3`. The receipt's bound capacity (and
/// therefore its priority/root digest) must differ from the capacity-2
/// receipt, even though all 3 preparations now fire in one tick.
#[test]
fn falsifier_capacity_change_changes_receipt_identity() {
    let plan_a = plan();
    let request_2 = request(2, masses(1, 2, 10), rail_a_handoff_digest());
    let execution_2 = plan_a.execute_with_cmca(&request_2).unwrap();

    let plan_b = plan();
    let request_3 = request(3, masses(1, 2, 10), rail_a_handoff_digest());
    let execution_3 = plan_b.execute_with_cmca(&request_3).unwrap();

    assert_ne!(
        execution_2.receipt.allocation.capacity, execution_3.receipt.allocation.capacity,
        "capacity must be receipt-bound"
    );
    assert_ne!(
        execution_2.receipt.root, execution_3.receipt.root,
        "capacity change must change the receipt root, even though all 3 \
         preparations now fire together at capacity 3"
    );

    let batches_3 = execution_3.execution.execution_batches().unwrap();
    assert_eq!(
        batches_3[0].len(),
        3,
        "capacity 3 must admit all 3 ready preparations in tick one -- got {batches_3:?}"
    );
}

/// Falsifier 3: reuse the capacity-2 receipt against a topologically
/// identical process with different action identities. Must refuse.
#[test]
fn falsifier_process_substitution_refuses() {
    const DOMAIN_2: &str = "(define (domain ticket15b)
        (:predicates (readied-a) (readied-b) (readied-c) (built))
        (:action ready-a :parameters () :precondition () :effect (readied-a))
        (:action ready-b :parameters () :precondition () :effect (readied-b))
        (:action ready-c :parameters () :precondition () :effect (readied-c))
        (:action build :parameters ()
            :precondition (and (readied-a) (readied-b) (readied-c))
            :effect (built)))";
    const PROBLEM_2: &str = "(define (problem ticket15bp) (:domain ticket15b) (:init)
        (:goal (built)))";

    let plan_a = plan();
    let request = request(2, masses(1, 2, 10), rail_a_handoff_digest());
    let execution_a = plan_a.execute_with_cmca(&request).unwrap();

    let plan_b = PddlPowlRuntime::new(PddlPowlConfig::default())
        .plan(DOMAIN_2, PROBLEM_2)
        .expect("topologically identical process with different action identities must plan");

    let cross = verify_cmca_execution(&execution_a.receipt, &plan_b, &request);
    assert!(
        cross.is_err(),
        "reusing ticket-15's receipt against a different-identity process must refuse -- got {cross:?}"
    );
    // Per BCINR-CMCA-G's own documented finding: since `request`'s masses
    // are keyed by ticket-15's real labels, mismatched labels are caught
    // at the mass-mapping gate (MissingActionMass) before the process
    // digest comparison is even reached -- an equally valid refusal of
    // reuse, not re-litigated here.
    match cross {
        Err(CmcaAllocationRefusal::MissingActionMass { .. }) => {}
        other => panic!("expected MissingActionMass, got {other:?}"),
    }
}

/// Falsifier 4: substitute a different Lean manifest digest at verify
/// time. Must refuse via `ProfileDigestMismatch`, before any scheduler
/// replay occurs.
#[test]
fn falsifier_wrong_lean_manifest_digest_refuses_before_replay() {
    let plan = plan();
    let sealed_with = request(2, masses(1, 2, 10), rail_a_handoff_digest());
    let execution = plan.execute_with_cmca(&sealed_with).unwrap();

    let different_digest = Digest::hash(b"a-different-lean-manifest-digest");
    assert_ne!(different_digest, rail_a_handoff_digest());
    let plan_for_verify = plan_for_verification();
    let verify_with = request(2, masses(1, 2, 10), different_digest);

    let result = verify_cmca_execution(&execution.receipt, &plan_for_verify, &verify_with);
    assert_eq!(result, Err(CmcaAllocationRefusal::ProfileDigestMismatch));
}

fn plan_for_verification() -> PddlPowlPlan {
    plan()
}

// ---------------------------------------------------------------------
// Runtime correspondence: the exact-rational oracle (`reference_escort`,
// BCINR-CMCA-H) and the fixed-point runtime (`allocate_pddl_powl_plan`,
// BCINR-CMCA-G) must agree on PRIORITY ORDER for this exact fixture's
// field -- "no float tolerance may define semantic identity." This does
// not re-derive H's bit-for-bit correspondence claims; it establishes
// the one relation ticket-15 actually depends on (high > medium > low)
// against the real oracle, for the real masses [1, 2, 10] this fixture
// uses, not [1,2,3,4] or [1,2,10] in the abstract.
// ---------------------------------------------------------------------

#[test]
fn exact_rational_oracle_and_fixed_point_runtime_agree_on_priority_order() {
    use bcinr_cmca::reference_escort::{escort, ReferenceLens};

    // The real exact-rational oracle, BCINR-CMCA-H's own type, on the
    // exact masses this fixture uses (in prepare-low/medium/high
    // declaration order).
    let oracle = escort(ReferenceLens::ExploitTwo, &[1, 2, 10])
        .expect("exploit2 on strictly positive masses must succeed");
    assert_eq!(oracle.len(), 3);
    assert!(
        oracle[0] < oracle[1] && oracle[1] < oracle[2],
        "oracle must order low < medium < high -- got {oracle:?}"
    );

    // The real fixed-point runtime, via the same allocate_pddl_powl_plan
    // this fixture's main test uses -- not re-derived, reused.
    let plan = plan();
    let request = request(2, masses(1, 2, 10), rail_a_handoff_digest());
    let allocation = allocate_pddl_powl_plan(&plan, &request)
        .expect("complete mass field over the 4 real production actions must allocate");

    let mut priority_by_label: BTreeMap<String, NonNegativeFixed> = BTreeMap::new();
    for (&slot, &priority) in &allocation.priority_map {
        let action = bcinr_pddl::production::action_for_slot(&plan.workflow, slot)
            .expect("every allocated slot must resolve to a real production action");
        priority_by_label.insert(action.label.clone(), priority);
    }

    let low = priority_by_label["prepare-low"];
    let medium = priority_by_label["prepare-medium"];
    let high = priority_by_label["prepare-high"];
    assert!(
        low < medium && medium < high,
        "fixed-point runtime must order prepare-low < prepare-medium < \
         prepare-high, matching the exact-rational oracle's order -- got \
         low={low:?}, medium={medium:?}, high={high:?}"
    );
}

// ---------------------------------------------------------------------
// Drift check: `RAIL_A_MANIFEST_DIGEST_HEX` above is a value copied from
// `~/mfw/docs/proof-status.md` at the time this fixture was written --
// exactly the kind of un-receipted handoff this whole checkpoint exists
// to eliminate. This test closes that gap by reading the real file
// directly and refusing to pass silently if it has drifted, instead of
// leaving the copy to go stale unnoticed. This is an environment
// precondition specific to this cross-repo integration checkpoint
// (`~/mfw` must be checked out as a sibling directory) -- it fails
// loudly, by design, rather than skipping quietly, if that's not true;
// this repository's own convention throughout this session has been
// "no silent skip," and a cross-repo correspondence check that quietly
// no-ops when the other repo is absent would defeat its own purpose.
// ---------------------------------------------------------------------

#[test]
fn rail_a_manifest_digest_matches_the_live_proof_status_md() {
    let home = std::env::var("HOME")
        .expect("HOME must be set to locate the sibling ~/mfw checkout this checkpoint depends on");
    let path = format!("{home}/mfw/docs/proof-status.md");
    let content = std::fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "could not read Rail A's handoff file at {path}: {error} -- \
             this checkpoint requires ~/mfw checked out as a sibling of ~/bcinr; \
             if that assumption is wrong for this environment, the fix is to \
             change how this test locates the file, not to skip the check"
        )
    });

    let join_math_row = content
        .lines()
        .find(|line| line.contains("JOIN-MATH") && line.contains("Rail A"))
        .unwrap_or_else(|| {
            panic!(
                "no JOIN-MATH (ECOSYSTEM-JOIN-001, Rail A) row found in {path} -- \
                 Rail A's own handoff format may have changed"
            )
        });

    let marker = "blake3:";
    let start = join_math_row
        .find(marker)
        .unwrap_or_else(|| panic!("JOIN-MATH row in {path} has no 'blake3:' digest"))
        + marker.len();
    let live_hex = &join_math_row[start..start + 64];

    assert_eq!(
        live_hex, RAIL_A_MANIFEST_DIGEST_HEX,
        "RAIL_A_MANIFEST_DIGEST_HEX has drifted from the live value in {path} -- \
         update the constant to match Rail A's current committed handoff"
    );

    // The digest this test just validated against the live file must be
    // the exact same value `rail_a_handoff_digest()` uses everywhere else
    // in this fixture -- not merely a string that happens to look right.
    let mut bytes = [0u8; 32];
    for (i, byte) in bytes.iter_mut().enumerate() {
        let pair = &live_hex[i * 2..i * 2 + 2];
        *byte = u8::from_str_radix(pair, 16).expect("live digest hex must be valid");
    }
    assert_eq!(Digest(bytes), rail_a_handoff_digest());
}
