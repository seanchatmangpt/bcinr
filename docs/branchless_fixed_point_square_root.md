# Branchless Fixed-Point Square Root in `bcinr`

In the `bcinr` (BranchlessCInRust) deterministic substrate, floating-point operations and hardware `sqrt` instructions are strictly forbidden. They introduce variable-latency side channels and violate the fundamental zero-branching ($CC=1$) architectural mandate. 

To compute the square root of `NonNegativeFixed` (Q16.16 format) numbers branchlessly, the runtime leverages two primary deterministic methodologies: **Constant-time Newton-Raphson Approximation** and **Digit-by-digit Reduction**.

## 1. Constant-Time Newton-Raphson Approximation (`isqrt_u32` & `q16_sqrt`)

For pure scalar square roots operating on the Q16.16 domain, `bcinr` employs a manually unrolled Newton-Raphson (NR) algorithm. For a Q16.16 value where $v = x / 65536$, the algorithm determines $\sqrt{x \cdot 65536} / 65536$ entirely via integer arithmetic.

### Branchless Seed Generation
A Newton-Raphson root-finding algorithm requires an initial estimate (seed). Floating-point math usually leverages an approximation instruction. `bcinr` synthesizes an initial estimate branchlessly via bit-length inspection using the `leading_zeros()` intrinsic:
```rust
// Initial estimate: 1 << ceil(bit_length / 2)
let shift = (32 - n.leading_zeros()) / 2;
let mut x = 1u32 << shift;
```

### Unrolled Iterations
The NR iteration step is $x_{n+1} = \frac{x_n + \frac{S}{x_n}}{2}$. To prevent a data-dependent loop backedge, the loop is manually unrolled exactly 5 times for the scaled 64-bit domain (or 4 times for standard 32-bit). The `.max(1)` construct (implemented branchlessly on target architectures) is used to prevent divide-by-zero panics without needing `if` statement guards:
```rust
// Fixed Newton-Raphson iterations
x = (x + n / x.max(1)) / 2;
x = (x + n / x.max(1)) / 2;
x = (x + n / x.max(1)) / 2;
x = (x + n / x.max(1)) / 2;
x = (x + n / x.max(1)) / 2; // Extra iteration for the scaled Q16 space
```

### Branchless Overshoot Correction
Because integer division truncates, the NR method can settle slightly above the true integer floor root (an overshoot). A branchless boolean correction is unconditionally computed and applied at the end:
```rust
// Correct for overshoot branchlessly
let too_big = ((x as u64) * (x as u64) > n as u64) as u32;
x - too_big
```

## 2. Digit-by-Digit Reduction (`fp_sqrt_u32_q16` / `norm_u32`)

For absolute worst-case bounding and strict magnitude constraints, `bcinr` uses a deterministic integer binary-search approach (digit-by-digit reduction), as implemented in `bcinr_logic::algorithms`.

An intermediate `u128` ensures scaling operations (`val << 16`) do not trigger overflow. The loop runs exactly 41 times unconditionally (since the scaled operand $< 2^{80}$, and the highest even power of four is $4^{40} = 2^{80}$).

```rust
let mut x = (val as u128) << 16;
let mut res = 0u128;
let mut bit = 1u128 << 80;
let mut k = 0u32;

while k < 41 { // Compile-time fixed bound, ensuring unrolled straight-line code
    let candidate = res + bit;
    let cond = x >= candidate;
    
    // Canonical mask generation: 0x0000000000000000 or 0xFFFFFFFFFFFFFFFF
    let m = (cond as u128).wrapping_neg();
    
    // Masked state evolution without branching
    x -= candidate & m;
    res = (res >> 1) + (bit & m);
    bit >>= 2;
    k += 1;
}
```

### Mask-Based Execution Breakdown
- **Canonical Masks**: The evaluation `x >= candidate` is cast to an integer and negated via `wrapping_neg()`. If `cond` is true (`1`), `m` securely overflows to all ones. If false (`0`), `m` remains all zeros.
- **Bitwise State Evolution**: Values unconditionally resolve using bitwise AND operations (`& m`). This mechanism ensures standard execution flow transforms from sequential conditional steps into fixed-width fieldwise masking, fully complying with `bcinr`'s constitutional ban on control-flow data dependency.
