//! # Stability candidate — Authority hop 4 of the C3 chain
//!
//! Per AGENTS.md §12 ("No runtime theorem discovery"), the authoritative runtime never
//! *discovers* a stability witness — it only *verifies* one supplied by the slow rail. This
//! module's [`derive_stability_candidate`] takes the comparison matrix `G`, a claimed
//! positive witness `d`, and the margin `delta` the slow rail already computed, and
//! verifies the static domination law
//!
//! ```text
//! G d <= (1 - delta) d      (elementwise)
//! ```
//!
//! producing a sealed [`StabilityCandidate`] only when the inequality actually holds for
//! the supplied values — the candidate is *derived* from a check, never asserted from the
//! caller's claim alone.

use crate::proposal::mix64;

/// Fixed dimension of the comparison matrix / witness vector this stage operates over.
/// Kept small and compile-time-fixed per AGENTS.md §13 (no unbounded/variable iteration).
pub const DIM: usize = 2;

/// Q16.16-scaled fixed-point unit used throughout this module's arithmetic (matches
/// `crate::fixed`'s scale so values can be compared 1:1 against `NonNegativeFixed`/
/// `SignedFixed` inputs converted via `value_bits()`).
pub const SCALE: i64 = 1 << 16;

/// Refusal reasons for [`derive_stability_candidate`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StabilityDerivationRefusal {
    /// The supplied witness `d` has a non-positive component — it is not a valid positive
    /// witness.
    WitnessNotPositive,
    /// The margin `delta` is outside the admissible `(0, 1)` range in Q16.16 fixed point.
    MarginOutOfRange,
    /// `G d <= (1 - delta) d` does not hold elementwise for the supplied `G`, `d`, `delta`.
    ContractionMarginInsufficient,
    /// The upstream jump analysis indicates a policy jump, which this stage refuses to
    /// derive a stability candidate for (a policy jump must be re-proposed, not silently
    /// treated as a stability question).
    UpstreamJumpNotStabilityRelevant,
}

/// A sealed, derived stability candidate binding the comparison matrix, positive witness,
/// margin, radii, q ceiling, Gram distinguishability floor, dwell law identity,
/// pricing-loop bound, and the identity of the comparison derivation that produced it.
///
/// Constructible only via [`derive_stability_candidate`], and only when the domination
/// witness actually verifies — this is a derived fact, not an asserted one.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct StabilityCandidate {
    g: [[i64; DIM]; DIM],
    d: [i64; DIM],
    margin_delta: i64,
    noise_radius: i64,
    switch_radius: i64,
    q_ceiling: i64,
    gram_distinguishability_floor: i64,
    dwell_law_id: u64,
    pricing_loop_bound: i64,
    comparison_derivation_identity: u64,
    candidate_digest: u64,
}

impl StabilityCandidate {
    #[inline(always)]
    pub fn g(&self) -> [[i64; DIM]; DIM] {
        self.g
    }
    #[inline(always)]
    pub fn d(&self) -> [i64; DIM] {
        self.d
    }
    #[inline(always)]
    pub fn margin_delta(&self) -> i64 {
        self.margin_delta
    }
    #[inline(always)]
    pub fn noise_radius(&self) -> i64 {
        self.noise_radius
    }
    #[inline(always)]
    pub fn switch_radius(&self) -> i64 {
        self.switch_radius
    }
    #[inline(always)]
    pub fn q_ceiling(&self) -> i64 {
        self.q_ceiling
    }
    #[inline(always)]
    pub fn gram_distinguishability_floor(&self) -> i64 {
        self.gram_distinguishability_floor
    }
    #[inline(always)]
    pub fn dwell_law_id(&self) -> u64 {
        self.dwell_law_id
    }
    #[inline(always)]
    pub fn pricing_loop_bound(&self) -> i64 {
        self.pricing_loop_bound
    }
    #[inline(always)]
    pub fn comparison_derivation_identity(&self) -> u64 {
        self.comparison_derivation_identity
    }
    #[inline(always)]
    pub fn candidate_digest(&self) -> u64 {
        self.candidate_digest
    }

