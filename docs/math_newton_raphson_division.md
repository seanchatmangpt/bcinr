Here is the documentation on the Newton-Raphson approximation technique used for division in `crates/bcinr-cmca/src/fixed.rs`.

```markdown
# Newton-Raphson Division in `saturating_div`

The `saturating_div` function in `crates/bcinr-cmca/src/fixed.rs` implements division using the Newton-Raphson method to compute the reciprocal of the denominator without relying on hardware division instructions, ensuring deterministic, branchless execution.

## Initial Guess Generation

The initial guess (`x0`) for the reciprocal is generated using a linear approximation after normalizing the denominator:
1. **Normalization**: The denominator `d` is normalized using its leading zeros to shift its most significant bit to a fixed position:
   ```rust
   let lz = d.leading_zeros();
   let d_norm = d << lz;
   ```
2. **Linear Approximation**: A first-order approximation is computed using precalculated scale and coefficient constants:
   ```rust
   let a_scale = 13021703673752174592u64;
   let b_coeff = 2021160080u64;
   let x0 = a_scale.wrapping_sub(b_coeff.wrapping_mul(d_norm as u64));
   ```

## Static Unrolled Iterations

For the Q16.16 fixed-point format, the algorithm requires exactly **3 static unrolled iterations** to converge to the required precision. Each iteration computes the error `e` and refines the reciprocal estimate `x`:

```rust
// Iteration 1
let e0 = (1i128 << 94) - (d_norm as i128) * (x0 as i128);
let x1 = ((x0 as i128) + (((x0 as i128) * (e0 >> 32)) >> 62)) as u64;

// Iteration 2
let e1 = (1i128 << 94) - (d_norm as i128) * (x1 as i128);
let x2 = ((x1 as i128) + (((x1 as i128) * (e1 >> 32)) >> 62)) as u64;

// Iteration 3
let e2 = (1i128 << 94) - (d_norm as i128) * (x2 as i128);
let x3 = ((x2 as i128) + (((x2 as i128) * (e2 >> 32)) >> 62)) as u64;
```
After the 3rd iteration, `x3` is multiplied by the numerator and scaled accordingly to produce the quotient. A final branchless correction step handles any residual rounding error.
```
