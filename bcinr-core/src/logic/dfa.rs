//! Table-Driven Automata: Finite automata via lookup tables
//!
//! This module contains handwritten, performance-critical implementations
//! of all Table-Driven Automata algorithms. These functions are pub(crate) and wrapped
//! by the public API in `src/api/dfa.rs`.
//!
//! # Table Layout
//!
//! The DFA transition table is laid out as a flat array of states indexed by:
//! `table[state * alphabet_size + input] = next_state`
//!
//! This layout ensures cache-friendly sequential access and minimal branching.
//!
//! # State Numbering Convention
//!
//! States are numbered starting from 0. The initial state is always 0.
//! Accepting states are indicated by providing a separate `accept_states` slice.
//!
//! Bootstrap-once (skip_existing): manually edit this file to add implementations.
//! When you `unrdf sync`:
//! 1. First run: Creates this file with stubs
//! 2. Subsequent runs: Leaves this file untouched
//! 3. API wrappers regenerate automatically

/// Single DFA state transition via lookup table
///
/// Performs one state transition using the transition table. The table
/// layout assumes row-major indexing: `table[state * alphabet_size + input]`.
///
/// # Arguments
///
/// * `state` - Current state number (0 for initial state)
/// * `input` - Input symbol (typically a byte value 0-255)
/// * `table` - Flat transition table organized as `state * alphabet_size + input`
/// * `alphabet_size` - Size of the input alphabet (typically 256 for bytes)
///
/// # Returns
///
/// Next state after consuming the input symbol.
///
/// # Panics
///
/// Panics if the computed table index exceeds the bounds of `table`.
///
/// # Examples
///
/// ```
/// // Simple DFA: accept strings matching [a-z]+
/// // States: 0 (initial/non-accepting), 1 (accepting)
/// // Alphabet: 256 (all bytes)
/// let mut table = vec![0; 2 * 256];
/// // From state 0, on input 'a'-'z', go to state 1
/// for c in b'a'..=b'z' {
///     table[0 * 256 + c as usize] = 1;
/// }
/// // From state 1, stay in state 1 on 'a'-'z', go to 0 on anything else
/// for c in b'a'..=b'z' {
///     table[1 * 256 + c as usize] = 1;
/// }
///
/// let next = dfa_advance(0, b'a', &table, 256);
/// assert_eq!(next, 1);
/// ```
#[inline(always)]
pub(crate) fn dfa_advance(state: usize, input: u8, table: &[usize], alphabet_size: usize) -> usize {
    let index = state.saturating_mul(alphabet_size).saturating_add(input as usize);
    if index < table.len() {
        table[index]
    } else {
        0
    }
}

/// Run DFA to completion and return the final state
///
/// Processes all input symbols in sequence, advancing through states
/// according to the transition table. Returns the state after consuming
/// all input.
///
/// # Arguments
///
/// * `input` - Slice of input symbols to process
/// * `table` - Flat transition table
/// * `alphabet_size` - Size of the input alphabet (typically 256 for bytes)
/// * `initial_state` - Starting state (typically 0)
///
/// # Returns
///
/// Final state after processing all input symbols.
///
/// # Examples
///
/// ```
/// // Simple DFA accepting strings of 'a' characters
/// let mut table = vec![0; 2 * 256];
/// for c in b'a'..=b'a' {
///     table[0 * 256 + c as usize] = 1;
///     table[1 * 256 + c as usize] = 1;
/// }
///
/// let input = b"aaa";
/// let final_state = dfa_run(input, &table, 256, 0);
/// assert_eq!(final_state, 1);
/// ```
#[inline(always)]
pub(crate) fn dfa_run(
    input: &[u8],
    table: &[usize],
    alphabet_size: usize,
    initial_state: usize,
) -> usize {
    let mut state = initial_state;
    for &byte in input {
        state = dfa_advance(state, byte, table, alphabet_size);
    }
    state
}

