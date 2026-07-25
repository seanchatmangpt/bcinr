# Branchless Option and Result Handling in BCINR (Rule 8)

Under the BCINR constitution's **Rule 8 (Absolute CC=1 law)**, control-flow branches are strictly forbidden in authoritative code. This bans standard Rust paradigms like `?`, `unwrap`, `expect`, early returns, and `match` or `if let` blocks on `Option` and `Result`. 

To support typed refusals and optional values without violating CC=1, the codebase relies on a **branchless accumulation and array indexing** pattern.

## 1. Bitwise Accumulation (No Early Returns)
Instead of short-circuiting when an error or `None` state is encountered, algorithms unconditionally execute their entire path. They accumulate states (like "found" or "error") into integer masks.

For example, in `crates/bcinr-logic/src/parse.rs`, parsing loop runs exactly `N` times. Error states are accumulated via bitwise OR:
```rust
let mut err = (len == 0 || len > 8) as u32;
(0..8).for_each(|i| {
    // ... logic executes unconditionally ...
    
    // Accumulate error bitlessly if an invalid state occurs
    err |= (!(is_digit | is_upper | is_lower) & (i < len) as u32) & 1;
});
```

## 2. Array Indexing for Return Values
To return a `Result` or `Option` without a branch, the runtime constructs both variants and selects the correct one using constant-time array indexing, converting a boolean condition (0 or 1) to a `usize` index.

### Result Example
When parsing succeeds or fails, both `Err` and `Ok` are instantiated in a 2-element array. If `err == 0`, it resolves to index `1` (`Ok`); if `err != 0`, it resolves to index `0` (`Err`).

```rust
// From crates/bcinr-logic/src/parse.rs
[Err(()), Ok(acc as u64)][(err == 0) as usize]
```

For typed refusals, the same pattern is used mapped to explicit error codes:
```rust
// From crates/bcinr-cmca/src/allocator.rs
let is_ok = const_eq_u32(err_code, u32::MAX);
let outcomes = [Err(err_val), Ok(pi_res)];
outcomes[(is_ok as usize) & 1]
```

### Option Example
Similarly, when retrieving from a branchless data structure like `PackedKeyTable`, both `None` and `Some` are instantiated and selected via the aggregated `found` mask:

```rust
// From crates/bcinr-logic/src/autonomic/packed_key_table.rs
let mut result = V::default();
let mut found = 0usize;
(0..N).for_each(|i| {
    let is_match = (i < self.len && self.hashes[i] == hash) as usize;
    result = [result, self.values[i]][is_match]; // array-based reassignment
    found |= is_match;
});
// Return Option without a branch
[None, Some(result)][found]
```

## 3. Branchless Unwrap Equivalents (Mask Selection)
When an operation needs to "unwrap or default", the substrate defers to bitwise mask selection rather than `unwrap_or`. 

Instead of `if valid { candidate } else { current }`, it utilizes bitwise selection functions (like `select_u32`) or array indexing:
```rust
let mask = CanonicalMask::from_lsb(condition);
// Selects the bitwise fields of `a` or `b` unconditionally based on mask
NonNegativeFixed::from_parts(
    mask.select_u32(a.value_bits(), b.value_bits()),
    mask.select_faults(a.faults(), b.faults()), 
)
```

By deferring all semantic choices to bitwise arithmetic and fixed-width array lookups, `bcinr` preserves deterministic execution time regardless of the success/failure state or presence of optional values.
