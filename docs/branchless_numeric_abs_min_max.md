I have researched the branchless `Absolute Value` and `Min/Max` implementations in `crates/bcinr-logic/src/fix.rs` and `int.rs`. Here is how they are computed, strictly adhering to the project's branchless requirements (especially Rule 14 and Rule 9):

### 1. `abs_diff_u32` (Absolute Difference) in `int.rs`
The absolute difference computes `abs(a - b)` branchlessly by first deriving a full-width mask from a comparison predicate, effectively computing `min(a, b)` and `max(a, b)`, and then subtracting the two.

```rust
pub const fn abs_diff_u32(a: u32, b: u32) -> u32 {
    // mask is all-ones if a < b, else all-zeros.
    let mask = 0u32.wrapping_sub((a < b) as u32);
    // lo = min(a, b), hi = max(a, b) -- branchless select.
    let lo = (a & mask) | (b & !mask);
    let hi = (b & mask) | (a & !mask);
    hi - lo
}
```
**Mechanism:**
- Evaluates `(a < b) as u32`, yielding `1` or `0`.
- Uses `0u32.wrapping_sub(...)` to turn `1` into `0xFFFFFFFF` (all ones) and `0` into `0x00000000` (all zeros), creating a full-width mask as required by Rule 9.
- Selects the minimum (`lo`) and maximum (`hi`) using the bitwise select pattern `(x & mask) | (y & !mask)`.
- Subtracts `lo` from `hi` securely (guaranteed not to wrap).

### 2. `clamp_u64` (Min/Max) in `int.rs`
This implementation computes `min(max(val, lo), hi)` using an XOR-based branchless bitwise select.

```rust
pub const fn clamp_u64(val: u64, lo: u64, hi: u64) -> u64 {
    // Branchless max(val, lo): if val < lo, choose lo, else choose val.
    let after_lo = {
        let diff = lo ^ val;
        let mask = 0u64.wrapping_sub((val < lo) as u64);
        val ^ (diff & mask)
    };
    // Branchless min(after_lo, hi): if hi < after_lo, choose hi, else choose after_lo.
    {
        let diff = after_lo ^ hi;
        let mask = 0u64.wrapping_sub((hi < after_lo) as u64);
        after_lo ^ (diff & mask)
    }
}
```
**Mechanism:**
- It uses the XOR swapping trick to avoid the bitwise NOT operation: `val ^ ((lo ^ val) & mask)`.
- If the mask is all ones (`0xFFFFFFFFFFFFFFFF`), it evaluates to `val ^ lo ^ val = lo`.
- If the mask is all zeros (`0x00`), it evaluates to `val ^ 0 = val`.

### 3. `clamp_u32` (Min/Max) in `fix.rs`
Similar to the `abs_diff_u32`, this directly uses the standard bitwise OR/AND select mandated by the Mask-Based Execution Law (Rule 9).

```rust
pub const fn clamp_u32(val: u32, min: u32, max: u32) -> u32 {
    let mut res = val;
    let lt_min = (res < min) as u32;
    res = (min & 0u32.wrapping_sub(lt_min)) | (res & !0u32.wrapping_sub(lt_min));
    let gt_max = (res > max) as u32;
    res = (max & 0u32.wrapping_sub(gt_max)) | (res & !0u32.wrapping_sub(gt_max));
    res
}
```
**Mechanism:**
- `lt_min` and `gt_max` dynamically construct masks that filter out the values branchlessly.
- Follows the exact mathematical equivalent of `select(m, a, b) = (m AND a) OR (NOT m AND b)`.

### Rule 14 & Rule 9 Compliance Validation:
- **No Branches:** Completely avoids `if/else`, complying with the absolute $CC=1$ rule.
- **Full-Width Masks (Rule 9):** Derives `0xFFFFFFFF` via `.wrapping_sub()` on `0` or `1`.
- **Deterministic and Bounded (Rule 14):** Execution is bounded, uses explicit wrapping semantics natively (`wrapping_sub`), entirely architecture-agnostic with strictly defined width types (`u32`, `u64`).
