Here is the research on the `UnsupportedDomain` typed refusal from the BCINR deterministic substrate, as mandated by Rule 18 in `AGENTS.md`.

# The `UnsupportedDomain` Typed Refusal

In the BCINR framework, **Rule 18** dictates that all rejected authoritative operations must produce bounded, strictly typed refusal codes. `UnsupportedDomain` is one of these mandatory typed refusals. Human-readable text and dynamic error messages are expressly forbidden in the hot path.

## What is it?
`UnsupportedDomain` is emitted when a mathematical function or authoritative operation receives an input that falls outside its rigorously defined and proven mathematical bounds (its "admitted domain"). Examples include calculating the logarithm of zero or executing a saturating division by zero. 

When an input breaches this admissible domain, the runtime must explicitly reject the unadmitted input via this typed refusal. Rule 18 explicitly forbids the following behaviors on unsupported inputs:
* Panicking or unwinding
* Silently clamping outside an admitted policy
* Dropping a factor
* Falling back to a simpler algorithm
* Mutating partial state
* Returning a plausible default

## Role in the Runtime

The `UnsupportedDomain` refusal plays a crucial role in maintaining BCINR's "hard substrate" guarantees, particularly its axiomatic determinism (Rule 1) and state isolation (Rule 10). Furthermore, due to the **Radon Law** (Absolute branchless execution, CC=1), the runtime handles `UnsupportedDomain` conditions entirely without traditional control flow (e.g., no `if input <= 0 { return Err(UnsupportedDomain); }`). 

Its branchless propagation mechanisms are structured as follows:

1. **Branchless Condition Masking:** Out-of-bounds conditions are evaluated into full-width bitmasks (`CanonicalMask`) via SWAR (SIMD Within A Register) and bitwise polynomials, avoiding boolean control flow.
2. **Hardware Trap Avoidance:** The hot path executes the full arithmetic calculation to avoid hardware panics. If the domain is mathematically unsupported, the implementation branchlessly selects a safe fallback value to feed the mathematical pipeline, and then clamps the final calculated result.
3. **Sticky Fault Accumulation:** Rather than halting execution or returning an early `Result`, the typed refusal is accumulated via a `NumericFaultSet`. The out-of-bounds mask branchlessly sets an `INVALID_DOMAIN` fault bit. This fault state acts as a join-semilattice, tracking seamlessly across chained computations without any "first-error-wins" short-circuiting.
4. **Sealed Return Types:** The final operation returns a fixed-width struct (like `SignedFixed`) that pairs the safely-computed scalar data with the accumulated error state.
5. **State Selection & Translation:** At the absolute boundary of the hot path (e.g., during state admission in `allocator.rs`), the accumulated numeric fault set is evaluated. If the `INVALID_DOMAIN` bit is present, the authoritative root translates it to the strict `StabilityRefusal::UnsupportedDomain` code, permanently terminating the transaction and rejecting the operation. The persistent state is left bit-for-bit unchanged via masked commits.
