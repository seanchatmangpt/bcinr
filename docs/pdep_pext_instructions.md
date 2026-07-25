# PDEP/PEXT Instructions in BCINR

## What are PDEP and PEXT?
Parallel Bit Deposit (`PDEP`) and Parallel Bit Extract (`PEXT`) are advanced bit-manipulation hardware instructions (part of the x86 BMI2 extension). 
- **PEXT (Parallel Bit Extract):** Uses a mask to extract scattered bits from a source operand and packs them contiguously into the least significant bits of the destination.
- **PDEP (Parallel Bit Deposit):** Performs the inverse operation. It takes contiguous bits from the least significant portion of a source operand and scatters (deposits) them into the spatial positions specified by a mask, leaving all other bits as zeros.

## Value for Branchless Logic
In `bcinr`, the deterministic substrate strictly mandates branchless execution ($CC=1$) and mask-based state selection. Operations that isolate, filter, or re-route sparse data bits typically require variable-bound loops or data-dependent `if`/`match` branching, which are strictly prohibited. 

`PDEP` and `PEXT` are highly valuable because they condense complex, data-dependent bit routing into a single, constant-time arithmetic operation. They directly enable the constitutional mandate of "Bit-parallel mechanics over byte-sequential control flow," allowing developers to implement highly efficient SWAR (SIMD Within A Register) patterns and mask-based state transitions without violating zero-branch structural laws.

## Strict Rules Governing Their Use
According to the `bcinr` constitution (`AGENTS.md`), the use of `PDEP`/`PEXT` falls under the exclusive authority of `@von_neumann_bypass` (Architect of Arithmetic Logic, Rule 4). 

Furthermore, Rule 22 (Feature and target matrix) imposes strict compliance rules for architecture-specific instructions. Any authoritative use of `PDEP`/`PEXT` requires:

1. **An Admitted Target Capability:** The specific hardware instructions can only be used where the target architecture explicitly supports them and the capability is formally admitted.
2. **A Lawful Fallback Target or Typed Refusal:** The code must provide either a bounded typed refusal (if the hardware lacks support) or a lawful software fallback. Critically, **the fallback implementation must satisfy the same structural laws**—meaning the software fallback must also be mathematically proven, completely branchless ($CC=1$), and allocation-free.
3. **Separate Disassembly Evidence:** Because the execution path depends on architecture-specific capabilities, its inclusion requires a separate object-code audit. The engineer must produce explicit disassembly evidence proving that the compiler successfully emitted the `PDEP`/`PEXT` instructions and did not inject any conditional jumps or hidden branching.
