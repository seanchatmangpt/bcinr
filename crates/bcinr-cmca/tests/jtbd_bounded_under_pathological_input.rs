#![allow(unused_imports)]

//! JTBD Track 4 — Bounded execution / DoS-shaped smoke check (SPECULATIVE, inferred JTBD).
//!
//! This module is a **determinism/boundedness SMOKE check**: ONE pathological-but-admitted
//! input, ONE wall-clock timing measurement. It is explicitly **NOT**:
//! - a formal worst-case-execution-time (WCET) proof,
//! - an exhaustive adversarial fuzzing campaign,
//! - a proof against algorithmic-complexity attacks in general.
//!
//! It is Chicago-style TDD: every assertion is a state-based check against the REAL,
//! unmodified [`bcinr_cmca::allocator::allocate`] authoritative root and REAL collaborator
//! types (`PackedSemanticState`, `LensSpec`, `NonNegativeFixed`, `RefusalSet`,
//! `NumericFaultSet`, `AllocationOutcome`). No mock or stub of any CMCA internal is used
//! anywhere in this file. Construction pattern reused from
//! `jtbd_multi_agent_resource_governance.rs` (flat N-way sibling registry, real
//! `CERTIFICATE_DIGEST`, real `AdaptiveUpdate::admit_adaptive_update` proof).
//!
//! ## The constructed input, and why it is "maximally adversarial but admitted"
//!
//! `allocate()`'s signature (`src/allocator.rs`) takes a fixed-shape set of arguments; there
//! is no separate "certification attempt" call to combine with a "plain allocation" call —
//! certification is represented by the `proof: Option<&AdaptiveUpdate<CertifiedLearning>>`
//! parameter of this SAME call, and by the certificate `digest: [u8; 32]` parameter compared
//! against `CERTIFICATE_DIGEST`. So "a rejected certificate attempt combined with a plain
//! allocation in one call" is not a separate combination question here — it collapses into
//! "pass a proof, but make the certificate/dwell/price checks inside the SAME `allocate()`
//! call fail," which this test exercises directly.
//!
//! Read from `src/allocator.rs`'s real gating logic (`allocate`, near its end):
//!
//! ```text
//! let has_refusal = has_error & !degrade_to_certified_selection;   // degrade = proof.is_none()
//! let gated_refusals = RefusalSet::EMPTY
//!     .union(RefusalSet::DIGEST_MISMATCH.masked(digest_err as u32))
//!     .union(RefusalSet::DWELL_UNSATISFIED.masked(dwell_err as u32))
//!     .union(RefusalSet::PROPOSAL_REJECTED.masked((!gd_ok | lr_err | beta_err | eta_err | q_err | price_err) as u32))
//!     .union(RefusalSet::AUTHORITY_MISSING.masked(degrade_to_certified_selection as u32))
//!     .masked(has_refusal as u32);
//! ```
//!
//! This exposes two REAL combination limits in the current implementation, verified by
//! reading the source (not assumed):
//!
//! 1. **`AUTHORITY_MISSING` cannot combine with any other refusal bit in one call.**
//!    `AUTHORITY_MISSING` is masked by `degrade_to_certified_selection` (`proof.is_none()`),
//!    but `has_refusal` — the outer mask gating the WHOLE `gated_refusals` union, including
//!    `AUTHORITY_MISSING` itself — requires `!degrade_to_certified_selection` (`proof.is_some()`).
//!    Those two conditions are mutually exclusive, so whenever `AUTHORITY_MISSING`'s own inner
//!    mask would be `1`, the outer `has_refusal` mask is provably `0`, zeroing the entire set.
//!    This test does not assert this is a bug (out of Track 4's scope to fix or characterize
//!    further) — it simply documents why this run supplies `proof = Some(..)` and therefore
//!    cannot also observe `AUTHORITY_MISSING`.
//! 2. **`RefusalSet::NO_LEAVES` cannot combine with a candidate that has any nonzero share.**
//!    `NO_LEAVES` requires the structural leaf count `nl == 0`, i.e. a forest with zero
//!    leaves — which under `allocate()`'s tree semantics also zeroes every `pi_res` entry
//!    (see the `nl_is_zero` masked-select over `pi_res` in `src/allocator.rs`). Constructing
//!    `nl == 0` also requires a `parent` shape where every node has a child (no leaves at
//!    all), which is a different structural regime than "one call with maximal simultaneous
//!    faults on real per-object values" — this test does not attempt it, since a zero-leaf
//!    forest gives nothing else to be pathological about.
//!
//! Given those two real, source-verified exclusions, the closest real combination the actual
//! API permits, all inside ONE `allocate()` call, is:
//!
//! - **Flat N-way sibling forest** (`parent = [-1; N]`, identical shape to
//!   `jtbd_multi_agent_resource_governance.rs` and `case_studies.rs`'s case-study-1): this is
//!   the shape JTBD_READINESS_REPORT.md's prior round found produces
//!   `NumericFaultSet::RANGE_VIOLATION` on every returned candidate share and a documented
//!   65532-vs-65536 conservation shortfall — i.e. genuinely "near-saturated values" already
//!   observed to stress the numeric path, reused rather than reinvented.
//! - **Zero-priced-sum**: `mu = [NonNegativeFixed::ZERO; N]`, `costs = [NonNegativeFixed::ZERO; N]`
//!   for every leaf except one, which is instead driven to `NonNegativeFixed::MAX` — far past
//!   the local `mu_max` envelope threshold (`6_553_600` value-bits) computed inside `allocate`
//!   — to trigger `price_err` (folded into `RefusalSet::PROPOSAL_REJECTED`).
//! - **A supplied-but-rejected certificate**: `proof = Some(get_proof())` (a REAL
//!   `AdaptiveUpdate<CertifiedLearning>`, admitted the same way every other test in this crate
//!   admits one) combined with a deliberately WRONG `digest` (`[0xFFu8; 32]`, guaranteed not to
//!   equal the real `CERTIFICATE_DIGEST` byte-for-byte) to trigger `digest_err`
//!   (`RefusalSet::DIGEST_MISMATCH`).
//! - **A dwell violation**: `tau_d = 0`, below the real
//!   `stability_profile::MODE_DWELL_ROUNDS_MIN`, to trigger `dwell_err`
//!   (`RefusalSet::DWELL_UNSATISFIED`).
//!
//! All four of `DIGEST_MISMATCH`, `DWELL_UNSATISFIED`, `PROPOSAL_REJECTED` and the numeric
//! `RANGE_VIOLATION` fault are real, independent gates inside the SAME `allocate()` call, and
//! (per the source excerpt above) all three `RefusalSet` bits are additionally gated by the
//! SAME `has_refusal` mask, so observing any one of them under `proof = Some(..)` is itself
//! evidence the other two are reachable through the identical mask, not merely asserted.
//!
//! This input does not violate any hard structural precondition: `N`, `Q`, `K` are the real
//! compile-time constants from `case_studies`, `parent` is a valid (if degenerate) flat
//! forest, and every `NonNegativeFixed`/`SignedFixed` value used is a real, safely-constructed
//! value of its type (no raw-byte forgery, no unsafe).

