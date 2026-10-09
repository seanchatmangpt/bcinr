//! Hostile verification for root-surface refusals with no prior exact-variant
//! coverage (lane 7): `DmeRouteRefusal`/`GallRouteRefusal` identity laws,
//! `LensSelectionRefusal::QMagnitudeExceeded`, and the `StabilityRefusal`
//! wire-code codec. Constitution §18/§19: every refusal is asserted against
//! its EXACT typed variant; each guarded check carries a `non-vacuity:`
//! comment naming the guarded source path and the plausible mutation that
//! would fail the assertion.

use bcinr_cmca::allocator::{allocate_single_lens, LensSelectionRefusal as LensRefusal};
use bcinr_cmca::fixed::{NonNegativeFixed, SignedFixed};
use bcinr_cmca::generated::consequence_mass::case_studies::{LENS_REGISTRY, N, Q};
use bcinr_cmca::{
    select_dme_route, select_gall_route, AuthorityStanding, ConsequenceClass, DmeRouteRefusal,
    DmeRouteRequest, GallRouteRefusal, GallRouteRequest, GallWorkIdentity, LensSelectionRefusal,
    RouteClass, StabilityRefusal, WorkKnowledge, WorkStanding,
};

// ---------------------------------------------------------------------------
// DmeRouteRefusal (src/dme_route.rs)
// ---------------------------------------------------------------------------

fn candidate(
    route: RouteClass,
    cost: u64,
    budget: u64,
    required: u64,
) -> bcinr_cmca::RouteCandidate {
    bcinr_cmca::RouteCandidate {
        route,
        cost_units: cost,
        capability_fit: true,
        budget_units: budget,
        required_units: required,
        evidence_fit: true,
        consequence_fit: true,
    }
}

fn dme_request() -> DmeRouteRequest {
    DmeRouteRequest {
        request_id: "req-lane7".into(),
        semantic_subject: "urn:test:subject:lane7".into(),
        standing: WorkStanding::Admitted,
        knowledge: WorkKnowledge::Known,
        consequence: ConsequenceClass::Construct,
        deadline_class: "DEFERABLE".into(),
        evidence_obligation: "EXACT_SUBJECT_RECEIPT".into(),
        frontier_escalation_admitted: false,
        routes: vec![
            candidate(RouteClass::KnownDeterministic, 1, 10, 1),
            candidate(RouteClass::UnknownLocal, 10, 100, 20),
        ],
    }
}

// non-vacuity: src/dme_route.rs:157 — replacing `request.semantic_subject.trim()
// .is_empty()` with `request.semantic_subject.is_empty()` (dropping the trim)
// makes the second assert stop failing; deleting the whole guard makes both
// asserts stop failing.
#[test]
fn dme_refuses_blank_semantic_subject_exactly() {
    assert!(select_dme_route(&dme_request()).is_ok());

    let mut whitespace_subject = dme_request();
    whitespace_subject.semantic_subject = "   ".into();
    assert_eq!(
        select_dme_route(&whitespace_subject),
        Err(DmeRouteRefusal::InvalidSemanticSubject)
    );

    let mut empty_subject = dme_request();
    empty_subject.semantic_subject = String::new();
    assert_eq!(
        select_dme_route(&empty_subject),
        Err(DmeRouteRefusal::InvalidSemanticSubject)
    );
}

// non-vacuity: src/dme_route.rs:163-170 — dropping the
// `.ok_or(DmeRouteRefusal::KnownWithoutDeterministicRoute)?` (or widening the
// filter to admit UnknownLocal routes into a Known selection) makes the assert
// stop failing. The positive control proves the fixture's deterministic route
// is lawful, so the refusal comes from its removal alone.
#[test]
fn dme_known_knowledge_without_deterministic_route_is_refused_exactly() {
    assert!(select_dme_route(&dme_request()).is_ok());

    let mut no_deterministic = dme_request();
    no_deterministic
        .routes
        .retain(|c| c.route != RouteClass::KnownDeterministic);
    assert_eq!(
        select_dme_route(&no_deterministic),
        Err(DmeRouteRefusal::KnownWithoutDeterministicRoute)
    );
}

// ---------------------------------------------------------------------------
// GallRouteRefusal (src/dme_route.rs:224-242)
// ---------------------------------------------------------------------------

