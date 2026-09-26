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
    /// Deferred exploration: the work is parked for a later bounded epoch instead of
    /// being expanded now (DME class `UNKNOWN_DEFERRED`).
    UnknownDeferred,
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

    fn canonical_key(&self) -> CanonicalKey {
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

/// Typed exhaustion witness: why no candidate route was lawful.
///
/// Counts are over the *canonical* candidate set (sorted, exact duplicates
/// collapsed), so the witness is invariant under transport reordering and duplicate
/// delivery. Exhaustion never widens a budget: `max_budget_shortfall_units` reports
/// how far the largest requirement exceeds its admitted budget, and the only lawful
/// response is a new upstream admission, never a self-granted increase.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DmeExhaustionWitness {
    /// Distinct canonical candidates presented.
    pub candidates: u64,
    /// Candidates whose route class is ineligible for the work's knowledge class
    /// (including `Refused`, and any class other than `KnownDeterministic` for KNOWN work).
    pub class_ineligible: u64,
    /// Frontier candidates presented while frontier escalation is not admitted.
    pub frontier_unadmitted: u64,
    /// Class-eligible candidates without capability fit.
    pub capability_unfit: u64,
    /// Class-eligible candidates without evidence fit.
    pub evidence_unfit: u64,
    /// Class-eligible candidates without consequence fit.
    pub consequence_unfit: u64,
    /// Class-eligible candidates whose requirement exceeds their budget.
    pub budget_exhausted: u64,
    /// Candidates lawful in every respect except the budget or the frontier
    /// admission: exhaustion that only a new upstream admission can lift.
    pub blocked_only_by_bound: u64,
    /// Largest `required_units - budget_units` over class-eligible candidates.
    pub max_budget_shortfall_units: u64,
}

/// Typed refusal of the selector.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DmeRouteRefusal {
    /// Work standing is not `Admitted`.
    RequestNotAdmitted,
    /// Request identity is blank.
    InvalidRequestId,
    /// Semantic subject is blank.
    InvalidSemanticSubject,
    /// Consequence class is `Unknown`.
    UnknownConsequenceClass,
    /// KNOWN work with no lawful deterministic route; carries the exhaustion witness.
    KnownWithoutDeterministicRoute(DmeExhaustionWitness),
    /// UNKNOWN work with no lawful admitted route; carries the exhaustion witness.
    UnknownWithoutLawfulRoute(DmeExhaustionWitness),
}

impl DmeRouteRefusal {
    /// The exhaustion witness, when the refusal is a bound/fit exhaustion rather
    /// than an admission refusal.
    pub fn exhaustion(&self) -> Option<&DmeExhaustionWitness> {
        match self {
            Self::KnownWithoutDeterministicRoute(w) | Self::UnknownWithoutLawfulRoute(w) => Some(w),
            _ => None,
        }
    }
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
        RouteClass::UnknownDeferred => 3,
        RouteClass::UnknownFrontier => 4,
        RouteClass::Refused => 5,
    }
}

/// Total canonical key of a candidate (class rank, cost, budget, requirement, fits).
type CanonicalKey = (u8, u64, u64, u64, bool, bool, bool);
/// Selection key: cost, class rank, canonical key.
type SelectionKey = (u64, u8, CanonicalKey);

/// Selection order: least cost first, then class rank, then the canonical key
/// (a total order, so ties are resolved identically under any input order).
fn selection_key(c: &RouteCandidate) -> SelectionKey {
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

const DECISION_DOMAIN: &str = "bcinr-cmca/dme-route-decision/v3";

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

    let class_eligible = |c: &RouteCandidate| class_eligible(request, c);
    let eligible = |c: &RouteCandidate| class_eligible(c) && c.lawful();
    let selected = routes
        .iter()
        .filter(|c| eligible(c))
        .min_by_key(|c| selection_key(c))
        .ok_or_else(|| {
            let witness = exhaustion_witness(request, &routes);
            match request.knowledge {
                WorkKnowledge::Known => DmeRouteRefusal::KnownWithoutDeterministicRoute(witness),
                WorkKnowledge::Unknown => DmeRouteRefusal::UnknownWithoutLawfulRoute(witness),
            }
        })?;
    // A class is refused only when no candidate of that class is eligible, so the
    // selected class can never also appear as refused.
    let refused = canonical_classes(
        routes
            .iter()
            .filter(|c| !eligible(c))
            .map(|c| c.route)
            .filter(|class| !routes.iter().any(|c| c.route == *class && eligible(c)))
            .collect(),
    );
    let reason = match request.knowledge {
        WorkKnowledge::Known => {
            "KNOWN work selected admitted deterministic machinery; model routes are ineligible"
        }
        WorkKnowledge::Unknown => {
            "UNKNOWN work selected the least-cost lawful admitted route under finite resource/evidence bounds"
        }
    };
    let explanation = RouteExplanation {
        selected_cost_units: Some(selected.cost_units),
        considered,
        refused,
        reason: reason.into(),
    };
    Ok(seal(request, &routes, selected.route, explanation))
}

