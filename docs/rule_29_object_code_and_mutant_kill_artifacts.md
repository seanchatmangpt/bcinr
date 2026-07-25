According to `AGENTS.md` (specifically Rule 19, Rule 20, and Rule 29), `OBJECT_CODE_AUDIT.md` and `MUTANT_KILL_MATRIX.md` are mandatory evidence artifacts required for every authoritative feature. They ensure the code adheres to the project's strict deterministic and branchless constraints.

### `OBJECT_CODE_AUDIT.md` (Defined in Rule 20)
**Why it is required:**
Source-level analysis (like checking for `if` statements or enforcing `CC=1`) is necessary but insufficient on its own. An exact, production-profile disassembly audit is required to prove that the final compiled machine code respects the substrate laws, ensuring no hidden branches, loop backedges, or allocations were introduced by the compiler.

**What it contains:**
An audit of all supported release targets inspecting:
- All authoritative root symbols and transitive helper symbols
- Panic and bounds-check symbols, allocator symbols
- Conditional jumps, loop backedges, indirect calls
- Floating-point instructions, division instructions, and unexpected runtime library calls

It must include a table of evidence in the following format for each symbol:
| Symbol | CC | Conditional jumps | Loop backedges | Panic path | Allocator | Standing |

---

### `MUTANT_KILL_MATRIX.md` (Defined in Rule 19)
**Why it is required:**
To prove that the test suite actually enforces the mathematical contracts and load-bearing laws of the implementation. The project operates on the principle that "a suite that cannot kill a plausible mutant is itself defective." This matrix guarantees that the tests are not just executing the code, but actively catching specific violations.

**What it contains:**
A ledger detailing adversarial hostile testing. For every implementation file, at least three independent, syntactically plausible mutants must be injected to alter a meaningful law (e.g., dropping a factor, bypassing a refusal). The matrix records how the system responded, verifying that the mutant triggered the expected typed refusal or independent oracle mismatch.

The mutant ledger must explicitly contain:
- mutant id
- source file
- changed law
- exact mutation
- expected detection
- actual detection
- test name
- receipt digest
- standing
