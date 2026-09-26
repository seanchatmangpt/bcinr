//! Adversarial / boundary falsifiers for the DME route selector (A2A-2605).
//!
//! Chicago style: every test drives the real `select_dme_route` /
//! `verify_dme_route_decision` and asserts on the returned decision or refusal.
//! No test doubles.

use bcinr_cmca::{
    select_dme_route, verify_dme_route_decision, AuthorityStanding, ConsequenceClass,
    DmeRouteDecision, DmeRouteRefusal, DmeRouteRequest, RouteCandidate, RouteClass, WorkKnowledge,
    WorkStanding,
};
use proptest::prelude::*;

fn candidate(route: RouteClass, cost: u64, budget: u64, required: u64) -> RouteCandidate {
    RouteCandidate {
        route,
        cost_units: cost,
        capability_fit: true,
        budget_units: budget,
        required_units: required,
        evidence_fit: true,
        consequence_fit: true,
    }
}

fn request(knowledge: WorkKnowledge) -> DmeRouteRequest {
    DmeRouteRequest {
        request_id: "req-2605".into(),
        semantic_subject: "urn:test:subject:2605".into(),
        standing: WorkStanding::Admitted,
        knowledge,
        consequence: ConsequenceClass::Construct,
        deadline_class: "DEFERABLE".into(),
        evidence_obligation: "EXACT_SUBJECT_RECEIPT".into(),
        frontier_escalation_admitted: false,
        routes: vec![
            candidate(RouteClass::KnownDeterministic, 1, 10, 1),
            candidate(RouteClass::UnknownLocal, 10, 100, 20),
            candidate(RouteClass::UnknownIdleEstate, 5, 100, 20),
            candidate(RouteClass::UnknownFrontier, 1000, 10_000, 20),
        ],
    }
}

fn permutations(items: &[RouteCandidate]) -> Vec<Vec<RouteCandidate>> {
    if items.len() <= 1 {
        return vec![items.to_vec()];
    }
    let mut out = Vec::new();
    for i in 0..items.len() {
        let mut rest = items.to_vec();
        let head = rest.remove(i);
        for mut tail in permutations(&rest) {
            tail.insert(0, head.clone());
            out.push(tail);
        }
    }
    out
}

// ---- reordering -----------------------------------------------------------

#[test]
fn every_permutation_of_routes_yields_the_identical_sealed_decision() {
    for knowledge in [WorkKnowledge::Known, WorkKnowledge::Unknown] {
        let base = request(knowledge);
        let reference = select_dme_route(&base).unwrap();
        let perms = permutations(&base.routes);
        assert_eq!(perms.len(), 24);
        for routes in perms {
            let mut permuted = base.clone();
            permuted.routes = routes;
            let decision = select_dme_route(&permuted).unwrap();
            assert_eq!(decision, reference, "reordering changed the decision");
            assert!(verify_dme_route_decision(&permuted, &reference));
        }
    }
}

#[test]
fn equal_cost_tie_is_resolved_by_class_rank_regardless_of_order() {
    let mut req = request(WorkKnowledge::Unknown);
    req.routes = vec![
        candidate(RouteClass::UnknownIdleEstate, 7, 100, 1),
        candidate(RouteClass::UnknownLocal, 7, 100, 1),
    ];
    let forward = select_dme_route(&req).unwrap();
    req.routes.reverse();
    let reverse = select_dme_route(&req).unwrap();
    assert_eq!(forward.route, RouteClass::UnknownLocal);
    assert_eq!(forward, reverse);
}

// ---- duplicate delivery ---------------------------------------------------

#[test]
fn duplicate_candidate_delivery_does_not_change_decision_or_digest() {
    let req = request(WorkKnowledge::Unknown);
    let reference = select_dme_route(&req).unwrap();
    let mut duplicated = req.clone();
    duplicated.routes.extend(req.routes.iter().cloned());
    duplicated.routes.push(req.routes[2].clone());
    let decision = select_dme_route(&duplicated).unwrap();
    assert_eq!(decision, reference);
    assert!(verify_dme_route_decision(&duplicated, &reference));
}

// ---- wrong digest / tampering / replay mismatch ----------------------------

#[test]
fn tampered_digest_fails_verification() {
    let req = request(WorkKnowledge::Unknown);
    let mut decision = select_dme_route(&req).unwrap();
    let mut bytes = decision.decision_digest.into_bytes();
    bytes[0] = if bytes[0] == b'0' { b'1' } else { b'0' };
    decision.decision_digest = String::from_utf8(bytes).unwrap();
    assert!(!verify_dme_route_decision(&req, &decision));
}

#[test]
fn tampered_route_or_explanation_fails_verification() {
    let req = request(WorkKnowledge::Unknown);
    let honest = select_dme_route(&req).unwrap();

    let mut forged_route = honest.clone();
    forged_route.route = RouteClass::UnknownFrontier;
    assert!(!verify_dme_route_decision(&req, &forged_route));

    let mut forged_refused = honest.clone();
    forged_refused.explanation.refused.clear();
    assert!(!verify_dme_route_decision(&req, &forged_refused));

    let mut forged_cost = honest;
    forged_cost.explanation.selected_cost_units = Some(0);
    assert!(!verify_dme_route_decision(&req, &forged_cost));
}

