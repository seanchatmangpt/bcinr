# CHEAT-022: Silent Fallback Avoidance

In the `bcinr` deterministic substrate constitution, **Silent Fallback Avoidance** (`CHEAT-022`) explicitly prohibits the practice of circumventing primary branchless fixed-point constraints by silently degrading or falling back to a simpler, branching, or floating-point algorithm when an edge case or unsupported condition is encountered. 

Instead of hiding the failure to maintain operations, the authoritative runtime must strictly return a **Typed Refusal** while leaving the underlying state bit-for-bit unchanged.

## The Mandate of Section 18: Typed Refusals

Under the core laws governing `bcinr` (as defined in `AGENTS.md`), the runtime operates as a deterministic bounded state machine. When an operation exceeds its mathematical or operational limits, the system must not attempt to dynamically "fix" or gloss over the problem. 

The constitution states unequivocally that **no unsupported input may:**
* Panic;
* Silently clamp outside the admitted policy;
* Drop a factor;
* **Fall back to a simpler algorithm;**
* Mutate partial state;
* Return a plausible default.

Instead, the implementation must produce a bounded typed refusal code (e.g., `UnsupportedDomain`, `NumericRangeExceeded`, `ContractionMarginInsufficient`, or `BranchlessContractFailed`). Human-readable text is banned from the hot path; only the strict bounded typed refusal is permitted.

## The Prohibition of Structural Degradation

A common anti-pattern is for developers to maintain uptime by introducing a conditional branch that defaults to an easier approach (like floating point math or simplified branching) when the optimized fixed-point branchless math risks overflow or fails a constraint check.

```rust
// ❌ PROHIBITED: CHEAT-022 (Silent Fallback)
// Violates CC=1, introduces branches, and degrades structural integrity.
let result = if branchless_path_is_safe(input) {
    optimized_branchless_compute(input)
} else {
    // Falling back to a simpler/branching algorithm is illegal
    simpler_branching_fallback(input) 
};
```

This immediately violates the **Radon Law ($CC=1$)** and the absolute runtime mandate of `no data-dependent branches` and `no floating-point operations`. Any sequential semantic decisions must be transformed into full-width masks and bitwise arithmetic selection. 

```rust
// ✅ ADMITTED: Mask-based execution and Typed Refusal
// The transition is fixed-width, masked, and structurally deterministic.
let valid_mask = derive_admission_mask(input);
let candidate = optimized_branchless_compute(input);

// State is committed only if valid; otherwise, it remains strictly unchanged.
// The rejection is signaled upstream via a Typed Refusal instead of a fallback workaround.
let next = State::select(valid_mask, candidate, current_state);
```

## Lawful Fallbacks vs. Unbounded Silent Fallbacks

The constitution does acknowledge the concept of fallbacks, but they are strictly mathematically gated and must adhere to the same laws as the primary path:

1. **Hardware Fallbacks (Section 22):** If an architecture-specific instruction (e.g., PDEP/PEXT) is absent, the fallback implementation **must satisfy the exact same structural laws**. It cannot be a simpler branching equivalent; it must still be bounded, branchless, and $CC=1$. If a lawful fallback target cannot be constructed, it must yield a Typed Refusal.
2. **Learning/Adaptive Fallbacks (Section 11):** When the environment transitions and learning is frozen, the system does not fall back to branching defaults. "The frozen fallback must be implemented by masked state selection, not branching."
3. **Approximations (Section 14):** Floating-point fallbacks are universally banned. Every numeric approximation requires a strict domain, maximum absolute error, and bounding envelope. "No epsilon may be inserted silently."

## Strict State Isolation (Section 10)

A silent fallback often attempts to speculatively mutate partial state in order to limp along. Under the `bcinr` contract, persistent state must never be mutated speculatively. If the fixed-point constraints fail and a `Typed Refusal` is triggered, the operation must leave the persistent state **bit-for-bit unchanged**.

By enforcing `CHEAT-022`, `@von_neumann_bypass` (Architect of Arithmetic Logic) and `@turing_machine` (Enforcer of Determinism) jointly ensure that the authoritative instruction shape strictly adheres to the fundamental mathematical contract, preserving the project's Substrate Integrity Score (SIS) without compromise.
