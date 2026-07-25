# Mask-Based State Commit Protocol in BCINR

In accordance with the "No mutation before complete admission" law and the deterministic, branchless `CC=1` mandate, the physical execution of `select(m_admitted, x_candidate, x_t)` for persistent structures ensures safety through fieldwise masked commits and Bitset Calculus without speculative memory mutation or heap allocation.

## 1. Candidate State Pre-computation
Persistent state is never mutated speculatively. Instead of cloning via heap allocation, the implementation copies the current state `x_t` into a fixed-size stack value or scratch structure, or simply computes the candidate state structurally. The entire candidate transition `x_candidate` is formed off-target.

## 2. Deriving the Admission Mask (`m_admitted`)
All admission predicates are verified unconditionally. Any invalidation or error conditions are accumulated and evaluated into a boolean predicate (e.g., `is_enabled` or `is_valid`). This boolean is reduced mathematically to a full-width mask (such as via underflow `0u64.wrapping_sub(is_enabled as u64)`) such that:
- `true` maps to `0xFFFFFFFFFFFFFFFF` (all 1s)
- `false` maps to `0x0000000000000000` (all 0s)

## 3. Fieldwise Masked Commit (`select`)
The physical commit applies the branchless bitwise polynomial selection across the fields of the persistent structure:
`select(m, a, b) = (m & a) | (!m & b)`

Instead of using `if valid { state = candidate; }`, the engine unconditionally iterates over the fields or words of the structure (e.g., statically unrolled over an array of `u64` words) and assigns:
```rust
// SWAR selection: (candidate & mask) | (current & !mask)
next.words[i] = (candidate.words[i] & mask) | (current.words[i] & !mask);
```

## 4. Unconditional Memory Overwrite
The result of the SWAR selection unconditionally overwrites the persistent state. 
- If `m_admitted` is `0xFFFFFFFFFFFFFFFF`, the state updates to `x_candidate`.
- If `m_admitted` is `0x0000000000000000`, the state is bit-for-bit identical to `x_t`.

This approach ensures execution safely updates persistent memory in constant time, entirely independent of the runtime data semantics, achieving structural bounds on memory access and execution work.
