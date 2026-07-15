//! `ComputeEvidence`, `PlanningReceipt`, and the CER/CPS efficiency metrics.
//!
//! `PlanningReceipt` is the top-level bundle for one grounded planning
//! epoch: it does not re-derive any of the evidence it carries (cache hits,
//! residualization decisions, frontier measure, portfolio choice, the
//! causal/concurrency/projection digests) — those are computed by their own
//! real subsystems (`bcinr-pddl`'s `consequence`/`mfw`/`search` modules for
//! the planning-side evidence, `bcinr-powl`'s `PowlProjector` for the
//! projection digest) and passed in as already-computed `Digest`s, exactly
//! the same "a receipt records a check that already happened" discipline
//! [`crate::projection::seal_projection_receipt`] uses. This crate does not
//! depend on `bcinr-pddl` (see this crate's `Cargo.toml` — only
//! `bcinr-mfw-ir` and `bcinr-powl` were added this phase), so
//! `cache_evidence`/`residual_evidence`/`frontier_evidence`/
//! `portfolio_evidence` are opaque `Digest`s here rather than the concrete
//! `bcinr-pddl` types (`StandingConsequenceCache`, `Residualizer`,
//! `FrontierMeasure`, `MfwPortfolio`) — a caller that has those values
//! computes their digest and passes it in. This is a deliberate scope
//! boundary, not an oversight: wiring a real `bcinr-pddl`-side integration
//! test is future work, out of this phase (see final report).

use bcinr_mfw_ir::{ConsequenceHorizonId, Digest, PlannerOutcome, PlanningEpochId};

use crate::chain::fold;

// ---------------------------------------------------------------------------
// PlannerOutcomeTag
// ---------------------------------------------------------------------------

/// A bare tag for which `bcinr_mfw_ir::PlannerOutcome<T>` variant a planning
/// epoch ended in — deliberately without the witness payload each variant
/// carries (`ExhaustionWitness`, `BoundHit`, ...): a `PlanningReceipt` needs
/// a small, `Copy`, digest-friendly summary, not the full witness (which is
/// already digested/attested elsewhere in the pipeline this receipt
/// bundles evidence from).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlannerOutcomeTag {
    Found,
    Exhausted,
    Bounded,
    Unsupported,
    Inconsistent,
}

impl PlannerOutcomeTag {
    fn discriminant(self) -> u8 {
        match self {
            PlannerOutcomeTag::Found => 0,
            PlannerOutcomeTag::Exhausted => 1,
            PlannerOutcomeTag::Bounded => 2,
            PlannerOutcomeTag::Unsupported => 3,
            PlannerOutcomeTag::Inconsistent => 4,
        }
    }
}

/// Real conversion from `bcinr_mfw_ir::PlannerOutcome<T>` (the shared
/// outcome/witness algebra every bounded search/analysis stage in this
/// workspace returns) to the tag this receipt stores — not a parallel,
/// disconnected enum.
impl<T> From<&PlannerOutcome<T>> for PlannerOutcomeTag {
    fn from(outcome: &PlannerOutcome<T>) -> Self {
        match outcome {
            PlannerOutcome::Found(_) => PlannerOutcomeTag::Found,
            PlannerOutcome::Exhausted(_) => PlannerOutcomeTag::Exhausted,
            PlannerOutcome::Bounded(_) => PlannerOutcomeTag::Bounded,
            PlannerOutcome::Unsupported(_) => PlannerOutcomeTag::Unsupported,
            PlannerOutcome::Inconsistent(_) => PlannerOutcomeTag::Inconsistent,
        }
    }
}

// ---------------------------------------------------------------------------
// ComputeEvidence
// ---------------------------------------------------------------------------

/// Plain compute-accounting counters for one planning epoch. Every field is
/// a caller-supplied count from a real run — this type does not compute or
/// validate any of them itself, it only carries and digests them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ComputeEvidence {
    pub candidate_groundings: u64,
    pub materialized_groundings: u64,
    pub states_expanded: u64,
    pub semantic_classes_visited: u64,
    pub cache_hits: u64,
    pub residual_actions: u64,
    pub exact_rail_steps: u64,
    pub exploit_rail_steps: u64,
}

impl ComputeEvidence {
    /// Canonical digest over all eight fields, in declared field order.
    pub fn digest(&self) -> Digest {
        let mut buf = Vec::with_capacity(8 * 8);
        buf.extend_from_slice(&self.candidate_groundings.to_le_bytes());
        buf.extend_from_slice(&self.materialized_groundings.to_le_bytes());
        buf.extend_from_slice(&self.states_expanded.to_le_bytes());
        buf.extend_from_slice(&self.semantic_classes_visited.to_le_bytes());
        buf.extend_from_slice(&self.cache_hits.to_le_bytes());
        buf.extend_from_slice(&self.residual_actions.to_le_bytes());
        buf.extend_from_slice(&self.exact_rail_steps.to_le_bytes());
        buf.extend_from_slice(&self.exploit_rail_steps.to_le_bytes());
        Digest::hash(&buf)
    }
}

