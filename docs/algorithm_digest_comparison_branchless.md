# Branchless Digest and Equality Comparison in BCINR

In accordance with **Rule 14 (Numeric-law requirements)** and **Rule 8 (Absolute CC=1 law)** of the BCINR Deterministic Substrate Constitution, digest comparison and equality checks are implemented without control-flow branches, short-circuiting, or data-dependent loops. 

The codebase implements these mathematical properties using fixed-width bitwise polynomials and macro-unrolled iteration.

## 1. Branchless Integer Equality (`const_eq_u32`)

The foundational primitive for bounded equality checking is `const_eq_u32`, located in `crates/bcinr-cmca/src/allocator.rs`. Instead of using an `if` statement or `match` block, it uses bitwise XOR and two's complement mechanics to resolve equality into a fixed `1` (true) or `0` (false).

```rust
#[inline(always)]
pub fn const_eq_u32(a: u32, b: u32) -> u32 {
    let x = core::hint::black_box(a) ^ core::hint::black_box(b);
    let nonzero = (x | x.wrapping_neg()) >> 31;
    1u32.wrapping_sub(nonzero)
}
```

### Mathematical Enforcement:
*   **Deterministic Evaluation:** The XOR operation (`^`) yields `0` only if `a` and `b` are identical. 
*   **Sign-Bit Extraction:** If `x` is non-zero, `x | x.wrapping_neg()` forces the most significant bit (MSB) to `1`. Shifting right by 31 bits (`>> 31`) isolates this MSB, yielding `1` for any non-zero difference and `0` for no difference.
*   **Mathematical Contract:** The final `1u32.wrapping_sub(...)` strictly ensures the codomain is bounded to `{0, 1}`, satisfying the exact postcondition required for mask-based execution (Rule 9).

## 2. 256-bit Digest Comparison (`digest_eq_mask` and Macro Unrolling)

Comparing 32-byte structures (e.g., cryptographic receipts, certificates) is typically prone to short-circuiting loops. BCINR enforces bounded execution (Rule 13) and branchlessness via two main techniques.

### A. Fixed-Width Unrolled Accumulation
In `crates/bcinr-cmca/src/allocator.rs`, comparing the `digest` to a `CERTIFICATE_DIGEST` is enforced sequentially but purely bitwise, utilizing `unroll_32_static!`:

```rust
let mut digest_match = 1u32;
unroll_32_static!(i, {
    digest_match &= const_eq_u32(
        digest[i & 31] as u32,
        crate::generated::stability_profile::CERTIFICATE_DIGEST[i & 31] as u32,
    );
});
let digest_err = const_eq_u32(digest_match, 0) != 0;
```
*   **No Unbounded Execution (Rule 13):** The loop is completely unrolled at compile time. There are no iterator short-circuits (`take_while`, `break`, early `return`).
*   **Bit-Parallel Mechanics:** The boolean AND (`&=`) accumulates the correctness of all 32 bytes simultaneously without branching on a premature mismatch. 

### B. Mask Generation (`digest_eq_mask`)
According to `docs/innovations/zero_allocation_branchless_receipt_validation.md`, 256-bit comparisons are also implemented by casting 32-byte arrays into 64-bit words, enabling strict hardware-level execution masks:

```rust
#[inline(always)]
pub fn digest_eq_mask(a: &Digest, b: &Digest) -> u64 {
    let w0 = u64::from_le_bytes([a.0[0], ... , a.0[7]]);
    // ... w1, w2, w3 extraction
    let u0 = u64::from_le_bytes([b.0[0], ... , b.0[7]]);
    // ... u1, u2, u3 extraction
    
    let diff = (w0 ^ u0) | (w1 ^ u1) | (w2 ^ u2) | (w3 ^ u3);
    let is_eq = (diff == 0) as u64;
    0u64.wrapping_sub(is_eq)
}
```
*   **Mask-Based Execution Law (Rule 9):** `0u64.wrapping_sub(is_eq)` yields `0xFFFF_FFFF_FFFF_FFFF` (all ones) when digests match, and `0x0` when they differ. This enables subsequent `select` operations (e.g., `(m & a) | (~m & b)`) entirely devoid of conditional jumps (`CC=1`).
*   **Zero-Allocation Bound:** The operations operate directly on fixed memory bounds on the stack, complying with the absolute requirement for `0 heap allocation`. 

## Summary
Both implementations align strictly with the constitution's `@von_neumann_bypass` dictates. All comparisons transform what would normally be semantic decisions into branchless arithmetic selection and bounded execution work, ensuring timing attacks are structurally impossible and execution is 100% deterministic.
