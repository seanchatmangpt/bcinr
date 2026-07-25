# Fixed-Point KL Divergence Accumulation in CMCA Observatory

The CMCA (Component Measure Co-Allocation) observatory uses a highly constrained fixed-point math engine to calculate Kullback-Leibler (KL) divergence ($\kappa_v$) scores. Due to strict constraints on the deterministic substrate (e.g., $CC=1$ cyclomatic complexity, no data-dependent branching, zero allocation), the accumulation of these scores relies entirely on branchless masking, loop unrolling, log-domain arithmetic, and widening fixed-point operations.

Here is how the CMCA observatory calculates and accumulates KL divergence without branching and without hitting numeric overflow:

### 1. Loop Unrolling and Fixed-Bounded Domains
Instead of variable-length loops or tree traversals, the observatory calculates KL divergence over statically bounded arrays representing the graph structure (`N` nodes, `K` measure heads). It utilizes macro-based loop unrolling (e.g., `unroll_8_static!`, `unroll_4_static!`) alongside precomputed boolean adjacency matrices (like `is_subtree_leaf_v`) passed into the `measure_kappa` function. This eliminates runtime branching (`while`, `for`) for topology traversal.

### 2. Branchless Selection via Bitmasking
Conditional logic (like checking if a node is a child or a leaf) is replaced by the bitwise `const_select_u32` function. 
- It computes a full-width mask by checking if the condition is non-zero: `let mask = 0u32.wrapping_sub(cond_val);`
- Then it uses the mask to perform the selection: `(a & mask) | (b & !mask)`.
This allows the calculation to include or exclude terms from the divergence sum unconditionally:
```rust
let term_selected = const_select_u32(is_child, (term >> 16) as u32, 0) as i32 as i64;
kappa_i64 = kappa_i64.wrapping_add(term_selected);
```

### 3. Log-Sum-Exp Trick to Avoid Exponentiation Overflow
KL divergence requires computing probabilities, which can easily underflow or overflow in 32-bit Q16.16 fixed-point representation. To accumulate these safely, the engine computes normalized probability distributions $s_{\text{leaf}}$ and $s_{\text{meas}}$ using the **Log-Sum-Exp** numerical stabilization technique branchlessly:
1. It computes the maximum log-probability in the bounded domain (`x_max_meas`).
2. It subtracts this maximum from all elements before exponentiation: `a_prime = x.wrapping_sub(x_max_meas)`.
3. It accumulates the sum of exponentials `sum_exp_meas`.
4. It computes the final log aggregate as `l_meas = x_max_meas.wrapping_add(sum_exp_meas.log2())`.
This guarantees that the inputs to `exp2()` are always $\le 0$, effectively mapping to $[0, 1]$ and preventing fixed-point overflow during exponentiation.

### 4. Avoiding Multiplication Overflow via `i64` Widening
The KL divergence term per child is $s_{\text{leaf}}(c) \cdot \log_2 \left( \frac{s_{\text{leaf}}(c)}{s_{\text{meas}}(c)} \right)$. 
In the code, this expands to:
```rust
let log_ratio = l_c.wrapping_sub(l_meas);
```
When calculating the term `s_leaf_c * log_ratio`, both are 32-bit fixed-point numbers. To prevent overflow during multiplication:
1. The numbers are widened and cast to `i64`.
2. A `wrapping_mul` is performed, utilizing the 64-bit capacity.
3. The result is then shifted back by 16 bits (`>> 16`) to retain the Q16.16 fractional scale before being downcasted.
```rust
let term = (s_leaf_c.value_bits() as i64).wrapping_mul(log_ratio as i64);
```

### 5. Wrapping Arithmetic
Across the entire calculation, standard operators (`+`, `-`, `*`) are banned because they can inject hidden panic branches (bounds checking) into the Rust AST. Instead, explicit `wrapping_add`, `wrapping_sub`, and `wrapping_mul` are employed. Any logical clamping must be performed branchlessly.

### 6. Branchless Non-Negative Clipping
Because KL divergence is mathematically strictly non-negative ($\ge 0$), fixed-point approximation error could theoretically result in a small negative accumulation. The observatory enforces mathematical correctness by clipping the final `kappa_i64` variable to 0 branchlessly:
```rust
let kappa_clipped = const_select_u32((kappa_i64 < 0) as u32, 0, kappa_i64 as u32);
let kappa = NonNegativeFixed::from_value_bits(kappa_clipped);
```

This yields a perfectly deterministic, $CC=1$, fixed-width calculation of Kullback-Leibler divergence.
