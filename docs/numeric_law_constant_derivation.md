# Rule 14: Numeric-Law Requirements and Constant Derivation

In BCINR, the deterministic computational substrate demands strict mathematical and structural guarantees for any arithmetic operation. Rule 14 governs authoritative arithmetic to ensure bit-for-bit reproducibility and branchless stability. Central to these requirements is the "Missing Epsilon Law" and strict policies against arbitrary numeric constants.

## Why Implicit (Silent) Epsilons are Banned

Silently inserting a small `epsilon` value (e.g., adding `1e-9` to a denominator to prevent division-by-zero) is strictly prohibited. The reasoning includes:

1. **Bypasses Algorithmic Correctness**: Division-by-zero hazards must be resolved fundamentally in the mathematical algorithm rather than masked by small numerical hacks. For instance, the algorithm should be mapped to the log domain or handled via bitwise masking and fixed-width reciprocal approximations. Legacy tricks (like `.unwrap_or(0)`) are equally banned as they introduce conditional jumps or uncertified behavior.
2. **Violates the Error Envelope**: Every approximation in BCINR must have a formally declared numeric error envelope detailing its domain, codomain, maximum absolute error, maximum relative error, and boundary behavior. A silent epsilon acts as an uncertified approximation that secretly alters numeric bounds and error envelopes without mathematical proof.
3. **Breaches Deterministic Substrate Laws**: BCINR demands 100% physically predictable, interchangeable parts. Hot-path execution must be zero-allocation, branchless (`CC=1`), and rely entirely on pure fixed-point or log-domain integer operations. Arbitrary limits and unverified floats compromise the deterministic state machine.

## Why Every Constant Must Be "Named, Derived, Admitted, and Included in the Influence Digest"

Any smoothing factor, clamp constant, or scaling epsilon cannot be an arbitrary or hidden literal. A raw literal triggers a `CHEAT-003: Magic Constants` violation. Instead, constants must be explicitly managed through four rigorous steps:

1. **Named**: The constant must have an explicit, unambiguous identifier (e.g., `epsilon_drift`). This allows it to be referenced formally in Hoare contracts, structural proofs, and adversarial test expectations.
2. **Derived**: It cannot be an arbitrarily chosen "tuning number." It must have a concrete mathematical or structural derivation by the `@hoare_oracle` that formally justifies its exact value and bounds.
3. **Admitted**: The constant must be formally accepted into the authoritative runtime policy through strict structural gates. This means it has an independent reference oracle, is structurally branchless (audited by the `@turing_machine`), and is tested against hostile mutants by the `@armstrong_fault` (ensuring that corruption of the constant results in a typed `StabilityRefusal`).
4. **Included in the influence digest**: The exact value of the constant must be statically bound into the system's cryptographic influence digest (often tracked via the `numeric_profile_digest`). This ensures that if a constant is tweaked even slightly, the influence digest changes. A mismatch automatically triggers a refusal (e.g., `CMCA_CERTIFICATE_DIGEST_MISMATCH`), invalidating the Substrate Integrity Score (`SIS`), safely degrading the system to freeze learning, and preventing any hidden numerical drift in the hot path.

By enforcing these strict rules, BCINR ensures that all mathematical limits are rigorously audited, proven, and immune to hidden modifications, preserving the absolute determinism of the substrate.