fn gall_request() -> GallRouteRequest {
    let work_order = "urn:gall:work-order:lane7:001".to_string();
    let mut route_request = dme_request();
    route_request.semantic_subject = work_order.clone();

    GallRouteRequest {
        identity: GallWorkIdentity {
            work_order_iri: work_order,
            checkpoint_iri: "urn:gall:checkpoint:lane7:001".into(),
            graph_digest: format!("sha256:{}", "a".repeat(64)),
            repository_identity: "seanchatmangpt/xaas".into(),
            base_sha: "b".repeat(40),
        },
        route_request,
    }
}

// non-vacuity: src/dme_route.rs:227-236 — deleting any single identity guard
// (absolute_iri on work_order_iri / checkpoint_iri, sha256_digest shape,
// owner/repo split) makes the corresponding assert stop failing and lets a
// malformed identity reach route selection; a mutation reordering the guards
// still fails because each case tampers exactly one field.
#[test]
fn gall_route_refuses_each_malformed_identity_field_exactly() {
    assert!(select_gall_route(&gall_request()).is_ok());

    let mut no_iri = gall_request();
    no_iri.identity.work_order_iri = "not-an-absolute-iri".into();
    assert_eq!(
        select_gall_route(&no_iri),
        Err(GallRouteRefusal::InvalidWorkOrderIri)
    );

    let mut no_checkpoint = gall_request();
    no_checkpoint.identity.checkpoint_iri = String::new();
    assert_eq!(
        select_gall_route(&no_checkpoint),
        Err(GallRouteRefusal::InvalidCheckpointIri)
    );

    let mut wrong_digest_scheme = gall_request();
    wrong_digest_scheme.identity.graph_digest = format!("md5:{}", "a".repeat(64));
    assert_eq!(
        select_gall_route(&wrong_digest_scheme),
        Err(GallRouteRefusal::InvalidGraphDigest)
    );

    let mut short_digest = gall_request();
    short_digest.identity.graph_digest = format!("sha256:{}", "a".repeat(63));
    assert_eq!(
        select_gall_route(&short_digest),
        Err(GallRouteRefusal::InvalidGraphDigest)
    );

    let mut no_repo = gall_request();
    no_repo.identity.repository_identity = "seanchatmangpt".into();
    assert_eq!(
        select_gall_route(&no_repo),
        Err(GallRouteRefusal::InvalidRepositoryIdentity)
    );

    let mut deep_repo = gall_request();
    deep_repo.identity.repository_identity = "owner/repo/extra".into();
    assert_eq!(
        select_gall_route(&deep_repo),
        Err(GallRouteRefusal::InvalidRepositoryIdentity)
    );
}

// non-vacuity: src/dme_route.rs:244-246 — deleting the
// `.map_err(GallRouteRefusal::Route)?` propagation (or replacing it with a
// panic/unwrap on the inner decision) makes the assert stop failing: the exact
// nested variant proves both that the outer wrapper fired and that the inner
// RequestNotAdmitted refusal was carried through unmodified.
#[test]
fn gall_route_proposes_inner_dme_refusal_through_route_variant_exactly() {
    let mut req = gall_request();
    req.route_request.standing = WorkStanding::Candidate; // inner DME refuses
    assert_eq!(
        select_gall_route(&req),
        Err(GallRouteRefusal::Route(DmeRouteRefusal::RequestNotAdmitted))
    );
}

// ---------------------------------------------------------------------------
// LensSelectionRefusal::QMagnitudeExceeded (src/allocator/mod.rs:2599-2609)
// ---------------------------------------------------------------------------

