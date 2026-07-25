# The `EnvelopeViolated` Typed Refusal in BCINR

In the `bcinr` deterministic substrate, executing operations within formally certified boundaries is a critical requirement governed by the **Radon Law ($CC=1$)**. When a computation exceeds its defined bounds—such as maximum absolute/relative error limits, structural capacities, or numerical limits—the system must trigger a `StabilityRefusal::EnvelopeViolated` (or `RuntimeEnvelopeViolated`) refusal.

Crucially, this bounding check and refusal generation must occur without any conditional branching, early returns, or loops. The hot path structurally evaluates these envelopes and triggers refusals through a branchless, bit-parallel pipeline.

## 1. Mathematical Contracts and Numeric Error Envelopes

Every authoritative operation (owned by `@hoare_oracle`) defines an explicit **numeric error envelope**. Because floating-point operations are strictly prohibited in the hot path, operations use Q16.16 fixed-point arithmetic (e.g., `NonNegativeFixed` and `SignedFixed`). 

The error envelope formally declares the admissible domain, codomain, maximum absolute error, maximum relative error, and saturation behaviors (e.g., the piecewise linear $\log_2$ approximation has a max absolute error bound of $\approx 0.08607$).

## 2. Structural Evaluation via Branchless Masking

Instead of using `if / else` statements to check if an envelope is violated, the substrate evaluates conditions mathematically to produce a `CanonicalMask` (which evaluates to either `0x00000000` or `0xFFFFFFFF`).

For example, in the **Atomic Concurrent-Safe Bump Arena (ACSBA)**, checking if an allocation exceeds the arena capacity is computed branchlessly:
```rust
let within_capacity = (next_offset <= self.capacity) as u64;
let no_overflow = (next_offset >= old_offset) as u64;
let success = within_capacity & no_overflow;

// Produces 0xFFFFFFFFFFFFFFFF on success, 0x0 on failure
let mask = 0u64.wrapping_sub(success); 
```
For fixed-point operations (like `exp2`), numeric bounds (overflow and underflow limits) are checked using constant-time evaluators like `const_eq_u32` or `const_lt_u32`, yielding a similar boolean mask.

## 3. Branchless Fault Selection and Accumulation

Once the mask is generated, the hot path must record the refusal without short-circuiting. The substrate achieves this through bitwise fault sets like `NumericFaultSet` or `RefusalSet`.

Using the generated mask, the algorithm selects the appropriate fault code (such as `APPROX_ENVELOPE` or `RANGE_VIOLATION`) and unions it with the ongoing state:
```rust
let e = CanonicalMask::select_faults(
    const_eq_u32(is_overflow.raw() | is_underflow.raw(), 0),
    NumericFaultSet::EMPTY,
    NumericFaultSet::RANGE_VIOLATION, // Triggers if envelope bounds exceeded
);

// Accumulate fault state branchlessly
self.faults = self.faults.union(e);
```
The `.union()` operation is a simple bitwise OR. This satisfies the invariant that fault accumulation is a join-semilattice—it never uses "first-error-wins" or "last-error-wins" branching.

## 4. State Masking and Transmitting the Refusal

Under the **"No mutation before complete admission"** law, exceeding the certified boundary must completely block state updates. The same `mask` that generated the fault is used to select the result:
```rust
let res = is_overflow.select_u32(u32::MAX, is_underflow.select_u32(0, val_shifted));
```
If the envelope was violated, the result is clamped to a saturated value (like `0` or `u32::MAX`), and the actual persistent state update is bypassed via `select_nnf`.

Finally, the operation completes its fixed $O(1)$ cycle time and returns an opaque aggregation (like `AllocationOutcome`). The accumulated fault bits (e.g., `RANGE_VIOLATION` or `APPROX_ENVELOPE`) are unwrapped at the substrate boundary by mappers like `wrap_result`, converting the bitwise mask into a typed `StabilityRefusal::EnvelopeViolated` or `RuntimeEnvelopeViolated` enum. This cleanly signals the MAPE-K Autonomic Loop to initiate recovery, all while maintaining perfect zero-allocation, branchless determinism.