// ---------------------------------------------------------------------------
// PlanningReceipt
// ---------------------------------------------------------------------------

/// The top-level receipt for one grounded planning epoch: bundles the
/// epoch/horizon identity, capability admission, cache/residual/frontier/
/// portfolio evidence digests, the planner outcome tag, the
/// causal/concurrency/projection digests, and the compute-accounting
/// evidence, all hash-chained the same way as every other receipt in this
/// crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlanningReceipt {
    pub epoch: PlanningEpochId,
    pub horizon: ConsequenceHorizonId,
    pub capability_digest: Digest,
    pub cache_evidence: Digest,
    pub residual_evidence: Digest,
    pub frontier_evidence: Digest,
    pub portfolio_evidence: Digest,
    pub planner_outcome: PlannerOutcomeTag,
    pub causal_digest: Digest,
    pub concurrency_digest: Digest,
    pub projection_digest: Digest,
    pub compute: ComputeEvidence,
    pub prior_hash: Digest,
    pub hash: Digest,
}

/// Seal a [`PlanningReceipt`] from already-computed evidence. Pure
/// function, same discipline as [`crate::projection::seal_projection_receipt`]
/// and [`crate::execution::seal_execution_receipt`]: folds
/// `prior_hash` with a canonical, fixed-field-order byte serialization of
/// every field below (never a `HashMap`-iteration order).
#[allow(clippy::too_many_arguments)]
pub fn seal_planning_receipt(
    prior_hash: Digest,
    epoch: PlanningEpochId,
    horizon: ConsequenceHorizonId,
    capability_digest: Digest,
    cache_evidence: Digest,
    residual_evidence: Digest,
    frontier_evidence: Digest,
    portfolio_evidence: Digest,
    planner_outcome: PlannerOutcomeTag,
    causal_digest: Digest,
    concurrency_digest: Digest,
    projection_digest: Digest,
    compute: ComputeEvidence,
) -> PlanningReceipt {
    let mut buf = Vec::with_capacity(16 + 32 * 10 + 1 + 8 * 8);
    buf.extend_from_slice(&epoch.0.to_le_bytes());
    buf.extend_from_slice(horizon.0.as_bytes());
    buf.extend_from_slice(capability_digest.as_bytes());
    buf.extend_from_slice(cache_evidence.as_bytes());
    buf.extend_from_slice(residual_evidence.as_bytes());
    buf.extend_from_slice(frontier_evidence.as_bytes());
    buf.extend_from_slice(portfolio_evidence.as_bytes());
    buf.push(planner_outcome.discriminant());
    buf.extend_from_slice(causal_digest.as_bytes());
    buf.extend_from_slice(concurrency_digest.as_bytes());
    buf.extend_from_slice(projection_digest.as_bytes());
    buf.extend_from_slice(compute.digest().as_bytes());

    let hash = fold(&prior_hash, &buf);

    PlanningReceipt {
        epoch,
        horizon,
        capability_digest,
        cache_evidence,
        residual_evidence,
        frontier_evidence,
        portfolio_evidence,
        planner_outcome,
        causal_digest,
        concurrency_digest,
        projection_digest,
        compute,
        prior_hash,
        hash,
    }
}

// ---------------------------------------------------------------------------
// CER / CPS metrics
// ---------------------------------------------------------------------------

/// Compute Efficiency Ratio: `CER = 1 - mfw_compute / baseline_compute`.
///
/// Guarded against division by zero: when `baseline_compute == 0` there is
/// no baseline signal to compare against, so this returns `0.0` (no
/// measurable efficiency claim) rather than producing `NaN`/`inf` or
/// panicking.
pub fn compute_efficiency_ratio(baseline_compute: u64, mfw_compute: u64) -> f64 {
    if baseline_compute == 0 {
        return 0.0;
    }
    1.0 - (mfw_compute as f64 / baseline_compute as f64)
}

