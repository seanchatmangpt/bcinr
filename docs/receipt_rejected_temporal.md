# Branchless Evaluation of Temporal Window Bounds

In the `bcinr` deterministic substrate, executing temporal bounds checks (like Dwell Times) within the hot path must comply with the **Radon Law ($CC=1$)**. This mandate forbids any input-dependent control flow, such as `if elapsed < required { return Err(...); }`, because such statements compile into conditional jumps that cause variable execution time and violate the project's allocation-free, fixed-bound arithmetic invariants.

To resolve whether a temporal window has expired or is premature within the `ReceiptRejected` (and related `ModeDwellTimeViolated`) refusal context, the substrate employs **bitwise polynomial evaluation and canonical masking**. 

Here is the exact mechanism by which this is achieved:

## 1. Branchless Polynomial Comparison (`const_lt_u32`)

When checking if an elapsed time `tau_d` satisfies a required window (e.g., `MODE_DWELL_ROUNDS_MIN`), the substrate avoids standard comparison operators that might lower to a branch. Instead, it extracts the sign bit of the difference using two's-complement arithmetic.

The comparison function, `const_lt_u32(a, b)`, mathematically computes whether $a < b$:

```rust
#[inline(always)]
pub fn const_lt_u32(a: u32, b: u32) -> u32 {
    let a_bb = core::hint::black_box(a);
    let b_bb = core::hint::black_box(b);
    
    // Polynomial branchless `<` comparison
    // Isolates the sign bit after wrapping subtraction
    ((a_bb ^ ((a_bb ^ b_bb) | (a_bb.wrapping_sub(b_bb) ^ b_bb))) >> 31) & 1
}
```

- If `a < b` (i.e., the temporal window is **premature**), this polynomial produces a strict `1`.
- If `a >= b` (i.e., the temporal window has **expired / is satisfied**), it produces a strict `0`.

```rust
// Example: Checking if elapsed dwell rounds fail to meet the minimum
let dwell_err = const_lt_u32(tau_d, MODE_DWELL_ROUNDS_MIN);
```

## 2. Canonical Refusal Masking 

Once the mathematical condition yields a deterministic `0` or `1`, the substrate transforms this bit into a **canonical mask** spanning the entire integer width. 

This is accomplished by subtracting the condition bit from `0` using wrapping arithmetic:
`0u32.wrapping_sub(condition & 1)`

- If `condition == 1` (premature), `0 - 1 = 0xFFFFFFFF` (all `1`s).
- If `condition == 0` (satisfied), `0 - 0 = 0x00000000` (all `0`s).

This mask is used to conditionally apply the typed refusal (e.g., `DWELL_UNSATISFIED` or `RECEIPT_REJECTED`) without branching:

```rust
#[inline(always)]
pub const fn masked(self, condition: u32) -> Self {
    Self(self.0 & 0u32.wrapping_sub(condition & 1))
}
```

The resulting mask is then bitwise `OR`'d (`union`) into a global `RefusalSet`:

```rust
let gated_refusals = RefusalSet::EMPTY
    .union(RefusalSet::DWELL_UNSATISFIED.masked(dwell_err as u32));
```
*(If `dwell_err` is `0`, `0x00000000` is added, changing nothing. If `1`, the refusal bit is embedded into the set.)*

## 3. Mask-Based State Transitions

Finally, if the temporal window bounds are violated, the active refusal mask instructs a bitwise multiplexer (a `select` function) to revert the state to its previous version, while a lack of refusals commits the candidate state:

```rust
*prev_mode = const_select_u32(has_refusal as u32, *prev_mode, local_prev_mode);
```

When returning legacy `Result` types outside the pure authoritative root, the `1` or `0` flag is used as a direct array index to fetch the outcome branchlessly:

```rust
let outcomes = [Err(StabilityRefusal::ModeDwellTimeViolated), Ok(candidate_state)];
return outcomes[(is_ok as usize) & 1];
```

## Summary
By chaining `const_lt_u32` (sign-bit isolation via wrapping arithmetic), `wrapping_sub` (canonical mask generation), and bitwise unions, the substrate completely eliminates control flow for temporal evaluations. A premature temporal window dynamically manifests as an all-`1`s mask that mathematically blocks state mutation and inserts the rejection into the `RefusalSet`, strictly upholding the $CC=1$ mandate.
