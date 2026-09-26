//! Canonical DME route-selection contract for Chatman Multifractal Cascade Allocation.
//!
//! This module is deliberately a selector, not an executor. It maps an admitted
//! work package plus a finite policy/resource profile to one content-addressed
//! route decision. It grants no authority and performs no consequence.
//!
//! # Canonical identity (hardening, v26.9.26)
//!
//! The decision digest binds the *canonical* request: the candidate route list is
//! sorted by a total key and exact duplicates are collapsed before selection and
//! before hashing. Consequently:
//!
//! * reordering the candidate list (transport reordering) cannot change the decision
//!   or its digest;
//! * delivering the same candidate twice (duplicate delivery) cannot change the
//!   decision or its digest;
//! * changing any semantic input (consequence class, deadline class, evidence
//!   obligation, frontier admission, any candidate field) changes the digest, so a
//!   decision cannot be replayed against a request it was not computed from.

extern crate alloc;
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use serde::{Deserialize, Serialize};

/// Admission standing of the work package presented for routing.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkStanding {
    /// Admitted by an upstream court; the only standing the selector optimizes.
    Admitted,
    /// Proposed but not admitted; refused before optimization.
    Candidate,
    /// Explicitly refused upstream; refused before optimization.
    Refused,
}

/// Whether the work class is covered by admitted deterministic machinery.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkKnowledge {
    /// KNOWN: only deterministic machinery is eligible.
    Known,
    /// UNKNOWN: exploratory routes are eligible under finite bounds.
    Unknown,
}

/// Declared consequence class of the work package.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConsequenceClass {
    /// Read-only observation.
    Observe,
    /// Constructs a candidate artifact.
    Construct,
    /// Changes admitted state (still executed elsewhere, never here).
    Change,
    /// External DO (still executed elsewhere, never here).
    ExternalDo,
    /// Unclassified consequence; refused before optimization.
    Unknown,
}

/// Bounded execution class a route belongs to.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RouteClass {
    /// Admitted deterministic machinery.
    KnownDeterministic,
    /// Local exploratory estate.
    UnknownLocal,
    /// Idle-estate exploratory capacity.
    UnknownIdleEstate,
    /// Frontier escalation; eligible only when explicitly admitted.
    UnknownFrontier,
    /// A route that is never selectable.
    Refused,
}

/// Authority carried by a route decision. There is exactly one inhabitant.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthorityStanding {
    /// The decision grants no authority.
    None,
}

/// One candidate route with its finite resource/evidence fit.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteCandidate {
    /// Execution class of the route.
    pub route: RouteClass,
    /// Selection cost (lower is preferred).
    pub cost_units: u64,
    /// Capability profile covers the work.
    pub capability_fit: bool,
    /// Budget available on this route.
    pub budget_units: u64,
    /// Budget the work requires on this route.
    pub required_units: u64,
    /// Route can discharge the evidence obligation.
    pub evidence_fit: bool,
    /// Route is permitted for the declared consequence class.
    pub consequence_fit: bool,
}

impl RouteCandidate {
    /// A route is lawful iff every fit holds and the requirement fits the budget.
    pub fn lawful(&self) -> bool {
        self.capability_fit
            && self.evidence_fit
            && self.consequence_fit
            && self.required_units <= self.budget_units
    }

    fn canonical_key(&self) -> (u8, u64, u64, u64, bool, bool, bool) {
        (
            route_rank(self.route),
            self.cost_units,
            self.budget_units,
            self.required_units,
            self.capability_fit,
            self.evidence_fit,
            self.consequence_fit,
        )
    }
}

