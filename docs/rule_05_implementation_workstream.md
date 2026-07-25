I have read `AGENTS.md` and reviewed the "Implementation" workstream under Rule 5. Here are the specific details:

**Implementation Workstream Output**
Under Rule 5 (Mandatory decomposition protocol), the exact output of the "Implementation" workstream is **branchless bounded code**.

**The Role of `@von_neumann_bypass`**
`@von_neumann_bypass` is the owner of the Implementation workstream. As defined under Rule 4, their specific role is the **"Architect of Arithmetic Logic"** and the **"Authoritative implementation owner."**

They hold exclusive authority over:
* Branchless arithmetic design
* SWAR construction
* SIMD shuffles
* PDEP/PEXT use where admitted
* Mask-based state selection
* Fixed-point mechanics
* Const-generic and generated unrolling

**Required Behavior:**
`@von_neumann_bypass` must transform sequential semantic decisions into masks, arithmetic selection, fixed lookup tables, generated straight-line code, and fixed-width state transitions. They are strictly prohibited from hiding branches in abstractions, adhering to the standard of "Bit-parallel mechanics over byte-sequential control flow."
