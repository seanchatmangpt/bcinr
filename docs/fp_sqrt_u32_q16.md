# Branchless Fixed-Point Square Root in `bcinr`

In the `bcinr` deterministic substrate, fixed-point square root operations such as `fp_sqrt_u32_q16` must strictly adhere to mathematical laws and absolute branchless constraints. These operations are required to compute roots accurately without floating-point math, architecture-specific instructions, or control-flow dependencies.

## Implementation Mechanics

The primary method used for strict exact integer calculation of square roots is **Digit-by-Digit Reduction**. The `fp_sqrt_u32_q16` function computes the root of a scaled `u32` value (converted to Q16 format by shifting left by 16 bits).

To prevent overflows during internal calculation, the operands are cast to `u128`. The algorithm iterates over exactly 41 unconditional steps. This bound is mathematically derived since the input domain is scaled to a maximum of $< 2^{80}$, and the highest possible even power of four is $4^{40} = 2^{80}$.

## Avoiding Conditional Branching (Rules 8, 9, & 13)

To ensure the $CC=1$ cyclomatic complexity mandate and eliminate branching, the implementation relies entirely on **Mask-Based Execution** rather than data-dependent control flow:

1. **Fixed Bounded Execution (Rule 13):** The internal logic relies on a fixed loop (`while k < 41`). There is no `break`, `continue`, or dynamic convergence condition. This enables the compiler or macro system to completely unroll the loop into straight-line object code without loop backedges.
2. **Canonical Mask Generation (Rule 9):** Instead of an `if (x >= candidate)` branch, the algorithm evaluates the boolean condition, casts it to an integer, and algebraically produces a canonical mask using `.wrapping_neg()`:
   ```rust
   let cond = x >= candidate;
   let m = (cond as u128).wrapping_neg(); // 0xFFFFFFFFFFFFFFFF if true, 0x0000000000000000 if false
   ```
3. **Masked State Evolution:** The mathematical state is updated unconditionally using bitwise operations against the generated mask. This ensures that the exact same operations and execution latency occur regardless of the input:
   ```rust
   x -= candidate & m;
   res = (res >> 1) + (bit & m);
   ```

## Satisfying Numeric Laws (Rule 14)

Under `AGENTS.md` Rule 14, every numerical function must satisfy rigorous authoritative criteria. `fp_sqrt_u32_q16` accomplishes this through:

- **Free of NaN and Infinity:** By computing strictly in fixed-width `u64` and `u128` integer arithmetic, floating-point numbers are completely excluded from the runtime. Concepts like `NaN` and `Infinity` are fundamentally impossible to represent in this domain, sidestepping hardware NaN-checking side channels.
- **Deterministic & Fixed-Width:** The function maps `u64` inputs directly to a deterministic `u64` output across all domains. There is no architecture-dependent rounding; the digit-by-digit approach ensures exact truncation bounds regardless of the target CPU.
- **Independent Oracle Proof:** The primitive is bounded by an independent, non-authoritative reference oracle (`fp_sqrt_u32_q16_reference`). The oracle computes the root using a standard Newton-Raphson approximation loop with conditionals and division. The hot-path branchless implementation must match this oracle's mathematical contract exactly bit-for-bit across all boundary scenarios.
- **Hostile Mutations (Master of Failure Law):** To prove test suite adequacy, the test suite provides syntactically plausible independent mutants (e.g., bitwise inversion bluffs, operator-swap bluffs). The test framework ensures that any structural divergence in the branchless code immediately fails the oracle equivalence checks, satisfying the mutation ledger requirement.
