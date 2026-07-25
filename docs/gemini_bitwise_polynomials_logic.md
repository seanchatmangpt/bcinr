# Bitwise Polynomials and Branchless Logic in BCINR

According to the **Radon Law** (`GEMINI.md`) and the **Mask-based execution law** (Rule 9 in `AGENTS.md`), the BCINR substrate mandates that logic must be expressed as **bitwise polynomials**. This means completely eliminating control-flow branches (`if`, `match`, data-dependent loops) in favor of constant-time, deterministic data-flow operations.

## The Principle

In traditional programming, decisions are made using control flow:

```rust
// Prohibited control flow
if valid {
    state = candidate;
} else {
    state = current;
}
```

This creates a data-dependent branch, which violates the strict $CC=1$ (Cyclomatic Complexity of 1) requirement. Branches introduce non-deterministic execution times, vulnerability to timing side-channels, and pipeline stalls. 

To adhere to BCINR's laws, sequential semantic decisions must be transformed into bitwise polynomials—a sequence of arithmetic and bitwise operations that compute a mask, followed by a deterministic selection between values.

## Mask Generation

A predicate must evaluate to a full-width mask: $m \in \{0, 2^w - 1\}$, where $w$ is the bit-width of the word (e.g., 32 or 64 bits). The mask must be either all 1s (`0xFF...FF`) for a "true" condition, or all 0s (`0x00...00`) for a "false" condition.

Instead of branching on a condition, you compute the mask using bitwise polynomials. For instance:

- **Sign Extraction / Less-than-zero**: An arithmetic right shift of a signed integer by $w-1$ broadcasts the sign bit to all positions. If a number is negative, the mask becomes all 1s. If non-negative, the mask is all 0s.
- **Equality**: While compilers can sometimes map equality checks to branchless flag-setting instructions, pure bitwise polynomial approaches use arithmetic properties (like bitwise XORing two values and smearing the bits) to generate a full-width mask without relying on hidden compiler branches.

## The Selection Operation

Once a mask is generated, the selection between a `candidate` value and the `current` value is performed using the fundamental selection polynomial defined in Rule 9:

$$ \operatorname{select}(m, a, b) = (m \land a) \lor (\neg m \land b) $$

In Rust, this looks like:

```rust
// Required mask-based selection
let next = (mask & candidate) | (!mask & current);
```

- If `mask` is all 1s (True), `mask & candidate` yields `candidate`, and `!mask & current` yields `0`. The bitwise OR combines them to return `candidate`.
- If `mask` is all 0s (False), `mask & candidate` yields `0`, and `!mask & current` yields `current`. The bitwise OR combines them to return `current`.

## Fieldwise Structured State Commit

For complex structured states, this mask-based selection must be applied fieldwise across fixed-width structures. State mutations are executed as an unconditional masked commit:

1. **Compute**: The candidate state is computed unconditionally (even if it won't be used).
2. **Verify**: All predicates are evaluated branchlessly to derive a unified admission mask.
3. **Commit**: A fieldwise masked selection updates the state.

```rust
// Constructing the mask from branchless verification
let mask = valid_mask(...);

// Unconditional masked selection
let next = State::select(mask, candidate, current);
```

By doing this, a rejected operation leaves the persistent state bit-for-bit unchanged. Whether the candidate is accepted or rejected, the execution time, instruction trace, and computational work remain mathematically identical.
