# Zero-Branch Saturating Arithmetic in `bcinr`

In the `bcinr` deterministic substrate, absolute adherence to the Radon Law ($CC=1$) means that no public primitive can use `if`, `match`, or standard library methods that intrinsically hide branches like `checked_add` (which yields an `Option` and forces subsequent control-flow routing).

To achieve constant-time, zero-branch saturating arithmetic, `bcinr` relies on bitwise polynomial multiplexing, formalized in the **CanonicalMask** abstraction. This technique evaluates conditions mathematically and expands a boolean predicate into a full-width bitmask via `wrapping_sub`.

## The `CanonicalMask` and `wrapping_sub` Interplay

The core mathematical trick is converting a 1-bit boolean evaluation (0 or 1) into a 32-bit (or 64-bit) selection mask using two's-complement arithmetic:

```rust
// Internally in CanonicalMask::from_lsb
let mask = 0u32.wrapping_sub(condition_bit);
```

- **If `condition_bit` is `1` (True):** `0 - 1` underflows, resulting in `0xFFFFFFFF` (all 1s).
- **If `condition_bit` is `0` (False):** `0 - 0` is `0x00000000` (all 0s).

Once the mask is generated, it replaces `if-else` blocks entirely by multiplexing between two potential outcomes using bitwise `AND` (`&`) and `OR` (`|`).

---

## 1. Branchless Saturating Addition (`add_sat`)

A standard addition can overflow, requiring a clamp to the type's `MAX`. Instead of checking if the result is valid and branching, `add_sat` resolves the clamp purely via logic gates.

```rust
pub const fn add_sat(a: u32, b: u32) -> u32 {
    // 1. Perform wrapping addition
    let res = a.wrapping_add(b);
    
    // 2. Evaluate overflow condition: in unsigned addition, overflow occurs if res < a
    let overflow_bit = (res < a) as u32; 
    
    // 3. Generate the mask using wrapping_sub
    let mask = 0u32.wrapping_sub(overflow_bit); 
    
    // 4. Multiplex outcome
    res | mask
}
```

**Mathematical breakdown:**
- **No Overflow (`mask = 0x00000000`):** `res | 0x00000000 = res`. The original wrapping sum is returned.
- **Overflow (`mask = 0xFFFFFFFF`):** `res | 0xFFFFFFFF = 0xFFFFFFFF` (`u32::MAX`). The result is forced to maximum, successfully saturating.

Because the condition is evaluated mathematically and the mask is unconditionally applied, the execution path remains identical for all inputs.

---

## 2. Branchless Saturating Subtraction (`sub_sat`)

Similarly, saturating subtraction clamps to `0` when an underflow occurs. 

```rust
pub const fn sub_sat(a: u32, b: u32) -> u32 {
    let res = a.wrapping_sub(b);
    
    // In unsigned subtraction, underflow occurs if a < b (or res > a)
    let underflow_bit = (a < b) as u32;
    
    // Generate the mask
    let mask = 0u32.wrapping_sub(underflow_bit);
    
    // Multiplex outcome: if underflow, mask is all 1s. We invert the mask to force 0.
    res & !mask
}
```

**Mathematical breakdown:**
- **No Underflow (`mask = 0x00000000`):** `!mask = 0xFFFFFFFF`. `res & 0xFFFFFFFF = res`.
- **Underflow (`mask = 0xFFFFFFFF`):** `!mask = 0x00000000`. `res & 0x00000000 = 0`. The result is properly floored to zero.

---

## 3. Branchless `clamp`

Clamping a value between `min` and `max` requires sequentially choosing the lower bound if `val < min`, and the upper bound if `val > max`. In `bcinr`, this is executed by applying the `CanonicalMask` selection logic twice:

```rust
pub const fn clamp_u32(val: u32, min: u32, max: u32) -> u32 {
    let mut res = val;
    
    // 1. Lower bound check
    let lt_min = (res < min) as u32;
    let min_mask = 0u32.wrapping_sub(lt_min);
    res = (min & min_mask) | (res & !min_mask);
    
    // 2. Upper bound check
    let gt_max = (res > max) as u32;
    let max_mask = 0u32.wrapping_sub(gt_max);
    res = (max & max_mask) | (res & !max_mask);
    
    res
}
```

**Mathematical breakdown for `(a & mask) | (b & !mask)`:**
- **If Condition is True (`mask = 0xFFFFFFFF`):**
  - `a & 0xFFFFFFFF` yields `a`. 
  - `b & 0x00000000` yields `0`. 
  - The bitwise `OR` gives `a | 0 = a`.
- **If Condition is False (`mask = 0x00000000`):**
  - `a & 0x00000000` yields `0`. 
  - `b & 0xFFFFFFFF` yields `b`. 
  - The bitwise `OR` gives `0 | b = b`.

In `clamp`, this technique forces `res` to seamlessly take the shape of `min` if `lt_min` evaluates to 1, and subsequently forces it to `max` if `gt_max` evaluates to 1. All paths process exactly the same polynomial operations, completely eliminating cyclic redundancy and remaining immune to timing side-channels.
