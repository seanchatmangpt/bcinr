Here is the explanation for why all smoothing constants, clamp constants, and normalization variables must be included in the influence digest under Rule 14 (Numeric-law requirements), based on my research in the `bcinr` documentation:

### Why Constants Must Be Included in the Influence Digest

Under **Rule 14**, the deterministic substrate prohibits any "silent epsilons" or unexplained "magic constants" (which also violates Rule 16 CHEAT-003). All smoothing factors, clamp limits, and normalization variables fundamentally impact decision boundaries, geometric scaling properties, and semantic mass.

Therefore, they must be rigorously **named, derived, admitted, and included in the influence digest** for the following reasons:

1. **Cryptographic Tracking & Provenance:** The exact values of these constants are statically bound into the system's cryptographic influence digest (often tracked via `numeric_profile_digest` in the generation manifest). This guarantees that every algorithmic tuning parameter is explicitly accounted for in state changes.
2. **Preventing Silent Drift:** By including constants in the digest, any undocumented mutation, numerical drift, or arbitrary tweaking of a value will immediately alter the artifact digest. It prevents "silent alterations" to the system's operational envelope.
3. **Automatic Enforcement (Typed Refusal):** If a constant is altered, the resulting digest mismatch automatically triggers a strict refusal (such as `CMCA_CERTIFICATE_DIGEST_MISMATCH` or `CMCA_RUNTIME_ENVELOPE_VIOLATED`). 
4. **Safe Degradation:** This refusal ensures the system safely degrades by invalidating the Substrate Integrity Score (SIS) and freezing adaptive state transitions (e.g., falling back to `CertifiedSelectionOnly` mode), thereby preventing hidden numerical corruption in the hot path.
