# Rule 9: Mask-Based Execution Law in BCINR

According to BCINR's constitution (Rule 9), the authoritative instruction shape must not depend on semantic input. This means traditional branching (`if valid { candidate } else { current }`) is strictly forbidden, as it causes CPU pipeline stalls and variable execution times based on data.

Instead, BCINR requires mapping runtime predicates to **full-width bitmasks** ($m \in \{0, 2^w-1\}$) and multiplexing state mathematically using bitwise selection: `select(m, a, b)`.

## 1. Transforming Runtime Predicates into Full-Width Masks

A mask $m$ represents a boolean state across all bits of a word (e.g., 32-bit):
- **True:** $m = 2^w - 1$ (all ones, e.g., `0xFFFFFFFF`)
- **False:** $m = 0$ (all zeros, e.g., `0x00000000`)

To transform predicates into masks branchlessly, BCINR leverages wrapping integer arithmetic and bitwise properties.

### Example: Less-Than Mask (`lt_mask_u32`)
When evaluating `a < b`, the transformation works as follows:
```rust
// (a < b) as u32 produces 0 or 1
// wrapping_sub converts 0 -> 0x00000000, 1 -> 0xFFFFFFFF
0u32.wrapping_sub((a < b) as u32)
```
- **If `a < b` is true (1):** `0 - 1 = 0xFFFFFFFF`
- **If `a < b` is false (0):** `0 - 0 = 0x00000000`

At the architecture level (e.g., x86-64), the compiler translates this to a branchless `SETB` (Set Byte if Below) followed by `NEG` (Negate), entirely avoiding a control-flow jump.

### Example: Equality Mask (`eq_mask_u32`)
Equality testing (`a == b`) can be done purely through bit manipulation:
```rust
let x = a ^ b; // 0 if equal, non-zero otherwise
// (x | -x) sets the high bit if x != 0
let non_zero_msb = (x | x.wrapping_neg()) >> 31; 
non_zero_msb.wrapping_sub(1)
```
- **If `a == b`:** `x = 0`, `non_zero_msb = 0`. `0 - 1 = 0xFFFFFFFF`.
- **If `a != b`:** `x != 0`, `non_zero_msb = 1`. `1 - 1 = 0x00000000`.

## 2. Mathematical Selection: `(m & a) | (~m & b)`

Once a predicate is converted into a full-width mask $m$, conditional assignment `if m { a } else { b }` is executed using the identity:
`M(m, a, b) = (m & a) | (~m & b)`

Because $m$ is either all ones or all zeros, it perfectly isolates the target value while zeroing out the other.

### Scenario A: Condition is True ($m =$ `0xFFFFFFFF`)
- `m & a` $\rightarrow$ `0xFFFFFFFF & a` $=$ `a` (preserves all bits of `a`)
- `~m & b` $\rightarrow$ `0x00000000 & b` $=$ `0` (destroys all bits of `b`)
- **Result:** `a | 0 = a`

### Scenario B: Condition is False ($m =$ `0x00000000`)
- `m & a` $\rightarrow$ `0x00000000 & a` $=$ `0` (destroys all bits of `a`)
- `~m & b` $\rightarrow$ `0xFFFFFFFF & b` $=$ `b` (preserves all bits of `b`)
- **Result:** `0 | b = b`

## Why this is strictly enforced
By enforcing this arithmetic form:
1. **Unconditional Evaluation:** Both paths (or values) are evaluated unconditionally, ensuring constant execution time (cyclomatic complexity $CC=1$).
2. **No Data-Dependent Branches:** Branch prediction is eliminated. There are no jumps based on input data, mathematically preventing timing side-channels and pipeline stalls.
3. **Formal Verification:** The resulting straight-line code maps perfectly to bounded bit-vector SMT solvers and Hoare logic postconditions.
