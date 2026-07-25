# Branchless Fixed-Point Modulo Operations in BCINR

In the `bcinr` architecture, the **Radon Law ($CC=1$)** strictly prohibits the use of variable-latency hardware instructions, including integer division (`div`) and modulo (`rem`). These instructions execute iterative subtract-and-shift algorithms that vary in clock cycles depending on the operands, introducing critical timing side-channels and violating deterministic execution laws.

To perform deterministic wrapping and modulo operations over Q16.16 fixed-point numbers without branching or variable latency, the substrate utilizes three primary bitwise arithmetic strategies depending on the divisor type:

## 1. Modulo by Powers of 2 (Fractional Extraction & Modulo 1.0)
When performing a modulo operation where the divisor is a power of 2 (or when extracting the fractional part of a Q16.16 number, which is effectively $X \pmod{1.0}$), the system completely bypasses division in favor of bit-shifts and two's complement wrapping.

For example, in the `exp2` approximation on signed Q16.16 numbers (`crates/bcinr-cmca/src/fixed.rs`):
```rust
let x = self.val;
let ip = x >> 16;                               // Arithmetic shift preserves sign
let fp = x.wrapping_sub(ip.wrapping_shl(16));   // Branchless modulo 1.0
```
This bitwise combination isolates the fractional part `fp` safely for both positive and negative values in a single constant-time cycle. Similarly, modulo $2^{64}$ operations (such as those in `modular_sub_u64.rs`) are implemented natively using `wrapping_sub`, mapping directly to constant-time ALU overflow mechanics.

## 2. Modulo by Compile-Time Constants (e.g., Trigonometric Domains)
When the divisor is known at compile time (such as $2\pi$ for trigonometric folding in `crates/bcinr-logic/src/fix.rs`), the Rust compiler (`rustc`/LLVM) is permitted to optimize the `%` operator. However, the runtime must still enforce branchless behavior for negative numbers.

LLVM optimizes modulo by a constant into a sequence of constant-time **magic reciprocal multiplications** and shifts, avoiding the hardware `rem` instruction entirely. To handle negative numbers branchlessly, `bcinr` projects the result back into the positive domain using a bitwise sign mask:

```rust
// Constant TWO_PI is known at compile-time, converting % to reciprocal multiplication
let raw = theta % TWO_PI; 

// Branchless correction for negative values:
// Shift right by 31 to broadcast the sign bit (-1 if negative, 0 if positive)
let neg_mask = raw >> 31; 

// If negative, neg_mask is 0xFFFFFFFF, selecting TWO_PI to wrap the value.
// If positive, neg_mask is 0x00000000, adding 0.
let wrapped = raw + (TWO_PI & neg_mask);
```
This translates a conditional wrapping requirement into a fully deterministic, branchless instruction sequence.

## 3. Dynamic Arbitrary Modulo (The Newton-Raphson Remainder)
When evaluating modulo with an unknown, dynamic divisor at runtime, the substrate utilizes its **Branchless Division Replacement** engine (documented in `docs/branchless_division_replacement.md`).

Rather than invoking a hardware `div`/`rem`, the substrate computes a highly precise reciprocal of the divisor $D$ using constant-time Newton-Raphson iterations ($X_3 \approx 2^{94} / d_{\text{norm}}$). 

The mathematical remainder (the modulo result) is then derived directly from the uncorrected quotient $q$ using purely bitwise wrapping arithmetic:
```rust
let rem = ((self.val as u64) << 16).wrapping_sub(q.wrapping_mul(d as u64)) as i64;
```
Because the reciprocal approximation can result in a quotient that differs by exactly $\pm 1$ LSB, the substrate branchlessly inspects the sign bits of the remainder and the divisor difference to assert if the quotient overshot or undershot:
```rust
let is_lt = ((rem >> 63) & 1) as u64;              // 1 if remainder is negative
let diff = rem.wrapping_sub(d as i64);
let is_ge = (((!diff) >> 63) & 1) as u64;          // 1 if remainder >= divisor
```
The true quotient (and effectively, the modulo remainder) is then corrected branchlessly using these extracted bitmasks, fully satisfying the requirement for deterministic, constant-time execution over the entire domain.
