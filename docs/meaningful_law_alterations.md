# Meaningful Law Alterations in Hostile Mutants

**Jurisdiction**: `@armstrong_fault` (Master of Failure Law)  
**Reference**: `AGENTS.md` — Rule 19 (Hostile mutation protocol)

In the BCINR Deterministic Substrate, the structural integrity of the authoritative runtime is verified through hostile mutation. Under the jurisdiction of `@armstrong_fault`, every authoritative implementation must be tested against at least three independent, syntactically plausible mutants. 

A mutant is not considered valid if it merely introduces a trivial syntax error, fails to compile, or is detected by a weak `assert_ne!(baseline, mutant)`. Instead, it must represent a **meaningful law alteration**—a corruption of the mathematical contract, execution bounds, or strict deterministic mechanics of the function.

## Plausible Mutants: Specific Faults

The following faults constitute plausible, meaningful law alterations that adversarial tests must successfully kill via **typed refusals** or exact oracle postcondition mismatches:

### 1. Sign Inversion
Reversing the polarity of an arithmetic operation (e.g., swapping `+` and `-`, or negating a coefficient). In a branchless substrate, this tests whether the mathematical laws of monotonicity, energy conservation, or valid output ranges are strictly enforced and properly caught by the independent oracle.

### 2. Dropped Factor
Omitting a crucial multiplier, scale factor, or weighting coefficient in a fixed-point calculation. This ensures the arithmetic logic properly bounds its numeric error envelope and that omitting a step in the SWAR (SIMD Within A Register) pipeline violates a specific mathematical postcondition.

### 3. Incorrect Mask
Applying the wrong bitmask during branchless state selection (`select(m, a, b)`). Since the substrate prohibits data-dependent branches, state transitions rely entirely on masks. An incorrect mask mutant verifies that leaking state or failing to properly isolate admitted candidates triggers a failure.

### 4. Normalization Omission
Failing to normalize a vector, value, or intermediate state before using it in bounded arithmetic. This tests the strict bounds-checking of the valid input domain and codomain.

### 5. Index Skew
Introducing an off-by-one error, stride misalignment, or corrupted index calculation. In fixed-width, bounded execution work, an index skew must predictably violate structural boundaries or lookup table contracts.

### 6. Stale Digest Acceptance
Failing to reject an outdated, invalid, or mismatched cryptographic/state digest. This directly attacks the **ReceiptSound law** (Rule 11) and ensures that the autonomic loop strictly requires an `AcceptedCertificate` and `AcceptedEnvelopeReceipt` before processing.

### 7. State Mutation Before Admission
Speculatively modifying persistent state before all verification predicates are checked and the final admission mask is derived. This verifies adherence to Rule 10 ("No mutation before complete admission"), ensuring rejected operations leave the state bit-for-bit unchanged.

### 8. Truncation of a Bounded Table
Prematurely truncating or improperly bounding a fixed-size lookup table used for branchless evaluation. This tests the **Full-domain requirement**, ensuring the implementation properly covers the entire $2^{64}$ domain without falling back to runtime loops or branches.

### 9. Bypassed Refusal
Removing or short-circuiting a mandatory failure path that should yield a typed refusal. This ensures that invalid states cannot silently proceed, testing the exhaustive requirement for strict typed refusals (e.g., `Err(StabilityRefusal::ContractionMarginInsufficient)`).

### 10. Incorrect Clamp
Applying the wrong bounds, maximums, or minimums in a clamp operation. Constant-time clamp boundaries are critical to numeric-law requirements, and altering them must trip the declared error envelope.

### 11. Unsupported Fallback
Silently falling back to a simpler algorithm or dropping to a non-authoritative processing mode when unsupported input is detected. The substrate forbids returning plausible defaults or silently clamping outside admitted policies; this mutant verifies that unsupported inputs generate strict typed refusals instead.

## The Typed-Refusal Requirement

A test suite that relies on `assert_ne!` to detect these meaningful alterations is defective (Rule 4, CHEAT-009). The mutation protocol strictly requires that when a mutant executes, it must either:
1. Trigger a specific **typed refusal** (e.g., `DigestMismatch`, `ControlStateUnadmitted`).
2. Be mathematically identified by the independent `@hoare_oracle` as violating an exact postcondition.
