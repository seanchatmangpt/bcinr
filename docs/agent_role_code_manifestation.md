I used `grep_search` to look for the literal agent handles (`@hoare_oracle`, `@turing_machine`, `@armstrong_fault`, and `@von_neumann_bypass`) across the rust source code (`.rs` files) in `/Users/sac/bcinr/crates/`. 

Interestingly, **the exact `@handle` strings do not appear anywhere in the Rust source code or test files themselves.** 

Instead, my searches revealed that these "transcendent constructs" manifest strictly through structural code annotations, specific attributes, and testing nomenclature rather than literal developer usernames:

### 1. `@hoare_oracle` (Oracle of Invariants)
Rather than tagging the agent handle, the axiomatic contracts manifest as **Hoare-logic proof annotations** inline with the code and in documentation:
- **Comments/Docs:** Appears as `/// # Hoare contract` or block comments like `// # AXIOMATIC PROOF: Hoare-logic Analysis`.
- **Line-by-Line Verification:** Heavily manifests as `// Hoare-logic Verification Line X: [Proof statement]` across complex data structures (e.g., `lock_free_slab.rs`, `bump_arena.rs`).
- **Test Nomenclature:** In the test suite (like `hostile_mutants.rs`), this manifests as "dedicated oracles" and test classifications like `KILLED_BY_INTENDED_ORACLE` to prove the mathematical boundaries hold.

### 2. `@turing_machine` (Enforcer of Determinism)
The enforcer of branchlessness and `CC=1` (Cyclomatic Complexity) manifests through the **"Radon Law"**:
- **Annotations:** Code is heavily annotated with assertions satisfying this enforcer's deterministic requirements, manifesting exactly as: `// Hoare-logic Verification Line X: Radon Law verified.` (or `Radon Law satisfied.`). This explicitly asserts that the structural auditor has proven the code is completely branchless.

### 3. `@armstrong_fault` (Master of Failure Law)
The adversarial testing and mutation architecture manifests directly in the source code through **conditional compilation flags** and **hostile mutant tests**:
- **Attributes:** Deliberate mathematical faults are injected directly into the production code using feature gates like `#[cfg(feature = "mutant_1")]`, `#[cfg(not(feature = "mutant_7"))]`, etc. (found extensively in `bcinr-cmca/src/allocator.rs`, `fixed.rs`, `observatory.rs`).
- **Test Names:** Manifests in tests via files like `tests/hostile_mutants.rs` and dedicated killing functions (e.g., `kill_mutant_7_saturating_div_false_zero`) that ensure these faults yield exact typed refusals.

### 4. `@von_neumann_bypass` (Architect of Arithmetic Logic)
The implementation of branchless bounded code manifests architecturally rather than through text markers:
- **Modules & Logic:** Manifests through the total absence of standard control flow in favor of explicit branchless primitives. We see this in the crate structures (`mask.rs`, `ct.rs`, `swar.rs`, `simd.rs` in `bcinr-logic/src/`) which replace standard von Neumann branching with bit-parallel, constant-time arithmetic logic.