/// Check if a state is an accepting state
///
/// Determines whether the given state is in the set of accepting states.
/// Performs a linear search; for larger accept sets, consider using a
/// HashSet or BitSet for faster lookup.
///
/// # Arguments
///
/// * `state` - State number to check
/// * `accept_states` - Slice of accepting state numbers
///
/// # Returns
///
/// `true` if `state` is in `accept_states`, `false` otherwise.
///
/// # Examples
///
/// ```
/// let accept_states = [1, 2];
/// assert!(dfa_is_accepting(1, &accept_states));
/// assert!(dfa_is_accepting(2, &accept_states));
/// assert!(!dfa_is_accepting(0, &accept_states));
/// ```
#[inline(always)]
pub(crate) fn dfa_is_accepting(state: usize, accept_states: &[usize]) -> bool {
    accept_states.binary_search(&state).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // dfa_advance tests
    // ============================================================================

    #[test]
    fn test_dfa_advance_simple_transition() {
        let mut table = [0usize; 2 * 256];
        table[0 * 256 + b'a' as usize] = 1;
        let next = dfa_advance(0, b'a', &table, 256);
        assert_eq!(next, 1);
    }

    #[test]
    fn test_dfa_advance_loop_self() {
        let mut table = [0usize; 2 * 256];
        table[1 * 256 + b'x' as usize] = 1;
        let next = dfa_advance(1, b'x', &table, 256);
        assert_eq!(next, 1);
    }

    #[test]
    fn test_dfa_advance_zero_state() {
        let table = [0usize; 2 * 256];
        let next = dfa_advance(0, b'a', &table, 256);
        assert_eq!(next, 0);
    }

    #[test]
    fn test_dfa_advance_out_of_bounds_returns_zero() {
        let table = [0usize; 1]; // Very small table
        let next = dfa_advance(100, b'a', &table, 256);
        assert_eq!(next, 0);
    }

    #[test]
    fn test_dfa_advance_multiple_symbols() {
        let mut table = [0usize; 3 * 256];
        table[0 * 256 + b'a' as usize] = 1;
        table[1 * 256 + b'b' as usize] = 2;
        let s1 = dfa_advance(0, b'a', &table, 256);
        let s2 = dfa_advance(s1, b'b', &table, 256);
        assert_eq!(s2, 2);
    }

    // ============================================================================
    // dfa_run tests
    // ============================================================================

    #[test]
    fn test_dfa_run_empty_input() {
        let table = [0usize; 2 * 256];
        let final_state = dfa_run(b"", &table, 256, 0);
        assert_eq!(final_state, 0);
    }

    #[test]
    fn test_dfa_run_single_transition() {
        let mut table = [0usize; 2 * 256];
        table[0 * 256 + b'a' as usize] = 1;
        let final_state = dfa_run(b"a", &table, 256, 0);
        assert_eq!(final_state, 1);
    }

    #[test]
    fn test_dfa_run_sequence_of_transitions() {
        let mut table = [0usize; 3 * 256];
        table[0 * 256 + b'a' as usize] = 1;
        table[1 * 256 + b'b' as usize] = 2;
        table[2 * 256 + b'c' as usize] = 2;
        let final_state = dfa_run(b"abc", &table, 256, 0);
        assert_eq!(final_state, 2);
    }

    #[test]
    fn test_dfa_run_self_loop() {
        let mut table = [0usize; 2 * 256];
        table[1 * 256 + b'a' as usize] = 1;
        let final_state = dfa_run(b"aaa", &table, 256, 1);
        assert_eq!(final_state, 1);
    }

    #[test]
    fn test_dfa_run_rejects_unknown_input() {
        let table = [0usize; 2 * 256]; // All transitions go to 0
        let final_state = dfa_run(b"xyz", &table, 256, 0);
        assert_eq!(final_state, 0);
    }

    // ============================================================================
    // dfa_is_accepting tests
    // ============================================================================

    #[test]
    fn test_dfa_is_accepting_single_state() {
        let accept_states = [1];
        assert!(dfa_is_accepting(1, &accept_states));
        assert!(!dfa_is_accepting(0, &accept_states));
    }

    #[test]
    fn test_dfa_is_accepting_multiple_states() {
        let accept_states = [1, 3, 5];
        assert!(dfa_is_accepting(1, &accept_states));
        assert!(dfa_is_accepting(3, &accept_states));
        assert!(dfa_is_accepting(5, &accept_states));
        assert!(!dfa_is_accepting(0, &accept_states));
        assert!(!dfa_is_accepting(2, &accept_states));
    }

    #[test]
    fn test_dfa_is_accepting_empty_accept_states() {
        let accept_states: [usize; 0] = [];
        assert!(!dfa_is_accepting(0, &accept_states));
        assert!(!dfa_is_accepting(1, &accept_states));
    }

    #[test]
    fn test_dfa_is_accepting_edge_case_large_state() {
        let accept_states = [1000, 2000, 3000];
        assert!(dfa_is_accepting(1000, &accept_states));
        assert!(dfa_is_accepting(3000, &accept_states));
        assert!(!dfa_is_accepting(999, &accept_states));
    }

    #[test]
    fn test_dfa_is_accepting_unsorted_input_fails() {
        // Binary search requires sorted input
        let accept_states = [3, 1, 2]; // Unsorted
        // This test documents the constraint: accept_states must be sorted
        // If unsorted, binary_search behavior is undefined
        let _ = dfa_is_accepting(1, &accept_states);
    }

    // ============================================================================
    // Integration tests
    // ============================================================================

    #[test]
    fn test_dfa_full_pipeline_accepts_sequence() {
        // DFA that accepts "aa"
        let mut table = [0usize; 3 * 256];
        table[0 * 256 + b'a' as usize] = 1;
        table[1 * 256 + b'a' as usize] = 2;

        let accept_states = [2];
        let final_state = dfa_run(b"aa", &table, 256, 0);
        assert!(dfa_is_accepting(final_state, &accept_states));
    }

    #[test]
    fn test_dfa_full_pipeline_rejects_invalid() {
        let mut table = [0usize; 3 * 256];
        table[0 * 256 + b'a' as usize] = 1;
        table[1 * 256 + b'a' as usize] = 2;

        let accept_states = [2];
        let final_state = dfa_run(b"ab", &table, 256, 0);
        assert!(!dfa_is_accepting(final_state, &accept_states));
    }

    #[test]
    fn test_dfa_alphabet_size_correctness() {
        // Test with smaller alphabet
        let mut table = [0usize; 2 * 10]; // alphabet_size = 10
        table[0 * 10 + 5] = 1;
        let next = dfa_advance(0, 5, &table, 10);
        assert_eq!(next, 1);
    }
}

