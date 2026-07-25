# Rule 14: Numeric-law requirements in BCINR

In the BCINR deterministic substrate, **Rule 14 ("Numeric-law requirements")** mandates that all authoritative arithmetic must be fixed-width, deterministic, free of floating-point numbers or NaNs, branchless, and fully bounded by a declared mathematical error envelope. 

Within this strict environment, **Eigenvalue lower bounds** and **KL accumulation** are singled out for special scrutiny due to the immense difficulty of enforcing fixed-point correctness without control flow.

## 1. Eigenvalue Lower Bounds

Eigenvalue bounds are critical for system stability, as they dictate the Gram Distinguishability lower bound ($\underline\gamma_{\min}^{+}$) in the CMCA-RDF Observatory. 
* **Numerical Confidence Limits**: When estimating the smallest positive eigenvalue of the Gram matrix, both statistical and numerical error ($\varepsilon_\Gamma$) must be strictly separated and conservatively subtracted. 
* **Degenerate Scaling Prevention**: Learner activation explicitly requires this lower bound to be strictly greater than a threshold ($\epsilon_{\mathrm{gram}}$). Incorrect approximations could cause false numerical confidence, leading to degenerate scaling or false proofs of system contractivity, which would compromise the substrate's stability.
* **Strict Monotonicity Requirements**: Without exact bounds on numeric errors, approximations could lack strict monotonicity, causing the system’s gradient logic to react oppositely to extreme conditions and bounce unpredictably.

## 2. KL (Kullback-Leibler) Accumulation

KL accumulation acts as an extreme stress test for fixed-width, branchless execution because it relies heavily on non-linear primitives and probabilities:
* **Vulnerability to Overflow/Underflow**: In 32-bit Q16.16 fixed-point math, small raw probabilities can easily underflow to zero, and intermediate exponentiations can overflow. Standard bounds-checking (`if kl > max`) is banned, forcing the engine to proactively stabilize numbers mathematically.
* **Restricted Primitives**: KL requires division, logarithms, and exponentials. Division must be converted to log-domain subtraction. Exponentials require the Log-Sum-Exp technique with branchless maximum-finding over bounded arrays to ensure inputs are $\le 0$.
* **Type Widening Hazards**: Multiplying fixed-point probabilities by log-ratios would immediately overflow 32-bit registers. The math engine must branchlessly cast up to 64-bit, perform `wrapping_mul`, bit-shift right to retain the fractional scale, and safely downcast.
* **Non-Negativity Invariant (Clamping without Branches)**: Mathematically, KL divergence is always $\ge 0$. Fixed-point approximation errors can result in small negative accumulations. Since `if (kl < 0) kl = 0;` violates $CC=1$, the non-negativity invariant must be enforced via complex branchless bitmasking (deriving a full-width mask from the sign bit and using bitwise selection).
* **Accumulation Without Control Flow**: The engine must use static macro loop unrolling over bounded domains and branchless boolean matrices to create bitmasks that unconditionally zero out non-included graph nodes during accumulation.
