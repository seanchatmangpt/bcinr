# Rule 22: Feature and Target Matrix

In the `bcinr` deterministic substrate, Rule 22 mandates that all verification gates must run across **every** supported combination of features, profiles, and architectures.

## Why all gates must run across every supported combination
The overarching philosophy of `bcinr` is whole-call-graph branchlessness and zero-allocation determinism (the "Radon Law"). Executing verification gates on a single configuration is structurally insufficient because passing one feature configuration does not establish universal repository standing. 

1. **Compiler Backend Variance**: Rust code that compiles to branchless, straight-line assembly on `x86_64` might trigger implicit conditional jumps, loop backedges, or runtime library calls on other architectures depending on the LLVM backend.
2. **Feature-Bound Instability**: A project might perfectly conform to `#![no_std]` and branchless execution under `default features`, but activating `all features` or using `no default features` could silently link branching primitives, panic handlers, or heap allocations.
3. **Eradicating Dead-Path Compliance (CHEAT-007)**: By demanding audits on all supported target configurations, every compilation path and fallback is exposed to structural enforcement (the `@turing_machine` enforcer). Developers cannot bypass constitutional laws by hiding an unlawful, branching loop behind a `#[cfg(...)]` fallback for an unsupported architecture.
4. **Profile Variations**: Object code differs between profiles. Exact production-profile disassembly audits are necessary to prove that the final release machine code contains no loop backedges or conditional jumps. 
5. **Generated Code Stability**: The matrix must include a `generated clean tree` to ensure that generated code is perfectly reproducible and identical byte-for-byte, preventing unexplained drift or manual edits.

## Requirements for Architecture-Specific Instructions (e.g., PDEP/PEXT)
When employing architecture-specific hardware intrinsics (like BMI2's `PDEP` or `PEXT`), the implementation must strictly provide:

1. **An admitted target capability**: The target capability must be explicitly acknowledged and authorized.
2. **A lawful fallback target or typed refusal**: If the instruction is unavailable on the executing hardware, the system must either execute a lawful fallback or yield a typed refusal (e.g., `SupportMismatch`). It cannot fall back to a simpler, branching algorithm.
3. **Separate disassembly evidence**: The architecture-specific implementation must have its object-code independently audited and proven compliant, separate from the fallback's audit.
4. **Lawful fallback compliance**: Any fallback implementation is not exempt from the core laws—it must satisfy the **exact same structural laws** (e.g., whole-call-graph branchlessness, zero heap allocation, CC=1) as the hardware-accelerated version.
