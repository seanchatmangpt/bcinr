# Zero-Allocation Rejection Architecture in BCINR

According to **AGENTS.md Rule 10 (No mutation before complete admission)**, persistent state must never be mutated speculatively. If a transaction is rejected or invalid, the persistent state must be left byte-for-byte unchanged. In the BCINR deterministic computational substrate, this is achieved without early returns, control-flow branching, or heap allocations.

The architecture guarantees this through a strict transactional pipeline:
`current immutable state → fixed-size candidate state → verify all predicates → derive admission mask → fieldwise masked commit`

## 1. Buffering in Zero-Allocation Scratch Space
Since heap allocation is strictly forbidden (Rule 3), "cloning the state" to prepare a speculative update cannot involve dynamic memory (`Box`, `Vec`, or `malloc`). Instead, the **candidate state** is computed structurally into:
- **Stack-Allocated Structures:** Fixed-size local arrays or struct variables bounded at compile-time.
- **BumpArena:** A branchless bump allocator that manages a contiguous, pre-allocated fixed memory span. It acts as an $O(1)$ deterministic scratch pad for complex, variable-sized intermediate calculations without ever touching the heap.

This strictly isolates the speculative computation from the persistent state.

## 2. Deriving the Admission Mask (No Early Returns)
Predicates, structural invariants, and rules are evaluated without `if`, `else`, or `return Err(...)`. All checks execute sequentially and their outcomes are mathematically accumulated into a single **admission mask** (or a gate variable, like `has_refusal`). 

This enforces the Radon Law ($CC=1$) by transforming discrete logical rejections into bit-parallel polynomials that always take the exact same execution time regardless of validity.

## 3. Fieldwise Masked Commit
The actual mutation of the persistent state only happens at the very end of the cycle via a masked `select` operation:
$$x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t)$$

Using bitwise selection (e.g., `(mask & candidate) | (!mask & current)`), the candidate state is conditionally committed. 
- If **admitted** (mask = `0xFFFFFFFF`), the candidate bits overwrite the persistent bits.
- If **rejected** (mask = `0x00000000`), the candidate bits are mathematically dropped, and the persistent state is rewritten with its exact previous value.

This ensures the persistent state remains bit-for-bit unchanged on rejection without branching.

## 4. Branchless $O(1)$ Memory Reclamation
Rejected speculative state lingering in the `BumpArena` is reclaimed in a single constant-time arithmetic operation:
`self.arena.offset &= !trigger_mask;`

When triggered, this bitwise operation zero-clears the arena's cursor instantly, reclaiming the contiguous block of scratch space for the next cycle. No garbage collection or heap cleanup is required, maintaining perfect execution predictability and avoiding memory leaks.
