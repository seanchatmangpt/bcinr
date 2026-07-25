# Branchless Handling of `UnsupportedDomain` Typed Refusals in BCINR

In `bcinr`, the deterministic substrate mandates strict compliance with the **Radon Law** ($CC=1$), requiring that hot-path execution contains zero data-dependent branches, early returns (`?`), or panics. When a mathematical function receives input outside its admitted domain—such as calculating the logarithm of zero, or a saturating division by zero—the runtime must reject the mutation and issue an `UnsupportedDomain` typed refusal. 

Rather than branching with an `if input <= 0 { return Err(UnsupportedDomain); }`, `bcinr` accomplishes this entirely through bitwise polynomials and SWAR (SIMD Within A Register) masking.

Here is the step-by-step branchless mechanism, utilizing `NonNegativeFixed::log2()` (from `crates/bcinr-cmca/src/fixed.rs`) as the primary example:

## 1. Branchless Condition Masking
Instead of evaluating boundary conditions into traditional boolean types used for control flow, `bcinr` transforms semantic decisions into a `CanonicalMask` (a strict `0xFFFFFFFF` for true, or `0x00000000` for false).
```rust
let is_zero = const_eq_u32(self.val, 0);
```

## 2. Safe Hardware Execution (Hardware Trap Avoidance)
To maintain constant-time execution without triggering architecture-dependent hardware panics, the hot path must execute the full arithmetic calculation regardless of the input's validity. If the domain is mathematically unsupported, the implementation branchlessly selects a safe fallback value (or substitute operands, like a `1` denominator) to feed the mathematical pipeline. It then clamps the final calculated result.
```rust
// A safe clamped value (-1048576) is branchlessly selected if is_zero is true
let computed = (res as u32).wrapping_sub(16 << 16) as i32;
let safe_val = is_zero.select_i32(-1048576, computed);
```

## 3. "Sticky" Fault Accumulation
Instead of halting execution or returning a standard Rust `Result`, the typed refusal is accumulated via the `NumericFaultSet`. The mask generated in Step 1 is used to branchlessly select the `INVALID_DOMAIN` fault bit. Because fault accumulation is a join-semilattice under bitwise union, the fault state tracks seamlessly across a chained computation without any "first-error-wins" short-circuiting.
```rust
let e = CanonicalMask::select_faults(
    is_zero,
    NumericFaultSet::DIVIDE_BY_ZERO.union(NumericFaultSet::INVALID_DOMAIN),
    NumericFaultSet::EMPTY,
);
```

## 4. Sealed Return Type Structs
The final operation completes by returning a fixed-width struct (`SignedFixed` or `NonNegativeFixed`) which permanently pairs the safely-computed scalar data value with the accumulated error state.
```rust
SignedFixed {
    val: safe_val,
    faults: self.faults.union(e), // Merge upstream faults with the new domain fault
}
```

## 5. Hot Path Boundary Unpacking
The mathematical operation resolves successfully from the Rust compiler's perspective without generating jump instructions. It is only at the absolute boundary of the hot path (in `allocator.rs` during state admission) that the overarching `AllocationOutcome` is evaluated. If the accumulated numeric fault set contains the `INVALID_DOMAIN` bit, the authoritative root translates this to the legacy `StabilityRefusal::UnsupportedDomain` code, rejects the operation, and leaves the persistent state bit-for-bit unchanged as required by the `AGENTS.md` admission laws.
