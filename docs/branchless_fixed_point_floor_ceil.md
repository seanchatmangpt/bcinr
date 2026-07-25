# Branchless Fixed-Point Floor and Ceil Operations in BCINR

In the `bcinr` deterministic substrate, numerical stability and strict `CC=1` (Cyclomatic Complexity = 1) enforcement require that non-linear operations—such as `floor` and `ceil`—be executed without any data-dependent control flow, branches, or conditional jumps. This is especially crucial when handling signed fixed-point types like `SignedFixed` (Q16.16 represented as a signed `i32`).

The substrate mathematically evaluates `floor` and `ceil` by exploiting fundamental properties of two's complement arithmetic and arithmetic bit-shifting.

## The Floor Operation (`to_num`)

The mathematical floor operation, $\lfloor x \rfloor$, maps a real number to the greatest integer less than or equal to $x$.

### Implementation
For a Q16.16 fixed-point number `x` (stored as an `i32` where the lower 16 bits represent the fractional part and the upper 16 bits represent the integer part), the floor operation is achieved via a simple arithmetic right shift:

```rust
// Extracted from bcinr's SignedFixed::to_num()
#[inline(always)]
pub const fn to_num(self) -> i32 {
    self.val >> 16
}
```

### Handling Negative Values Properly
In Rust, right-shifting a signed integer (`>>`) performs an **Arithmetic Shift Right (ASR)**, which preserves the sign bit. Mathematically, an ASR effectively divides the integer by $2^{16}$ and rounds towards negative infinity. 
- Positive example: $1.5$ is represented as `98304`. `98304 >> 16 = 1`.
- Negative example: $-1.5$ is represented as `-98304`. `-98304 >> 16 = -2`. 
- Negative integer: $-1.0$ is represented as `-65536`. `-65536 >> 16 = -1`.

Because two's complement shifts naturally round towards negative infinity, no `if x < 0` branching is required to correct the rounding direction. The operation is intrinsically branchless.

## The Ceil Operation

The mathematical ceiling operation, $\lceil x \rceil$, maps a real number to the least integer greater than or equal to $x$. 

### Implementation
Branchless ceiling evaluation relies on the mathematical identity:
$$ \lceil x \rceil = \lfloor x + 1 - \epsilon \rfloor $$
where $\epsilon$ is the smallest representable positive value in the fixed-point system.

For a Q16.16 system:
- $1$ (the whole number) is represented as `1 << 16 = 65536`.
- $\epsilon$ is the smallest fraction, represented by `1` (which is $2^{-16}$).
- Therefore, $1 - \epsilon$ is represented by `65536 - 1 = 65535`.

To compute the ceiling branchlessly:
```rust
#[inline(always)]
pub const fn ceil(self) -> i32 {
    // Add 65535 (wrapping to prevent overflow panics) and apply floor
    (self.val.wrapping_add(65535)) >> 16
}
```

### Handling Negative Values Properly
Because the underlying shift (`>> 16`) always rounds toward negative infinity, the pre-addition of `65535` perfectly offsets the fraction without overshooting whole numbers:
- Positive fraction: $1.000015$ (represented as `65537`). `(65537 + 65535) >> 16 = 131072 >> 16 = 2`.
- Exact positive integer: $1.0$ (represented as `65536`). `(65536 + 65535) >> 16 = 131071 >> 16 = 1`.
- Negative fraction: $-0.5$ (represented as `-32768`). `(-32768 + 65535) >> 16 = 32767 >> 16 = 0`.
- Exact negative integer: $-1.0$ (represented as `-65536`). `(-65536 + 65535) >> 16 = -1 >> 16 = -1`.

In every scenario—positive, negative, fractional, or exact integer—the exact mathematical bounds are achieved with 100% linear control flow.

## Exact Conservation Constraints

When `bcinr` divides fixed-point budgets (e.g., allocation uniform exploration floors), it cannot tolerate residual rounding errors. Instead of using a standard floating-point `floor` and accumulating float error, `bcinr` forces exact mathematical conservation. 

As seen in `cmca_generated.rs` and `allocator.rs`:
```rust
let q_floor = 65536u32 / nl_safe;
let r_floor = 65536u32 - q_floor * nl_safe;
```
Here, integer division intrinsically provides a mathematically perfect floor ($\lfloor \text{budget} / N \rfloor$), and a remainder is computed branchlessly to ensure exactly `65536` units are distributed, perfectly satisfying the substrate's zero-sum budget conservation requirements.
