# Auto-Select Tool Evaluation Cyclomatic Complexity (CC=1) Proof

The `evaluate_candidate` and overarching `select` function in `/Users/sac/mfw/mfw-auto-select/src/lib.rs` strictly adhere to the $CC=1$ rule (Radon Law) mandated by the BCINR deterministic substrate constitution. The implementation is entirely branchless, zero-allocation, and operates with a fixed instruction shape. 

Below is the step-by-step structural cyclomatic complexity proof showing exactly why there are no branches in the execution.

## 1. Branchless Mask Generation
Sequential semantic decisions are transformed into full-width bitmasks rather than control-flow jumps.
* **Admissibility Check:** `let is_admissible = (admissible & bit) == bit;` computes a boolean flag.
* **Mask Derivation:** `let admissible_mask = 0u32.wrapping_sub(is_admissible as u32);` safely maps the boolean (0 or 1) into a full-width 32-bit mask (`0x00000000` or `0xFFFFFFFF`) via two's complement arithmetic (underflow). This inherently bypasses any need for an `if` block.

## 2. Unconditional Execution (SWAR)
The candidate is scored mathematically (`mass.wrapping_mul(mass)`) and evaluated irrespective of whether it is ultimately admissible.
* Using `select_u32(admissible_mask, score, 0)`, the function replaces conditional control-flow logic with a bitwise polynomial equivalent to `(mask & a) | (!mask & b)`. This executes in fixed time without jump instructions.

## 3. Branchless State Mutation and Comparisons
Comparisons do not branch or yield an early-return.
* **Relational Mask:** `lt_mask_u32(current_best_score, active_score)` generates a selection mask based on magnitude (e.g. relying on sign-bit extraction from subtraction) without inducing a hardware branch.
* **Commit State:** The state updates (`next_best_score` and `next_best_id`) utilize `select_u32` with the `update_mask` to fieldwise commit the candidate's metrics if it is strictly superior, or to retain the existing metrics otherwise. This fully satisfies the "Mask-based execution law".

## 4. Loop Backedge Elimination
The `select` function avoids unbounded execution, iterators, and data-dependent loop terminations:
* Instead of using a `for` or `while` loop over the slice of candidates—which inherently introduces conditional branches—it utilizes **compile-time static unrolling**.
* It explicitly cascades `evaluate_candidate` over `input.candidates[0]` through `input.candidates[7]` in straight-line code.
* No `break`, `continue`, or variable iteration bounds exist. The resulting object code will compile into a direct linear pipeline free of runtime loop backedges.

## Conclusion
The full authoritative call graph (`select` -> `evaluate_candidate` -> `select_u32` & `lt_mask_u32`) contains zero instances of `if`, `match`, `while`, early returns, or `Result/Option`-based error handling control flow. The cyclomatic complexity is statically proven to be exactly 1.
