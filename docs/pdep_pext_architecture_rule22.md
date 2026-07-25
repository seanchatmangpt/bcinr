# Rule 22: Architecture-Specific Instructions in BCINR (`PDEP`/`PEXT`)

Based on the project's documentation (`docs/architecture_feature_gating_pdep.md` and `docs/pdep_pext_branchless_instructions.md`), here is how BCINR strictly governs the use of architecture-specific hardware intrinsics like `PDEP` and `PEXT` under **Rule 22 (Feature and Target Matrix)**.

## Architecture Target Capability Requirements
`PDEP` (Parallel Bit Deposit / Scatter) and `PEXT` (Parallel Bit Extract / Gather) are advanced BMI2 bit manipulation instructions used to enforce the **Radon Law ($CC=1$)**. They enable sequential semantic decisions to be evaluated through constant-time bit-parallel logic without requiring control flow branches.

Rule 22 strictly dictates that any architecture-specific instruction must have:
1. **An admitted target capability** (e.g., conditionally compiling with `#[target_feature(enable = "...")]`).
2. **A lawful fallback target or a typed refusal**.
3. **Separate disassembly evidence** for each target to definitively prove that neither compilation path introduces a branch, panic, or loop backedge into the final machine code.

## Handling Unsupported Architectures

When a target architecture lacks native hardware support for `PDEP` or `PEXT`, a naïve branching software fallback (like a `while` loop with an `if` condition) is **strictly prohibited**. It introduces Jump Conditional Code (JCC) violations and timing side-channels, destroying the deterministic substrate.

Instead, BCINR handles unsupported architectures via two strictly enforced mechanisms:

### 1. The Branchless Fallback
The compliant fallbacks utilize **fully unrolled, 6-stage constant-time parallel-prefix algorithms** (adapted from *Hacker's Delight*):
- `expand_bits_u64.rs` for `PDEP`
- `compress_bits_u64.rs` / `bext_u64.rs` for `PEXT`

These software fallbacks consist purely of primitive bitwise arithmetic (`^`, `|`, `&`, `<<`, `>>`) without loops or branches. This guarantees they satisfy the exact same structural laws ($CC=1$) as the primary hardware intrinsics, ensuring execution time remains completely data-independent.

### 2. Typed Refusal (The Ultimate Guardrail)
If a target lacks native support and an $O(1)$ constant-time software fallback *cannot* be constructed, the runtime is forbidden from degrading into a branching loop, panicking, or silently truncating data.

Instead, the primitive must immediately yield a deterministic **Typed Refusal** (e.g., returning `Err(SupportMismatch)`). This uncompromising requirement ensures that the system's execution shape never silently violates the deterministic runtime laws.
