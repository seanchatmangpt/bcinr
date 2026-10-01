//! Canonical DME route-selection contract for Chatman Multifractal Cascade Allocation.
//!
//! This module is deliberately a selector, not an executor. It maps an admitted
//! work package plus a finite policy/resource profile to one content-addressed
//! route decision. It grants no authority and performs no consequence.

extern crate alloc;
use alloc::{string::{String, ToString}, vec::Vec};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkStanding { Admitted, Candidate, Refused }

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkKnowledge { Known, Unknown }

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConsequenceClass { Observe, Construct, Change, ExternalDo, Unknown }

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RouteClass { KnownDeterministic, UnknownLocal, UnknownIdleEstate, UnknownFrontier, Refused }

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthorityStanding { None }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteCandidate {
    pub route: RouteClass,
    pub cost_units: u64,
    pub capability_fit: bool,
    pub budget_units: u64,
    pub required_units: u64,
    pub evidence_fit: bool,
    pub consequence_fit: bool,
}
impl RouteCandidate {
    pub fn lawful(&self) -> bool {
        self.capability_fit && self.evidence_fit && self.consequence_fit && self.required_units <= self.budget_units
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DmeRouteRequest {
    pub request_id: String,
    pub semantic_subject: String,
    pub standing: WorkStanding,
    pub knowledge: WorkKnowledge,
    pub consequence: ConsequenceClass,
    pub deadline_class: String,
    pub evidence_obligation: String,
    pub frontier_escalation_admitted: bool,
    pub routes: Vec<RouteCandidate>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteExplanation {
    pub selected_cost_units: Option<u64>,
    pub considered: Vec<RouteClass>,
    pub refused: Vec<RouteClass>,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DmeRouteDecision {
    pub request_id: String,
    pub semantic_subject: String,
    pub route: RouteClass,
    pub authority: AuthorityStanding,
    pub explanation: RouteExplanation,
    pub decision_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DmeRouteRefusal {
    RequestNotAdmitted,
    InvalidSemanticSubject,
    UnknownConsequenceClass,
    KnownWithoutDeterministicRoute,
    UnknownWithoutLawfulRoute,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GallWorkIdentity {
    pub work_order_iri: String,
    pub checkpoint_iri: String,
    pub graph_digest: String,
    pub repository_identity: String,
    pub base_sha: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GallRouteRequest {
    pub identity: GallWorkIdentity,
    pub route_request: DmeRouteRequest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GallRouteDecision {
    pub identity: GallWorkIdentity,
    pub route_decision: DmeRouteDecision,
    pub authority: AuthorityStanding,
    pub decision_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GallRouteRefusal {
    InvalidWorkOrderIri,
    InvalidCheckpointIri,
    InvalidGraphDigest,
    InvalidRepositoryIdentity,
    InvalidBaseSha,
    SubjectMismatch,
    Route(DmeRouteRefusal),
}

impl core::fmt::Display for DmeRouteRefusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { core::fmt::Debug::fmt(self, f) }
}
#[cfg(feature = "std")]
impl std::error::Error for DmeRouteRefusal {}

fn route_rank(route: RouteClass) -> u8 {
    match route {
        RouteClass::KnownDeterministic => 0,
        RouteClass::UnknownLocal => 1,
        RouteClass::UnknownIdleEstate => 2,
        RouteClass::UnknownFrontier => 3,
        RouteClass::Refused => 4,
    }
}
fn canonical_digest<T: Serialize>(value: &T) -> String {
    let bytes = serde_json::to_vec(value).expect("DME route decision serialization is infallible");
    blake3::hash(&bytes).to_hex().to_string()
}
#[derive(Serialize)]
struct DecisionBody<'a> {
    request_id: &'a str,
    semantic_subject: &'a str,
    route: RouteClass,
    authority: AuthorityStanding,
    explanation: &'a RouteExplanation,
}
fn seal(request: &DmeRouteRequest, route: RouteClass, explanation: RouteExplanation) -> DmeRouteDecision {
    let authority = AuthorityStanding::None;
    let body = DecisionBody { request_id: &request.request_id, semantic_subject: &request.semantic_subject, route, authority, explanation: &explanation };
    let decision_digest = canonical_digest(&body);
    DmeRouteDecision { request_id: request.request_id.clone(), semantic_subject: request.semantic_subject.clone(), route, authority, explanation, decision_digest }
}

/// SELECT the least-cost lawful route. This function grants no authority and performs no DO.
pub fn select_dme_route(request: &DmeRouteRequest) -> Result<DmeRouteDecision, DmeRouteRefusal> {
    if request.standing != WorkStanding::Admitted { return Err(DmeRouteRefusal::RequestNotAdmitted); }
    if request.semantic_subject.trim().is_empty() { return Err(DmeRouteRefusal::InvalidSemanticSubject); }
    if request.consequence == ConsequenceClass::Unknown { return Err(DmeRouteRefusal::UnknownConsequenceClass); }

    let mut considered = request.routes.iter().map(|c| c.route).collect::<Vec<_>>();
    considered.sort_by_key(|route| route_rank(*route));
    considered.dedup();

    match request.knowledge {
        WorkKnowledge::Known => {
            let selected = request.routes.iter()
                .filter(|c| c.route == RouteClass::KnownDeterministic && c.lawful())
                .min_by_key(|c| (c.cost_units, route_rank(c.route)))
                .ok_or(DmeRouteRefusal::KnownWithoutDeterministicRoute)?;
            let refused = request.routes.iter().filter(|c| c.route != RouteClass::KnownDeterministic).map(|c| c.route).collect();
            Ok(seal(request, RouteClass::KnownDeterministic, RouteExplanation {
                selected_cost_units: Some(selected.cost_units), considered, refused,
                reason: "KNOWN work selected admitted deterministic machinery; model routes are ineligible".into(),
            }))
        }
        WorkKnowledge::Unknown => {
            let selected = request.routes.iter()
                .filter(|c| c.route != RouteClass::KnownDeterministic && c.route != RouteClass::Refused)
                .filter(|c| c.route != RouteClass::UnknownFrontier || request.frontier_escalation_admitted)
                .filter(|c| c.lawful())
                .min_by_key(|c| (c.cost_units, route_rank(c.route)))
                .ok_or(DmeRouteRefusal::UnknownWithoutLawfulRoute)?;
            let refused = request.routes.iter()
                .filter(|c| c.route == RouteClass::KnownDeterministic || !c.lawful() || (c.route == RouteClass::UnknownFrontier && !request.frontier_escalation_admitted))
                .map(|c| c.route).collect();
            Ok(seal(request, selected.route, RouteExplanation {
                selected_cost_units: Some(selected.cost_units), considered, refused,
                reason: "UNKNOWN work selected the least-cost lawful admitted route under finite resource/evidence bounds".into(),
            }))
        }
    }
}

pub fn verify_dme_route_decision(request: &DmeRouteRequest, decision: &DmeRouteDecision) -> bool {
    select_dme_route(request).map(|expected| expected == *decision).unwrap_or(false)
}

fn absolute_iri(value: &str) -> bool {
    !value.is_empty() && value.contains(':')
}

fn sha256_digest(value: &str) -> bool {
    let Some(("sha256", hex)) = value.split_once(':') else { return false; };
    hex.len() == 64 && hex.bytes().all(|b| b.is_ascii_hexdigit())
}

fn git_sha(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

fn repo_identity(value: &str) -> bool {
    let mut parts = value.split('/');
    matches!(
        (parts.next(), parts.next(), parts.next()),
        (Some(owner), Some(repo), None) if !owner.is_empty() && !repo.is_empty()
    )
}

/// Compose the existing bounded selector with exact GALL work-order identity.
///
/// This adds no new optimizer and no execution path. The inner CMCA decision
/// remains authoritative for route selection; the outer receipt only binds
/// that SELECT result to the exact semantic subject.
pub fn select_gall_route(request: &GallRouteRequest) -> Result<GallRouteDecision, GallRouteRefusal> {
    let id = &request.identity;

    if !absolute_iri(&id.work_order_iri) {
        return Err(GallRouteRefusal::InvalidWorkOrderIri);
    }
    if !absolute_iri(&id.checkpoint_iri) {
        return Err(GallRouteRefusal::InvalidCheckpointIri);
    }
    if !sha256_digest(&id.graph_digest) {
        return Err(GallRouteRefusal::InvalidGraphDigest);
    }
    if !repo_identity(&id.repository_identity) {
        return Err(GallRouteRefusal::InvalidRepositoryIdentity);
    }
    if !git_sha(&id.base_sha) {
        return Err(GallRouteRefusal::InvalidBaseSha);
    }
    if request.route_request.semantic_subject != id.work_order_iri {
        return Err(GallRouteRefusal::SubjectMismatch);
    }

    let route_decision =
        select_dme_route(&request.route_request).map_err(GallRouteRefusal::Route)?;
    let authority = AuthorityStanding::None;

    #[derive(Serialize)]
    struct GallDecisionBody<'a> {
        identity: &'a GallWorkIdentity,
        route_decision: &'a DmeRouteDecision,
        authority: AuthorityStanding,
    }

    let body = GallDecisionBody {
        identity: id,
        route_decision: &route_decision,
        authority,
    };
    let decision_digest = canonical_digest(&body);

    Ok(GallRouteDecision {
        identity: id.clone(),
        route_decision,
        authority,
        decision_digest,
    })
}

pub fn verify_gall_route_decision(
    request: &GallRouteRequest,
    decision: &GallRouteDecision,
) -> bool {
    select_gall_route(request)
        .map(|expected| expected == *decision)
        .unwrap_or(false)
}

