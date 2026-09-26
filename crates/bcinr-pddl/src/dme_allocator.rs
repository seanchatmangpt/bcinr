//! DME allocator-facing interface (RFC closure item 1, A2A-2605).
//!
//! One stable surface over the machinery this crate already owns, composed into the
//! canonical CMCA route selector (`bcinr_cmca::select_dme_route`) instead of a second
//! selector:
//!
//! * [`CostVector`] (lexicographic, admitted-first) orders rails; a non-admitted cost
//!   vector removes the rail's capability fit.
//! * [`MassVector`] / [`FrontierBoxes`] / [`q_lens`] weight rails with equal cost rank:
//!   the higher `L_q` frontier mass wins the tie.
//! * [`FairRailScheduler`] produces the bounded exploit/exact tick schedule for
//!   exploratory work, so the exact rail is never starved beyond `max_gap`.
//! * [`ConsequenceHorizon`] identity is bound into the evidence obligation and so into
//!   the sealed decision digest.
//!
//! Laws kept here (RFC Chicago falsifiers):
//!
//! * Candidate PDDL/domain text never reaches a route: [`WorkAdmission::Candidate`] is
//!   routed with standing `Candidate` and refused; the only way to an admitted package
//!   is [`admit_domain_work`] (the real `admit_candidate_domain` gate).
//! * Requesting more search mass never raises a budget: every rail carries the
//!   profile's admitted budget unchanged; over-budget demand is classified `BLOCKED`
//!   with an exhaustion witness.
//! * Hard ceilings: fan-out ([`MAX_ALLOCATOR_RAILS`]), budget
//!   ([`MAX_ALLOCATOR_BUDGET_UNITS`]) and schedule length
//!   ([`MAX_ALLOCATOR_SCHEDULE_TICKS`]) refuse with a typed [`BoundHit`].
//! * Nothing here carries authority: SELECT only, never DO.

use bcinr_cmca::{
    classify_dme_work, AuthorityStanding, ConsequenceClass, DmeRouteRequest, DmeWorkClass,
    DmeWorkClassification, RouteCandidate, RouteClass, WorkKnowledge, WorkStanding,
};
use bcinr_mfw_ir::{BoundHit, BoundKind, ConsequenceHorizonId};

use crate::capability_router::CostVector;
use crate::consequence::ConsequenceHorizon;
use crate::error::Pddl8Error;
use crate::llm_bridge::{admit_candidate_domain, AdmittedDomain};
use crate::mfw::{q_lens, FrontierBoxes, MassVector, QLensError, QValue};
use crate::search::{FairRailScheduler, RailSelection};

/// Hard fan-out ceiling: rails presented to one allocation.
pub const MAX_ALLOCATOR_RAILS: usize = 64;
/// Hard ceiling on the admitted budget of one allocation.
pub const MAX_ALLOCATOR_BUDGET_UNITS: u64 = 1 << 32;
/// Hard ceiling on the fair-rail schedule length of one allocation.
pub const MAX_ALLOCATOR_SCHEDULE_TICKS: usize = 1024;
/// Resolution of the q-lens weight inside one cost rank.
const WEIGHT_SCALE: u64 = 1_000_000;

/// Admission witness of a work package. Constructible only from an admitted domain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdmittedWork {
    witness: String,
}

impl AdmittedWork {
    /// Bind an already-admitted domain.
    pub fn from_admitted_domain(domain: &AdmittedDomain) -> Self {
        Self {
            witness: domain.witness.clone(),
        }
    }

    /// The admission witness (BLAKE3 hex of the admitted domain structure).
    pub fn witness(&self) -> &str {
        &self.witness
    }
}

/// Admission of the work presented for allocation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkAdmission {
    /// Admitted through the substrate's admission gate.
    Admitted(AdmittedWork),
    /// Candidate text (e.g. LLM-proposed PDDL). Never routed.
    Candidate { text: String },
}

/// Run candidate domain text through the real admission gate.
pub fn admit_domain_work(text: &str) -> Result<WorkAdmission, Pddl8Error> {
    let domain = admit_candidate_domain(text)?;
    Ok(WorkAdmission::Admitted(AdmittedWork::from_admitted_domain(
        &domain,
    )))
}

