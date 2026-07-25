# Branchless Mask and Scan Primitives in BCINR

The BCINR substrate enforces absolute branchlessness (the Radon Law, $CC=1$) to prevent data-dependent pipeline stalls, ensure deterministic execution work, and eliminate timing side-channels. In `bcinr-logic`, all scanning and masking primitives achieve this by encoding logic as polynomial arithmetic, using two main techniques: the **all-ones/all-zeros mask convention** and **SWAR (SIMD Within A Register)**.

---

## 1. Mask Primitives (`mask.rs`)

Mask calculus replaces conditional jumps (`if`/`else`) with bitwise arithmetic. The foundational convention is:
- `0xFFFF_FFFF` (all ones) means **True**
- `0x0000_0000` (all zeros) means **False**

### Conditional Selection
The workhorse function is `select_u32(mask, a, b)`, which implements the B-Calculus identity $M(c, a, b) = (c \land a) \lor (\neg c \land b)$.
```rust
pub const fn select_u32(mask: u32, a: u32, b: u32) -> u32 {
    (mask & a) | (!mask & b)
}
```
Both `a` and `b` are evaluated unconditionally. The selection is derived purely through bitwise operations, ensuring completely static execution paths.

### Equality and Zero Detection
To derive masks branchlessly, the library uses two's-complement identities:
- **`is_zero_mask_u32(x)`**: Leverages the fact that `(x | -x)` forces the sign bit to `1` for all non-zero numbers.
  ```rust
  let non_zero_msb = (x | x.wrapping_neg()) >> 31;
  non_zero_msb.wrapping_sub(1)
  ```
  If `x == 0`, `non_zero_msb` is `0`. Subtracting `1` wraps it to `0xFFFF_FFFF`. If `x != 0`, `non_zero_msb` is `1`. Subtracting `1` yields `0x0000_0000`.
- **`eq_mask_u32(a, b)`**: Reuses the above logic on `x = a ^ b`.

### Ordering and Arithmetic
- **`lt_mask_u32(a, b)`**: Uses `0u32.wrapping_sub((a < b) as u32)`. The Rust compiler (`rustc` / LLVM) robustly lowers this into a branchless `SETB` + `NEG` sequence on x86-64.
- **Min / Max**: Simply compose `lt_mask_u32` with `select_u32`.
- **`abs_i32(x)`**: Broadcasts the sign bit into a mask via an arithmetic right shift (`mask = x >> 31`), then conditionally negates the value with `(x ^ mask).wrapping_sub(mask)`.

---

## 2. Scan Primitives (`scan.rs`)

Scanning operations conventionally use `while` loops or `break`/`return`, violating $CC=1$. BCINR resolves this by calculating state for the *entire* maximum bounded domain (or slice size) unconditionally, using data-independent loop bounds.

### SWAR Acceleration and Zero-Byte Detection
To keep operations efficient while avoiding early returns, BCINR packs 8 bytes into a `u64` and uses SWAR tricks.
The foundational technique is branchless zero-byte detection:
```rust
v.wrapping_sub(0x0101_0101_0101_0101) & !v & 0x8080_8080_8080_8080
```
This formula evaluates to a 64-bit integer where the high bit (`0x80`) is set in each byte lane that originally contained `0x00`. To find a specific character, the word is XORed with a broadcast mask of the target character, turning all occurrences into zero-bytes.

### Notable Scan Implementations
- **`swar_find_all_positions(bytes, target)`**: Uses the SWAR XOR trick to locate the target byte, then shifts and extracts the resulting `0x80` bits to return a packed `u64` bitmask of the target's locations—all without a single comparison branch.
- **`skip_spaces(bytes)`**: Instead of breaking on a non-space, it iterates the entire slice. The running offset accumulates via `is_space & mask`, where `mask = (offset == i) as usize`. The `mask` evaluates to `1` only if every preceding character was also a space, organically halting the counter's growth branchlessly.
- **`count_leading_eq_u8(bytes, target)`**: Processes 8 bytes at a time, calculating `all_match` using SWAR. An accumulation lock `done` is maintained: `done |= 1 - all_match`. The count updates via `count += 8 * all_match * (1 - done)`. Once a non-matching byte is found, `done` becomes `1`, and all subsequent loop iterations multiply their updates by `0`, leaving the total count pristine.
- **`is_ascii_u64_slice(bytes)`**: Processes 8-byte chunks, ORing `val & 0x8080_8080_8080_8080` into a rolling accumulator. Returns `accumulator == 0` at the end, confirming no MSB was ever set across the whole scan.

### Parallel Prefix Scans (Hillis-Steele / Blelloch)
In addition to byte scanning, `scan.rs` provides unrolled branchless algorithms for parallel prefix sums and max scans (`prefix_sum_u32x16`, `exclusive_scan_u32x16`). These replace data-dependent accumulations with fully unrolled static network paths, reducing $O(N)$ sequential dependencies into $O(\log N)$ logarithmic passes.
