//! Table-Driven Automata — public API
//!
//! Finite automata via lookup tables
//!
//! This module is 100% generated. Do not edit directly.
//! Modifications will be overwritten on the next `unrdf sync`.
//!
//! This is a thin wrapper layer around `crate::logic::dfa`.
//! All functions are inline for zero-cost abstraction.

use crate::logic::dfa as inner;

/// dfa_advance from the Table-Driven Automata family
///
/// Single DFA state transition via lookup table.
/// Table layout assumes row-major indexing: `table[state * alphabet_size + input]`.
///
/// # Arguments
///
/// * `state` - Current state number
/// * `input` - Input symbol (typically a byte 0-255)
/// * `table` - Flat transition table
/// * `alphabet_size` - Size of input alphabet (typically 256)
///
/// # Returns
///
/// Next state after consuming the input symbol.
#[inline(always)]
pub fn dfa_advance(state: usize, input: u8, table: &[usize], alphabet_size: usize) -> usize {
    inner::dfa_advance(state, input, table, alphabet_size)
}

/// dfa_run from the Table-Driven Automata family
///
/// Run DFA to completion and return the final state.
/// Processes all input symbols in sequence according to the transition table.
///
/// # Arguments
///
/// * `input` - Slice of input symbols to process
/// * `table` - Flat transition table
/// * `alphabet_size` - Size of input alphabet (typically 256)
/// * `initial_state` - Starting state (typically 0)
///
/// # Returns
///
/// Final state after processing all input symbols.
#[inline(always)]
pub fn dfa_run(
    input: &[u8],
    table: &[usize],
    alphabet_size: usize,
    initial_state: usize,
) -> usize {
    inner::dfa_run(input, table, alphabet_size, initial_state)
}

/// dfa_is_accepting from the Table-Driven Automata family
///
/// Check if a state is an accepting state.
/// Performs binary search on sorted accept_states slice.
///
/// # Arguments
///
/// * `state` - State number to check
/// * `accept_states` - Sorted slice of accepting state numbers
///
/// # Returns
///
/// `true` if `state` is in `accept_states`, `false` otherwise.
#[inline(always)]
pub fn dfa_is_accepting(state: usize, accept_states: &[usize]) -> bool {
    inner::dfa_is_accepting(state, accept_states)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_symbols_exist() {
        // Compile-time check: if this test compiles, the API contract is satisfied
    }

    #[test]
    fn test_api_dfa_advance() {
        let mut table = [0usize; 2 * 256];
        table[0 * 256 + b'a' as usize] = 1;
        let result = dfa_advance(0, b'a', &table, 256);
        assert_eq!(result, 1);
    }

    #[test]
    fn test_api_dfa_run() {
        let mut table = [0usize; 2 * 256];
        table[0 * 256 + b'a' as usize] = 1;
        table[1 * 256 + b'a' as usize] = 1;
        let result = dfa_run(b"aa", &table, 256, 0);
        assert_eq!(result, 1);
    }

    #[test]
    fn test_api_dfa_is_accepting() {
        let accept_states = [1, 2, 3];
        assert!(dfa_is_accepting(1, &accept_states));
        assert!(!dfa_is_accepting(0, &accept_states));
    }
}
