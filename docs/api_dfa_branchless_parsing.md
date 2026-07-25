# Branchless DFA Parsing in `bcinr-logic::dfa`

The deterministic finite automaton (DFA) implementation achieves strict CC=1 (Cyclomatic Complexity 1) and predictable execution latency by removing all `if`, `match`, and early-exit loop structures. Instead, it relies on mathematical operations, bitwise masking, and unconditional iteration.

## 1. Branchless State Transitions (`dfa_advance`)

The `dfa_advance` function determines the next state using a flat transition table without any conditional jumps or panic paths:

```rust
pub fn dfa_advance(state: usize, input: u8, table: &[usize], alphabet_size: usize) -> usize {
    let index = state
        .wrapping_mul(alphabet_size)
        .wrapping_add(input as usize);
    let mask = (index < table.len()) as usize;
    let safe_idx = index & (0usize.wrapping_sub(mask));
    table[safe_idx] & (0usize.wrapping_sub(mask))
}
```

**How it works branchlessly:**
- **Index Calculation:** Computes the conceptual 2D array index (`state * alphabet_size + input`) using `wrapping_mul` and `wrapping_add` to prevent overflow panics.
- **Bounds Masking:** Compares the index against `table.len()` and casts the boolean result to `usize` (`1` if valid, `0` if out-of-bounds).
- **Safe Indexing:** Computes `0usize.wrapping_sub(mask)`. 
  - If `mask == 1`, this results in `usize::MAX` (all 1s).
  - If `mask == 0`, this results in `0` (all 0s).
  Applying a bitwise AND (`index & ...`) forces the array lookup to index `0` if the calculated index is out-of-bounds, preventing runtime bounds-checking panics.
- **Output Masking:** Looks up the transition in `table[safe_idx]`. If the original index was out-of-bounds, the lookup result is masked with `0`, silently defaulting to state `0` (typically the start/reject state) instead of branching to an error handler.

## 2. Unconditional Input Consumption (`dfa_run`)

The `dfa_run` function consumes the entire input stream sequentially without early termination:

```rust
pub fn dfa_run(table: &[usize], alphabet_size: usize, initial_state: usize, input: &[u8]) -> usize {
    let mut state = initial_state;
    input.iter().for_each(|&b| {
        state = dfa_advance(state, b, table, alphabet_size);
    });
    state
}
```

**How it works branchlessly:**
- It uses a functional `.for_each()` loop over the input byte slice.
- There are no `break` conditions, meaning latency is purely proportional to input length and unaffected by the internal path of states being traversed, closing potential timing side-channels.

## 3. Branchless Acceptance Checking (`dfa_is_accepting`)

The `dfa_is_accepting` function checks if the current state is within the list of valid accepting states without short-circuiting:

```rust
pub fn dfa_is_accepting(state: usize, accept_states: &[usize]) -> bool {
    let mut found = 0usize;
    (0..accept_states.len()).for_each(|i| {
        found |= (accept_states[i] == state) as usize;
    });
    found != 0
}
```

**How it works branchlessly:**
- **Full Iteration:** Iterates through the entire `accept_states` slice using `.for_each()` regardless of whether a match has already been found.
- **Bitwise Accumulation:** Casts the equality comparison (`accept_states[i] == state`) to `usize` (0 or 1) and accumulates it into `found` using bitwise OR (`|=`).
- **Final Evaluation:** Returns true only if the accumulated `found` value is non-zero, safely verifying the state without any internal conditionals.
