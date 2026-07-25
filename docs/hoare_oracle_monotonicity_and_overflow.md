# `@hoare_oracle` Proof Obligations: Monotonicity and Overflow

In the BCINR framework, the `@hoare_oracle` (Rule 4) acts as the Axiomatic proof lead and specification owner. Every primitive must be defined by a formal Hoare contract $ \{P(x)\} \quad f(x) \quad \{Q(x,f(x))\} $. Among the strict list of contract requirements, the **monotonicity law (where applicable)** and **overflow behavior** hold unique importance for achieving BCINR's foundational goal: deterministic, branchless, zero-allocation execution.

Here is why these two specific mathematical properties are mission-critical for verifying fixed-width execution and bounds checking within this substrate.

## 1. The Monotonicity Law

The monotonicity law requires proving that a function preserves a given order (e.g., if $a \le b$, then $f(a) \le f(b)$). In a branchless, deterministic substrate, monotonicity is not just a nice-to-have mathematical trait; it is a structural necessity for safety.

* **Branchless Bounds Checking:** Without the ability to use control flow (`if`/`match`/`panic`) to verify bounds dynamically, the system relies on mathematical constraints. If a function is proven to be monotonic, validating the minimum and maximum boundaries of the input domain statically guarantees the boundaries of the codomain. The runtime does not need to search or check intermediate values for spikes that might exceed fixed-width limits.
* **Deterministic Convergence and Stability:** For operations like accumulation, normalization, or fixed-point approximations (Rule 14), monotonicity ensures predictability. It guarantees that an increase in input strictly corresponds to a bounded, predictable change in output. This prevents adversarial manipulation where a maliciously crafted input causes a non-linear arithmetic anomaly to bypass a threshold.
* **Eliminating Algorithmic Search:** Rule 12 prohibits "runtime theorem discovery." Monotonicity allows the runtime to verify static domination (e.g., $\widehat G \le G_{\mathrm{certified}}$) simply by comparing packed values, knowing the underlying arithmetic will not unexpectedly invert or violate constraints.

## 2. Overflow Behavior

In a fixed-width environment (Rule 14) running without standard Rust safety nets (Rule 8: no bounds-check panic paths, no checked arithmetic with branch-bearing handling), overflow management must be absolute, deterministic, and proven.

* **No Panic Paths (`CC=1`):** A standard arithmetic overflow in Rust normally triggers a panic in debug mode or wraps silently in release mode. Because BCINR strictly prohibits panic paths and unwinding, overflow cannot be left to default compiler behavior or dynamic `Result`-based handling. It must be explicitly defined as either mathematically *saturating* (e.g., weights, distances, physical limits) or *wrapping* (e.g., hashing, cryptographic operations).
* **Adversarial Resiliency:** The `@armstrong_fault` role (Master of Failure Law) relies on the oracle's contracts to generate hostile mutants. If overflow behavior is not formally specified as part of the postcondition $Q(x,f(x))$, it is impossible to write typed refusals or verification tests for boundaries. The Hoare contract provides the exact arithmetic baseline that proves a saturating addition or fixed-width clamp executes flawlessly at $2^{64}-1$.
* **Mask-Based Execution Intersections:** When state transitions are managed via branchless masks (Rule 9: $m \land a \lor \neg m \land b$), overflow behavior feeds directly into mask generation. If an input exceeds a mathematical domain, the overflow contract determines whether the mask saturates the value to a bound or triggers a `StabilityRefusal`. Explicit overflow specification guarantees that this selection remains a fixed bitwise operation rather than an undefined semantic state.

## Conclusion

In a system governed by the Radon Law (Cyclomatic Complexity = 1) and fixed bounded memory, you cannot *react* to out-of-bounds or unexpected numeric behaviors at runtime. You must *preordain* them algebraically. 

The `@hoare_oracle` mandates the **monotonicity law** and **overflow behavior** because they translate semantic safety into structural guarantees. They allow the compiler, bit-vector solvers, and the `bcinr-cheat-scanner` to mathematically prove that a fixed-width, branchless instruction sequence will never exceed its memory bounds, branch, or panic, regardless of the inputs provided.
