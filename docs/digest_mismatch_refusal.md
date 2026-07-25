# Structural Evaluation of Digest Mismatches in the Deterministic Hot Path

In compliance with the BCINR Constitution (specifically the **Radon Law**: $CC=1$, no data-dependent branches, no loop backedges), the authoritative hot path structurally evaluates cryptographic digest mismatches (such as an envelope digest or `RDF_INPUT_DIGEST`) strictly through branchless arithmetic and bitmask composition.

This mechanism avoids `if` conditions, short-circuit returns, and timing side-channels, ensuring the execution clock cycles are identical regardless of whether the digest is valid or corrupted.

## 1. Loop-Free Byte Traversal

When evaluating a 32-byte hash (like BLAKE3), looping constructs (`for`, `while`) are prohibited because they compile into conditional jump instructions (loop backedges). Instead, `bcinr` leverages a static unrolling macro (`unroll_32_static!`) to inline the 32 byte-comparisons into a sequential straight-line pipeline:

```rust
let mut digest_match = 1u32;
unroll_32_static!(i, {
    digest_match &= const_eq_u32(
        digest[i & 31] as u32,
        expected_digest[i & 31] as u32,
    );
});
```

The accumulator `digest_match` starts at `1` and remains `1` only if every single bitwise-AND operation returns `1`. If even one byte differs, `digest_match` becomes `0`.

## 2. Branchless Equality Operation (`const_eq_u32`)

Inside the unrolled block, byte equality is evaluated using `const_eq_u32`. This avoids comparison instructions (`cmp`, `je`) by mathematically extracting the truth value using two's complement behavior:

```rust
#[inline(always)]
pub fn const_eq_u32(a: u32, b: u32) -> u32 {
    let x = core::hint::black_box(a) ^ core::hint::black_box(b);
    let nonzero = (x | x.wrapping_neg()) >> 31;
    1u32.wrapping_sub(nonzero)
}
```

- `a ^ b` (XOR) evaluates to `0` if and only if the bytes are identical.
- `x.wrapping_neg()` forces the most significant bit (sign bit) to `1` for any non-zero value.
- Shifting right by 31 (`>> 31`) yields `1` if `x` was non-zero, and `0` if `x` was zero.
- `1 - nonzero` yields `1` for equality and `0` for inequality. 

## 3. Masked Refusal Composition

With the accumulation complete, the hot path maps the result to a boolean-like `u32` error flag, extracting `1` if `digest_match == 0` (meaning there was a mismatch):

```rust
let digest_err = const_eq_u32(digest_match, 0) != 0;
```

Rather than using a branch (`if digest_err { return Err(DIGEST_MISMATCH); }`), the refusal is integrated into the operation's resulting `RefusalSet` via conditional bitmasking:

```rust
let gated_refusals = RefusalSet::EMPTY
    .union(RefusalSet::DIGEST_MISMATCH.masked(digest_err as u32))
    // ... unioned with other masked refusals
```

The `.masked(condition)` method applies a bitwise AND between the structural `DIGEST_MISMATCH` flag and the expanded `digest_err` mask. 
- If `digest_err` is `1`, the mask preserves the `DIGEST_MISMATCH` bits. 
- If `digest_err` is `0`, the mask unconditionally zeros the bits.

## 4. Branchless State Protection

Finally, because the hot path executes entirely without early returns, a mismatch means the state computation continues as if normal. However, Invariant 5 of the hot path demands that rejected inputs leave the persistent state *bit-for-bit unchanged*. 

To enforce this, a master `has_refusal` gate is computed by OR-ing all failure masks, which then drives a masked state-selection (e.g., `select_nnf` or `const_select_u32`) that selects between the candidate state and the original state:

```rust
let has_refusal = (has_error | ... ) & ...;

unroll_8_static!(v, {
    weights[v] = select_nnf(
        has_refusal as u32,
        weights[v],        // Keep old state if refused
        local_weights[v],  // Apply new state if admitted
    );
});
```

By substituting semantic decisions with data-driven masks, the deterministic execution graph evaluates the `DigestMismatch` securely, returning a fully-typed mathematical refusal structurally bound to the $CC=1$ rule.
