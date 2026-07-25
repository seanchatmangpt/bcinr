# No Runtime Algorithm Search in BCINR

The `bcinr` deterministic substrate constitution strictly prohibits **runtime algorithm search** as part of its absolute runtime laws (Rule 3). This principle is central to maintaining the branchless, bounded, and deterministic nature of the authoritative hot path.

## Why is Runtime Algorithm Search Prohibited?

1. **Fixed Instruction Shape:** The core mandate of `bcinr` requires a mathematical guarantee: `admitted input -> fixed instruction shape -> deterministic output`. Dynamically falling back to a simpler algorithm based on input complexity inherently creates varying execution paths, violating the rule that the instruction shape must not depend on semantic input.
2. **The Radon Law ($CC=1$):** A runtime algorithm search fundamentally requires conditional evaluation (e.g., `if complexity > threshold, use algorithm B`) or variable loop boundaries (e.g., iterating until a threshold is met). This violates the absolute prohibition on data-dependent branches and data-dependent loop termination.
3. **Bounded Execution Work:** The substrate guarantees "fixed bounded execution work" and "fixed bounded memory access." Searching for a viable algorithmic path or adaptively discovering bounds destroys the guarantee of exact, constant-time execution bounds and exposes timing side-channels.
4. **Prevention of Silent Fallbacks (CHEAT-022):** The constitution explicitly forbids circumventing constraints by silently degrading to a simpler, branching, or floating-point algorithm when an edge case is encountered. Glossing over failures is banned; the runtime must strictly enforce its mathematical envelope.

## How Algorithm Selection is Handled Without Branching

Because the hot path cannot branch or conditionally switch algorithms, `bcinr` employs a combination of off-path derivation, mask-based execution, and strict refusal policies:

### 1. Slow Rail Derivation vs. Hot Path Verification
Runtime theorem discovery is explicitly banned (Section 12). Instead of discovering the right algorithm, optimal thresholds, or stability parameters on the fly, this work is offloaded to the non-authoritative **"slow rail"**:
- The slow rail is permitted to allocate, branch, and perform dynamic algorithm searches (e.g., eigenvalue search, optimization over weighting vectors) to derive a certificate or mathematical witness.
- The authoritative hot path merely **verifies** this supplied witness using constant-time arithmetic ("The hot path compares packed values only").

### 2. Mask-Based Execution (Bit-Parallel Selection)
When an operation fundamentally involves distinct logical pathways, sequential semantic decisions must be transformed into mathematical masks (Section 9):
- The runtime evaluates all valid state transitions in parallel using generated, straight-line code.
- Full-width masks ($m \in \{0, 2^w-1\}$) are derived from the input predicates.
- The correct result is applied using branchless selection: $\operatorname{select}(m, a, b) = (m \land a) \lor (\neg m \land b)$.

### 3. Equivalent Structural Fallbacks
When a fallback is strictly necessary—such as for hardware-specific instructions (e.g., SIMD vs. portable scalar)—the fallback implementation must satisfy the **exact same structural laws** (Section 22). It cannot be a simpler branching equivalent; it must still be bounded, branchless, and possess $CC=1$.

### 4. Typed Refusals
If an input exceeds the admitted complexity, unsupported domain, or fails mathematical validation, the runtime does not search for a degraded fallback algorithm. Instead, it securely terminates the transition by returning a **Typed Refusal** (e.g., `UnsupportedDomain`, `NumericRangeExceeded`, `ContractionMarginInsufficient`). The persistent state is left bit-for-bit unchanged via masked commits, strictly enforcing the state isolation boundary (Section 10).
