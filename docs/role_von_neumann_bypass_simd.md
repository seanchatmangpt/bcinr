Here is the research regarding the role of `@von_neumann_bypass` based on `AGENTS.md`:

### Role of `@von_neumann_bypass` (Architect of Arithmetic Logic)

Under **Rule 4** of the constitution, `@von_neumann_bypass` operates as the **Authoritative implementation owner**. 

#### Exclusive Authority
This agent holds exclusive authority over structural and arithmetic implementations designed to avoid branching. Their domain specifically includes:
* **SIMD shuffles**
* **PDEP/PEXT use where admitted**
* Branchless arithmetic design
* SWAR (SIMD Within A Register) construction
* Mask-based state selection
* Fixed-point mechanics
* Const-generic and generated unrolling

#### Required Behavior & Standards
The Architect of Arithmetic Logic is tasked with translating all sequential semantic decisions into deterministic, constant-time operations. This includes using:
* Masks and arithmetic selection
* Fixed lookup tables
* Generated straight-line code
* Fixed-width state transitions

**The Governing Standard:** *"Bit-parallel mechanics over byte-sequential control flow."* The implementation must never hide branches in abstractions. 

#### Rules on Architecture-Specific Instructions (Rule 22)
Although `@von_neumann_bypass` governs the use of "PDEP/PEXT where admitted", these architecture-specific instructions come with stringent requirements under Rule 22. Any use of PDEP/PEXT must have:
1. An admitted target capability.
2. A lawful fallback target (which must satisfy the same structural and branchless laws) OR a typed refusal.
3. Separate disassembly evidence.
