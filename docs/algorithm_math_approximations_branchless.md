# Branchless Numeric Primitives in `bcinr-logic`

After exploring the `crates/bcinr-logic/src/algorithms/` directory, I found two algorithms implementing **logarithm** and **reciprocal** semantics in accordance with **Rule 14 (Numeric-law requirements)**.

## 1. Fixed-Point Logarithm (`fixed_point_log2.rs`)
**Primitive Found:** `logarithm`

### Mathematical Law
The function calculates a fixed-point unsigned estimation of $\log_2(\text{val})$, returned in Qx.fb format.
- **Integer part:** $\lfloor\log_2(\text{val})\rfloor = 63 - \text{clz}(\text{val})$.
- **Fractional part:** Defined by the high `fb` bits of the mantissa after the implicit leading one.
- **Boundary Behavior:** `val == 0` maps strictly to `0`.

### Branchless Enforcement (Rule 14 & Rule 9)
The implementation replaces all data-dependent logic with mathematical bitwise operations:
- **Zero-Handling Masking (Rule 9):** Instead of `if val == 0`, a non-zero bit is derived: `nz = ((val | val.wrapping_neg()) >> 63) & 1`.
- **Branchless Integer Part:** `ip = 63u64.wrapping_sub(lz as u64) & nz.wrapping_neg();`. The `.wrapping_neg()` creates a full width mask (`0xFF...FF` if non-zero, `0x00...00` if zero). This ensures the log of `0` is `0` without any `cmp/jmp` instructions.
- **Constant-Time Mantissa Extraction:** The implicit leading `1` is shifted out using `val.wrapping_shl(lz.wrapping_add(1))`. If `val` is `0`, this unconditionally zeroes out the mantissa.
- **Wrapping Arithmetic:** All additions and shifts are explicitly executed as `.wrapping_shl()` and `.wrapping_add()` to prevent Rust's hidden branch-bearing bounds/overflow checks (Rule 8).
- **Independent Oracle & Hostile Mutants (Rule 15 & 19):** Tested comprehensively against `fixed_point_log2_reference` (which explicitly branches via `if val == 0`), proving the bit-parallel hot path identically matches the mathematical definition.

## 2. Reciprocal Division (`base85_encode_ascii85.rs`)
**Primitive Found:** `reciprocal`

### Mathematical Law
Base-85 digits are extracted by sequentially dividing a 32-bit chunk by $85$.

### Branchless Enforcement (Rule 14 & Rule 13)
Hardware integer division (`/`) or modulo (`%`) often leverages variable-latency microcode or branching logic. To execute in constant time:
- **Reciprocal Multiplication:** True division by 85 is bypassed using an exact reciprocal constant multiplier. The mathematical law $x / 85$ is enforced branchlessly via: `(x.wrapping_mul(0xC0C0C0C1)) >> 38`.
- **Unrolled Straight-Line Code (Rule 13):** There is no loop. The digit extraction unrolls into sequential, deterministic quotient/remainder steps.
- **Bounded Error:** Because the input domain is fixed-width (a 32-bit word, `x & 0xFFFF_FFFF`), the specific magic number reciprocal `0xC0C0C0C1` mathematically guarantees $0$ relative error over the entire admitted domain.
