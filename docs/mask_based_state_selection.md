# Rule 9: Mask-Based Execution Law

In the BCINR Deterministic Substrate, **Rule 9** dictates how runtime conditional logic must be transformed into arithmetic operations to achieve a Cyclomatic Complexity of 1 (CC=1) and entirely branchless algorithmics. 

Traditional conditional logic (like `if / else`) relies on CPU Jump (JCC) instructions, which introduce data-dependent control flow, branch prediction overhead, and timing side-channels. Under Rule 9, these branches are strictly prohibited. 

Instead, runtime predicates must be evaluated into **full-width bitmasks**:
- **True:** All bits are `1` (e.g., `0xFFFFFFFF` for 32-bit width).
- **False:** All bits are `0` (e.g., `0x00000000`).

Selection then takes a pure mathematical bitwise form:
`select(m, a, b) = (m & a) | (~m & b)`

Because both branches of logic are unconditionally evaluated as straight-line arithmetic, the CPU's instruction shape remains fixed and deterministic, regardless of the semantic input.

---

# Selecting Structured State (Fieldwise)

When updating a `struct` or other compound data type ("structured state"), you cannot conditionally assign or copy the entire structure at once if it might compile down to a branch or variable-time memory copy. Instead, the constitution mandates that selection must be:

1. **Fieldwise:** The bitwise mask selection is applied independently to *every individual primitive field* within the struct.
2. **Fixed-width:** All operations are performed on primitives with a known, constant bit-width (e.g., `u32` or `u64`).

This guarantees that the bitwise polynomial is uniformly applied across the exact same number of bits every time, executing in a fixed number of CPU cycles.

---

# How `State::select` Works

The BCINR constitution outlaws the branching shape:
```rust
// PROHIBITED
if valid {
    candidate
} else {
    current
}
```

Instead, it dictates the **"Required shape"** for structured state transitions:
```rust
// REQUIRED
let mask = valid_mask(...);
let next = State::select(mask, candidate, current);
```

While `State::select` operates at the struct level, under the hood, it applies the mathematical selection formula to each field atomically. Here is exactly how it works:

1. **The Inputs:** It takes the full-width `mask`, the `candidate` struct, and the `current` (fallback) struct.
2. **Fieldwise Arithmetic:** For every primitive field in the struct, it applies `(mask & candidate.field) | (!mask & current.field)`.
3. **The Result:** 
    - **When valid (mask is all 1s):** `(!mask)` becomes all 0s. The formula simplifies to `candidate.field | 0`, effectively selecting the `candidate` state.
    - **When invalid (mask is all 0s):** `(mask & candidate.field)` becomes 0. The formula simplifies to `0 | current.field`, preserving the `current` state.

By pushing the conditional logic down to fieldwise bitwise operators, `State::select` evaluates both the "accepted" and "rejected" state transitions in parallel, leaving the persistent state bit-for-bit unchanged on rejection without ever branching the CPU's control flow.
