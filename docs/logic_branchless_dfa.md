# Branchless DFA Execution in BCINR

The `bcinr-logic` crate implements Deterministic Finite Automata (DFA) under the strict Radon Law ($CC=1$). This means no `if`, `match`, or early-exit loops are permitted. All logic must be expressed as bitwise polynomials to ensure deterministic, constant-time latency.

## 1. DFA Data Modeling

Instead of pointers, references, or branching node architectures, a DFA is modeled as a **flat, 1D array transition table**. 
The formula to compute the next index is strictly algebraic:
`index = state * alphabet_size + input`

This design forces every state transition to be an $O(1)$ memory lookup, eliminating data-dependent jumps. 

## 2. Branchless Advancement (`dfa_advance`)

In a conventional implementation, bounds checking would require an `if` statement to handle out-of-range inputs or states. Under $CC=1$, out-of-bounds safety is achieved using arithmetic masking.

```rust
pub fn dfa_advance(state: usize, input: u8, table: &[usize], alphabet_size: usize) -> usize {
    let index = state
        .wrapping_mul(alphabet_size)
        .wrapping_add(input as usize);
        
    // 1 if in bounds, 0 if out of bounds
    let mask = (index < table.len()) as usize;
    
    // 0usize.wrapping_sub(mask) creates an all-1s mask if mask=1, or all-0s if mask=0.
    let safe_idx = index & (0usize.wrapping_sub(mask));
    
    // Safely look up index 0 if out of bounds, and mask the result to 0.
    table[safe_idx] & (0usize.wrapping_sub(mask))
}
```
If the transition is out of bounds, `mask` evaluates to `0`. `0usize.wrapping_sub(0)` produces `0x0000...`. The index falls back to `0`, ensuring memory safety, and the final state returned is completely zeroed out to reflect a deterministic fallback state, all without a single CPU branch.

## 3. Unconditional Acceptance Checking (`dfa_is_accepting`)

Normally, checking if a state is an accepting state would involve iterating through a list and exiting early upon finding a match (e.g. using `contains()`). This violates the $CC=1$ rule because the execution time depends on when the match is found.

Instead, the `bcinr` DFA implements an exhaustive bitwise folding check:
```rust
pub fn dfa_is_accepting(state: usize, accept_states: &[usize]) -> bool {
    let mut found = 0usize;
    // Unconditional iteration over the entire slice
    (0..accept_states.len()).for_each(|i| {
        found |= (accept_states[i] == state) as usize;
    });
    found != 0
}
```
This guarantees fixed execution work ($O(N)$ for $N$ accepting states) regardless of the provided state.

## 4. Policy-Gated Transitions (`ConstantShapePolicyDfa`)

DFA execution can be integrated with security rules using the `ConstantShapePolicyDfa` (from `patterns/policy_dfa.rs`), which evaluates DFA state transitions against a policy mask.

Instead of rejecting a transition via an `if` block, a bitmask policy gating approach is used:
```rust
pub fn step(&self, current_state: usize, input: u8) -> (usize, u64) {
    let next = dfa_advance(current_state, input, self.table, self.alphabet_size);
    
    // Create a bitmask representing the state
    let state_bit = 1u64.wrapping_shl((next as u32) & 0x3F);
    
    // Intersect with a blacklist mask
    let blacklisted = self.blacklisted_states_mask & state_bit;
    
    // Yield an all-1s mask if valid (blacklisted == 0), or all-0s if invalid
    let allowed_mask = PolicyGuard::mask_eq(blacklisted, 0);
    
    // Multiplex between the valid next state and the error state
    let gated_state =
        ((next as u64 & allowed_mask) | (self.error_state as u64 & !allowed_mask)) as usize;
        
    (gated_state, allowed_mask)
}
```
This forces the execution into a constant-shape CPU pipeline. If the target state violates the policy, the arithmetic gracefully falls back to the `error_state` via the bitwise `|` multiplexer, upholding the mission of a deterministic computational substrate.