/// Finite allocation profile. The budget is admitted upstream; nothing here raises it.
#[derive(Clone, Copy, Debug)]
pub struct AllocatorProfile {
    /// q-lens exponent for frontier-mass weighting.
    pub q: QValue,
    /// Admitted budget carried by every rail.
    pub admitted_budget_units: u64,
    /// Fair-rail fairness floor: exact rail at least every `max_gap` ticks.
    pub fair_rail_max_gap: usize,
    /// Length of the fair-rail schedule for exploratory work.
    pub schedule_ticks: usize,
    /// Whether frontier escalation is admitted.
    pub frontier_escalation_admitted: bool,
}

/// One candidate rail with its cost, frontier mass and requested search mass.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AllocatorRail {
    pub route: RouteClass,
    pub cost: CostVector,
    pub mass: MassVector,
    /// Search mass the rail requests. It is compared against the admitted budget; it
    /// never changes the budget.
    pub requested_units: u64,
    pub capability_fit: bool,
    pub evidence_fit: bool,
    pub consequence_fit: bool,
}

/// A work package presented for allocation.
#[derive(Clone, Debug, PartialEq)]
pub struct AllocatorWorkPackage {
    pub request_id: String,
    pub semantic_subject: String,
    pub admission: WorkAdmission,
    pub knowledge: WorkKnowledge,
    pub consequence: ConsequenceClass,
    pub deadline_class: String,
    pub evidence_obligation: String,
    pub rails: Vec<AllocatorRail>,
}

/// The allocation: the DME work classification (decision or typed refusal), the
/// exact route request it was computed from, the fair-rail schedule and the bound
/// consequence horizon.
#[derive(Clone, Debug, PartialEq)]
pub struct DmeAllocation {
    pub classification: DmeWorkClassification,
    pub route_request: DmeRouteRequest,
    pub rail_schedule: Vec<RailSelection>,
    pub horizon: ConsequenceHorizonId,
    pub authority: AuthorityStanding,
}

/// Typed refusals of the allocator itself (route refusals live in the classification).
#[derive(Clone, Debug, PartialEq)]
pub enum AllocatorRefusal {
    /// More rails than [`MAX_ALLOCATOR_RAILS`].
    FanOutCeiling(BoundHit),
    /// Admitted budget above [`MAX_ALLOCATOR_BUDGET_UNITS`].
    BudgetCeiling(BoundHit),
    /// Schedule length above [`MAX_ALLOCATOR_SCHEDULE_TICKS`].
    ScheduleCeiling(BoundHit),
    /// q-lens normalization refused.
    QLens(QLensError),
}

fn ceiling(kind: BoundKind, limit: u64, observed: u64) -> BoundHit {
    BoundHit {
        kind,
        limit,
        observed,
    }
}

/// Dense rank of each rail's cost vector (equal vectors share a rank), so the rank is
/// independent of presentation order.
fn cost_ranks(rails: &[AllocatorRail]) -> Vec<u64> {
    let mut distinct: Vec<CostVector> = rails.iter().map(|r| r.cost).collect();
    distinct.sort();
    distinct.dedup();
    rails
        .iter()
        .map(|r| distinct.binary_search(&r.cost).unwrap_or(distinct.len()) as u64)
        .collect()
}

/// q-lens weight of each rail's frontier mass, scaled to `0..=WEIGHT_SCALE`. Rails
/// whose mass does not project to a positive value get weight 0.
fn q_weights(q: QValue, rails: &[AllocatorRail]) -> Result<Vec<u64>, AllocatorRefusal> {
    let mut boxes = FrontierBoxes::new();
    for (index, rail) in rails.iter().enumerate() {
        boxes.insert(index, rail.mass);
    }
    let mut weights = vec![0u64; rails.len()];
    let distribution = match boxes.positive_distribution() {
        Ok(distribution) => distribution,
        Err(QLensError::EmptyDistribution) => return Ok(weights),
        Err(other) => return Err(AllocatorRefusal::QLens(other)),
    };
    let weighted = q_lens(q, &distribution).map_err(AllocatorRefusal::QLens)?;
    for (index, weight) in weighted.entries() {
        weights[*index] = (weight.get() * WEIGHT_SCALE as f64).round() as u64;
    }
    Ok(weights)
}

