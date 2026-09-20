use bcinr_cmca::{
    select_dme_route, select_gall_route, verify_dme_route_decision, verify_gall_route_decision,
    AuthorityStanding, ConsequenceClass, DmeRouteRefusal, DmeRouteRequest, GallRouteRefusal,
    GallRouteRequest, GallWorkIdentity, RouteCandidate, RouteClass, WorkKnowledge, WorkStanding,
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

#[test]
fn known_task_selects_deterministic_even_when_frontier_is_available() {
    let mut req = request(WorkKnowledge::Known);
    req.frontier_escalation_admitted = true;
    let decision = select_dme_route(&req).unwrap();
    assert_eq!(decision.route, RouteClass::KnownDeterministic);
    assert_eq!(decision.authority, AuthorityStanding::None);
    assert!(decision.explanation.refused.contains(&RouteClass::UnknownFrontier));
}

#[test]
fn exhausted_local_budget_does_not_silently_spill_to_frontier() {
    let mut req = request(WorkKnowledge::Unknown);
    req.routes.retain(|r| r.route != RouteClass::UnknownIdleEstate);
    req.routes.iter_mut().find(|r| r.route == RouteClass::UnknownLocal).unwrap().budget_units = 1;
    req.frontier_escalation_admitted = false;
    assert_eq!(
        select_dme_route(&req),
        Err(DmeRouteRefusal::UnknownWithoutLawfulRoute)
    );

    req.frontier_escalation_admitted = true;
    let decision = select_dme_route(&req).unwrap();
    assert_eq!(decision.route, RouteClass::UnknownFrontier);
}

#[test]
fn identical_admitted_inputs_produce_identical_content_addressed_decisions() {
    let req = request(WorkKnowledge::Unknown);
    let first = select_dme_route(&req).unwrap();
    let second = select_dme_route(&req).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.route, RouteClass::UnknownIdleEstate);
    assert_eq!(first.decision_digest.len(), 64);
    assert!(verify_dme_route_decision(&req, &first));
}

#[test]
fn unknown_consequence_class_is_refused_before_optimization() {
    let mut req = request(WorkKnowledge::Unknown);
    req.consequence = ConsequenceClass::Unknown;
    assert_eq!(
        select_dme_route(&req),
        Err(DmeRouteRefusal::UnknownConsequenceClass)
    );
}

#[test]
fn selector_output_carries_structural_no_authority() {
    let req = request(WorkKnowledge::Unknown);
    let decision = select_dme_route(&req).unwrap();
    assert_eq!(decision.authority, AuthorityStanding::None);
    assert_ne!(decision.route, RouteClass::KnownDeterministic);
}

#[test]
fn unknown_prefers_lower_cost_idle_estate_over_more_expensive_local() {
    let req = request(WorkKnowledge::Unknown);
    let decision = select_dme_route(&req).unwrap();
    assert_eq!(decision.route, RouteClass::UnknownIdleEstate);
    assert_eq!(decision.explanation.selected_cost_units, Some(5));
}

#[test]
fn candidate_standing_is_not_optimized() {
    let mut req = request(WorkKnowledge::Unknown);
    req.standing = WorkStanding::Candidate;
    assert_eq!(select_dme_route(&req), Err(DmeRouteRefusal::RequestNotAdmitted));
}


fn gall_request() -> GallRouteRequest {
    let work_order = "urn:gall:work-order:xaas:001".to_string();
    let mut route_request = request(WorkKnowledge::Unknown);
    route_request.semantic_subject = work_order.clone();

    GallRouteRequest {
        identity: GallWorkIdentity {
            work_order_iri: work_order,
            checkpoint_iri: "urn:gall:checkpoint:xaas:001".into(),
            graph_digest: format!("sha256:{}", "a".repeat(64)),
            repository_identity: "seanchatmangpt/xaas".into(),
            base_sha: "b".repeat(40),
        },
        route_request,
    }
}

#[test]
fn gall_route_reuses_cmca_selector_and_binds_exact_subject() {
    let req = gall_request();
    let decision = select_gall_route(&req).unwrap();

    assert_eq!(decision.route_decision.route, RouteClass::UnknownIdleEstate);
    assert_eq!(decision.authority, AuthorityStanding::None);
    assert_eq!(decision.identity.work_order_iri, req.identity.work_order_iri);
    assert!(verify_gall_route_decision(&req, &decision));
}

#[test]
fn moved_work_order_subject_is_refused_before_route_selection() {
    let mut req = gall_request();
    req.route_request.semantic_subject = "urn:gall:work-order:other".into();

    assert_eq!(select_gall_route(&req), Err(GallRouteRefusal::SubjectMismatch));
}

#[test]
fn branch_name_cannot_replace_exact_base_sha() {
    let mut req = gall_request();
    req.identity.base_sha = "main".into();

    assert_eq!(select_gall_route(&req), Err(GallRouteRefusal::InvalidBaseSha));
}
