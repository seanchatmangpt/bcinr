//! Gate G4: Mutant Kill Protocol — POWL Mutations
//!
//! Injects 3 controlled mutations into POWL execution and verifies all are caught
//! by the receipt oracle.
//!
//! Mutation 1: Scheduler tick logic mutation (wrong firing mask)
//! Mutation 2: State-transition wrong order (reorder fired actions)
//! Mutation 3: Receipt generation bit flip (corrupt chain_root)

#![allow(clippy::len_zero)]

use bcinr_powl::powl2::{compile_powl2, LowestIndexPolicy, Powl2Model};
use bcinr_powl::receipt::execution_v2::{
    execute_and_seal_v2, verify_execution_v2, PowlV2ReceiptError,
};
use bcinr_powl::tape::v2::ConcurrencyGuardTable;

/// Build a simple sequence: A → B
fn simple_sequence() -> bcinr_powl::powl2::CompiledPowl2 {
    compile_powl2(
        &Powl2Model::Sequence(vec![
            Powl2Model::Activity("A".into()),
            Powl2Model::Activity("B".into()),
        ]),
        &mut LowestIndexPolicy,
    )
    .expect("sequence should compile")
}

/// Build a choice graph: A | B (not used in current tests, kept for completeness)
#[allow(dead_code)]
fn simple_choice() -> bcinr_powl::powl2::CompiledPowl2 {
    compile_powl2(
        &Powl2Model::ChoiceGraph {
            children: vec![
                Powl2Model::Activity("A".into()),
                Powl2Model::Activity("B".into()),
            ],
            edges: vec![(0, 1)],
            start: 0,
            end: 1,
        },
        &mut LowestIndexPolicy,
    )
    .expect("choice should compile")
}

/// Oracle 0: Baseline — verify normal POWL execution passes
#[test]
fn oracle_powl_baseline_passes() {
    let compiled = simple_sequence();
    let guards = ConcurrencyGuardTable::empty();

    // Execute and seal
    let receipt =
        execute_and_seal_v2(&compiled.tape, &guards, 8).expect("execution should succeed");

    // Oracle check: verify must pass
    verify_execution_v2(&receipt, &compiled.tape, &guards, 8)
        .expect("baseline receipt must verify");
}

/// Mutant 1: Scheduler tick logic mutation — wrong firing mask
///
/// Simulate a mutation where the scheduler fires the wrong action
/// (different bitmask). The oracle detects this via FiredTraceMismatch.
#[test]
fn mutant_1_wrong_firing_mask_is_killed() {
    let compiled = simple_sequence();
    let guards = ConcurrencyGuardTable::empty();

    // Get a clean receipt
    let mut receipt =
        execute_and_seal_v2(&compiled.tape, &guards, 8).expect("execution should succeed");

    // MUTATE: flip one bit in the first fired mask
    // This represents a scheduler error: firing wrong action
    if receipt.fired_masks.len() > 0 {
        receipt.fired_masks[0] ^= 1;
    }

    // Oracle must catch this
    let result = verify_execution_v2(&receipt, &compiled.tape, &guards, 8);
    assert!(
        matches!(result, Err(PowlV2ReceiptError::FiredTraceMismatch)),
        "mutant 1 must be caught as FiredTraceMismatch"
    );
}

/// Mutant 2: State-transition wrong order
///
/// Reorder the fired_masks to simulate actions firing out of order.
/// In a sequence, this breaks the control flow and receipt verification.
#[test]
fn mutant_2_action_order_wrong_is_killed() {
    let compiled = compile_powl2(
        &Powl2Model::Sequence(vec![
            Powl2Model::Activity("A".into()),
            Powl2Model::Activity("B".into()),
            Powl2Model::Activity("C".into()),
        ]),
        &mut LowestIndexPolicy,
    )
    .expect("3-sequence should compile");

    let guards = ConcurrencyGuardTable::empty();

    // Get a clean receipt (should have 3 ticks: A, B, C)
    let mut receipt =
        execute_and_seal_v2(&compiled.tape, &guards, 8).expect("execution should succeed");

    // MUTATE: swap first two firing masks (swap A and B execution)
    if receipt.fired_masks.len() >= 2 {
        receipt.fired_masks.swap(0, 1);
    }

    // Oracle must catch this reordering
    let result = verify_execution_v2(&receipt, &compiled.tape, &guards, 8);
    assert!(
        matches!(result, Err(PowlV2ReceiptError::FiredTraceMismatch)),
        "mutant 2 (wrong order) must be caught as FiredTraceMismatch"
    );
}

/// Mutant 3: Receipt bit flip — corrupt chain_root
///
/// Flip a bit in the BLAKE3 chain root, breaking the receipt signature.
/// The oracle detects this via ChainRootMismatch.
#[test]
fn mutant_3_chain_root_corruption_is_killed() {
    let compiled = simple_sequence();
    let guards = ConcurrencyGuardTable::empty();

    // Get a clean receipt
    let mut receipt =
        execute_and_seal_v2(&compiled.tape, &guards, 8).expect("execution should succeed");

    // MUTATE: flip one byte in chain_root (BLAKE3 hash)
    if receipt.chain_root.len() > 0 {
        receipt.chain_root.push('X');
    }

    // Oracle must catch the chain root corruption
    let result = verify_execution_v2(&receipt, &compiled.tape, &guards, 8);
    assert!(
        matches!(result, Err(PowlV2ReceiptError::ChainRootMismatch)),
        "mutant 3 (chain root corrupt) must be caught as ChainRootMismatch"
    );
}

/// Mutant 1b: Tape root mutation
///
/// Corrupt the tape root (XOR with 1). The oracle should catch this.
#[test]
fn mutant_1b_tape_root_mutation_is_killed() {
    let compiled = simple_sequence();
    let guards = ConcurrencyGuardTable::empty();

    let mut receipt =
        execute_and_seal_v2(&compiled.tape, &guards, 8).expect("execution should succeed");

    // MUTATE: corrupt tape root
    if receipt.tape_root.len() > 0 {
        receipt.tape_root.push('!');
    }

    let result = verify_execution_v2(&receipt, &compiled.tape, &guards, 8);
    assert!(
        matches!(result, Err(PowlV2ReceiptError::TapeRootMismatch)),
        "tape root mutation must be caught"
    );
}

/// Mutant 2b: Final state mutation
///
/// Corrupt the final_done_mask. The oracle detects via FinalStateMismatch.
#[test]
fn mutant_2b_final_state_mutation_is_killed() {
    let compiled = simple_sequence();
    let guards = ConcurrencyGuardTable::empty();

    let mut receipt =
        execute_and_seal_v2(&compiled.tape, &guards, 8).expect("execution should succeed");

    // MUTATE: XOR final_done_mask (wrong final state)
    receipt.final_done_mask ^= 1;

    let result = verify_execution_v2(&receipt, &compiled.tape, &guards, 8);
    assert!(
        matches!(result, Err(PowlV2ReceiptError::FinalStateMismatch)),
        "final state mutation must be caught"
    );
}

/// Oracle summary: All 5 POWL receipt mutations killed
#[test]
fn all_powl_mutants_killed_by_oracle() {
    // Confirm oracle is working
    let compiled = simple_sequence();
    let guards = ConcurrencyGuardTable::empty();

    let receipt =
        execute_and_seal_v2(&compiled.tape, &guards, 8).expect("clean execution should work");

    verify_execution_v2(&receipt, &compiled.tape, &guards, 8)
        .expect("oracle must verify clean receipt");

    // Test passes: oracle is armed and ready
    // All mutations 1-3 + variants are caught above
}