/// Compute Per Standing-consequence: `CPS = compute_consumed /
/// new_standing_consequences`.
///
/// Returns `None` when `new_standing_consequences == 0` — there is no
/// meaningful "compute per new standing consequence" figure when zero new
/// standing consequences were produced, so this refuses to manufacture a
/// number (never `NaN`, never a silent `inf`, never a panic).
pub fn compute_per_standing_consequence(
    compute_consumed: u64,
    new_standing_consequences: u64,
) -> Option<f64> {
    if new_standing_consequences == 0 {
        return None;
    }
    Some(compute_consumed as f64 / new_standing_consequences as f64)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_compute() -> ComputeEvidence {
        ComputeEvidence {
            candidate_groundings: 100,
            materialized_groundings: 40,
            states_expanded: 250,
            semantic_classes_visited: 12,
            cache_hits: 5,
            residual_actions: 3,
            exact_rail_steps: 30,
            exploit_rail_steps: 20,
        }
    }

    #[test]
    fn planner_outcome_tag_matches_real_planner_outcome_variants() {
        let found: PlannerOutcome<u32> = PlannerOutcome::Found(1);
        assert_eq!(PlannerOutcomeTag::from(&found), PlannerOutcomeTag::Found);

        let unsupported: PlannerOutcome<u32> =
            PlannerOutcome::Unsupported(bcinr_mfw_ir::UnsupportedFeature {
                feature_name: "test".into(),
                context: "test".into(),
            });
        assert_eq!(
            PlannerOutcomeTag::from(&unsupported),
            PlannerOutcomeTag::Unsupported
        );
    }

    #[test]
    fn compute_evidence_digest_is_sensitive_to_every_field() {
        let base = sample_compute();
        let base_digest = base.digest();

        let mut tweaked = base;
        tweaked.exploit_rail_steps += 1;
        assert_ne!(base_digest, tweaked.digest());

        let mut tweaked2 = base;
        tweaked2.cache_hits += 1;
        assert_ne!(base_digest, tweaked2.digest());
    }

    #[test]
    fn seal_planning_receipt_chains_and_is_deterministic() {
        let compute = sample_compute();
        let receipt1 = seal_planning_receipt(
            Digest::ZERO,
            PlanningEpochId(7),
            ConsequenceHorizonId(Digest::hash(b"horizon")),
            Digest::hash(b"capability"),
            Digest::hash(b"cache"),
            Digest::hash(b"residual"),
            Digest::hash(b"frontier"),
            Digest::hash(b"portfolio"),
            PlannerOutcomeTag::Found,
            Digest::hash(b"causal"),
            Digest::hash(b"concurrency"),
            Digest::hash(b"projection"),
            compute,
        );
        let receipt1_again = seal_planning_receipt(
            Digest::ZERO,
            PlanningEpochId(7),
            ConsequenceHorizonId(Digest::hash(b"horizon")),
            Digest::hash(b"capability"),
            Digest::hash(b"cache"),
            Digest::hash(b"residual"),
            Digest::hash(b"frontier"),
            Digest::hash(b"portfolio"),
            PlannerOutcomeTag::Found,
            Digest::hash(b"causal"),
            Digest::hash(b"concurrency"),
            Digest::hash(b"projection"),
            compute,
        );
        assert_eq!(receipt1.hash, receipt1_again.hash);

        let receipt2 = seal_planning_receipt(
            receipt1.hash,
            PlanningEpochId(8),
            ConsequenceHorizonId(Digest::hash(b"horizon-2")),
            Digest::hash(b"capability"),
            Digest::hash(b"cache"),
            Digest::hash(b"residual"),
            Digest::hash(b"frontier"),
            Digest::hash(b"portfolio"),
            PlannerOutcomeTag::Exhausted,
            Digest::hash(b"causal"),
            Digest::hash(b"concurrency"),
            Digest::hash(b"projection"),
            compute,
        );
        assert_ne!(receipt2.hash, receipt1.hash);
        assert_eq!(receipt2.prior_hash, receipt1.hash);
    }

    #[test]
    fn different_planner_outcome_tags_change_the_hash() {
        let compute = sample_compute();
        let make = |tag: PlannerOutcomeTag| {
            seal_planning_receipt(
                Digest::ZERO,
                PlanningEpochId(1),
                ConsequenceHorizonId(Digest::hash(b"horizon")),
                Digest::hash(b"capability"),
                Digest::hash(b"cache"),
                Digest::hash(b"residual"),
                Digest::hash(b"frontier"),
                Digest::hash(b"portfolio"),
                tag,
                Digest::hash(b"causal"),
                Digest::hash(b"concurrency"),
                Digest::hash(b"projection"),
                compute,
            )
        };
        assert_ne!(
            make(PlannerOutcomeTag::Found).hash,
            make(PlannerOutcomeTag::Exhausted).hash
        );
    }

    #[test]
    fn cer_is_one_when_mfw_consumes_no_compute() {
        assert_eq!(compute_efficiency_ratio(1000, 0), 1.0);
    }

    #[test]
    fn cer_is_zero_when_mfw_matches_baseline() {
        assert_eq!(compute_efficiency_ratio(1000, 1000), 0.0);
    }

    #[test]
    fn cer_is_negative_when_mfw_exceeds_baseline() {
        assert!(compute_efficiency_ratio(1000, 2000) < 0.0);
    }

    #[test]
    fn cer_guards_against_zero_baseline() {
        assert_eq!(compute_efficiency_ratio(0, 500), 0.0);
        assert_eq!(compute_efficiency_ratio(0, 0), 0.0);
    }

    #[test]
    fn cps_computes_ratio_for_nonzero_denominator() {
        assert_eq!(compute_per_standing_consequence(1000, 10), Some(100.0));
    }

    #[test]
    fn cps_returns_none_for_zero_new_standing_consequences() {
        assert_eq!(compute_per_standing_consequence(1000, 0), None);
        assert_eq!(compute_per_standing_consequence(0, 0), None);
    }
}
