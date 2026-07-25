Here is the documentation on the Mask Calculus, as requested:

# Mask Calculus

The **Mask Calculus** provides the foundation for branchless conditional selection in the B-Calculus framework. It replaces traditional control-flow branches (like `if`/`else`), which are vulnerable to pipeline stalls from CPU mispredictions, with pure bitwise arithmetic that always executes with a fixed latency.

## Mask Convention

The system represents boolean conditions using an **all-ones/all-zeros convention** rather than `1` and `0`. A valid mask is guaranteed to be one of two values (e.g., for 32-bit operations):
- **True:** `0xFFFF_FFFF` (all ones)
- **False:** `0x0000_0000` (all zeros)

## Algebraic Implementation of AND, OR, and NOT

Because masks rigidly follow the all-ones or all-zeros invariant across every bit, logical boolean operations can be algebraically implemented using single-instruction bitwise operators:

- **AND:** Implemented as bitwise AND (`mask_a & mask_b`).
- **OR:** Implemented as bitwise OR (`mask_a | mask_b`).
- **NOT:** Implemented as bitwise NOT (`!mask` in Rust, or `~mask`).

This bit-parallel algebraic structure completely eliminates the need for logical short-circuiting or branching.

## B-Calculus Notation

In the formal B-Calculus framework, the core mask operation (equivalent to a conditional select) is defined as:

$$M(c, a, b) = (c \ \& \ a) \ | \ (\sim c \ \& \ b)$$

Where:
- $c$ is the mask condition (`0` or `!0`)
- $a$ is the value selected if true
- $b$ is the value selected if false

This identity forms the bedrock of all conditionals in the library. Higher-level primitives such as `min`, `max`, `abs`, and `clamp` are entirely expressed in terms of $M$.

## Implementation Families

The `bcinr-logic` (and `bcinr-api`) crates expose several function families to support this calculus natively:

| Family | Description |
|--------|-------------|
| **Selection** | `select_u32`, `select_u64` — Performs $M(c, a, b)$ using an existing mask. |
| **Mask Generation** | `eq_mask_u32` (a == b), `lt_mask_u32` (a < b), `is_zero_mask_u32` (x == 0), `nonzero_mask_u32` (x != 0) — Safely generates the `0xFFFF_FFFF` or `0x0000_0000` patterns from comparisons. |
| **Branchless Math** | `min_u32`, `max_u32`, `abs_i32` — Composes mask generation and selection into higher-level arithmetic primitives. |

## Performance Profile

All Mask Calculus operations execute in $O(1)$ time with a data-independent instruction count. For example, `lt_mask_u32` typically compiles to a branchless `SETB` + `NEG` sequence on x86-64. Since no control-flow instructions are generated, the CPU can issue these instructions optimally alongside unrelated work with zero risk of branch misprediction.
