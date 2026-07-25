Based on my review of `AGENTS.md` in the `bcinr` repository, here is the breakdown of what the "generated clean tree" and "test profile where relevant" targets entail for the project's gates under Rules 22 and 23:

### 1. Generated clean tree
Governed by Rule 21 ("Generated-code law"), this target ensures that all generated source code is strictly reproducible and untampered. Running gates on a "generated clean tree" entails:
* Wiping and strictly regenerating all authoritative code from scratch.
* Hashing and verifying that the freshly synthesized output is byte-for-byte identical to the committed code.
* Ensuring there is no "unexplained drift" or manual tampering (hand-editing generated code is explicitly prohibited).
* The subsequent structural gates (e.g., `CC=1`, cheat scanner, object-code audit) are then run on this freshly verified tree to ensure generated branches haven't evaded structural laws.

### 2. Test profile where relevant
Because `bcinr` enforces strict deterministic constraints via Rust object-code audits, the gates must be evaluated not just in `release` but also in test environments. This entails:
* **Isolation of Test/Slow Rail Code:** Rule 6 allows "test references" and "test-only oracles" to branch and allocate. The gates must prove that when `#[cfg(test)]` or test features are active, these allocations and panic paths do not accidentally link into or execute within the "authoritative hot path."
* **Adversarial Hostile Mutation:** Governed by Rule 19, the hostile mutation protocol must run the test suite and verify expected "typed refusals." The test profile is required here to confirm that corrupted implementations produce explicit, typed domain rejections (e.g., `Err(...)`) rather than unhandled panics.
* **Structural Guarantees in Testing:** Verifying that the authoritative logic still maintains zero data-dependent branches, zero heap allocations, and deterministic execution when built under the test compiler profile.
