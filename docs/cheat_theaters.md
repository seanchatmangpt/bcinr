# Anti-Cheat "Theater" Rules in BCINR

The BCINR deterministic substrate enforces strict anti-cheat rules to prevent the falsification or misrepresentation of verification evidence. Among these are the "Theater" rules, which specifically target practices that create the *illusion* of compliance without providing mathematical or mechanical proof.

## CHEAT-008: Benchmark Theater
**Definition:** Benchmarking a stub, constant-folded path, dead result, or reduced problem not equivalent to production.
**Meaning:** This violation occurs when performance metrics are artificially inflated by benchmarking code that doesn't do the actual work. For example, if a compiler optimizes away a benchmark because its result is unused (dead result), or if the problem space is artificially reduced. The benchmark must test the actual, unelided hot path with equivalent complexity and scale to a real production workload.

## CHEAT-009: Mutant Theater
**Definition:** Creating mutants that cannot compile, are trivially different, or are detected only by `assert_ne!`.
**Meaning:** Hostile mutation testing requires injecting plausible faults into the implementation to ensure the test suite can catch them. "Mutant theater" happens when the mutations are invalid (they don't even compile, meaning the compiler catches them instead of the tests), completely trivial, or verified lazily (e.g., merely checking that the output changed with `assert_ne!` rather than asserting a specific mathematically sound typed refusal or explicit oracle mismatch). A valid mutant must alter a meaningful law (like a dropped factor or incorrect mask) and be caught by the independent oracle or typed refusal boundary.

## CHEAT-010: Gate-Jurisdiction Theater
**Definition:** Reporting a passing scanner that does not inspect the relevant crate, file, generated output, feature set, or target.
**Meaning:** This occurs when a verification gate (like the cheat scanner or object-code auditor) technically returns a "pass" or green status, but it didn't actually scan the code in question. A green build is meaningless if the jurisdiction of the scan was incomplete. To avoid this, one must prove that the verification tools successfully analyzed the exact files, generated rust code, specific feature flags, and target architectures affected by a change.

## Why Fabricating Verification Evidence is an Absolute Failure

In the BCINR constitution, fabricating verification evidence (which includes any form of "theater") is classified as an **absolute failure**. According to Section 24 (Substrate Integrity Score) and the broader constitutional principles:

1. **SIS Drops to 0:** Any absolute failure immediately forces the Substrate Integrity Score to 0 and triggers `MaturityScrutiny`.
2. **Cannot Be Masked:** No weighted average or passing score in other areas can conceal a constitutional violation. 
3. **Breach of the Deterministic Contract:** BCINR operates on the principle that "Agent agreement is not evidence." The substrate's integrity relies entirely on reproducible, mechanical artifacts and independent structural proofs. Faking this evidence compromises the fundamental contract of the system, invalidating the guarantees of branchless, bounded, and deterministic execution.
