# Branchless Enforcement of `clamp` and `min_max` Mathematical Laws

In the `bcinr-logic` crate, clamp operations (such as `clamp_i64` and `clamp_slice_branchless`) are mathematically defined to restrict an input value `val` within an inclusive range `[lo, hi]` derived from an auxiliary parameter `aux`.

According to the project's **Radon Law ($CC=1$)**, no public primitive may contain an `if`, `match`, or data-dependent `loop`. These mathematical bounds must be enforced in constant time without any control flow hazards.

## `clamp_i64` (Signed 64-bit Clamping)

**Location**: `crates/bcinr-logic/src/algorithms/clamp_i64.rs`

### The Mathematical Law
The algorithm must securely clamp a 64-bit integer `val` into an inclusive boundary. The boundaries are extracted from the lower and upper 32 bits of `aux` (sign-extended to 64 bits). Because the caller does not guarantee the order of the extracted bounds, the algorithm must dynamically determine the minimum (`lo`) and maximum (`hi`) bounds before applying the clamp.

### Branchless Enforcement
```rust
pub fn clamp_i64(val: u64, aux: u64) -> u64 {
    let v = val as i64;
    let a = (aux as i32) as i64; 
    let b = ((aux >> 32) as i32) as i64; 

    // 1. Branchless Ordering
    let lo = a.min(b);
    let hi = a.max(b);
    
    // 2. Branchless Clamping
    v.max(lo).min(hi) as u64
}
```

1. **Eliminating the `if` for Bounds Ordering:** Instead of using an `if a <= b { (a, b) } else { (b, a) }`, the code utilizes Rust's intrinsic `min` and `max` operations. These operations compile down to bitwise abstractions or native architecture conditional moves (e.g., `cmov` on x86, or native `min`/`max` vector instructions), which possess no instruction jumps or branch prediction penalties.
2. **Eliminating the `if` for the Clamp Check:** A typical branching clamp looks like `if v < lo { lo } else if v > hi { hi } else { v }`. To avoid this data-dependent branch, the algorithm composes two branchless bounds checks: `v.max(lo)` securely elevates any under-bound values to `lo`, and chaining `.min(hi)` forcibly lowers any over-bound values to `hi`. 

## `clamp_slice_branchless` (Unsigned 64-bit Clamping)

**Location**: `crates/bcinr-logic/src/algorithms/clamp_slice_branchless.rs`

### The Mathematical Law
The algorithm restricts an unsigned `val` to a fixed window where `min` is the upper 32 bits of `aux` and `max` is the lower 32 bits of `aux`.

### Branchless Enforcement
```rust
pub fn clamp_slice_branchless(val: u64, aux: u64) -> u64 {
    let min = aux >> 32;
    let max = aux & 0xFFFFFFFF;
    (val.max(min)).min(max)
}
```
Like `clamp_i64`, this entirely skips conditional boundary checking. The bounds are isolated via bitwise shifts and masking (`>> 32` and `& 0xFFFFFFFF`), and the clamp is mathematically resolved using composite `min`/`max` selections. This ensures that the generated CPU instructions form a single linear basic block of execution, achieving perfect $CC=1$ cyclomatic complexity.
