//! RFC closure item 1 (A2A-2605): the allocator-facing interface composes the real
//! CostVector / MassVector / q-lens / FairRailScheduler / ConsequenceHorizon
//! machinery with the canonical CMCA route selector.
//!
//! Chicago style: real admission gate (`admit_candidate_domain`), real q-lens, real
//! fair-rail scheduler, real horizons, real selector. No test doubles.
#![cfg(feature = "mfw-planner")]

use bcinr_cmca::{
    AuthorityStanding, ConsequenceClass, DmeRouteRefusal, DmeWorkClass, RouteClass, WorkKnowledge,
};
use bcinr_mfw_ir::{BoundHit, BoundKind};
use bcinr_pddl::capability_router::CostVector;
use bcinr_pddl::dme_allocator::{
    admit_domain_work, allocate, AllocatorProfile, AllocatorRail, AllocatorRefusal,
    AllocatorWorkPackage, WorkAdmission, MAX_ALLOCATOR_BUDGET_UNITS, MAX_ALLOCATOR_RAILS,
    MAX_ALLOCATOR_SCHEDULE_TICKS,
};
use bcinr_pddl::{
    ConsequenceHorizon, GoalReachabilityHorizon, MassVector, MinimumMakespanHorizon, QValue,
    RailSelection,
};

const DOMAIN: &str = "(define (domain blocks) (:requirements :strips) (:predicates (on ?x ?y) (ontable ?x) (clear ?x) (holding ?x) (handempty)) (:action pick-up :parameters (?x) :precondition (and (clear ?x) (ontable ?x) (handempty)) :effect (and (holding ?x) (not (clear ?x)) (not (ontable ?x)) (not (handempty)))) (:action put-down :parameters (?x) :precondition (holding ?x) :effect (and (not (holding ?x)) (clear ?x) (ontable ?x) (handempty))))";

fn cost(latency_ms: u64) -> CostVector {
    CostVector {
        admitted: true,
        unreceipted_mutation_risk: 0,
        human_attention_seconds: 1.0,
        token_cost: 0,
        latency_ms,
        context_switches: 1,
    }
}

fn mass(goal: f64) -> MassVector {
    MassVector {
        unresolved_goal_mass: goal,
        candidate_action_mass: 0.0,
        semantic_novelty_mass: 0.0,
        resource_pressure_mass: 0.0,
        temporal_pressure_mass: 0.0,
        cache_novelty_mass: 0.0,
    }
}

fn rail(route: RouteClass, latency_ms: u64, goal_mass: f64, requested: u64) -> AllocatorRail {
    AllocatorRail {
        route,
        cost: cost(latency_ms),
        mass: mass(goal_mass),
        requested_units: requested,
        capability_fit: true,
        evidence_fit: true,
        consequence_fit: true,
    }
}

fn profile() -> AllocatorProfile {
    AllocatorProfile {
        q: QValue::new(2.0).unwrap(),
        admitted_budget_units: 100,
        fair_rail_max_gap: 2,
        schedule_ticks: 9,
        frontier_escalation_admitted: false,
    }
}

fn package(knowledge: WorkKnowledge, rails: Vec<AllocatorRail>) -> AllocatorWorkPackage {
    AllocatorWorkPackage {
        request_id: "alloc-1".into(),
        semantic_subject: "urn:test:alloc".into(),
        admission: admit_domain_work(DOMAIN).expect("real domain admits"),
        knowledge,
        consequence: ConsequenceClass::Construct,
        deadline_class: "DEFERABLE".into(),
        evidence_obligation: "EXACT_SUBJECT_RECEIPT".into(),
        rails,
    }
}

/// RFC falsifier: "candidate PDDL/domain text bypasses admission".
#[test]
fn candidate_domain_text_never_reaches_a_route() {
    let mut pkg = package(
        WorkKnowledge::Known,
        vec![rail(RouteClass::KnownDeterministic, 1, 1.0, 1)],
    );
    // Even text that WOULD admit is refused while it is still a candidate.
    pkg.admission = WorkAdmission::Candidate {
        text: DOMAIN.into(),
    };
    let alloc = allocate(&profile(), &pkg, &GoalReachabilityHorizon).unwrap();
    assert_eq!(alloc.classification.class, DmeWorkClass::Refused);
    assert_eq!(
        alloc.classification.refusal,
        Some(DmeRouteRefusal::RequestNotAdmitted)
    );
    assert!(alloc.classification.decision.is_none());
    assert!(alloc.rail_schedule.is_empty());

    // Malformed text cannot become admitted work at all.
    assert!(admit_domain_work("(define (domain broken) (:predicates").is_err());

    // The same text through the real gate is admitted and routed.
    pkg.admission = admit_domain_work(DOMAIN).unwrap();
    let admitted = allocate(&profile(), &pkg, &GoalReachabilityHorizon).unwrap();
    assert_eq!(admitted.classification.class, DmeWorkClass::Known);
}