/// Whether a candidate's route class may serve the request's knowledge class.
fn class_eligible(request: &DmeRouteRequest, c: &RouteCandidate) -> bool {
    match request.knowledge {
        WorkKnowledge::Known => c.route == RouteClass::KnownDeterministic,
        WorkKnowledge::Unknown => {
            c.route != RouteClass::KnownDeterministic
                && c.route != RouteClass::Refused
                && (c.route != RouteClass::UnknownFrontier || request.frontier_escalation_admitted)
        }
    }
}

fn exhaustion_witness(
    request: &DmeRouteRequest,
    routes: &[RouteCandidate],
) -> DmeExhaustionWitness {
    let mut w = DmeExhaustionWitness {
        candidates: routes.len() as u64,
        ..DmeExhaustionWitness::default()
    };
    for c in routes {
        let frontier_gated = request.knowledge == WorkKnowledge::Unknown
            && c.route == RouteClass::UnknownFrontier
            && !request.frontier_escalation_admitted;
        if frontier_gated {
            w.frontier_unadmitted += 1;
        }
        let fits = c.capability_fit && c.evidence_fit && c.consequence_fit;
        if !class_eligible(request, c) {
            w.class_ineligible += 1;
            if frontier_gated && fits && c.required_units <= c.budget_units {
                w.blocked_only_by_bound += 1;
            }
            continue;
        }
        w.capability_unfit += u64::from(!c.capability_fit);
        w.evidence_unfit += u64::from(!c.evidence_fit);
        w.consequence_unfit += u64::from(!c.consequence_fit);
        if c.required_units > c.budget_units {
            w.budget_exhausted += 1;
            w.max_budget_shortfall_units = w
                .max_budget_shortfall_units
                .max(c.required_units - c.budget_units);
            if fits {
                w.blocked_only_by_bound += 1;
            }
        }
    }
    w
}

/// DME work class of a routed request (RFC closure item 2).
///
/// None of these classes carries execution authority; the classification is a
/// SELECT-level description of what the substrate may do next.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DmeWorkClass {
    /// KNOWN work routed to admitted deterministic machinery.
    Known,
    /// UNKNOWN work routed to local (or idle-estate) exploration.
    UnknownLocal,
    /// UNKNOWN work deferred to a later bounded epoch.
    UnknownDeferred,
    /// UNKNOWN work escalated to an admitted frontier route.
    UnknownFrontier,
    /// Refused: not admitted, malformed, unclassified consequence, or no lawful route
    /// for a reason other than a bound or a missing capability.
    Refused,
    /// Blocked: a route would be lawful except for a finite bound (budget) or a
    /// missing frontier admission; only a new upstream admission can lift it.
    Blocked,
    /// Unsupported: no presented route has the capability to serve the work.
    Unsupported,
}

/// A work classification: the class plus the decision or the typed refusal behind it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DmeWorkClassification {
    /// DME work class.
    pub class: DmeWorkClass,
    /// The sealed route decision, when a route was selected.
    pub decision: Option<DmeRouteDecision>,
    /// The typed refusal (with exhaustion witness where applicable), otherwise.
    pub refusal: Option<DmeRouteRefusal>,
    /// Always [`AuthorityStanding::None`].
    pub authority: AuthorityStanding,
}

/// Classify a request into one of the seven DME work classes. SELECT only.
pub fn classify_dme_work(request: &DmeRouteRequest) -> DmeWorkClassification {
    match select_dme_route(request) {
        Ok(decision) => {
            let class = match decision.route {
                RouteClass::KnownDeterministic => DmeWorkClass::Known,
                RouteClass::UnknownLocal | RouteClass::UnknownIdleEstate => {
                    DmeWorkClass::UnknownLocal
                }
                RouteClass::UnknownDeferred => DmeWorkClass::UnknownDeferred,
                RouteClass::UnknownFrontier => DmeWorkClass::UnknownFrontier,
                // Unreachable by construction (Refused is never eligible); classified
                // as Refused rather than panicking.
                RouteClass::Refused => DmeWorkClass::Refused,
            };
            DmeWorkClassification {
                class,
                decision: Some(decision),
                refusal: None,
                authority: AuthorityStanding::None,
            }
        }
        Err(refusal) => {
            let class = match refusal.exhaustion() {
                None => DmeWorkClass::Refused,
                Some(w) => {
                    let eligible = w.candidates - w.class_ineligible;
                    if w.blocked_only_by_bound > 0 {
                        DmeWorkClass::Blocked
                    } else if eligible == w.capability_unfit {
                        DmeWorkClass::Unsupported
                    } else {
                        DmeWorkClass::Refused
                    }
                }
            };
            DmeWorkClassification {
                class,
                decision: None,
                refusal: Some(refusal),
                authority: AuthorityStanding::None,
            }
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
