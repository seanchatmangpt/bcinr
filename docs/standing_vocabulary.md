### 1. The Exact Bounded Standing Values
According to **Rule 28** in `AGENTS.md` (and extended by rule supplements), the permitted standing vocabulary is strictly bounded to the following values. These represent the *verification and maturity level* of the project's code, not its runtime execution state:

- **`PROVEN`**: A specific theorem is machine-checked or exhaustively established over its declared domain.
- **`INVARIANT`**: True by construction or type exclusion.
- **`ALIVE`**: The implementation executes and passes all declared gates in the pinned environment.
- **`SOURCE_BRANCHLESS_PARTIAL`**: Source appears branchless, but complete object-code standing is not established.
- **`BRANCHLESS_ALIVE`**: The authoritative call graph passes source, complexity, allocation, panic, and disassembly audits.
- **`REPORTED_ALIVE`**: An agent reports success, but independent reproduction has not occurred.
- **`PARTIAL_ALIVE`**: Some required gates remain incomplete.
- **`UNKNOWN`**: Evidence is insufficient.
- **`REFUSED`**: The input or configuration is outside the admitted domain.
- **`BUILD_BROKEN`**: The pinned build fails.

**Additional Permitted Extensions:**
- **`MUTATION_GATE_FAILED`** (From Rule 19): A project-wide failure standing assigned when a hostile mutant survives the verification matrix (test suite). It acts as an emergency brake that immediately blocks all feature work.
- **`REPORTED`** (From `.claude/rules/10-standing-and-evidence.md`): A claim made by an implementation agent about its own work, not yet mechanically reproduced by an independent verifier.

---

### 2. The State Machine Category Clarification
There is a fundamental category distinction to be made regarding how these states are used. 

The terms `ALIVE`, `MUTATION_GATE_FAILED`, etc., are **not** states in the compiled Rust substrate's runtime state machine. They are **project/repository metadata states** used by human developers, agents, and CI pipelines to govern the project's verification lifecycle. Transitions between these standing states are handled procedurally (e.g., via the mutant ledger or Markdown standing reports), not by the Rust codebase itself.

### How the *Actual* Substrate State Machine Transitions (Branchlessly)
For the actual runtime data processing (the "substrate state machine"), the repository mandates a strictly deterministic, branchless transition protocol governed by **Rule 9 (Mask-based execution law)** and **Rule 10 (No mutation before complete admission)**.

The true runtime state machine (which handles structures like `AdmittedControlState`) transitions without branching via **full-width bitmasks and bitwise selection**:

1. **Immutable Candidate Derivation**: The system computes a fixed-size candidate state from the current state.
2. **Predicate Verification**: The system evaluates all predicates and derivations, transforming boolean decisions into full-width masks (where $m \in \{0, 2^w-1\}$).
3. **Masked Commit (Selection)**: The persistent state is atomically committed without `if`/`else` control flow using a fixed-width `select` equivalent to:
   $$ (m \land a) \lor (\neg m \land b) $$
   Or mathematically represented as:
   $$ x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t) $$

Thus, while the **project's verification ledger** transitions procedurally between `ALIVE` and `MUTATION_GATE_FAILED` based on audit outcomes, the **runtime Rust substrate** transitions between logic states exclusively via bit-parallel masks and arithmetic selection over continuous polynomial paths.
