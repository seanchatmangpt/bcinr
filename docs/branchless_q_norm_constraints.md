# Branchless Q-Norm Arithmetic Constraints in `bcinr`

In `bcinr`, floating-point arithmetic (`f64`), standard control flow branches (`if`, `match`, data-dependent `loop`), and heap allocations are strictly forbidden in the Hot Path. This enforces the project's foundational mandate of **deterministic, sub-10ns, branchless execution** with a Cyclomatic Complexity (CC) of 1.

To adhere to these laws while computing vector magnitudes and Euclidean Q-norms, `bcinr` implements specialized fixed-point integer mathematics (typically using a Q16.16 format for fractions, or pure scaled integers). Rather than using hardware instructions or floating-point libraries, it employs two principal branchless strategies: **Unrolled Newton-Raphson Iteration** and **Digit-by-digit Reduction**.

Here is how the Hot Path computes these Q-norms branchlessly.

## 1. Branchless 2D Euclidean Magnitude (`norm_u32`)

For computing the magnitude of a 2D vector, $||V|| = \lfloor\sqrt{x^2 + y^2}\rfloor$, `bcinr` implements a deterministic, digit-by-digit integer square root algorithm rather than floating-point math.

Located in `norm_u32.rs`, the calculation operates on an intermediate `u128` to guarantee that squaring `u32`/`u64` values does not trigger overflow. 

```rust
// Extracted from bcinr_logic::algorithms::norm_u32
let mut val_sq = x * x + y * y;
let mut res = 0u128;
let mut bit = 1u128 << 64; // Highest even power of 4 for the 130-bit domain
let mut k = 0u32;

while k < 33 {
    let candidate = res + bit;
    let cond = val_sq >= candidate;
    // Produce a full-width canonical mask (0x000...000 or 0xFFF...FFF)
    let m = (cond as u128).wrapping_neg();
    
    // Masked evaluation prevents branching
    val_sq -= candidate & m;
    res = (res >> 1) + (bit & m);
    bit >>= 2;
    k += 1;
}
```

**Key Mechanisms:**
- **Fixed-Bounded Loops:** Although a `while` loop is used, it executes exactly 33 times unconditionally. This compiles down into an unrolled straight-line sequence, complying with the $CC=1$ mandate.
- **Canonical Masks:** Standard `if (val_sq >= candidate)` logic is eradicated. The boolean condition is cast to an integer and negated via `wrapping_neg()` to generate a bitmask (`m`). 
- **Bitwise Math:** State transitions (`val_sq -= candidate & m`) apply the results securely without branch-prediction hazards.

## 2. Fixed-Point Newton-Raphson (`isqrt_u32` & `q16_sqrt`)

For pure scalar square roots, particularly operating in Q16.16 fixed-point space (`fix.rs`), `bcinr` deploys manually unrolled Newton-Raphson iterations. 

### Step A: Branchless Seed Generation
A Newton-Raphson root-finding algorithm requires an initial estimate (seed). Floating-point math usually leverages an approximation instruction. `bcinr` synthesizes an initial estimate branchlessly via bit-length inspection:

```rust
// Initial estimate: 1 << ceil(bit_length / 2)
let shift = (32 - n.leading_zeros()) / 2;
let mut x = 1u32 << shift;
```

### Step B: Unrolled Iterations & Division-by-Zero Defense
The classical NR iteration for square roots is $x_{n+1} = \frac{x_n + \frac{S}{x_n}}{2}$. 
Because `x` could theoretically dip, causing a divide-by-zero panic, `bcinr` uses `.max(1)`. Since `.max()` is usually implemented branchlessly on standard architectures, this protects the denominator. The loop is manually unrolled 4 times, assuring convergence for the full 32-bit domain without risking a variable loop backedge:

```rust
// Four fixed Newton-Raphson iterations
x = (x + n / x.max(1)) / 2;
x = (x + n / x.max(1)) / 2;
x = (x + n / x.max(1)) / 2;
x = (x + n / x.max(1)) / 2;
```

### Step C: Branchless Overshoot Correction
Because integer division truncates, Newton-Raphson can settle slightly above the true integer floor root (an overshoot). To fix this, `bcinr` calculates the error unconditionally and subtracts it:

```rust
// Correct for overshoot (branchless): subtract 1 if x*x > n
let too_big = ((x as u64) * (x as u64) > n as u64) as u32;
x - too_big
```

## 3. Summary of Q16.16 Handling
For Q16.16 fractions (where $v = x / 65536$), finding $\sqrt{v}$ implies finding $\sqrt{x \cdot 65536} / 65536$. The library (`q16_sqrt`) applies a bitwise shift (`x << 16` onto a `u64` intermediate) and repeats the identical NR unrolling pattern (with 5 iterations for the 64-bit domain space).

By combining bitwise logical masks, zero-loop-backedge iteration, and deterministic precision bounds, `bcinr` seamlessly computes magnitudes, maintaining 100% compliance with the restrictive substrate constitution.