use bcinr_cmca::allocator::{
    allocate, AdaptiveUpdate, AdmittedControlState, CertificateReceipt, CertifiedLearning,
    EnvelopeReceipt, OutcomeReceipt, RefusalSet,
};
use bcinr_cmca::fixed::{NonNegativeFixed, NumericFaultSet};
use bcinr_cmca::generated_artifact::case_studies::{ETA, LAMBDA, LENS_REGISTRY, N, OBJECT_REGISTRY, Q};

use std::time::{Duration, Instant};

/// Real, unmocked certified-learning proof — identical construction to every other real test
/// in this crate (`jtbd_multi_agent_resource_governance.rs`, `case_studies.rs`).
fn get_proof() -> Option<AdaptiveUpdate<CertifiedLearning>> {
    AdaptiveUpdate::admit_adaptive_update(
        AdmittedControlState::admit_control_state(0),
        CertificateReceipt::admit_certificate(0),
        EnvelopeReceipt::admit_envelope(0),
        OutcomeReceipt::admit_outcome(0),
        NonNegativeFixed::ZERO,
        NonNegativeFixed::ONE,
        CertifiedLearning::admit_learning(),
    )
}

/// Generous-but-real wall-clock bound. `allocate()` is documented ($O(K \cdot Q \cdot N^2)$,
/// $N=8, K=4, Q=4$) as branchless and $O(1)$ in the constants this crate compiles with; 100ms
/// is orders of magnitude above any plausible single-call cost on any real machine, so a
/// breach here is strong (not proof-grade — see module doc) evidence of an accidental
/// input-dependent loop or unbounded path, not measurement noise.
const WALL_CLOCK_BOUND: Duration = Duration::from_millis(100);

