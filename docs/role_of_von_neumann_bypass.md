# The Role of `@von_neumann_bypass` (Architect of Arithmetic Logic)

Based on the `AGENTS.md` BCINR Deterministic Substrate Constitution, the **`@von_neumann_bypass`** agent serves a critical function in enforcing the project's foundational mandate: maintaining a runtime strictly bound by branchless algorithmics and zero heap allocation. 

## Primary Role & Ownership
* **Title:** Architect of Arithmetic Logic
* **Designation:** Authoritative implementation owner. 
* **Write Authority:** Has exclusive write ownership over the project's **authoritative implementation** (branchless bounded code). 
* **Subagent Location:** Configured via `.claude/agents/von-neumann-bypass.md`.

## Exclusive Authorities
The `@von_neumann_bypass` agent possesses absolute authority over the design and implementation of non-branching data paths. Their jurisdiction covers:

1. **Branchless Arithmetic Design:** Architecting hot-path algorithms that avoid control-flow branches (no `if`, `match`, or loops) by mapping logical paths directly to mathematical instructions.
2. **SWAR (SIMD Within A Register) Construction:** Leveraging bitwise operations on wide standard integer registers (e.g., 64-bit) to execute parallel operations without dedicated SIMD hardware, ensuring consistent branchless evaluation.
3. **SIMD Shuffles:** Using Vector/SIMD permutation instructions to reorganize data across lanes deterministically, avoiding scalar loop-based data movement.
4. **Fixed-Point Mechanics:** Managing fractional representations and numeric arithmetic purely through integer operations, as the constitution completely bans floating-point operations.
5. **Generated Unrolling:** Authoring const-generic or explicitly generated straight-line code to circumvent variable-bound iterations, preventing any loop backedges in compiled object code.
6. **Mask-Based State Selection:** Deriving execution states using deterministic arithmetic masks (e.g., full-width `0` or `2^w-1`) instead of branches.
7. **PDEP/PEXT Use:** Structuring Bit Deposit (`PDEP`) and Bit Extract (`PEXT`) instructions where admitted to compress/expand sparse bits effectively without sequential branching.

## Required Implementations
Sequential semantic decisions cannot be processed conventionally. The Architect of Arithmetic Logic is tasked with strictly transforming them into:
* Masks
* Arithmetic selection
* Fixed lookup tables
* Generated straight-line code
* Fixed-width state transitions

Critically, the constitution specifies that implementations **must not hide branches in abstractions** (such as private wrappers, traits, or macro expansions).

## The Core Standard: "Bit-parallel mechanics over byte-sequential control flow."
This standard encapsulates the ethos of the Architect of Arithmetic Logic. 

* **Byte-sequential control flow** relies on executing different execution blocks byte-by-byte based on conditional logic (e.g., iterating with loops, checking limits, early returns, using `if/else`). 
* **Bit-parallel mechanics** shifts this paradigm to evaluate states synchronously. Rather than making run-time decisions that alter the instruction pointer, logic is expressed as mathematical polynomials and bitwise operations evaluated simultaneously across wide registers. 

By replacing "control flow" with "mechanics," the standard demands that all paths be evaluated. The correct outcome is selected arithmeticly at the end—using structures equivalent to `select(m, a, b) = (m & a) | (~m & b)`. This physically prevents timing side-channels, fulfills the Cyclomatic Complexity requirement of 1 ($CC=1$), and produces bounded, fixed deterministic mechanics.

## Cross-Functional Mandates
As part of the mandatory 4-way decomposition protocol, `@von_neumann_bypass` collaborates tightly but operates under the **No self-certification** law. They cannot self-certify the mathematical correctness, branchlessness, mutant adequacy, or final object code of their implementation. They receive proofs from `@hoare_oracle`, are audited by `@turing_machine`, and are tested through hostile mutations by `@armstrong_fault`.
