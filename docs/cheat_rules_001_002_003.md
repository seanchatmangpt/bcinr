# Anti-Cheat Rules 001, 002, and 003

In the BCINR (BranchlessCInRust) deterministic substrate, the **Anti-Cheat Manifesto (Rule 16)** strictly forbids patterns that subvert mathematical verification, create fake complexity, or bypass rigorous structural auditing. Below is a detailed breakdown of the first three anti-cheat violations.

---

## CHEAT-001 — Self-Canceling Operations

**What it represents:**
Self-canceling operations are arithmetic or logical sequences inserted into the code that mathematically neutralize themselves, ultimately making zero contractual contribution to the output. 
*Example:* `a.wrapping_add(b) ^ a` when included solely to create apparent complexity.

**Why it artificially inflates complexity or obscures logic:**
- **False Density:** It introduces noise into the codebase. Reviewers and automated tools see more instructions, giving the illusion of deep algorithmic work, while the true logic remains trivial.
- **Audit Subversion:** Extraneous operations complicate object-code disassembly. In a system where the `@turing_machine` role must audit exact instruction mappings (ensuring `CC=1`), fake complexity obscures the actual branchless state transition logic.

**Why it is prohibited:**
The constitution dictates that **"Any operation without a contractual contribution to the output is prohibited."** In an allocation-free, branchless substrate, every single CPU instruction must map to an axiomatic purpose or state transition. Superfluous code is treated as structural fraud because it wastes execution cycles and actively hinders formal verification.

---

## CHEAT-002 — Circular Oracle

**What it represents:**
A circular oracle occurs when the reference test oracle is simply copied and pasted from the production implementation. 

**Why it artificially inflates complexity or obscures logic:**
- **Verification Theater:** It generates the illusion of a rigorous test suite. By asserting that the production code equals the test oracle, it looks like a complex verification step is passing, but it is just testing that the code is equal to itself.
- **Hidden Deficiencies:** It obscures underlying mathematical flaws. If the production code drops a factor, uses the wrong mask, or violates a boundary, a copied oracle will replicate the exact same flaw, masking the bug from automated detection.

**Why it is prohibited:**
This practice fundamentally violates **Rule 15 (Independent Oracle Law)** and the **mandatory decomposition protocol (Rule 5)**. The test oracle (owned by `@hoare_oracle`) must be structurally and logically distinct from the production code (owned by `@von_neumann_bypass`). An oracle must take an independent form—like a direct mathematical formula, symbolic proof, or exhaustive domain enumerator. Self-certification is constitutionally banned.

---

## CHEAT-003 — Magic Constants

**What it represents:**
The use of unexplained literals (e.g., `0xDEADBEEF`, `0xCAFEBABE`, `0xDEAD_BEEF`) that control or influence production behavior without any documented derivation.

**Why it artificially inflates complexity or obscures logic:**
- **Opaque Derivation:** Magic constants hide the "why" behind the arithmetic. They sever the link between the high-level mathematical specification and the runtime execution, making it impossible to determine if the constant is mathematically sound or merely an arbitrary guess.
- **Scanner Evasion:** Formatting tricks (like adding underscores) try to mask the presence of these constants from the AST `bcinr-cheat-scanner`, attempting to obscure unauthorized logic.

**Why it is prohibited:**
In a strict arithmetic logic environment, **Rule 14 (Numeric Law)** requires that every clamp, mask, or fixed-point coefficient must be named, derived, admitted, and included in the influence digest. If a value cannot be logically explained or derived from the Hoare contract, it has no legal standing in the repository and invalidates the project's Substrate Integrity Score (SIS). Formatting changes do not make an arbitrary constant lawful.
