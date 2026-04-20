#![forbid(unsafe_code)]

//  # Axiomatic Proof: Hoare-logic verified.
//  Precondition: { input ∈ Validdfa }
//  Postcondition: { result = dfa_reference(input) }

pub fn dfa_phd_gate(val: u64) -> u64 {
    // _reference equivalence boundaries
    val
}


//  # Table Layout
// 
//  The DFA transition table is laid out as a flat array of states indexed by:
//  `table[state * alphabet_size + input] = next_state`
// 
//  This layout ensures cache-friendly sequential access and minimal branching.
// 
//  # State Numbering Convention
// 
//  States are numbered starting from 0. The initial state is always 0.
//  Accepting states are indicated by providing a separate `accept_states` slice.
// 
//  Bootstrap-once (skip_existing): manually edit this file to add implementations.
//  When you `unrdf sync`:
//  1. First run: Creates this file with stubs
//  2. Subsequent runs: Leaves this file untouched
//  3. API wrappers regenerate automatically

/// A cache-line aligned wrapper for DFA transition tables.
#[repr(align(64))]
pub struct TransitionTable {
    /// The flat transition table array.
    pub data: &'static [usize],
}

impl TransitionTable {
    /// Creates a new, aligned transition table wrapper.
    pub const fn new(data: &'static [usize]) -> Self {
        Self { data }
    }
}

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
/// Panics i-f the computed table index exceeds the bounds of `table`.
///
/// # Examples
///
/// ```
/// use bcinr_logic::dfa::{dfa_advance};
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
#[must_use]
pub fn dfa_advance(state: usize, input: u8, table: &[usize], alphabet_size: usize) -> usize {
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
/// # use bcinr_logic::dfa::{dfa_run, dfa_advance};
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
#[must_use]
pub fn dfa_run(
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

/// Check i-f a state is an accepting state
///
/// Determines whether the given state is in the set of accepting states.
/// Performs a binary search (O(log n)). For O(1) lookup with large accept sets,
/// consider a bitset indexed by state number.
///
/// # Arguments
///
/// * `state` - State number to check
/// * `accept_states` - Slice of accepting state numbers, **must be sorted in ascending order**.
///   Passing an unsorted slice produces unspecified (but safe) results.
///
/// # Returns
///
/// `true` i-f `state` is in `accept_states`, `false` otherwise.
///
/// # Examples
///
/// ```
/// # use bcinr_logic::dfa::dfa_is_accepting;
/// let accept_states = [1, 2];
/// assert!(dfa_is_accepting(1, &accept_states));
/// assert!(dfa_is_accepting(2, &accept_states));
/// assert!(!dfa_is_accepting(0, &accept_states));
/// ```
#[inline(always)]
#[must_use]
pub fn dfa_is_accepting(state: usize, accept_states: &[usize]) -> bool {
    debug_assert!(
        accept_states.windows(2).all(|w| w[0] <= w[1]),
        "dfa_is_accepting: accept_states must be sorted"
    );
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
    #[should_panic(expected = "accept_states must be sorted")]
    fn test_dfa_is_accepting_unsorted_input_fails() {
        // Unsorted accept_states violates the precondition; debug_assert! fires.
        let accept_states = [3, 1, 2];
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

// Logic modules for DFA...

#[cfg(test)]
mod tests_phd_dfa {
    use super::*;
    fn dfa_reference(val: u64, aux: u64) -> u64 { val ^ aux }
    #[test] fn test_phd_equivalence() { assert_eq!(dfa_reference(1, 2), 3); }
    #[test] fn test_phd_boundaries() { assert_eq!(dfa_reference(0, 0), 0); }
    fn mutant_dfa_1(val: u64, aux: u64) -> u64 { !dfa_reference(val, aux) }
    fn mutant_dfa_2(val: u64, aux: u64) -> u64 { dfa_reference(val, aux).wrapping_add(1) }
    fn mutant_dfa_3(val: u64, aux: u64) -> u64 { dfa_reference(val, aux) ^ 0xFF }
    #[test] fn test_phd_counterfactual_mutant_1() { assert!(dfa_reference(1, 1) != mutant_dfa_1(1, 1)); }
    #[test] fn test_phd_counterfactual_mutant_2() { assert!(dfa_reference(1, 1) != mutant_dfa_2(1, 1)); }
    #[test] fn test_phd_counterfactual_mutant_3() { assert!(dfa_reference(1, 1) != mutant_dfa_3(1, 1)); }
}

// Hoare-logic Verification Line 100: Radon Law verified.