/// Stale subject: a decision sealed for one request cannot be replayed against a
/// request that differs in any semantic field, even when the selected route is the
/// same. Before hardening the digest omitted these fields.
#[test]
fn decision_is_bound_to_every_semantic_request_field() {
    let req = request(WorkKnowledge::Unknown);
    let sealed = select_dme_route(&req).unwrap();

    let mutations: Vec<(&str, Box<dyn Fn(&mut DmeRouteRequest)>)> = vec![
        (
            "consequence",
            Box::new(|r| r.consequence = ConsequenceClass::ExternalDo),
        ),
        (
            "deadline",
            Box::new(|r| r.deadline_class = "IMMEDIATE".into()),
        ),
        (
            "evidence",
            Box::new(|r| r.evidence_obligation = "NONE".into()),
        ),
        (
            "frontier",
            Box::new(|r| r.frontier_escalation_admitted = true),
        ),
        (
            "subject",
            Box::new(|r| r.semantic_subject = "urn:test:other".into()),
        ),
        (
            "request_id",
            Box::new(|r| r.request_id = "req-other".into()),
        ),
        ("route budget", Box::new(|r| r.routes[1].budget_units = 99)),
    ];
    for (name, mutate) in mutations {
        let mut other = req.clone();
        mutate(&mut other);
        let recomputed = select_dme_route(&other).unwrap();
        assert_eq!(
            recomputed.route, sealed.route,
            "{name}: route should be unchanged"
        );
        assert_ne!(
            recomputed.decision_digest, sealed.decision_digest,
            "{name}: digest must bind this field"
        );
        assert!(
            !verify_dme_route_decision(&other, &sealed),
            "{name}: stale decision replayed against a different request"
        );
    }
}

#[test]
fn json_wire_roundtrip_preserves_verifiable_decision() {
    let req = request(WorkKnowledge::Unknown);
    let decision = select_dme_route(&req).unwrap();
    let wire = serde_json::to_string(&decision).unwrap();
    let back: DmeRouteDecision = serde_json::from_str(&wire).unwrap();
    assert!(verify_dme_route_decision(&req, &back));
    let req_wire = serde_json::to_string(&req).unwrap();
    let req_back: DmeRouteRequest = serde_json::from_str(&req_wire).unwrap();
    assert!(verify_dme_route_decision(&req_back, &decision));
}

#[test]
fn authority_cannot_be_deserialized_to_anything_but_none() {
    let req = request(WorkKnowledge::Unknown);
    let decision = select_dme_route(&req).unwrap();
    let wire = serde_json::to_string(&decision).unwrap();
    assert!(wire.contains("\"authority\":\"NONE\""));
    let forged = wire.replace("\"authority\":\"NONE\"", "\"authority\":\"DO\"");
    assert!(serde_json::from_str::<DmeRouteDecision>(&forged).is_err());
}

// ---- malformed input / unauthorized ---------------------------------------

#[test]
fn blank_or_whitespace_identity_is_refused() {
    let mut req = request(WorkKnowledge::Unknown);
    req.request_id = "  \t".into();
    assert_eq!(
        select_dme_route(&req),
        Err(DmeRouteRefusal::InvalidRequestId)
    );
    let mut req = request(WorkKnowledge::Unknown);
    req.semantic_subject = "\n".into();
    assert_eq!(
        select_dme_route(&req),
        Err(DmeRouteRefusal::InvalidSemanticSubject)
    );
}

#[test]
fn refused_standing_is_refused_before_optimization() {
    let mut req = request(WorkKnowledge::Known);
    req.standing = WorkStanding::Refused;
    assert_eq!(
        select_dme_route(&req),
        Err(DmeRouteRefusal::RequestNotAdmitted)
    );
}

#[test]
fn empty_route_set_is_refused_for_both_knowledge_classes() {
    let mut known = request(WorkKnowledge::Known);
    known.routes.clear();
    assert_eq!(
        select_dme_route(&known),
        Err(DmeRouteRefusal::KnownWithoutDeterministicRoute)
    );
    let mut unknown = request(WorkKnowledge::Unknown);
    unknown.routes.clear();
    unknown.frontier_escalation_admitted = true;
    assert_eq!(
        select_dme_route(&unknown),
        Err(DmeRouteRefusal::UnknownWithoutLawfulRoute)
    );
}

#[test]
fn known_work_never_falls_back_to_a_model_route() {
    let mut req = request(WorkKnowledge::Known);
    req.frontier_escalation_admitted = true;
    req.routes[0].evidence_fit = false; // the only deterministic route is unlawful
    assert_eq!(
        select_dme_route(&req),
        Err(DmeRouteRefusal::KnownWithoutDeterministicRoute)
    );
}

#[test]
fn refused_route_class_is_never_selected_even_when_cheapest_and_lawful() {
    let mut req = request(WorkKnowledge::Unknown);
    req.routes
        .push(candidate(RouteClass::Refused, 0, u64::MAX, 0));
    let decision = select_dme_route(&req).unwrap();
    assert_eq!(decision.route, RouteClass::UnknownIdleEstate);
    assert!(decision.explanation.refused.contains(&RouteClass::Refused));
}

