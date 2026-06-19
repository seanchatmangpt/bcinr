//! # DFA Pattern Matching Example
//!
//! Demonstrates `bcinr_logic::dfa`: constructing a transition table, running a
//! DFA on a byte sequence branchlessly, and testing acceptance.
//!
//! **Doc reference:** `crates/bcinr-logic/src/dfa.rs`
//! **Also see:** `examples/branchless_pipeline.rs` — mask/bitset composition.
//!
//! The DFA table is a flat array indexed by `state * alphabet_size + byte_input`.
//! `dfa_advance` does a single branchless table lookup per byte; `dfa_run` chains
//! advances; `dfa_is_accepting` tests the final state. These assertions would fail
//! if the wrong state were returned after any transition.
//!
//! **DFA constructed here**: recognizes sequences whose byte sum is odd.
//! - State 0: even popcount (initial, non-accepting)
//! - State 1: odd popcount (accepting)
//! - Alphabet: {0=zero_byte, 1=one_byte} → alphabet_size = 2
//! - Transitions: S0+0→S0, S0+1→S1, S1+0→S1, S1+1→S0
//! Table (flat): [0, 1, 1, 0]

use bcinr::dfa::{dfa_advance, dfa_is_accepting, dfa_run};

const ALPHABET_SIZE: usize = 2;

// Parity DFA: counts '1' bytes mod 2.
// Input bytes: 0=even byte, 1=odd byte.
// table[state * ALPHABET_SIZE + input] = next_state
const PARITY_TABLE: &[usize] = &[
    0, 1, // from state 0: 0-byte→0, 1-byte→1
    1, 0, // from state 1: 0-byte→1, 1-byte→0
];
const ACCEPTING: &[usize] = &[1]; // state 1 = odd count = accepting

fn main() {
    // --- dfa_advance: single-step branchless transitions ---
    let s0_on_zero = dfa_advance(0, 0, PARITY_TABLE, ALPHABET_SIZE);
    assert_eq!(s0_on_zero, 0, "S0 + 0 → S0");
    let s0_on_one = dfa_advance(0, 1, PARITY_TABLE, ALPHABET_SIZE);
    assert_eq!(s0_on_one, 1, "S0 + 1 → S1");
    let s1_on_one = dfa_advance(1, 1, PARITY_TABLE, ALPHABET_SIZE);
    assert_eq!(s1_on_one, 0, "S1 + 1 → S0 (two 1s = even)");
    println!("dfa_advance: S0+0={s0_on_zero}, S0+1={s0_on_one}, S1+1={s1_on_one}");

    // --- dfa_is_accepting: correct state recognition ---
    assert!(!dfa_is_accepting(0, ACCEPTING), "state 0 is not accepting");
    assert!(dfa_is_accepting(1, ACCEPTING), "state 1 is accepting");
    println!(
        "dfa_is_accepting(0)={}, dfa_is_accepting(1)={}",
        dfa_is_accepting(0, ACCEPTING),
        dfa_is_accepting(1, ACCEPTING)
    );

    // --- dfa_run on concrete sequences ---
    // [1, 1, 1] → 3 ones → odd count → state 1 (accept)
    let s = dfa_run(PARITY_TABLE, ALPHABET_SIZE, 0, &[1, 1, 1]);
    assert_eq!(s, 1, "three 1-bytes → odd count → state 1");
    assert!(dfa_is_accepting(s, ACCEPTING));
    println!(
        "dfa_run([1,1,1]) → state {s} (accepted={})",
        dfa_is_accepting(s, ACCEPTING)
    );

    // [1, 1] → 2 ones → even count → state 0 (reject)
    let s = dfa_run(PARITY_TABLE, ALPHABET_SIZE, 0, &[1, 1]);
    assert_eq!(s, 0, "two 1-bytes → even count → state 0");
    assert!(!dfa_is_accepting(s, ACCEPTING));
    println!(
        "dfa_run([1,1]) → state {s} (accepted={})",
        dfa_is_accepting(s, ACCEPTING)
    );

    // empty input → stays in initial state (not accepting)
    let s = dfa_run(PARITY_TABLE, ALPHABET_SIZE, 0, &[]);
    assert_eq!(s, 0, "empty input stays in state 0");
    assert!(!dfa_is_accepting(s, ACCEPTING));
    println!("dfa_run([]) → state {s}");

    // [0, 1, 0] → one 1 → odd → accept
    let s = dfa_run(PARITY_TABLE, ALPHABET_SIZE, 0, &[0, 1, 0]);
    assert_eq!(s, 1, "one 1-byte among zeros → accept");
    println!("dfa_run([0,1,0]) → state {s}");

    // out-of-bounds input byte (byte value ≥ alphabet_size) → safe return of 0
    let s_oob = dfa_advance(0, 99, PARITY_TABLE, ALPHABET_SIZE);
    println!("dfa_advance(S0, byte=99 [OOB]) → {s_oob} (branchlessly safe)");

    println!("\nAll DFA matching assertions passed.");
}
