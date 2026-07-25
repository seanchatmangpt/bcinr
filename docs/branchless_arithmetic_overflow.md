Here is the research report on how Rule 8 (`CC=1`) branchless arithmetic and overflow handling are implemented in the `bcinr` codebase.

### Overview of Branchless Arithmetic (Rule 8)

In the `bcinr` codebase, Rule 8 mandates that logic must be expressed without data-dependent control flow (no `if`, `match`, `?`, or branch-bearing `Option`/`Result`). To achieve safe arithmetic without panicking or bubbling up errors via branches, the codebase heavily utilizes bitwise logic, explicit wrapping operations (`wrapping_add`, `wrapping_sub`), masks, and a custom fault accumulation system (`NumericFaultSet`).

### 1. The Core Primitives: Canonical Masks and Selectors
Instead of conditionals, the substrate uses a `CanonicalMask` which evaluates to either `0x00000000` (FALSE) or `0xFFFFFFFF` (TRUE).
Conditions are calculated via bitwise polynomials that extract a flag (0 or 1), which is expanded into a full bitmask via `0u32.wrapping_sub(flag)`. 

Once a mask is created, conditional assignments are executed using the branchless bitwise select pattern:
```rust
#[inline(always)]
pub const fn select_u32(self, a: u32, b: u32) -> u32 {
    (a & self.0) | (b & !self.0)
}
```

### 2. Computing Overflow Branchlessly
Calculations natively use explicit `wrapping_add` or `overflowing_add`.

**For Unsigned Types (`u32` / Q16.16):**
The sum is computed directly. Overflow is detected via a branchless less-than check (`const_lt_u32(sum, a)`), which uses XOR and wrapping arithmetic to find if `sum < a`:
```rust
#[inline(always)]
pub const fn const_lt_u32(a: u32, b: u32) -> CanonicalMask {
    let diff = ((a ^ ((a ^ b) | (a.wrapping_sub(b) ^ b))) >> 31) & 1;
    CanonicalMask(0u32.wrapping_sub(diff))
}
```

**For Signed Types (`i32`):**
They leverage `overflowing_add` which returns a tuple `(sum, overflow_bool)`. The boolean is converted to an integer and negated into a canonical mask without branches:
```rust
let (sum, overflow) = self.val.overflowing_add(other.val);
// Cast bool to u32, subtract from 0 to get 0x00000000 or 0xFFFFFFFF
let overflow_mask = CanonicalMask(0u32.wrapping_sub(overflow as u32));
```

### 3. Saturation and Handling the "Error" State
When overflow occurs, standard Rust uses `Option` or `Result`, which inherently relies on enums and branching `match` statements. `bcinr` achieves this completely flatly:

```rust
// Example from saturating_add implementation:
let sat_val = is_neg.select_i32(i32::MIN, i32::MAX); 

// Select either the max/min bound or the computed sum:
let final_val = overflow_mask.select_i32(sat_val, sum);
```

For error tracking, instead of early-returns (`?`), it uses a bitwise union over a `NumericFaultSet`:
```rust
// Faults are accumulated via bitwise OR (never short-circuited).
let e = CanonicalMask::select_faults(
    overflow_mask,
    NumericFaultSet::OVERFLOW.union(NumericFaultSet::SATURATION),
    NumericFaultSet::EMPTY,
);

Self {
    val: final_val,
    faults: self.faults.union(other.faults).union(e),
}
```

### 4. Preventing Cross-Lane Carries in SWAR (SIMD Within A Register)
Rule 8 extends to parallel reductions using `u64` as packed vectors of 8 `u8`s (SWAR). Standard `+` would cause carries to overflow across byte-lane boundaries, causing data corruption (as documented in `reduce.rs`).

To branchlessly compare `u8` lanes safely without cross-lane pollution, `bcinr` uses lane-masking and a complex carry/borrow evaluation:
```rust
fn swar_byte_ge_mask(a: u64, b: u64) -> u64 {
    const HI: u64 = 0x8080_8080_8080_8080u64; // top bit of each lane
    const LO: u64 = 0x7F7F_7F7F_7F7F_7F7Fu64; // low 7 bits of each lane
    let a7 = a & HI;
    let b7 = b & HI;
    // Borrow into bit 7 of the low-7-bit subtraction: set iff a_lo7 >= b_lo7.
    let borrow = (a | HI).wrapping_sub(b & LO) & HI;
    // Fold in the real top bits to obtain the true 8-bit comparison in bit 7.
    let ge = ((a7 & !b7) | (!(a7 ^ b7) & borrow)) & HI;
    // Expand each lane's bit 7 to a full 0xFF mask, strictly within the lane:
    ge.wrapping_sub(ge >> 7) | ge
}
```
This polynomial isolates operations within sub-word lanes, ensuring bits carry cleanly without spilling over, fulfilling the rule requirement of deterministic output and fixed instruction shapes while completely avoiding `unwrap`, `expect`, or bounds-check panics.