/// An admitted work package presented for route selection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DmeRouteRequest {
    /// Request identity; must be non-blank.
    pub request_id: String,
    /// Semantic subject; must be non-blank.
    pub semantic_subject: String,
    /// Admission standing.
    pub standing: WorkStanding,
    /// KNOWN/UNKNOWN classification.
    pub knowledge: WorkKnowledge,
    /// Declared consequence class.
    pub consequence: ConsequenceClass,
    /// Deadline class (bound into the decision digest).
    pub deadline_class: String,
    /// Evidence obligation (bound into the decision digest).
    pub evidence_obligation: String,
    /// Whether frontier escalation is admitted for this request.
    pub frontier_escalation_admitted: bool,
    /// Candidate routes (order and exact duplicates are not semantic).
    pub routes: Vec<RouteCandidate>,
}

/// Why the selected route was chosen and which classes were refused.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteExplanation {
    /// Cost of the selected candidate.
    pub selected_cost_units: Option<u64>,
    /// Distinct route classes presented, in canonical rank order.
    pub considered: Vec<RouteClass>,
    /// Distinct route classes refused, in canonical rank order.
    pub refused: Vec<RouteClass>,
    /// Human-readable reason.
    pub reason: String,
}

/// A sealed, content-addressed route decision.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DmeRouteDecision {
    /// Request identity the decision was computed for.
    pub request_id: String,
    /// Semantic subject the decision was computed for.
    pub semantic_subject: String,
    /// Selected route class.
    pub route: RouteClass,
    /// Always [`AuthorityStanding::None`].
    pub authority: AuthorityStanding,
    /// Selection explanation.
    pub explanation: RouteExplanation,
    /// BLAKE3 hex digest over the canonical request and the decision body.
    pub decision_digest: String,
}

/// Typed refusal of the selector.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DmeRouteRefusal {
    /// Work standing is not `Admitted`.
    RequestNotAdmitted,
    /// Request identity is blank.
    InvalidRequestId,
    /// Semantic subject is blank.
    InvalidSemanticSubject,
    /// Consequence class is `Unknown`.
    UnknownConsequenceClass,
    /// KNOWN work with no lawful deterministic route.
    KnownWithoutDeterministicRoute,
    /// UNKNOWN work with no lawful admitted route.
    UnknownWithoutLawfulRoute,
}
impl core::fmt::Display for DmeRouteRefusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
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

/// Selection order: least cost first, then class rank, then the canonical key
/// (a total order, so ties are resolved identically under any input order).
fn selection_key(c: &RouteCandidate) -> (u64, u8, (u8, u64, u64, u64, bool, bool, bool)) {
    (c.cost_units, route_rank(c.route), c.canonical_key())
}

fn canonical_classes(mut classes: Vec<RouteClass>) -> Vec<RouteClass> {
    classes.sort_by_key(|route| route_rank(*route));
    classes.dedup();
    classes
}

/// Canonical candidate list: sorted by a total key, exact duplicates collapsed.
fn canonical_routes(routes: &[RouteCandidate]) -> Vec<RouteCandidate> {
    let mut routes = routes.to_vec();
    routes.sort_by_key(RouteCandidate::canonical_key);
    routes.dedup();
    routes
}

fn canonical_digest<T: Serialize>(value: &T) -> String {
    let bytes = serde_json::to_vec(value).expect("DME route decision serialization is infallible");
    blake3::hash(&bytes).to_hex().to_string()
}

#[derive(Serialize)]
struct CanonicalRequest<'a> {
    request_id: &'a str,
    semantic_subject: &'a str,
    standing: WorkStanding,
    knowledge: WorkKnowledge,
    consequence: ConsequenceClass,
    deadline_class: &'a str,
    evidence_obligation: &'a str,
    frontier_escalation_admitted: bool,
    routes: &'a [RouteCandidate],
}

#[derive(Serialize)]
struct DecisionBody<'a> {
    domain: &'static str,
    request: CanonicalRequest<'a>,
    route: RouteClass,
    authority: AuthorityStanding,
    explanation: &'a RouteExplanation,
}

const DECISION_DOMAIN: &str = "bcinr-cmca/dme-route-decision/v2";

