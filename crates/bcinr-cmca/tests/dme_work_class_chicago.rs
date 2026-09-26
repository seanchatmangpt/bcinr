//! RFC closure items 2 and 3 for the DME selector (A2A-2605).
//!
//! * Item 2: the seven DME work classes (KNOWN, UNKNOWN_LOCAL, UNKNOWN_DEFERRED,
//!   UNKNOWN_FRONTIER, REFUSED, BLOCKED, UNSUPPORTED) are all reachable from real
//!   requests, and none carries authority.
//! * Item 3: exhaustion returns a typed witness instead of silently expanding
//!   search, and asking for more search mass never raises a budget.
//!
//! Chicago style: every test drives the real `select_dme_route` /
//! `classify_dme_work` and asserts on returned state. No test doubles.

use bcinr_cmca::{
    classify_dme_work, select_dme_route, AuthorityStanding, ConsequenceClass, DmeExhaustionWitness,
    DmeRouteRefusal, DmeRouteRequest, DmeWorkClass, RouteCandidate, RouteClass, WorkKnowledge,
    WorkStanding,
};

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

fn request(knowledge: WorkKnowledge, routes: Vec<RouteCandidate>) -> DmeRouteRequest {
    DmeRouteRequest {
        request_id: "req-closure".into(),
        semantic_subject: "urn:test:subject:closure".into(),
        standing: WorkStanding::Admitted,
        knowledge,
        consequence: ConsequenceClass::Construct,
        deadline_class: "DEFERABLE".into(),
        evidence_obligation: "EXACT_SUBJECT_RECEIPT".into(),
        frontier_escalation_admitted: false,
        routes,
    }
}

#[test]
fn all_seven_dme_work_classes_are_reachable_and_authority_free() {
    let known = request(
        WorkKnowledge::Known,
        vec![candidate(RouteClass::KnownDeterministic, 1, 10, 1)],
    );
    let local = request(
        WorkKnowledge::Unknown,
        vec![candidate(RouteClass::UnknownLocal, 1, 10, 1)],
    );
    let deferred = request(
        WorkKnowledge::Unknown,
        vec![
            candidate(RouteClass::UnknownDeferred, 1, 10, 1),
            candidate(RouteClass::UnknownLocal, 5, 10, 1),
        ],
    );
    let mut frontier = request(
        WorkKnowledge::Unknown,
        vec![candidate(RouteClass::UnknownFrontier, 1, 10, 1)],
    );
    frontier.frontier_escalation_admitted = true;
    let mut refused = request(
        WorkKnowledge::Unknown,
        vec![candidate(RouteClass::UnknownLocal, 1, 10, 1)],
    );
    refused.standing = WorkStanding::Candidate;
    let blocked = request(
        WorkKnowledge::Unknown,
        vec![candidate(RouteClass::UnknownLocal, 1, 10, 11)],
    );
    let mut unsupported = request(
        WorkKnowledge::Unknown,
        vec![candidate(RouteClass::UnknownLocal, 1, 10, 1)],
    );
    unsupported.routes[0].capability_fit = false;

    let cases = [
        (known, DmeWorkClass::Known),
        (local, DmeWorkClass::UnknownLocal),
        (deferred, DmeWorkClass::UnknownDeferred),
        (frontier, DmeWorkClass::UnknownFrontier),
        (refused, DmeWorkClass::Refused),
        (blocked, DmeWorkClass::Blocked),
        (unsupported, DmeWorkClass::Unsupported),
    ];
    let mut seen = Vec::new();
    for (req, expected) in cases {
        let classification = classify_dme_work(&req);
        assert_eq!(classification.class, expected, "request {req:?}");
        assert_eq!(classification.authority, AuthorityStanding::None);
        assert_eq!(
            classification.decision.is_some(),
            classification.refusal.is_none(),
            "exactly one of decision/refusal"
        );
        if let Some(decision) = &classification.decision {
            assert_eq!(decision.authority, AuthorityStanding::None);
        }
        seen.push(classification.class);
    }
    seen.sort_by_key(|c| *c as u8);
    seen.dedup();
    assert_eq!(
        seen.len(),
        7,
        "all seven DME work classes reached: {seen:?}"
    );
}

