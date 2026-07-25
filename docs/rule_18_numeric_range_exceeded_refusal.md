Here is the detailed research on the `NumericRangeExceeded` typed refusal, based on my analysis of the `AGENTS.md` constitution and the project's internal documentation (specifically `docs/numeric_range_exceeded_refusal.md`).

# The `NumericRangeExceeded` Typed Refusal

In the BCINR deterministic computational substrate, `NumericRangeExceeded` is a **bounded typed refusal code**. It is part of the `StabilityRefusal` enum mandated by Rule 18 of `AGENTS.md`. 

Rule 18 strictly forbids human-readable text, panic paths, silent clamping, and fallback mechanisms in the hot path. Instead, any rejected operations—such as numerical values exceeding defined bounds—must output a bounded typed refusal. `NumericRangeExceeded` fulfills this role for arithmetic overflow and domain violations.

## Role in the Runtime

Because the BCINR runtime adheres to a strict $CC=1$ (Cyclomatic Complexity of 1) mandate across all authoritative functions, standard Rust overflow handling (e.g., `if overflow`, `unwrap`, panics, or early returns) is illegal. The `NumericRangeExceeded` typed refusal plays a vital role by allowing the runtime to gracefully log and handle mathematical bounds violations while preserving a purely **branchless, straight-line instruction stream**.

Mechanically, it operates via the following branchless sequence:

### 1. Canonical Masking
When a Q16.16 fixed-point calculation (such as addition or division) exceeds its absolute maximum bounds, the overflow condition is computed as a boolean but immediately cast into a `CanonicalMask` (evaluating strictly to `0xFFFFFFFF` for true, or `0x00000000` for false) using wrapping arithmetic.

### 2. Mathematical Saturation
Instead of branching, the `CanonicalMask` is used to mathematically select a saturated value (like `i32::MAX` or `i32::MIN`) through bitwise `AND` and `OR` operations, guaranteeing bounded output without conditionals.

### 3. Branchless Error Mapping
The same `CanonicalMask` used for saturation is reused to map the overflow condition directly to the `StabilityRefusal::NumericRangeExceeded` refusal code.
```rust
let e = overflow_mask.select_u32(StabilityRefusal::NumericRangeExceeded as u32, u32::MAX);
```

### 4. Sticky Error Accumulation
To ensure the error propagates up without an early `return Err(...)` (which introduces branching), BCINR relies on a **Sticky Error Accumulator**. The Q16.16 structures couple their computational bits with an internal error state. Using a `branchless_err_acc` utility, the new `NumericRangeExceeded` state is bitwise-unioned with any pre-existing errors in the chain.

The structure deterministically returns both the saturated safe value and the accumulated error state. The refusal naturally surfaces at the end of the operation chain, enabling later validation components to explicitly handle the `NumericRangeExceeded` refusal without ever breaking the rigid $CC=1$ execution graph.
