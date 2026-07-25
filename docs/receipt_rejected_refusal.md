# The `ReceiptRejected` Typed Refusal in `bcinr`

In the deterministic `bcinr` substrate, execution in the hot path is governed by the **Radon Law ($CC=1$)**, meaning no input-dependent loops, conditional jumps, or early returns are permitted. To evaluate the internal validity of a structural receipt—and mathematically trigger a `ReceiptRejected` refusal (or an equivalent `RefusalSet` flag) if it is invalid—the system employs **O(1) branchless arithmetic and bitwise masking**.

Here is how the system mathematically verifies constraints (like cryptographic bounds or temporal windows) and constructs the refusal branchlessly.

## 1. Branchless Evaluation of Internal Validity

Whether verifying a cryptographic digest match, structural bounds, or elapsed temporal windows (dwell times), the hot path transforms all boolean validation checks into pure mathematical projections yielding a `1` (valid/invalid) or `0`.

### Digest and Cryptographic Bound Verification
Instead of using an `if actual == expected` branch, the system evaluates equality using bitwise XOR. For multiple bounds or structural elements, the differences are bitwise OR'd together. 
If all elements match, the entire expression evaluates to `0`. 

```rust
let digests_match = (((state.digest ^ cert.digest)
    | (state.digest ^ env.digest)
    | (state.digest ^ outcome.digest))
    == 0) as u32;
```
*(For byte arrays, this is done via a fixed unrolled macro like `unroll_32_static!`, XORing each byte and accumulating the results into a single condition flag without early exits).*

### Temporal Windows and Numeric Thresholds
To evaluate bounded windows (e.g., verifying if $elapsed \ge required$), the system avoids control flow by using bit-shifts and two's-complement arithmetic to extract the sign bit.

```rust
// Branchless Less-Than check: Returns 1 if a < b, else 0.
#[inline(always)]
pub fn const_lt_u32(a: u32, b: u32) -> u32 {
    let a_bb = core::hint::black_box(a);
    let b_bb = core::hint::black_box(b);
    ((a_bb ^ ((a_bb ^ b_bb) | (a_bb.wrapping_sub(b_bb) ^ b_bb))) >> 31) & 1
}

// Example: Checking a temporal window dwell time
let dwell_err = const_lt_u32(elapsed_tau, required_tau);
```

## 2. Mathematically Triggering the Typed Refusal

Once constraints have been reduced to a boolean bit (`1` indicating an error or `0` indicating valid), the system mathematically constructs the Typed Refusal without any `if` statements.

### The Masking Operation
The system applies a two's-complement masking function to either emit the refusal bits or drop them entirely. 

```rust
#[inline(always)]
pub const fn masked(self, condition: u32) -> Self {
    // 0u32.wrapping_sub(1) yields 0xFFFFFFFF (all 1s)
    // 0u32.wrapping_sub(0) yields 0x00000000 (all 0s)
    Self(self.0 & 0u32.wrapping_sub(condition & 1))
}
```
* If `condition == 1` (error), `0 - 1 = -1` (or `0xFFFFFFFF` in unsigned binary). `self.0 & 0xFFFFFFFF` returns the refusal bits.
* If `condition == 0` (no error), `0 - 0 = 0`. `self.0 & 0x00000000` returns `0` (an empty refusal).

### Aggregation (Union)
The resulting masked refusal is combined with other system refusals via a bitwise `union` (`OR`).

```rust
let mut refusals = RefusalSet::EMPTY;

// If receipt is rejected, the bits are added. If valid, 0 is added (no-op).
let receipt_invalid_condition = /* branchless computation result */;
refusals = refusals.union(RefusalSet::RECEIPT_REJECTED.masked(receipt_invalid_condition));
```

### Result Mapping via Array Indexing
If a component needs to return a single legacy `enum` (like `StabilityRefusal`) or a sealed constructor struct (like `Option<AdaptiveUpdate>`), it uses the $1/0$ condition to index an array of outcomes directly:

```rust
let outcomes = [
    Err(StabilityRefusal::ReceiptRejected),
    Ok(pi_res)
];
// (is_ok as usize) & 1 ensures the index is strictly 0 or 1
return outcomes[(is_ok as usize) & 1];
```

## Summary
The `bcinr` authoritative engine never "checks and returns". It evaluates **all** receipt constraints bitwise, maps failures to `1` or `0`, dynamically generates an all-1s or all-0s bitmask, and mathematically merges the `ReceiptRejected` flag into a global `RefusalSet`. The state mutations that follow then use branchless selectors (`select_nnf`) to either commit the transaction or silently preserve the exact previous state based on whether the `RefusalSet` remains completely empty.
