# Branchless Error Handling (Rule 8)

The `bcinr` codebase strictly enforces the Absolute `CC=1` law (Rule 8 from `AGENTS.md`), which forbids control-flow branches such as early returns, `?`, `unwrap`, and `match`-based `Result`/`Option` short-circuiting. To adhere to this rule, the codebase uses a branchless error accumulation pattern.

Instead of terminating early when an error occurs, the implementation unconditionally executes the entire path, aggregating error states into bitmasks (fault bits). The final `Result` is constructed without conditional jumps using array indexing.

## 1. Bitwise Error Accumulation
Errors are captured by accumulating fault flags using bitwise OR (`|`). 

In parsing operations (e.g., `parse_hex_u32` in `crates/bcinr-logic/src/parse.rs`), a `u32` error mask accumulates faults across a fixed number of loop iterations (via `for_each`):
```rust
let mut err = (len == 0 || len > 8) as u32;
(0..8).for_each(|i| {
    // ... logic ...
    // Accumulate error if the character is not a valid digit
    err |= (!(is_digit | is_upper | is_lower) & (i < len) as u32) & 1;
    // ...
});
```

For more complex domain errors, structures like `RefusalSet` (`crates/bcinr-cmca/src/allocator.rs`) wrap a `u32` and accumulate specific fault bits via branchless operations:
```rust
#[inline(always)]
pub const fn union(self, other: Self) -> Self {
    Self(self.0 | other.0) // Bitwise OR accumulation
}

#[inline(always)]
pub const fn masked(self, condition: u32) -> Self {
    // Zeroes `self` unless `condition` is `1` using a mask subtraction
    Self(self.0 & 0u32.wrapping_sub(condition & 1))
}
```

## 2. Array Indexing for Result Construction
To return a `Result` without invoking `if` or `match`, the final success/failure evaluation uses constant-time array indexing. The condition boolean (0 or 1) is used as an index to pick between the `Err` and `Ok` variants.

From `crates/bcinr-logic/src/parse.rs`:
```rust
// If err == 0, index 1 (Ok) is returned. 
// If err != 0, index 0 (Err) is returned.
[Err(()), Ok(res)][(err == 0) as usize]
```

From `wrap_result` in `crates/bcinr-cmca/src/allocator.rs`, showing a similar approach mapped to a typed `StabilityRefusal`:
```rust
pub fn wrap_result(
    pi_res: [NonNegativeFixed; N],
    err_code: u32,
) -> Result<[NonNegativeFixed; N], StabilityRefusal> {
    let err_val = REFUSALS[(err_code as usize) & 31];
    let is_ok = const_eq_u32(err_code, u32::MAX);
    let outcomes = [Err(err_val), Ok(pi_res)];
    outcomes[(is_ok as usize) & 1]
}
```

## 3. Mask-Based State Selection
When errors are encountered mid-operation and a state rollback or fallback is necessary, the substrate never branches using `if err { ... } else { ... }`. It uses a fully-evaluated branchless select to apply or discard faults (`crates/bcinr-cmca/src/allocator.rs`):
```rust
#[inline(always)]
fn select_nnf(condition: u32, a: NonNegativeFixed, b: NonNegativeFixed) -> NonNegativeFixed {
    let mask = CanonicalMask::from_lsb(condition);
    NonNegativeFixed::from_parts(
        mask.select_u32(a.value_bits(), b.value_bits()),
        mask.select_faults(a.faults(), b.faults()), // Picks the fault set corresponding to the selected branch
    )
}
```

By deferring all decisions to bitwise arithmetic and array access, `bcinr` entirely eliminates CPU branching caused by error handling, preserving constant-time (`CC=1`) determinism.
