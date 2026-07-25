# Missing Epsilon Law: Numeric Requirements in BCINR

In the BCINR deterministic substrate, the handling of numeric limits, numeric boundaries, and approximations is strictly governed by the **Numeric-Law Requirements**. Central to these requirements is the "Missing Epsilon Law," which dictates how epsilons, smoothing factors, and clamp constants must be treated.

## Why Silent Epsilons are Prohibited

Silently inserting a small `epsilon` value (e.g., adding `1e-9` to a denominator) to prevent division-by-zero panics or numerical instability is **strictly prohibited** for the following reasons:

1. **Bypasses Algorithmic Correctness**: Division-by-zero hazards must be resolved fundamentally in the mathematical algorithm rather than masked by small numerical hacks. For instance, rather than adding an epsilon, the algorithm should be mapped to the log domain (where division becomes a branchless subtraction from a pre-computed `log-sum-exp` denominator) or handled via bitwise masking and fixed-width reciprocal approximation bounds. Legacy tricks like `0/1` division truncation (`b | ((b == 0) as i32)`) or `.unwrap_or(0)` are equally banned as they introduce conditional jumps or uncertified behavior.
2. **Violates the Error Envelope**: Every approximation in BCINR must have a formally declared numeric error envelope detailing its domain, codomain, maximum absolute error, maximum relative error, and boundary behavior. A silent epsilon acts as an uncertified approximation that silently alters the numeric bounds and error envelopes without mathematical proof.
3. **Breaches Deterministic Substrate Laws**: BCINR demands 100% physically predictable, interchangeable parts. Hot-path execution must be mathematically sound, zero-allocation, branchless (`CC=1`), and rely entirely on pure integer log-domain or fixed-point SWAR (SIMD Within A Register) operations. Arbitrary floating-point limits and "magic numbers" have no place in a deterministically verified branchless state machine.

## Explicit Handling of Clamping Constants

Any smoothing factor, $\varepsilon_{\mathrm{sum}}$, or clamp constant used in the codebase cannot be an arbitrary or hidden literal. According to the architecture's mandate, it must be explicitly managed through four rigorous steps:

1. **Named**: The constant must have an explicit, unambiguous identifier (e.g., `epsilon_on`, `epsilon_gram`, `epsilon_drift`) rather than appearing as a raw literal (which triggers a `CHEAT-003: Magic constants` violation).
2. **Derived**: The constant must be mathematically derived by the `@hoare_oracle` and supported by a formal proof or derivation artifact that justifies its exact value and bounds.
3. **Admitted**: It must pass through rigorous verification gates, meaning it has an independent reference oracle, is structurally branchless, and is tested against hostile mutants that invert or corrupt the constant to ensure the system returns a typed `StabilityRefusal` instead of proceeding with invalid logic.
4. **Included in the Influence Digest**: The exact value of the constant must be statically bound into the system's influence digest (often tracked via the `numeric_profile_digest` within the generation manifest). This ensures that any mutation or drift in the constant alters the artifact digest. A digest mismatch automatically triggers a `CMCA_CERTIFICATE_DIGEST_MISMATCH` or `CMCA_RUNTIME_ENVELOPE_VIOLATED` refusal, safely degrading the system to `CertifiedSelectionOnly` and freezing learning.

By enforcing the Missing Epsilon Law, BCINR ensures that all mathematical limits are strictly audited, rigorously bounded, and immune to hidden numerical drift.
