# Branchless DFA Parsing in `bcinr-logic`

The logic libraries in BCINR implement Deterministic Finite Automata (DFA) parsing following strict branchless, deterministic execution laws (Radon Law, $CC=1$, zero allocations). This guarantees identical execution latency across all input symbols, effectively eliminating timing side-channels.

## Core Primitives

The base DFA building blocks are defined in `crates/bcinr-logic/src/dfa.rs`:

1.  **Flat Transition Tables**: DFAs are modeled as flat transition arrays of `usize`. The next state is retrieved by calculating `table[state * alphabet_size + input]`.
2.  **`dfa_advance`**: Steps the DFA by one symbol. It uses safe-index masking—a bitwise operation derived from bounds checking (`index < table.len()`). If an out-of-bounds input is provided, the index and result are safely zeroed via bitwise AND, returning state 0 without panicking or branching.
3.  **`dfa_run`**: A wrapper loop that consumes a byte slice and feeds each byte sequentially into `dfa_advance`. Because each step is fixed-time, the function’s execution time is strictly proportional to the input slice length.
4.  **`dfa_is_accepting`**: Verifies if a given state is within a set of accepting states. It iterates over the entire `accept_states` array and accumulates a bitwise-OR flag without any early exits or jumps.

## Policy-Gated Transitions (`ConstantShapePolicyDfa`)

Found in `crates/bcinr-logic/src/patterns/policy_dfa.rs`, this pattern extends the basic DFA with branchless policy enforcement:

-   **Bitmask Validation**: A `blacklisted_states_mask` (`u64`) defines disallowed states, capping the supported state count at 64.
-   **Mask-Based Substitution**: If `dfa_advance` yields a blacklisted state, the implementation uses `PolicyGuard` to generate a 64-bit mask. This mask is then used to branchlessly substitute the resulting state with a predefined `error_state` via a bitwise selection `(next & allowed_mask) | (error_state & !allowed_mask)`.
-   **Timing Contract**: Requires a tight bound—under 8 cycles (~2 ns) per symbol lookup, or under 200 ns in aggregate per step.

## Usage Example

`bcinr/examples/dfa_matching.rs` illustrates a basic implementation—a parity checker that evaluates sequences of `0` and `1` bytes. It uses a 2-element alphabet and flat array mapping `[0, 1, 1, 0]` to track the odd/even count of `1`s entirely branchlessly.
