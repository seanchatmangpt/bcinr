# Branchless Digest Comparisons in `bcinr`

Per the project's strict Radon Law ($CC=1$) and numeric hot-path invariants, the allocator cannot use early-return loops (e.g., `if a[i] != b[i] { return Err(...) }`) for validating security certificates and digests. All validation logic must run in constant time using straight-line arithmetic and bitwise operations.

In `bcinr`, this is implemented differently for bounded `u64` digests and full `[u8; 32]` certificate hashes.

## Comparing 32-byte Arrays

For 32-byte digest verification (e.g., matching a provided `digest` against a statically compiled `CERTIFICATE_DIGEST` profile), `bcinr-cmca` uses a completely unrolled macro iteration combined with a branchless boolean equality helper:

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

### The Arithmetic Mechanism: `const_eq_u32`

The core component powering the equality check without branches is `const_eq_u32`, which converts an equality test into a `1` or `0` integer mask:

```rust
#[inline(always)]
pub fn const_eq_u32(a: u32, b: u32) -> u32 {
    let x = core::hint::black_box(a) ^ core::hint::black_box(b);
    let nonzero = (x | x.wrapping_neg()) >> 31;
    1u32.wrapping_sub(nonzero)
}
```

**How it works:**
1. **XOR:** `x = a ^ b` evaluates to `0` if and only if `a == b`.
2. **OR with Two's Complement:** `x | x.wrapping_neg()` exploits the properties of two's complement. If `x` is non-zero, either `x` or its negation will have the highest bit (the sign bit, bit 31) set to `1`. If `x` is `0`, both are `0`, and the MSB remains `0`.
3. **Shift:** `>> 31` shifts the MSB down to the LSB, resulting in `1` if the inputs differed, or `0` if they were identical.
4. **Negation:** `1u32.wrapping_sub(nonzero)` flips the semantic meaning back: it yields `1` when `a == b`, and `0` when they differ.
5. **Black Box:** `core::hint::black_box` guarantees that LLVM optimization passes do not re-introduce conditional branches (like jumps) behind the scenes, enforcing the invariant that object code retains the exact fixed instruction shape.

As the macro unrolls through all 32 bytes, it continuously `&`s the result. A single mismatch forces the running `digest_match` boolean integer to `0`, producing the equivalent of a logical `false` in a strictly numeric form.

## Multi-Hash OR-Reduction for `u64` State Digests

When validating smaller `u64` structs (like `AdmittedControlState`, `CertificateReceipt`, `EnvelopeReceipt`, and `OutcomeReceipt`), `bcinr` leverages bitwise `XOR` and `OR` accumulation rather than iterative comparison:

```rust
let digests_ok = (((state.digest ^ cert.digest)
    | (state.digest ^ env.digest)
    | (state.digest ^ outcome.digest))
    == 0) as u32;
```

**How it works:**
1. Pairwise `XOR` evaluates to `0` only if both sides match.
2. The results are accumulated via bitwise `OR` (`|`). The combined bitmask will contain a `1` anywhere there is a discrepancy across *any* of the pairwise comparisons.
3. The result equals `0` if and only if all evaluated digests are strictly equal to `state.digest`. 
4. The output is cast to a `u32` representation of a boolean, keeping the logic confined to arithmetic paths and preventing the compiler from forming branching control flows based on the outcome.

Both strategies transform what is traditionally sequential conditional logic into strict algebraic formulas, directly satisfying Rule 14 and the $CC=1$ rule.
