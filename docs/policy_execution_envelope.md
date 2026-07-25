Here is the documentation detailing how the runtime boundary verifies that an input execution envelope does not violate execution bounds branchlessly, based on our findings in the `bcinr` codebase (specifically `docs/envelope_violated_refusal.md` and `docs/rule_18_envelope_violated_refusal.md`).

# Runtime Boundary Verification of Execution Envelopes in BCINR

In the `bcinr` deterministic substrate, executing operations within formally certified boundaries is a strict requirement governed by the **Radon Law ($CC=1$)**. When a computation risks exceeding its defined numerical limits, structural capacities, or approximation error boundaries, it must be rejected with a typed `EnvelopeViolated` refusal (`StabilityRefusal::RuntimeEnvelopeViolated`).

Crucially, this boundary checking and refusal generation is designed to happen without panics, early returns, loops, or `if/else` statements. Instead, the runtime evaluates execution bounds branchlessly using a bit-parallel pipeline:

## 1. Mathematical Contracts and Numeric Error Envelopes

Every authoritative operation is bound by an explicit **numeric error envelope**. Because floating-point operations are strictly prohibited in the hot path, operations use Q16.16 fixed-point arithmetic (e.g., `NonNegativeFixed` and `SignedFixed`). The error envelope formally defines the admissible domain, codomain, maximum absolute error, maximum relative error, and saturation behaviors.

## 2. Structural Evaluation via Branchless Masking

Instead of using `if/else` conditionals to determine if an envelope is violated, the substrate evaluates conditions mathematically to yield a boolean bit-mask (`CanonicalMask`). A successful check produces a mask of all ones (e.g., `0xFFFFFFFFFFFFFFFF`), and a failure yields a mask of all zeros (`0x0`).

For instance, fixed-point operations (like `exp2`) check numeric bounds (overflow and underflow limits) using constant-time evaluators like `const_eq_u32` or `const_lt_u32`. Structural boundaries, such as checking if an allocation exceeds an arena capacity, are computed branchlessly like so:
```rust
let within_capacity = (next_offset <= self.capacity) as u64;
let no_overflow = (next_offset >= old_offset) as u64;
let success = within_capacity & no_overflow;

// Produces 0xFFFFFFFFFFFFFFFF on success, 0x0 on failure
let mask = 0u64.wrapping_sub(success); 
```

## 3. Branchless Fault Selection and Accumulation

Once the branchless mask is generated, the hot path must record the refusal without short-circuiting. The substrate achieves this through bitwise fault sets like `NumericFaultSet` or `RefusalSet`.

Using the generated mask, the algorithm selects the appropriate fault code (e.g., `APPROX_ENVELOPE` or `RANGE_VIOLATION`) and unions it with the ongoing state:
```rust
let e = CanonicalMask::select_faults(
    const_eq_u32(is_overflow.raw() | is_underflow.raw(), 0),
    NumericFaultSet::EMPTY,
    NumericFaultSet::RANGE_VIOLATION, // Triggers if envelope bounds exceeded
);

// Accumulate fault state branchlessly
self.faults = self.faults.union(e);
```
The `.union()` operation is a simple bitwise OR. It satisfies the invariant that fault accumulation is a join-semilattice—it never uses "first-error-wins" or "last-error-wins" branching.

## 4. State Masking (Gating Mutation)

In accordance with the "No mutation before complete admission" rule, state updates are applied via a fixed-width selection function. If the mathematical envelope is violated, the generated mask evaluates to `0`, ensuring the rejected operation leaves the persistent state bit-for-bit unchanged: `next_state = select(mask, candidate_state, current_state)`. 

## 5. Signaling at the Boundary

The operation completes its fixed $O(1)$ cycle time without unwinding and returns an opaque aggregation (like `AllocationOutcome`). The accumulated fault bits (e.g., `RANGE_VIOLATION` or `APPROX_ENVELOPE`) are unwrapped at the substrate boundary by mappers like `wrap_result`. These converters safely translate the bitwise mask into the typed `StabilityRefusal::RuntimeEnvelopeViolated` enum. This cleanly signals the MAPE-K Autonomic Loop to initiate recovery while preserving perfect zero-allocation, branchless determinism.
