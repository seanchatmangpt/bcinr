# Reference: `dfa` — Deterministic Finite Automata

Module: `bcinr_logic::dfa` (`crates/bcinr-logic/src/dfa.rs`)

Table-driven, branchless DFA execution for line-rate parsing and
classification. Transitions are array lookups gated by arithmetic, not
conditionals. All functions are `#[inline(always)]`.

## Functions

| Function | Signature |
|----------|-----------|
| `dfa_advance` | `fn(state: usize, input: u8, table: &[usize], alphabet_size: usize) -> usize` |
| `dfa_run` | `fn(table: &[usize], alphabet_size: usize, initial_state: usize, input: &[u8]) -> usize` |
| `dfa_is_accepting` | `fn(state: usize, accept_states: &[usize]) -> bool` |

## `dfa_advance`

Single transition. Computes the flat table index
`state * alphabet_size + input` (wrapping arithmetic), then **masks** it:

```rust
let mask = (index < table.len()) as usize;
let safe_idx = index & (0usize.wrapping_sub(mask));
table[safe_idx] & (0usize.wrapping_sub(mask))
```

Returns the next state. **Out-of-range behaviour is defined, not a panic:**
if `index >= table.len()`, the index is folded to `0` and the result is `0`
(state 0 acts as the implicit dead/trap state). No branch, no bounds panic.

**Contract.** `table` is a row-major transition matrix of shape
`num_states * alphabet_size`. `input` is used as a column in `0..alphabet_size`;
callers feeding bytes with `alphabet_size == 256` index directly by byte
value.

## `dfa_run`

Folds `dfa_advance` over `input` from `initial_state`, returning the final
state. Note the argument order differs from `dfa_advance`: `(table,
alphabet_size, initial_state, input)`. Runs in `O(len)` transitions with a
branchless body per byte.

## `dfa_is_accepting`

Returns `true` iff `state` appears in `accept_states`. Scans the **entire**
`accept_states` slice (OR-accumulate, no early exit), so timing is
independent of where — or whether — a match occurs.

## Integrity / oracle

The verification scaffold (reference + 3 counterfactual mutants) is in the
`#[cfg(test)]` module; there is no exported `*_phd_gate`. See `phd_gates.md`.

## Complexity

| Function | Time | Space |
|----------|------|-------|
| `dfa_advance` | `O(1)` | `O(1)` |
| `dfa_run` | `O(len)` | `O(1)` |
| `dfa_is_accepting` | `O(k)`, `k = accept_states.len()` | `O(1)` |

## Cross-references

- Table lookups as a branch-replacement, and the memory-channel caveat:
  `explanation/theory-6.md`.
- Why the accept scan does not short-circuit: `explanation/theory-6.md`.
- Byte parsing helpers: `reference/ref-9.md` (`parse`).