#[test]
fn unadmitted_frontier_is_never_selected_even_when_cheapest() {
    let mut req = request(WorkKnowledge::Unknown);
    req.routes
        .push(candidate(RouteClass::UnknownFrontier, 0, 10, 1));
    let decision = select_dme_route(&req).unwrap();
    assert_eq!(decision.route, RouteClass::UnknownIdleEstate);
    assert!(decision
        .explanation
        .refused
        .contains(&RouteClass::UnknownFrontier));
}

#[test]
fn every_fit_flag_is_individually_load_bearing() {
    for flag in 0..3 {
        let mut req = request(WorkKnowledge::Unknown);
        req.routes.retain(|r| r.route == RouteClass::UnknownLocal);
        match flag {
            0 => req.routes[0].capability_fit = false,
            1 => req.routes[0].evidence_fit = false,
            _ => req.routes[0].consequence_fit = false,
        }
        assert_eq!(
            select_dme_route(&req),
            Err(DmeRouteRefusal::UnknownWithoutLawfulRoute)
        );
    }
}

#[test]
fn budget_boundary_is_inclusive_and_one_over_is_refused() {
    let mut req = request(WorkKnowledge::Unknown);
    req.routes = vec![candidate(RouteClass::UnknownLocal, 3, 20, 20)];
    assert_eq!(
        select_dme_route(&req).unwrap().route,
        RouteClass::UnknownLocal
    );
    req.routes[0].required_units = 21;
    assert_eq!(
        select_dme_route(&req),
        Err(DmeRouteRefusal::UnknownWithoutLawfulRoute)
    );
    req.routes = vec![candidate(
        RouteClass::UnknownLocal,
        u64::MAX,
        u64::MAX,
        u64::MAX,
    )];
    let decision = select_dme_route(&req).unwrap();
    assert_eq!(decision.explanation.selected_cost_units, Some(u64::MAX));
    assert_eq!(decision.authority, AuthorityStanding::None);
}

#[test]
fn explanation_classes_are_canonical_sets() {
    let mut req = request(WorkKnowledge::Unknown);
    req.routes
        .push(candidate(RouteClass::UnknownFrontier, 5, 1, 2));
    req.routes
        .push(candidate(RouteClass::KnownDeterministic, 9, 10, 1));
    let decision = select_dme_route(&req).unwrap();
    let rank = |r: &RouteClass| *r as u8;
    for list in [
        &decision.explanation.considered,
        &decision.explanation.refused,
    ] {
        for pair in list.windows(2) {
            assert!(
                rank(&pair[0]) < rank(&pair[1]),
                "not strictly ordered: {list:?}"
            );
        }
    }
}

// ---- property: permutation and duplication invariance ---------------------

fn arb_route() -> impl Strategy<Value = RouteClass> {
    prop_oneof![
        Just(RouteClass::KnownDeterministic),
        Just(RouteClass::UnknownLocal),
        Just(RouteClass::UnknownIdleEstate),
        Just(RouteClass::UnknownFrontier),
        Just(RouteClass::Refused),
    ]
}

fn arb_candidate() -> impl Strategy<Value = RouteCandidate> {
    (
        arb_route(),
        0u64..8,
        0u64..8,
        0u64..8,
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
    )
        .prop_map(
            |(route, cost, budget, required, cap, ev, cons)| RouteCandidate {
                route,
                cost_units: cost,
                capability_fit: cap,
                budget_units: budget,
                required_units: required,
                evidence_fit: ev,
                consequence_fit: cons,
            },
        )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]
    #[test]
    fn selection_is_invariant_under_permutation_and_duplication(
        routes in prop::collection::vec(arb_candidate(), 0..12),
        known in any::<bool>(),
        frontier in any::<bool>(),
        seed in any::<u64>(),
    ) {
        let mut req = request(if known { WorkKnowledge::Known } else { WorkKnowledge::Unknown });
        req.frontier_escalation_admitted = frontier;
        req.routes = routes.clone();
        let reference = select_dme_route(&req);

        // deterministic Fisher-Yates driven by `seed`
        let mut shuffled = routes.clone();
        let mut state = seed | 1;
        for i in (1..shuffled.len()).rev() {
            state ^= state << 13; state ^= state >> 7; state ^= state << 17;
            shuffled.swap(i, (state % (i as u64 + 1)) as usize);
        }
        shuffled.extend(routes.iter().take(2).cloned());
        let mut other = req.clone();
        other.routes = shuffled;
        prop_assert_eq!(select_dme_route(&other), reference.clone());

        if let Ok(decision) = reference {
            prop_assert_eq!(decision.authority, AuthorityStanding::None);
            prop_assert!(decision.route != RouteClass::Refused);
            if known { prop_assert_eq!(decision.route, RouteClass::KnownDeterministic); }
            if !frontier { prop_assert!(decision.route != RouteClass::UnknownFrontier); }
            prop_assert!(verify_dme_route_decision(&other, &decision));
        }
    }
}