test!(
    one_maximally_adversarial_admitted_call_stays_bounded_and_coherent,
    {
        // Arrange: flat N-way sibling forest (real OBJECT_REGISTRY, unmodified structure,
        // known from the prior JTBD round to stress the numeric path under N-way competing
        // demand) with one leaf's price driven to the real MAX value, far past allocate()'s
        // internal mu_max envelope.
        let mut weights = [[NonNegativeFixed::ONE; 2 * Q]; N];
        let payoffs = [[NonNegativeFixed::ZERO; 2 * Q]; N];
        let mut last_switch_t: u32 = 0;
        let mut prev_mode: u32 = 0;
        let parent = [-1; N];

        // Zero-priced-sum baseline for every leaf except one, which is saturated past the
        // envelope to force `price_err` (folded into `RefusalSet::PROPOSAL_REJECTED`).
        let mut mu = [NonNegativeFixed::ZERO; N];
        mu[0] = NonNegativeFixed::MAX;
        let costs = [NonNegativeFixed::ZERO; N];

        // Deliberately wrong certificate digest — guaranteed to mismatch the real
        // CERTIFICATE_DIGEST byte-for-byte (all bytes 0xFF, which no real BLAKE3-derived
        // digest constant in this codebase is byte-for-byte equal to).
        let wrong_digest = [0xFFu8; 32];

        // Dwell violation: 0 rounds is below any real minimum-dwell-rounds constant (which is
        // structurally >= 1 for the dwell mechanism to mean anything).
        let tau_d_below_minimum: u32 = 0;

        // A supplied-but-rejected certificate: proof IS present (so `has_refusal`'s
        // `!degrade_to_certified_selection` half is satisfied), but the digest/dwell/price
        // checks inside the same call independently fail.
        let proof = get_proof();

        // Act: one real, timed call to the authoritative allocator root.
        let start = Instant::now();
        let outcome = allocate(
            &OBJECT_REGISTRY,
            &LENS_REGISTRY,
            &LAMBDA,
            ETA,
            &parent,
            &mut weights,
            &payoffs,
            NonNegativeFixed::ZERO,
            NonNegativeFixed::ZERO,
            &mu,
            &costs,
            0,
            &mut last_switch_t,
            &mut prev_mode,
            tau_d_below_minimum,
            wrong_digest,
            proof.as_ref(),
        );
        let elapsed = start.elapsed();
        eprintln!(
            "[jtbd_bounded_under_pathological_input] elapsed={elapsed:?} refusals={:?} (bits={:#x}) numeric_faults=(bits={:#x})",
            outcome.refusals(),
            outcome.refusals().bits(),
            outcome.numeric_faults().bits()
        );

        // Assert 1 (boundedness smoke check): the call completed within the generous bound.
        // This is a single wall-clock sample on whatever machine runs this test, NOT a WCET
        // proof (see module doc) — reported as the real measured duration, not rounded up.
        assert!(
            elapsed < WALL_CLOCK_BOUND,
            "allocate() took {elapsed:?} on this pathological input, exceeding the generous \
             {WALL_CLOCK_BOUND:?} smoke-check bound — possible input-dependent unbounded path"
        );

        // Assert 2 (coherence — no panic): reaching this line at all, without the test process
        // aborting, is itself the first-order evidence the branchless contract held under this
        // input (numeric-hot-path.md Invariant 6: the authoritative root is total).

        // Assert 3 (specific expected refusal bits, not just "some refusal exists"):
        // DIGEST_MISMATCH (from the wrong certificate digest) and DWELL_UNSATISFIED (from
        // tau_d below the real minimum) must both be present, and co-occur as a SET rather
        // than collapsing to one — the authority-and-c3.md Invariant 2 property this crate's
        // own RefusalSet type exists to preserve.
        let refusals = outcome.refusals();
        assert!(
            refusals.contains(RefusalSet::DIGEST_MISMATCH),
            "expected DIGEST_MISMATCH from a byte-for-byte-wrong certificate digest; got {refusals:?}"
        );
        assert!(
            refusals.contains(RefusalSet::DWELL_UNSATISFIED),
            "expected DWELL_UNSATISFIED from tau_d={tau_d_below_minimum} below the real \
             minimum-dwell-rounds constant; got {refusals:?}"
        );
        assert!(
            refusals.contains(RefusalSet::PROPOSAL_REJECTED),
            "expected PROPOSAL_REJECTED from a leaf price (mu[0]=MAX) past allocate()'s \
             internal mu_max envelope; got {refusals:?}"
        );
        assert!(
            outcome.is_refused(),
            "the aggregate RefusalSet must be non-empty given the three named bits above"
        );

        // Assert 4 (specific expected numeric fault bit, not just "some fault exists"): the
        // flat N-way sibling forest is the exact shape JTBD_READINESS_REPORT.md's prior round
        // documented as producing RANGE_VIOLATION on every returned candidate share. This
        // assertion is reporting that same real, already-known behavior under a call that ALSO
        // carries a full refusal set — not introducing a new claim.
        let numeric_faults = outcome.numeric_faults();
        assert!(
            numeric_faults.bits() & NumericFaultSet::RANGE_VIOLATION.bits()
                == NumericFaultSet::RANGE_VIOLATION.bits(),
            "expected RANGE_VIOLATION under the flat N-way sibling forest shape (per \
             JTBD_READINESS_REPORT.md's prior-round finding); got {numeric_faults:?}"
        );

        // Assert 5 (well-formedness of output values — not garbage): every returned candidate
        // share is a validly-constructed NonNegativeFixed (structurally non-negative by type)
        // within the representable [0, ONE] unit-budget range. This does NOT assert exact
        // conservation (numeric-hot-path.md Invariant 4) — the prior JTBD round already
        // documented that the flat N-way shape currently conserves to 65532, not 65536, and
        // fixing that is out of this test's scope; this test only checks the values are
        // well-formed, not garbage/out-of-range.
        let shares = outcome.candidate();
        let budget = NonNegativeFixed::ONE.value_bits();
        for (i, share) in shares.iter().enumerate() {
            assert!(
                share.value_bits() <= budget,
                "workload {i} received {} bits, exceeding the whole unit budget {budget} — \
                 not a well-formed share",
                share.value_bits()
            );
        }
    }
);
use chicago_tdd_tools::test;
