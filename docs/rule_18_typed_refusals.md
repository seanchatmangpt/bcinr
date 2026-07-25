# Research: Rule 18 (Typed Refusals) in BCINR

According to **Rule 18 (Typed refusals)** of the BCINR `AGENTS.md` Constitution, all rejected authoritative operations must produce a bounded typed refusal code rather than using human-readable strings, panics, or partial processing. 

## Why Human-Readable Text is Banned from the Hot Path

The "hot path" (authoritative runtime) in BCINR must strictly adhere to the laws of being a deterministic, bounded, allocation-free execution substrate. Human-readable text is banned because:
1. **Zero Allocation:** Text formatting and passing string messages typically require dynamic heap allocations (e.g., `String`), which violates the absolute runtime law of "zero heap allocation".
2. **Fixed Instruction Shape & Memory:** The substrate guarantees fixed-width outputs and bounded memory access. Variable-length text strings violate these fixed-width mathematical contracts.
3. **Branchlessness ($CC=1$):** Creating, concatenating, or formatting strings introduces hidden branching and overhead. Using bounded, fixed-width enums or type codes ensures the failure path remains strictly branchless and deterministic.

## Required Categories for Typed Refusals

When an operation is rejected, it must return a specific error from bounded, required categories. Some of these include:
- `ContractViolation`
- `UnsupportedDomain`
- `NumericRangeExceeded`
- `DigestMismatch`
- `CertificateMissing`
- `CertificateStale`
- `EnvelopeViolated`
- `ContractionMarginInsufficient`
- `CheatDetected`

## Strict Handling of Unsupported Input

Rule 18 mandates that unsupported inputs must never trigger certain behaviors, because they violate fundamental constitutional laws of the BCINR architecture:

- **Must Never Panic:** 
  Rule 3 states there must be "no panic paths" and "no unwinding". Panicking disrupts deterministic execution, breaks the requirement of $CC=1$ (cyclomatic complexity of 1) by introducing implicit control-flow branches, and abruptly terminates bounded operations. 

- **Must Never Silently Clamp Outside Policy:** 
  Silently accepting out-of-bounds input or clamping it in a way not defined by an explicit mathematical contract destroys the project's axiomatic precision. An implementation must behave exactly as its strict mathematical proof dictates. If an input breaches the admissible domain, it must explicitly result in a typed refusal (like `UnsupportedDomain`), rather than returning a plausible but incorrect default.

- **Must Never Mutate Partial State:** 
  This violates **Rule 10 (No mutation before complete admission)**. The authoritative code uses purely transactional execution ("masked commit"). A rejected operation must leave the persistent state bit-for-bit unchanged. Speculatively modifying a field before the entire input is validated and admitted could leave the system in an invalid or corrupted state.