    // Invariant 3 (authority-and-c3.md) requires the seal to bind every enumerated
    // domain-specific identity (graph, payload, kernel, pricing, control-mode,
    // comparison-derivation, round, ...) explicitly and independently — collapsing
    // these into a bundling struct would obscure which bindings are actually checked.
    // Documented allow per AGENTS.md's "no undocumented allow" rule.
    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    fn seal(
        g: [[i64; DIM]; DIM],
        d: [i64; DIM],
        margin_delta: i64,
        noise_radius: i64,
        switch_radius: i64,
        q_ceiling: i64,
        gram_distinguishability_floor: i64,
        dwell_law_id: u64,
        pricing_loop_bound: i64,
        comparison_derivation_identity: u64,
    ) -> u64 {
        let mut acc = comparison_derivation_identity;
        for row in g.iter() {
            for &v in row.iter() {
                acc = mix64(acc, v as u64);
            }
        }
        for &v in d.iter() {
            acc = mix64(acc, v as u64);
        }
        acc = mix64(acc, margin_delta as u64);
        acc = mix64(acc, noise_radius as u64);
        acc = mix64(acc, switch_radius as u64);
        acc = mix64(acc, q_ceiling as u64);
        acc = mix64(acc, gram_distinguishability_floor as u64);
        acc = mix64(acc, dwell_law_id);
        acc = mix64(acc, pricing_loop_bound as u64);
        acc
    }
}

