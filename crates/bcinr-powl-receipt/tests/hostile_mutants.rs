#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unexpected_cfgs)]

use bcinr_mfw_ir::{Digest, EventSet};
use bcinr_powl::tape::v2::ConcurrencyGuardTable;
use bcinr_powl_receipt::execution::{
    seal_execution_receipt, verify_execution_receipt, ExecutionIntegrityError,
};

// M01: canonical_bytes drops `tick`
// M02: canonical_bytes drops `fired`
// M03: canonical_bytes drops `scheduler_decision_digest`

// Baseline verification: Ensure true code REJECTS the forgeries with HashMismatch.
#[cfg(not(any(feature = "mutant_1", feature = "mutant_2", feature = "mutant_3")))]
#[test]
fn verify_baseline_forged_receipts_rejected() {
    let guards = ConcurrencyGuardTable::empty();
    let fired = EventSet::empty().with(0);
    let original = seal_execution_receipt(
        Digest::ZERO,
        Digest::hash(b"model"),
        Digest::hash(b"compiled"),
        1,
        Digest::hash(b"decision"),
        fired,
        fired,
        &guards,
    )
    .unwrap();

    // Forge tick
    let mut forged_tick = original;
    forged_tick.tick = 999;
    let result_tick = verify_execution_receipt(&forged_tick, &guards);
    assert!(
        matches!(
            result_tick,
            Err(ExecutionIntegrityError::HashMismatch { .. })
        ),
        "Baseline must reject altered tick with HashMismatch"
    );

    // Forge fired set (must use an admissible set to bypass admissibility check and hit HashMismatch)
    let mut forged_fired = original;
    forged_fired.fired = EventSet::empty().with(1);
    let result_fired = verify_execution_receipt(&forged_fired, &guards);
    assert!(
        matches!(
            result_fired,
            Err(ExecutionIntegrityError::HashMismatch { .. })
        ),
        "Baseline must reject altered fired set with HashMismatch"
    );

    // Forge scheduler decision
    let mut forged_decision = original;
    forged_decision.scheduler_decision_digest = Digest::hash(b"forged-decision");
    let result_decision = verify_execution_receipt(&forged_decision, &guards);
    assert!(
        matches!(
            result_decision,
            Err(ExecutionIntegrityError::HashMismatch { .. })
        ),
        "Baseline must reject altered scheduler_decision_digest with HashMismatch"
    );
}

// Mutant 1: canonical_bytes drops `tick`.
// Proves this exact corruption allows a forged tick to pass verification.
#[cfg(feature = "mutant_1")]
#[test]
fn kill_mutant_1_accepts_forged_tick() {
    let guards = ConcurrencyGuardTable::empty();
    let fired = EventSet::empty().with(0);
    let original = seal_execution_receipt(
        Digest::ZERO,
        Digest::hash(b"model"),
        Digest::hash(b"compiled"),
        1,
        Digest::hash(b"decision"),
        fired,
        fired,
        &guards,
    )
    .unwrap();

    let mut forged_tick = original;
    forged_tick.tick = 999;

    let result = verify_execution_receipt(&forged_tick, &guards);
    assert_eq!(
        result,
        Ok(()),
        "Mutant 1 (dropped tick from digest) should falsely accept a receipt with an altered tick"
    );
}

// Mutant 2: canonical_bytes drops `fired`.
// Proves this exact corruption allows a forged admissible fired set to pass verification.
#[cfg(feature = "mutant_2")]
#[test]
fn kill_mutant_2_accepts_forged_fired() {
    let guards = ConcurrencyGuardTable::empty();
    let fired = EventSet::empty().with(0);
    let original = seal_execution_receipt(
        Digest::ZERO,
        Digest::hash(b"model"),
        Digest::hash(b"compiled"),
        1,
        Digest::hash(b"decision"),
        fired,
        fired,
        &guards,
    )
    .unwrap();

    let mut forged_fired = original;
    forged_fired.fired = EventSet::empty().with(1); // 1 is also admissible under empty guards

    let result = verify_execution_receipt(&forged_fired, &guards);
    assert_eq!(
        result,
        Ok(()),
        "Mutant 2 (dropped fired from digest) should falsely accept a receipt with an altered fired set"
    );
}

// Mutant 3: canonical_bytes drops `scheduler_decision_digest`.
// Proves this exact corruption allows a forged scheduler_decision_digest to pass verification.
#[cfg(feature = "mutant_3")]
#[test]
fn kill_mutant_3_accepts_forged_scheduler_decision() {
    let guards = ConcurrencyGuardTable::empty();
    let fired = EventSet::empty().with(0);
    let original = seal_execution_receipt(
        Digest::ZERO,
        Digest::hash(b"model"),
        Digest::hash(b"compiled"),
        1,
        Digest::hash(b"decision"),
        fired,
        fired,
        &guards,
    )
    .unwrap();

    let mut forged_decision = original;
    forged_decision.scheduler_decision_digest = Digest::hash(b"forged-decision");

    let result = verify_execution_receipt(&forged_decision, &guards);
    assert_eq!(
        result,
        Ok(()),
        "Mutant 3 (dropped scheduler_decision_digest from digest) should falsely accept a receipt with an altered decision digest"
    );
}