fn seal(
    request: &DmeRouteRequest,
    routes: &[RouteCandidate],
    route: RouteClass,
    explanation: RouteExplanation,
) -> DmeRouteDecision {
    let authority = AuthorityStanding::None;
    let body = DecisionBody {
        domain: DECISION_DOMAIN,
        request: CanonicalRequest {
            request_id: &request.request_id,
            semantic_subject: &request.semantic_subject,
            standing: request.standing,
            knowledge: request.knowledge,
            consequence: request.consequence,
            deadline_class: &request.deadline_class,
            evidence_obligation: &request.evidence_obligation,
            frontier_escalation_admitted: request.frontier_escalation_admitted,
            routes,
        },
        route,
        authority,
        explanation: &explanation,
    };
    let decision_digest = canonical_digest(&body);
    DmeRouteDecision {
        request_id: request.request_id.clone(),
        semantic_subject: request.semantic_subject.clone(),
        route,
        authority,
        explanation,
        decision_digest,
    }
}

/// SELECT the least-cost lawful route. This function grants no authority and performs no DO.
pub fn select_dme_route(request: &DmeRouteRequest) -> Result<DmeRouteDecision, DmeRouteRefusal> {
    if request.standing != WorkStanding::Admitted {
        return Err(DmeRouteRefusal::RequestNotAdmitted);
    }
    if request.request_id.trim().is_empty() {
        return Err(DmeRouteRefusal::InvalidRequestId);
    }
    if request.semantic_subject.trim().is_empty() {
        return Err(DmeRouteRefusal::InvalidSemanticSubject);
    }
    if request.consequence == ConsequenceClass::Unknown {
        return Err(DmeRouteRefusal::UnknownConsequenceClass);
    }

    let routes = canonical_routes(&request.routes);
    let considered = canonical_classes(routes.iter().map(|c| c.route).collect());

    match request.knowledge {
        WorkKnowledge::Known => {
            let selected = routes
                .iter()
                .filter(|c| c.route == RouteClass::KnownDeterministic && c.lawful())
                .min_by_key(|c| selection_key(c))
                .ok_or(DmeRouteRefusal::KnownWithoutDeterministicRoute)?;
            let refused = canonical_classes(
                routes
                    .iter()
                    .filter(|c| c.route != RouteClass::KnownDeterministic)
                    .map(|c| c.route)
                    .collect(),
            );
            let explanation = RouteExplanation {
                selected_cost_units: Some(selected.cost_units),
                considered,
                refused,
                reason: "KNOWN work selected admitted deterministic machinery; model routes are ineligible".into(),
            };
            Ok(seal(
                request,
                &routes,
                RouteClass::KnownDeterministic,
                explanation,
            ))
        }
        WorkKnowledge::Unknown => {
            let eligible = |c: &&RouteCandidate| {
                c.route != RouteClass::KnownDeterministic
                    && c.route != RouteClass::Refused
                    && (c.route != RouteClass::UnknownFrontier
                        || request.frontier_escalation_admitted)
                    && c.lawful()
            };
            let selected = routes
                .iter()
                .filter(eligible)
                .min_by_key(|c| selection_key(c))
                .ok_or(DmeRouteRefusal::UnknownWithoutLawfulRoute)?;
            let refused = canonical_classes(
                routes
                    .iter()
                    .filter(|c| !eligible(c))
                    .map(|c| c.route)
                    .collect(),
            );
            let explanation = RouteExplanation {
                selected_cost_units: Some(selected.cost_units),
                considered,
                refused,
                reason: "UNKNOWN work selected the least-cost lawful admitted route under finite resource/evidence bounds".into(),
            };
            Ok(seal(request, &routes, selected.route, explanation))
        }
    }
}

/// Replay check: recompute the decision from `request` and compare exactly
/// (route, explanation and digest). A decision sealed for a different request,
/// or with any tampered field, fails.
pub fn verify_dme_route_decision(request: &DmeRouteRequest, decision: &DmeRouteDecision) -> bool {
    select_dme_route(request)
        .map(|expected| expected == *decision)
        .unwrap_or(false)
}