/// RFC falsifier: "requesting additional search mass automatically increases the budget".
#[test]
fn requesting_more_search_mass_never_increases_the_admitted_budget() {
    for requested in [100u64, 101, 10_000, u64::MAX] {
        let pkg = package(
            WorkKnowledge::Unknown,
            vec![rail(RouteClass::UnknownLocal, 1, 1.0, requested)],
        );
        let alloc = allocate(&profile(), &pkg, &GoalReachabilityHorizon).unwrap();
        assert!(alloc
            .route_request
            .routes
            .iter()
            .all(|c| c.budget_units == profile().admitted_budget_units));
        if requested <= 100 {
            assert_eq!(alloc.classification.class, DmeWorkClass::UnknownLocal);
        } else {
            assert_eq!(alloc.classification.class, DmeWorkClass::Blocked);
            let witness = alloc
                .classification
                .refusal
                .as_ref()
                .unwrap()
                .exhaustion()
                .unwrap()
                .clone();
            assert_eq!(witness.max_budget_shortfall_units, requested - 100);
            assert!(
                alloc.rail_schedule.is_empty(),
                "blocked work is not expanded"
            );
        }
    }
}

/// RFC falsifier: "equivalent KNOWN work enters an exploratory LLM/search rail when an
/// admitted deterministic route exists".
#[test]
fn known_work_stays_on_deterministic_machinery_even_when_exploration_looks_better() {
    let mut p = profile();
    p.frontier_escalation_admitted = true;
    let pkg = package(
        WorkKnowledge::Known,
        vec![
            rail(RouteClass::KnownDeterministic, 50_000, 0.001, 1),
            rail(RouteClass::UnknownLocal, 1, 100.0, 1),
            rail(RouteClass::UnknownFrontier, 1, 1_000.0, 1),
        ],
    );
    let alloc = allocate(&p, &pkg, &GoalReachabilityHorizon).unwrap();
    assert_eq!(alloc.classification.class, DmeWorkClass::Known);
    assert_eq!(
        alloc.classification.decision.as_ref().unwrap().route,
        RouteClass::KnownDeterministic
    );
    assert!(alloc.rail_schedule.is_empty());
    assert_eq!(alloc.authority, AuthorityStanding::None);
}

#[test]
fn cost_vector_order_dominates_and_non_admitted_cost_loses_capability() {
    let pkg = package(
        WorkKnowledge::Unknown,
        vec![
            rail(RouteClass::UnknownLocal, 9_000, 100.0, 1),
            rail(RouteClass::UnknownIdleEstate, 10, 0.1, 1),
        ],
    );
    let alloc = allocate(&profile(), &pkg, &GoalReachabilityHorizon).unwrap();
    assert_eq!(
        alloc.classification.decision.unwrap().route,
        RouteClass::UnknownIdleEstate,
        "lower CostVector wins regardless of frontier mass"
    );

    let mut refused = pkg.clone();
    refused.rails[1].cost.admitted = false;
    let alloc = allocate(&profile(), &refused, &GoalReachabilityHorizon).unwrap();
    assert_eq!(
        alloc.classification.decision.unwrap().route,
        RouteClass::UnknownLocal
    );
}

#[test]
fn q_lens_frontier_mass_breaks_equal_cost_ties_and_flips_with_the_masses() {
    let pick = |local_mass: f64, idle_mass: f64| {
        let pkg = package(
            WorkKnowledge::Unknown,
            vec![
                rail(RouteClass::UnknownLocal, 10, local_mass, 1),
                rail(RouteClass::UnknownIdleEstate, 10, idle_mass, 1),
            ],
        );
        allocate(&profile(), &pkg, &GoalReachabilityHorizon)
            .unwrap()
            .classification
            .decision
            .unwrap()
            .route
    };
    assert_eq!(pick(1.0, 3.0), RouteClass::UnknownIdleEstate);
    assert_eq!(pick(3.0, 1.0), RouteClass::UnknownLocal);
}

#[test]
fn consequence_horizon_identity_is_bound_into_the_decision() {
    let pkg = package(
        WorkKnowledge::Unknown,
        vec![rail(RouteClass::UnknownLocal, 1, 1.0, 1)],
    );
    let goal = allocate(&profile(), &pkg, &GoalReachabilityHorizon).unwrap();
    let makespan = allocate(&profile(), &pkg, &MinimumMakespanHorizon).unwrap();
    assert_eq!(goal.horizon, GoalReachabilityHorizon.id());
    assert_eq!(makespan.horizon, MinimumMakespanHorizon.id());
    assert_ne!(
        goal.classification.decision.unwrap().decision_digest,
        makespan.classification.decision.unwrap().decision_digest
    );
}