/// Verifies the static domination witness `G d <= (1 - delta) d` (elementwise, Q16.16
/// fixed point) and, only if it holds, derives a sealed [`StabilityCandidate`] binding the
/// supplied matrix/witness/margin/radii/ceiling/floor/dwell-law/pricing-bound/derivation
/// identity.
///
/// This function does not search for a witness — see AGENTS.md §12. It only checks the one
/// supplied.
#[allow(clippy::too_many_arguments)]
pub fn derive_stability_candidate(
    upstream_kind: crate::jump::JumpKind,
    g: [[i64; DIM]; DIM],
    d: [i64; DIM],
    margin_delta: i64,
    noise_radius: i64,
    switch_radius: i64,
    q_ceiling: i64,
    gram_distinguishability_floor: i64,
    dwell_law_id: u64,
    pricing_loop_bound: i64,
    comparison_derivation_identity: u64,
) -> Result<StabilityCandidate, StabilityDerivationRefusal> {
    if matches!(upstream_kind, crate::jump::JumpKind::PolicyJump) {
        return Err(StabilityDerivationRefusal::UpstreamJumpNotStabilityRelevant);
    }
    for &di in d.iter() {
        if di <= 0 {
            return Err(StabilityDerivationRefusal::WitnessNotPositive);
        }
    }
    if margin_delta <= 0 || margin_delta >= SCALE {
        return Err(StabilityDerivationRefusal::MarginOutOfRange);
    }

    // G d, computed exactly in i128 to avoid overflow, then rescaled back to Q16.16.
    let mut gd = [0i64; DIM];
    for r in 0..DIM {
        let mut acc: i128 = 0;
        for c in 0..DIM {
            acc += (g[r][c] as i128) * (d[c] as i128);
        }
        gd[r] = (acc / SCALE as i128) as i64;
    }

    // (1 - delta) * d, in Q16.16.
    let one_minus_delta = SCALE - margin_delta;
    for i in 0..DIM {
        let bound = ((one_minus_delta as i128) * (d[i] as i128) / SCALE as i128) as i64;
        if gd[i] > bound {
            return Err(StabilityDerivationRefusal::ContractionMarginInsufficient);
        }
    }

    let candidate_digest = StabilityCandidate::seal(
        g,
        d,
        margin_delta,
        noise_radius,
        switch_radius,
        q_ceiling,
        gram_distinguishability_floor,
        dwell_law_id,
        pricing_loop_bound,
        comparison_derivation_identity,
    );

    Ok(StabilityCandidate {
        g,
        d,
        margin_delta,
        noise_radius,
        switch_radius,
        q_ceiling,
        gram_distinguishability_floor,
        dwell_law_id,
        pricing_loop_bound,
        comparison_derivation_identity,
        candidate_digest,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jump::JumpKind;

    fn contracting_g() -> [[i64; DIM]; DIM] {
        // G = 0.5 * I (Q16.16), contracts any positive d by exactly half.
        [[SCALE / 2, 0], [0, SCALE / 2]]
    }

    fn positive_d() -> [i64; DIM] {
        [SCALE, SCALE]
    }

    #[test]
    fn derives_candidate_when_witness_holds() {
        let res = derive_stability_candidate(
            JumpKind::FixedPointStateJump,
            contracting_g(),
            positive_d(),
            SCALE / 4, // delta = 0.25; need Gd <= 0.75*d; Gd = 0.5*d, holds
            0,
            0,
            SCALE,
            0,
            1,
            0,
            42,
        );
        assert!(res.is_ok());
    }

    #[test]
    fn refuses_when_margin_insufficient() {
        // delta so large that (1-delta) < 0.5 fails the 0.5-contracting G.
        let res = derive_stability_candidate(
            JumpKind::FixedPointStateJump,
            contracting_g(),
            positive_d(),
            (SCALE * 6) / 10, // delta = 0.6; need Gd <= 0.4*d, but Gd = 0.5*d -> fails
            0,
            0,
            SCALE,
            0,
            1,
            0,
            42,
        );
        assert_eq!(
            res,
            Err(StabilityDerivationRefusal::ContractionMarginInsufficient)
        );
    }

    #[test]
    fn refuses_on_nonpositive_witness() {
        let res = derive_stability_candidate(
            JumpKind::FixedPointStateJump,
            contracting_g(),
            [SCALE, 0], // second component non-positive
            SCALE / 4,
            0,
            0,
            SCALE,
            0,
            1,
            0,
            42,
        );
        assert_eq!(res, Err(StabilityDerivationRefusal::WitnessNotPositive));
    }

    #[test]
    fn refuses_on_margin_out_of_range() {
        let res = derive_stability_candidate(
            JumpKind::FixedPointStateJump,
            contracting_g(),
            positive_d(),
            0, // delta must be > 0
            0,
            0,
            SCALE,
            0,
            1,
            0,
            42,
        );
        assert_eq!(res, Err(StabilityDerivationRefusal::MarginOutOfRange));
    }

    #[test]
    fn refuses_policy_jumps_as_not_stability_relevant() {
        let res = derive_stability_candidate(
            JumpKind::PolicyJump,
            contracting_g(),
            positive_d(),
            SCALE / 4,
            0,
            0,
            SCALE,
            0,
            1,
            0,
            42,
        );
        assert_eq!(
            res,
            Err(StabilityDerivationRefusal::UpstreamJumpNotStabilityRelevant)
        );
    }

    #[test]
    fn candidate_digest_is_derived_from_all_bound_fields() {
        let a = derive_stability_candidate(
            JumpKind::FixedPointStateJump,
            contracting_g(),
            positive_d(),
            SCALE / 4,
            0,
            0,
            SCALE,
            0,
            1,
            0,
            42,
        )
        .unwrap();
        let b = derive_stability_candidate(
            JumpKind::FixedPointStateJump,
            contracting_g(),
            positive_d(),
            SCALE / 4,
            0,
            0,
            SCALE,
            0,
            1,
            0,
            43, // different derivation identity
        )
        .unwrap();
        assert_ne!(a.candidate_digest(), b.candidate_digest());
    }
}