// non-vacuity: src/allocator/mod.rs:2609 — deleting the
// `q.to_bits().unsigned_abs() > MAX_LENS_MAGNITUDE << 16` guard (or mutating
// MAX_LENS_MAGNITUDE so the boundary moves past the shipped lens registry)
// makes the refusal assert stop failing; the happy-path control on the same
// fixture proves the registry itself is in-bounds, so only the injected
// magnitude triggers the refusal.
#[test]
fn single_lens_refuses_q_magnitude_beyond_admitted_bound_exactly() {
    let weights = [[NonNegativeFixed::ONE; 2 * Q]; N];
    let parent = [-1i32; N];

    // The shipped lens registry is inside the bound: lens 0 selects cleanly.
    assert!(allocate_single_lens(
        &bcinr_cmca::generated::consequence_mass::case_studies::OBJECT_REGISTRY,
        &LENS_REGISTRY,
        0,
        0,
        &parent,
        &weights
    )
    .is_ok());

    // 17.0 in Q16.16 exceeds MAX_LENS_MAGNITUDE = 16 (16 << 16 = 1_048_576 bits).
    let out_of_bound = 17 * 65_536;
    let mut inflated = LENS_REGISTRY;
    inflated[0].q = SignedFixed {
        val: out_of_bound,
        err: u32::MAX,
    };
    assert_eq!(
        allocate_single_lens(
            &bcinr_cmca::generated::consequence_mass::case_studies::OBJECT_REGISTRY,
            &inflated,
            0,
            0,
            &parent,
            &weights
        ),
        Err(LensRefusal::QMagnitudeExceeded {
            q: SignedFixed {
                val: out_of_bound,
                err: u32::MAX
            }
        })
    );
    let _ = LensSelectionRefusal::MeasureIndexOutOfRange { measure: 0 }; // keep root re-export referenced
}

// ---------------------------------------------------------------------------
// StabilityRefusal wire-code codec (src/allocator/mod.rs:415-487)
// ---------------------------------------------------------------------------

// The u32 refusal codes cross receipt boundaries; the from_u32 table IS the
// law that a code and its typed variant mean the same thing on both sides.
// non-vacuity: src/allocator/mod.rs:415-455 — swapping any two entries of the
// from_u32 lookup (e.g. LearningFrozen with NumericRangeExceeded), dropping an
// entry (shifting later codes by one), or widening the in-bounds clamp past 22
// makes the corresponding exact-mapping assert stop failing. This pins the
// codec law only: production-path refusals for codes 0-21 are exercised by the
// allocate/observatory suites where reachable (see gap report).
#[test]
fn stability_refusal_wire_codes_map_to_exact_variants() {
    let expected: [(u32, StabilityRefusal); 22] = [
        (0, StabilityRefusal::CertificateMissing),
        (1, StabilityRefusal::BlockGainBoundExceeded),
        (2, StabilityRefusal::ContractionMarginInsufficient),
        (3, StabilityRefusal::LearningRateOutsideEnvelope),
        (4, StabilityRefusal::ModeDwellTimeViolated),
        (5, StabilityRefusal::QRangeDestabilizing),
        (6, StabilityRefusal::MassClampUnsafe),
        (7, StabilityRefusal::PriceGainUnsafe),
        (8, StabilityRefusal::StandingProjectionGainUnsafe),
        (9, StabilityRefusal::RuntimeEnvelopeViolated),
        (10, StabilityRefusal::CertificateDigestMismatch),
        (11, StabilityRefusal::ControlModeUncertified),
        (12, StabilityRefusal::ControlModeSwitchTooFast),
        (13, StabilityRefusal::YieldGainBoundViolated),
        (14, StabilityRefusal::RewardBoundViolated),
        (15, StabilityRefusal::ResourceResponseBoundViolated),
        (16, StabilityRefusal::StandingResetBoundViolated),
        (17, StabilityRefusal::LearningFrozen),
        (18, StabilityRefusal::NumericRangeExceeded),
        (19, StabilityRefusal::UnsupportedDomain),
        (20, StabilityRefusal::ContractViolation),
        (21, StabilityRefusal::ExploreFloorOutsideEnvelope),
    ];
    for (code, variant) in expected {
        assert_eq!(
            StabilityRefusal::from_u32(code),
            Some(variant),
            "code {code}"
        );
    }
    assert_eq!(StabilityRefusal::from_u32(22), None);
    assert_eq!(StabilityRefusal::from_u32(31), None);
    assert_eq!(StabilityRefusal::from_u32(u32::MAX), None);
}

// Keep the root-level re-export binding pinned: lib.rs re-exports
// StabilityRefusal from allocator; a rename there must break this file.
// non-vacuity: src/lib.rs:150 — deleting either re-exported name from the
// `pub use allocator::{...}` list fails this file's compilation.
#[test]
fn root_reexports_resolve_to_allocator_types() {
    let _both: (StabilityRefusal, AuthorityStanding) = (
        StabilityRefusal::CertificateMissing,
        AuthorityStanding::None,
    );
}
