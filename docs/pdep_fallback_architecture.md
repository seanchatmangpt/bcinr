# Rule 22: Architecture-Specific Instructions in BCINR (`PDEP`/`PEXT`)

Based on the BCINR Constitution (Rule 22: Feature and Target Matrix), the project strictly governs the use of architecture-specific hardware intrinsics like `PDEP` (Parallel Bit Deposit) and `PEXT` (Parallel Bit Extract). These BMI2 instructions are utilized to enforce the **Radon Law ($CC=1$)**, enabling sequential semantic decisions to be evaluated through constant-time bit-parallel logic without requiring control flow branches.

## Isolation and Capability Requirements

Under Rule 22, any architecture-specific instruction is isolated and strictly required to have:
1. **An admitted target capability**: Conditionally compiling via explicit capability gating, e.g., `#[target_feature(enable = "bmi2")]`.
2. **A lawful fallback target or a typed refusal**: A strict requirement to handle architectures where the hardware instruction is unavailable.
3. **Separate disassembly evidence**: Each target path must individually prove via disassembly that no branches, panics, or loop backedges are introduced into the final machine code.

## The PDEP/PEXT Fallback Architecture & Structural Laws ($CC=1$)

When a target architecture lacks native hardware support for `PDEP` or `PEXT`, degrading to a naïve branching software fallback (such as a `while` loop with an `if` condition) is **strictly prohibited**, as it introduces timing side-channels and violates the $CC=1$ deterministic requirement.

BCINR handles the fallback architecture using two strictly enforced mechanisms:

### 1. The Branchless Constant-Time Fallback
The compliant fallbacks rely on **fully unrolled, 6-stage constant-time parallel-prefix algorithms** (adapted from *Hacker's Delight*):
- `expand_bits_u64.rs` handles `PDEP`
- `compress_bits_u64.rs` / `bext_u64.rs` handle `PEXT`

To pass the structural laws ($CC=1$), these software fallbacks are constructed entirely of primitive bitwise arithmetic (`^`, `|`, `&`, `<<`, `>>`) with zero loops, conditionals, or branches. This guarantees they satisfy the exact same structural laws as the primary hardware intrinsics, ensuring execution time remains entirely data-independent. 

### 2. Typed Refusal (Guardrail)
If a target lacks native support and an $O(1)$ constant-time software fallback *cannot* be constructed or maintained, the runtime is forbidden from silently degrading into a branching loop, panicking, or truncating data.

Instead, the fallback path must immediately yield a deterministic **Typed Refusal** (e.g., returning `Err(SupportMismatch)`). This uncompromising requirement ensures that the execution shape never silently violates the project's rigid deterministic runtime laws.
