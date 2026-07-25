# Fixed-Point Logarithm (log2) Hoare Logic Laws

## 1. Domain and Codomain
- **Domain (Precondition)**: Admitted inputs `{ val, aux ∈ U64 }`.
- **Codomain (Postcondition)**: The output is an unsigned `Qx.fb` fixed-point representation mapped into `U64`. The integer part corresponds to `floor(log2(val)) = 63 - clz(val)`. The fractional part consists of the top `fb` (where `fb = aux & 63`) mantissa bits after the implicit leading one is dropped.
- **Zero-Input Handling**: The boundary condition `val == 0` explicitly and deterministically maps to `0` without utilizing panic paths or branching.

## 2. Absolute Error Limitations
For log-domain approximations used in normalization and relative entropy ($\kappa_q$) estimations, the fractional part drops the implicit leading one and utilizes a piecewise linear mapping ($\log_2(1+x) \approx x$). 

- **Maximum Absolute Error**: According to the substrate's numeric error envelope, the theoretical maximum absolute error bound for this piecewise linear $\log_2$ approximation is strictly bounded at:
  $$ \max |E_{\text{abs}}| = 1 - \frac{1}{\ln 2} + \frac{\ln(1/\ln 2)}{\ln 2} \approx 0.08607 $$

## 3. Independent Oracle Verification
The `@hoare_oracle` mandates that the production branchless implementation is structurally and logically validated against an independent reference, avoiding "circular oracle" violations.

- **Independent Derivation**: The reference oracle computes the logarithm using standard control flow (`if val == 0`), explicitly isolating the MSB (`63 - val.leading_zeros()`), and extracting the fractional bits by masking the bits strictly below the leading one (`val & ((1u64 << msb) - 1)`). This structurally independent approach verifies the production code's branchless bitwise operations (like `wrapping_shl`).
- **Hostile Mutation Testing**: To prove the integrity of the approximation bounds, the implementation is subjected to negative mutant testing (`@armstrong_fault`). The oracle confirms adversarial divergence by killing mathematically plausible but flawed mutants, such as the Identity Bluff (bitwise NOT of reference), Bit-skip Bluff (off-by-one errors), and Operator-swap Bluff (masking errors).
