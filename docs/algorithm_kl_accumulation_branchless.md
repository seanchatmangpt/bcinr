# Branchless Enforcement of KL Accumulation (Rule 14)

In the BCINR deterministic substrate, Rule 14 mandates that all authoritative arithmetic must be fixed-width, branchless ($CC=1$), fully bounded, and free of implicit panics. KL accumulation acts as an extreme stress test for these numerical laws, and its enforcement relies on several multi-layered, branchless techniques:

### 1. Loop Unrolling Over Bounded Domains
Rule 13 forbids unbounded loops (`while`, `for`), which would introduce cyclical branches. To accumulate the divergence score across dimensions or graph nodes, the engine uses **macro-based static loop unrolling** (e.g., `unroll_8_static!`, `unroll_4_static!`) over bounded domains. It unconditionally computes the complex math for *every* element rather than selectively traversing nodes.

### 2. Branchless Selection via Bitmasking
Any conditional logic (e.g., checking if a node should be included in the sum or if it is a child/leaf) is strictly forbidden. Instead, it relies on full-width bitmasks.
* A condition is evaluated to a `0` or `1`.
* It expands into a full-width mask: `let mask = 0u32.wrapping_sub(cond_val);`
* Bitwise selection (via primitives like `const_select_u32`) includes or zeroes-out terms unconditionally: `(term & mask) | (0 & !mask)`. The sum safely ignores zeroed elements using `wrapping_add`.

### 3. Log-Sum-Exp Trick for Overflow Prevention
KL divergence requires exponentiation and logarithmic probabilities, which can easily overflow or underflow 32-bit fixed-point (Q16.16) boundaries. Standard bounds-checking `if` statements are banned. The mathematical law is enforced via the **Log-Sum-Exp** stabilization technique:
* The engine branchlessly computes the maximum log-probability in the domain (`x_max_meas`).
* It subtracts this maximum from all elements before exponentiation (`x.wrapping_sub(x_max_meas)`).
* This mathematical invariant guarantees that all inputs to the `exp2()` function are $\le 0$, mapping them cleanly into the $[0, 1]$ interval to prevent fixed-point overflow.

### 4. Avoiding Multiplication Overflow via `i64` Widening
The core KL accumulation multiplies fixed-point values (a probability by a log-ratio). Doing this directly in a 32-bit space would overflow. Since conditional checks are banned, the calculation widens the types securely:
* 32-bit operands are widened to `i64`.
* A 64-bit `wrapping_mul` executes to prevent implicitly injected AST panic branches.
* The product is bit-shifted right (`>> 16`) to retain the fixed-point fractional scale before safely downcasting back to 32-bit.

### 5. Branchless Non-Negative Clipping (Invariant Enforcement)
Mathematically, KL divergence must be strictly non-negative ($\ge 0$). Fixed-point approximation errors, however, can cause small negative values. A standard `if (kl < 0) kl = 0;` violates the Cyclomatic Complexity constraint. Instead, the substrate enforces this mathematical invariant by deriving a full-width bitmask directly from the sign bit of the fixed-point value, conditionally clamping the final accumulation to zero using purely bitwise logic.
