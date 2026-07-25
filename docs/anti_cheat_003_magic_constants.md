# CHEAT-003: Magic Constants in `bcinr`

In the `bcinr` deterministic substrate, **CHEAT-003** refers to the strict prohibition of "magic constants"—any unexplained literal controlling production behavior (e.g., `0xDEADBEEF`, `0xCAFEBABE`, or arbitrary numeric values like `1e-9`).

## Why Unexplained Literals are Strictly Prohibited

1. **Lack of Mathematical Provenance:** The `bcinr` architecture is a civilizational-scale deterministic substrate where all logic must be expressed as mathematically proven, bitwise polynomials. Magic numbers bypass the required formal proofs and bounding analyses.
2. **Breach of "Contract with Teeth":** Every primitive requires a Hoare contract, an independent oracle, and strict proof obligations (managed by the `@hoare_oracle`). Unexplained literals obscure structural logic and cannot be formally verified.
3. **Violates the Error Envelope:** Every approximation in the hot path must have a formally declared numeric error envelope (domain, codomain, maximum absolute/relative error). Arbitrary limits or "silent epsilons" act as uncertified approximations that alter these numeric bounds without mathematical proof.
4. **Scanner Evasion is Impossible:** The `bcinr-cheat-scanner` parses the full Abstract Syntax Tree (AST), normalizes whitespace, strips numeric separators (e.g., `0xDEAD_BEEF`), and detects equivalent hex spellings natively. Superficial formatting changes will not bypass the requirement.

## How Constants Must Be Handled Instead (Rule 14)

Any constant, such as a smoothing factor or clamp boundary, must be strictly managed through four rigorous steps:

1. **Named:** The constant must be assigned an explicit, unambiguous identifier (e.g., `epsilon_on`, `epsilon_drift`) rather than appearing as a raw literal in the code.
2. **Derived:** The exact value and its bounds must be mathematically derived by the `@hoare_oracle` and supported by a formal proof or derivation artifact justifying its existence.
3. **Admitted:** The constant must pass rigorous verification gates. This includes having an independent reference oracle, being structurally branchless, and surviving adversarial testing against hostile mutants (designed by `@armstrong_fault`) that corrupt the constant to ensure the system safely returns a typed `StabilityRefusal`.
4. **Included in the Influence Digest:** The value must be statically bound into the system's influence digest (often tracked via `numeric_profile_digest` in the generation manifest). Any undocumented mutation or numerical drift will alter the digest, immediately triggering a `CMCA_CERTIFICATE_DIGEST_MISMATCH` or `CMCA_RUNTIME_ENVELOPE_VIOLATED` refusal and freezing adaptive state transitions.