#[test]
fn exhaustion_returns_a_typed_witness_not_a_bare_refusal() {
    let mut req = request(
        WorkKnowledge::Unknown,
        vec![
            candidate(RouteClass::UnknownLocal, 1, 10, 25),
            candidate(RouteClass::UnknownIdleEstate, 2, 10, 12),
            candidate(RouteClass::UnknownFrontier, 3, 100, 1),
            candidate(RouteClass::KnownDeterministic, 0, 10, 1),
        ],
    );
    req.routes[1].evidence_fit = false;
    let refusal = select_dme_route(&req).unwrap_err();
    let DmeRouteRefusal::UnknownWithoutLawfulRoute(witness) = &refusal else {
        panic!("expected exhaustion refusal, got {refusal:?}")
    };
    assert_eq!(
        *witness,
        DmeExhaustionWitness {
            candidates: 4,
            class_ineligible: 2, // KnownDeterministic + unadmitted frontier
            frontier_unadmitted: 1,
            capability_unfit: 0,
            evidence_unfit: 1,
            consequence_unfit: 0,
            budget_exhausted: 2,
            // local is lawful except its budget; frontier is lawful except admission
            blocked_only_by_bound: 2,
            max_budget_shortfall_units: 15,
        }
    );
    assert_eq!(refusal.exhaustion(), Some(witness));
    assert_eq!(classify_dme_work(&req).class, DmeWorkClass::Blocked);
    assert_eq!(
        select_dme_route(&request(WorkKnowledge::Unknown, vec![]))
            .unwrap_err()
            .exhaustion()
            .unwrap()
            .candidates,
        0
    );
}

#[test]
fn exhaustion_witness_is_invariant_under_reordering_and_duplicate_delivery() {
    let routes = vec![
        candidate(RouteClass::UnknownLocal, 1, 10, 25),
        candidate(RouteClass::UnknownIdleEstate, 2, 10, 12),
        candidate(RouteClass::UnknownFrontier, 3, 100, 1),
    ];
    let forward = select_dme_route(&request(WorkKnowledge::Unknown, routes.clone())).unwrap_err();
    let mut reversed = routes.clone();
    reversed.reverse();
    reversed.push(routes[0].clone());
    let other = select_dme_route(&request(WorkKnowledge::Unknown, reversed)).unwrap_err();
    assert_eq!(forward, other);
}

/// RFC falsifier: "requesting additional search mass automatically increases the budget".
#[test]
fn requesting_more_search_mass_never_increases_the_budget() {
    for requested in [10u64, 11, 100, 10_000, u64::MAX] {
        let req = request(
            WorkKnowledge::Unknown,
            vec![candidate(RouteClass::UnknownLocal, 1, 10, requested)],
        );
        let classification = classify_dme_work(&req);
        if requested <= 10 {
            assert_eq!(classification.class, DmeWorkClass::UnknownLocal);
            continue;
        }
        // Over-budget demand is BLOCKED with the exact shortfall, never admitted by
        // widening the budget.
        assert_eq!(classification.class, DmeWorkClass::Blocked);
        assert!(classification.decision.is_none());
        let witness = classification
            .refusal
            .as_ref()
            .unwrap()
            .exhaustion()
            .unwrap();
        assert_eq!(witness.max_budget_shortfall_units, requested - 10);
        assert_eq!(witness.budget_exhausted, 1);
    }
    // When a cheaper in-budget route exists, the over-budget demand still does not
    // widen anything: the in-budget route is selected and the request's budget is
    // bound into the digest unchanged.
    let mut req = request(
        WorkKnowledge::Unknown,
        vec![
            candidate(RouteClass::UnknownLocal, 1, 10, 1_000),
            candidate(RouteClass::UnknownIdleEstate, 9, 10, 10),
        ],
    );
    let decision = select_dme_route(&req).unwrap();
    assert_eq!(decision.route, RouteClass::UnknownIdleEstate);
    req.routes[0].budget_units = 1_000;
    assert_ne!(
        select_dme_route(&req).unwrap().decision_digest,
        decision.decision_digest
    );
}

#[test]
fn selected_class_never_also_appears_as_refused() {
    let mut req = request(
        WorkKnowledge::Unknown,
        vec![
            candidate(RouteClass::UnknownLocal, 3, 10, 1),
            candidate(RouteClass::UnknownLocal, 1, 10, 11), // same class, over budget
            candidate(RouteClass::UnknownFrontier, 0, 10, 1),
        ],
    );
    req.routes.push(candidate(RouteClass::Refused, 0, 10, 1));
    let decision = select_dme_route(&req).unwrap();
    assert_eq!(decision.route, RouteClass::UnknownLocal);
    assert_eq!(decision.explanation.selected_cost_units, Some(3));
    assert!(!decision.explanation.refused.contains(&decision.route));
    assert_eq!(
        decision.explanation.refused,
        vec![RouteClass::UnknownFrontier, RouteClass::Refused]
    );
}

#[test]
fn known_work_without_deterministic_machinery_is_unsupported_not_explored() {
    let mut req = request(
        WorkKnowledge::Known,
        vec![candidate(RouteClass::UnknownLocal, 0, 10, 1)],
    );
    req.frontier_escalation_admitted = true;
    req.routes
        .push(candidate(RouteClass::UnknownFrontier, 0, 10, 1));
    let classification = classify_dme_work(&req);
    assert_eq!(classification.class, DmeWorkClass::Unsupported);
    let witness = classification.refusal.unwrap();
    assert!(matches!(
        witness,
        DmeRouteRefusal::KnownWithoutDeterministicRoute(DmeExhaustionWitness {
            candidates: 2,
            class_ineligible: 2,
            ..
        })
    ));
}