/// SELECT an allocation. Grants no authority and performs no DO.
pub fn allocate<H: ConsequenceHorizon>(
    profile: &AllocatorProfile,
    package: &AllocatorWorkPackage,
    horizon: &H,
) -> Result<DmeAllocation, AllocatorRefusal> {
    if package.rails.len() > MAX_ALLOCATOR_RAILS {
        return Err(AllocatorRefusal::FanOutCeiling(ceiling(
            BoundKind::FrontierStates,
            MAX_ALLOCATOR_RAILS as u64,
            package.rails.len() as u64,
        )));
    }
    if profile.admitted_budget_units > MAX_ALLOCATOR_BUDGET_UNITS {
        return Err(AllocatorRefusal::BudgetCeiling(ceiling(
            BoundKind::SearchSteps,
            MAX_ALLOCATOR_BUDGET_UNITS,
            profile.admitted_budget_units,
        )));
    }
    if profile.schedule_ticks > MAX_ALLOCATOR_SCHEDULE_TICKS {
        return Err(AllocatorRefusal::ScheduleCeiling(ceiling(
            BoundKind::SearchSteps,
            MAX_ALLOCATOR_SCHEDULE_TICKS as u64,
            profile.schedule_ticks as u64,
        )));
    }

    let ranks = cost_ranks(&package.rails);
    let weights = q_weights(profile.q, &package.rails)?;
    let routes: Vec<RouteCandidate> = package
        .rails
        .iter()
        .zip(ranks.iter().zip(weights.iter()))
        .map(|(rail, (rank, weight))| RouteCandidate {
            route: rail.route,
            cost_units: rank * (WEIGHT_SCALE + 1) + (WEIGHT_SCALE - weight.min(&WEIGHT_SCALE)),
            capability_fit: rail.capability_fit && rail.cost.admitted,
            budget_units: profile.admitted_budget_units,
            required_units: rail.requested_units,
            evidence_fit: rail.evidence_fit,
            consequence_fit: rail.consequence_fit,
        })
        .collect();

    let (standing, admission_witness) = match &package.admission {
        WorkAdmission::Admitted(work) => (WorkStanding::Admitted, work.witness().to_string()),
        WorkAdmission::Candidate { .. } => (WorkStanding::Candidate, "unadmitted".to_string()),
    };
    let horizon_id = horizon.id();
    let route_request = DmeRouteRequest {
        request_id: package.request_id.clone(),
        semantic_subject: package.semantic_subject.clone(),
        standing,
        knowledge: package.knowledge,
        consequence: package.consequence,
        deadline_class: package.deadline_class.clone(),
        evidence_obligation: format!(
            "{}|horizon:{}|admission:{}",
            package.evidence_obligation,
            horizon_id.get(),
            admission_witness
        ),
        frontier_escalation_admitted: profile.frontier_escalation_admitted,
        routes,
    };
    let classification = classify_dme_work(&route_request);

    let exploratory = matches!(
        classification.class,
        DmeWorkClass::UnknownLocal | DmeWorkClass::UnknownDeferred | DmeWorkClass::UnknownFrontier
    );
    let rail_schedule = if exploratory {
        let exploit_rails = route_request
            .routes
            .iter()
            .filter(|c| c.route != RouteClass::KnownDeterministic && c.lawful())
            .count();
        let mut scheduler = FairRailScheduler::new(profile.fair_rail_max_gap, exploit_rails);
        (0..profile.schedule_ticks)
            .map(|_| scheduler.select())
            .collect()
    } else {
        Vec::new()
    };

    Ok(DmeAllocation {
        classification,
        route_request,
        rail_schedule,
        horizon: horizon_id,
        authority: AuthorityStanding::None,
    })
}