#[test]
fn exploratory_work_gets_a_fair_rail_schedule_that_never_starves_the_exact_rail() {
    let pkg = package(
        WorkKnowledge::Unknown,
        vec![
            rail(RouteClass::UnknownLocal, 1, 1.0, 1),
            rail(RouteClass::UnknownIdleEstate, 2, 1.0, 1),
        ],
    );
    let alloc = allocate(&profile(), &pkg, &GoalReachabilityHorizon).unwrap();
    assert_eq!(alloc.classification.class, DmeWorkClass::UnknownLocal);
    assert_eq!(alloc.rail_schedule.len(), 9);
    let mut gap = 0usize;
    for tick in &alloc.rail_schedule {
        match tick {
            RailSelection::Exact => gap = 0,
            RailSelection::Exploit(index) => {
                assert!(*index < 2);
                gap += 1;
                assert!(gap <= 2, "exact rail starved: {:?}", alloc.rail_schedule);
            }
        }
    }
    assert!(alloc.rail_schedule.contains(&RailSelection::Exact));
}

#[test]
fn hard_ceilings_refuse_with_typed_witnesses() {
    let many = vec![rail(RouteClass::UnknownLocal, 1, 1.0, 1); MAX_ALLOCATOR_RAILS + 1];
    assert_eq!(
        allocate(
            &profile(),
            &package(WorkKnowledge::Unknown, many),
            &GoalReachabilityHorizon
        ),
        Err(AllocatorRefusal::FanOutCeiling(BoundHit {
            kind: BoundKind::FrontierStates,
            limit: MAX_ALLOCATOR_RAILS as u64,
            observed: MAX_ALLOCATOR_RAILS as u64 + 1,
        }))
    );
    let exactly = vec![rail(RouteClass::UnknownLocal, 1, 1.0, 1); MAX_ALLOCATOR_RAILS];
    assert!(allocate(
        &profile(),
        &package(WorkKnowledge::Unknown, exactly),
        &GoalReachabilityHorizon
    )
    .is_ok());

    let pkg = package(
        WorkKnowledge::Unknown,
        vec![rail(RouteClass::UnknownLocal, 1, 1.0, 1)],
    );
    let mut wide = profile();
    wide.admitted_budget_units = MAX_ALLOCATOR_BUDGET_UNITS + 1;
    assert!(matches!(
        allocate(&wide, &pkg, &GoalReachabilityHorizon),
        Err(AllocatorRefusal::BudgetCeiling(BoundHit { observed, .. })) if observed == MAX_ALLOCATOR_BUDGET_UNITS + 1
    ));
    let mut long = profile();
    long.schedule_ticks = MAX_ALLOCATOR_SCHEDULE_TICKS + 1;
    assert!(matches!(
        allocate(&long, &pkg, &GoalReachabilityHorizon),
        Err(AllocatorRefusal::ScheduleCeiling(_))
    ));
}

#[test]
fn allocation_is_deterministic_and_invariant_under_rail_order() {
    let rails = vec![
        rail(RouteClass::UnknownLocal, 10, 1.0, 1),
        rail(RouteClass::UnknownIdleEstate, 10, 3.0, 1),
        rail(RouteClass::UnknownDeferred, 20, 5.0, 1),
    ];
    let forward = allocate(
        &profile(),
        &package(WorkKnowledge::Unknown, rails.clone()),
        &GoalReachabilityHorizon,
    )
    .unwrap();
    let mut reversed = rails;
    reversed.reverse();
    let backward = allocate(
        &profile(),
        &package(WorkKnowledge::Unknown, reversed),
        &GoalReachabilityHorizon,
    )
    .unwrap();
    assert_eq!(
        forward.classification.decision.unwrap().decision_digest,
        backward.classification.decision.unwrap().decision_digest
    );
}

#[test]
fn deferred_work_class_is_reachable_through_the_allocator() {
    let pkg = package(
        WorkKnowledge::Unknown,
        vec![
            rail(RouteClass::UnknownDeferred, 1, 1.0, 1),
            rail(RouteClass::UnknownLocal, 10, 1.0, 1),
        ],
    );
    let alloc = allocate(&profile(), &pkg, &GoalReachabilityHorizon).unwrap();
    assert_eq!(alloc.classification.class, DmeWorkClass::UnknownDeferred);
    assert_eq!(alloc.classification.authority, AuthorityStanding::None);
}
