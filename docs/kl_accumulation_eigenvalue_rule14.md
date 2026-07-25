# Rule 14: Structural Mathematics in BCINR

In the BCINR deterministic substrate, **Rule 14 (Numeric-law requirements)** mandates that all authoritative arithmetic must be fixed-width, deterministic, free of floating-point numbers or NaNs, branchless ($CC=1$), and fully bounded by a declared mathematical error envelope. 

Within this extreme environment, **KL accumulation** and **Eigenvalue lower bounds** are singled out for special scrutiny because enforcing mathematical invariants without control flow (e.g., bounds checking or dynamic loops) is incredibly challenging.

Here is how BCINR enforces these operations structurally:

## 1. KL (Kullback-Leibler) Accumulation

KL accumulation acts as an extreme stress test for fixed-width, branchless execution because it relies heavily on non-linear primitives (division, logarithms, exponentials) and probabilities that can easily overflow or underflow. The hot path calculates it branchlessly using the following layered techniques:

* **Loop Unrolling Over Bounded Domains**: Unbounded loops (`while`, `for`) introduce cyclical branches and are forbidden. The engine uses macro-based static loop unrolling (e.g., `unroll_8_static!`) over statically bounded domains, unconditionally computing the math for *every* element rather than selectively traversing nodes.
* **Branchless Selection via Bitmasking**: Standard conditional logic (e.g., checking if a node should be included) is strictly banned. The engine evaluates conditions to `0` or `1`, expands them into full-width masks (e.g., `let mask = 0u32.wrapping_sub(cond_val);`), and uses bitwise selection (`(term & mask) | (0 & !mask)`) to safely include or zero-out terms.
* **Log-Sum-Exp Trick for Overflow Prevention**: Standard bounds-checking `if` statements are banned. To prevent exponentiation from overflowing 32-bit Q16.16 fixed-point boundaries, the engine branchlessly finds the maximum log-probability in the domain and subtracts it from all elements before exponentiation. This mathematical invariant guarantees all inputs to the `exp2()` function are $\le 0$, mapping them securely into the $[0, 1]$ interval.
* **i64 Widening to Prevent Multiplication Overflow**: Core KL accumulation multiplies a fixed-point probability by a log-ratio, which would overflow a 32-bit space. Operands are cast up to 64-bit, multiplied via `wrapping_mul`, bit-shifted right (`>> 16`) to retain the fractional scale, and safely downcasted without AST panic branches.
* **Branchless Non-Negative Clipping**: Mathematically, KL divergence must be $\ge 0$, but fixed-point approximations can cause small negative accumulations. A standard `if (kl < 0) kl = 0;` violates $CC=1$. Instead, the substrate derives a full-width bitmask directly from the sign bit of the fixed-point value to conditionally clamp the final accumulation to zero using bitwise logic.

## 2. Eigenvalue Lower Bounds (Gram Distinguishability)

Eigenvalue lower bounds dictate the Gram Distinguishability lower bound ($\underline\gamma_{\min}^{+}$), which is critical for system stability. However, calculating eigenvalues is an iterative, branching process (e.g., spectral-radius estimation, power iteration) that fundamentally violates the substrate's constitutional laws (Rule 12 forbids "runtime theorem discovery").

Instead of calculating them directly on the hot path, BCINR uses an **Observe-Verify** paradigm:

* **Slow Rail Theorem Discovery**: The iterative mathematical heavy lifting is relegated to the out-of-band "Slow Rail". This asynchronous environment (allowed to branch and allocate) calculates the spectral radius ($\rho(G_a) < 1$) and the conservative lower bound on the smallest positive eigenvalue of the Gram matrix. It strictly separates statistical and numerical errors to prevent degenerate scaling.
* **The AcceptedCertificate (Witness)**: The Slow Rail packages its findings into a static, cryptographically digested witness containing fixed-point bounds like the certified gain matrix ($G_{\mathrm{certified}}$), a scaling vector ($d$), and the contraction margin ($\delta$).
* **Hot Path Branchless Verification**: The Authoritative Runtime does **not** search for eigenvalues. It simply takes the witness and *verifies* it branchlessly using fixed-point matrix-vector multiplication (e.g., $\widehat G \leq G_{\mathrm{certified}}$ and $G_{\mathrm{certified}} d \leq (1-\delta)d$).
* **Safe Homeostasis**: If the $O(1)$ arithmetic confirms these bounds and the state digest matches, the admission mask evaluates to full width ($1$). If the inequalities fail, the mask evaluates to $0$. This mathematically freezes learning (`LearningFrozen`) and leaves the state unchanged, enforcing stability without a single `if` statement.
