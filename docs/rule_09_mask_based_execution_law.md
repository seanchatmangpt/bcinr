# Rule 9: Mask-Based Execution Law

In the BCINR Deterministic Substrate, **Rule 9 (Mask-based execution law)** enforces that runtime predicates must never use traditional control flow (like `if`/`else`). Instead, sequential semantic decisions must be transformed into full-width masks, and state transitions must be executed using bitwise arithmetic.

## Mathematical Equivalence of `select(m, a, b)`

A runtime predicate must be resolved into a mask $m$ where all bits are either 0 or 1 across the full width of the word (i.e., $m \in \{0, 2^w-1\}$). 

The `select` operation is mathematically defined as:
$$ \operatorname{select}(m, a, b) = (m \land a) \lor (\neg m \land b) $$

**How it works structurally:**
- When the condition is **true**, the mask $m$ is all `1`s (e.g., `0xFFFFFFFF`). 
  - $m \land a$ preserves the bits of $a$. 
  - $\neg m$ is all `0`s, so $\neg m \land b$ evaluates to `0`. 
  - The result is $a \lor 0 = a$.
- When the condition is **false**, the mask $m$ is all `0`s. 
  - $m \land a$ evaluates to `0`. 
  - $\neg m$ is all `1`s, so $\neg m \land b$ preserves the bits of $b$. 
  - The result is $0 \lor b = b$.

This equivalence allows the CPU to evaluate both potential outcomes and bitwise-merge them, achieving conditional logic in absolute constant time with $CC=1$ (zero control-flow branches).

## Fieldwise Fixed-Width Selection for Structured State

When updating complex data types ("structured state"), the rule dictates two constraints:
1. **Fieldwise:** You cannot use branching to swap structs or bypass the update if a condition is false. Instead, you must explicitly push the mask down to evaluate `select(m, candidate_field, current_field)` on every single constituent field of the struct.
2. **Fixed-width:** Every property in the struct must have a strictly bounded, constant compile-time size (e.g., `u32`, `i64`). Because selection relies on exact bitwise operations, dynamically sized types (like heap allocations) cannot be used.

By applying the same bit-parallel mask independently to every fixed-width field, the whole structured state transitions cleanly without requiring jumps, allocations, or data-dependent runtime paths.

## Implementation Shape

**Prohibited (Uses Branching):**
```rust
if valid {
    candidate
} else {
    current
}
```

**Required Shape (Mask-Based Selection):**
```rust
let mask = valid_mask(...);
let next = State::select(mask, candidate, current);
```

*(Note: The mask implementation itself must also pass strict object-code inspection to ensure the compiler hasn't sneakily emitted conditional jumps to evaluate it.)*