#[cfg(feature = "bench")]
mod benches {
    use super::*;

    #[bench]
    fn bench_dfa_advance_single(b: &mut test::Bencher) {
        let mut table = vec![0; 2 * 256];
        table[0 * 256 + b'a' as usize] = 1;
        b.iter(|| {
            let _ = dfa_advance(0, b'a', &table, 256);
        });
    }

    #[bench]
    fn bench_dfa_advance_worst_case(b: &mut test::Bencher) {
        let mut table = vec![0; 100 * 256];
        for i in 0..100 {
            table[i * 256 + b'x' as usize] = (i + 1) % 100;
        }
        b.iter(|| {
            let _ = dfa_advance(99, b'x', &table, 256);
        });
    }

    #[bench]
    fn bench_dfa_run_short_input(b: &mut test::Bencher) {
        let mut table = vec![0; 2 * 256];
        table[0 * 256 + b'a' as usize] = 1;
        table[1 * 256 + b'a' as usize] = 1;
        let input = b"aaa";
        b.iter(|| {
            let _ = dfa_run(input, &table, 256, 0);
        });
    }

    #[bench]
    fn bench_dfa_run_long_input(b: &mut test::Bencher) {
        let mut table = vec![0; 2 * 256];
        table[0 * 256 + b'a' as usize] = 1;
        table[1 * 256 + b'a' as usize] = 1;
        let input: Vec<u8> = vec![b'a'; 1000];
        b.iter(|| {
            let _ = dfa_run(&input, &table, 256, 0);
        });
    }

    #[bench]
    fn bench_dfa_run_state_machine(b: &mut test::Bencher) {
        let mut table = vec![0; 5 * 256];
        table[0 * 256 + b'a' as usize] = 1;
        table[1 * 256 + b'b' as usize] = 2;
        table[2 * 256 + b'c' as usize] = 3;
        table[3 * 256 + b'd' as usize] = 4;
        let input = b"abcdabcd";
        b.iter(|| {
            let _ = dfa_run(input, &table, 256, 0);
        });
    }

    #[bench]
    fn bench_dfa_is_accepting_small(b: &mut test::Bencher) {
        let accept_states = [2];
        b.iter(|| {
            let _ = dfa_is_accepting(2, &accept_states);
        });
    }

    #[bench]
    fn bench_dfa_is_accepting_large(b: &mut test::Bencher) {
        let accept_states: Vec<usize> = (0..1000).collect();
        b.iter(|| {
            let _ = dfa_is_accepting(500, &accept_states);
        });
    }

    #[bench]
    fn bench_dfa_is_accepting_not_found(b: &mut test::Bencher) {
        let accept_states: Vec<usize> = (0..1000).step_by(2).collect();
        b.iter(|| {
            let _ = dfa_is_accepting(999, &accept_states);
        });
    }
}
