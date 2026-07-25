# bcinr-cheat-scanner: Enforcing the Anti-Cheat Manifesto

The `bcinr-cheat-scanner` is a rigorous compliance tool within the BCINR deterministic substrate. Its primary mandate is to automatically enforce the **Anti-Cheat Manifesto** (Rule 16 in `AGENTS.md`) and other constitutional laws (e.g., Rule 7, Rule 10, Rule 18). It guarantees that the mathematical rigor and zero-branching laws of the project are not subverted through obfuscation, testing theater, or artificial complexity.

The scanner runs during the `cargo make scan-cheats` pipeline step and fails the build immediately if any violations are found.

## How it Works

The scanner (`tools/bcinr-cheat-scanner/src/main.rs`) operates across multiple analytical layers:
1. **AST Analysis**: Parses Rust code into an Abstract Syntax Tree using the `syn` crate to logically inspect program structure, expressions, and macro definitions.
2. **Text/String Analysis**: Scans source code text, doc comments, and test files for banned literals, mock claims, and required invariant checks.
3. **Call-Graph & Metadata Analysis**: Inspects the workspace dependency tree using `cargo metadata` to catch violations hidden in transitive dependencies.

## Enforced Rules and Mechanisms

Here is how the scanner implements detection for specific cheats outlined in the manifesto:

### 1. Structural and Logic Cheats
*   **CHEAT-001 (Self-Canceling Operations)**: Uses AST traversal to detect binary expressions that logically cancel themselves out (e.g., `A ^ A`, `A - A`, or `A.wrapping_add(B) ^ A`). This prevents developers from artificially inflating code complexity without altering functionality.
*   **CHEAT-002 (Circular Oracles)**: Prevents mathematical oracles from simply being copies of the production code. The scanner stringifies the AST bodies of reference functions (ending in `_oracle` or `_reference`) and compares them against their production counterparts to ensure they are structurally distinct.
*   **CHEAT-020 (Mutation Before Admission)**: Uses AST inspection on critical hot-path functions (like `allocate`) to ensure persistent state fields (e.g., `weights`) are not speculatively modified before all validation and admission checks are completed.

### 2. Theater and Fake Compliance
*   **CHEAT-007 (Dead Path Compliance)**: Scans the AST for compliant, branchless dummy code hidden inside unreachable blocks (e.g., `if false { ... }`) while the true active execution path remains unlawful.
*   **CHEAT-008 (Benchmark Theater)**: Analyzes criterion benchmark closures to ensure that calls to branchless functions feed their results into `core::hint::black_box`. This prevents LLVM from optimizing away the benchmarked logic (constant folding).
*   **CHEAT-009 (Mutant Theater)**: Analyzes test files to ensure that counterfactual mutant tests don't just use weak `assert_ne!` assertions. It requires tests to verify exact, typed refusal codes (e.g., `Err(StabilityRefusal::)`, `.is_refused()`) or explicitly cite a named violated postcondition.
*   **CHEAT-031 (Black Box Branchlessness Claim)**: Detects invalid textual claims in documentation asserting that `core::hint::black_box` *guarantees* machine-level branchlessness. The constitution acknowledges that LLVM optimization passes can rewrite bitwise logic back into branches; therefore, only object-code audits (disassembly) can prove true branchlessness.

### 3. Evasion and Obfuscation
*   **CHEAT-003 (Magic Constants)**: Scans both the AST and text for unexplained hexadecimal literals (e.g., `0xDEADBEEF`, `0xCAFEBABE`) attempting to bypass certified and derived configuration constants.
*   **CHEAT-006 (Scanner Evasion)**: Inspects `macro_rules!` definitions to ensure `if` or `match` statements are not hidden inside macro expansions in an attempt to evade structural AST branch-detection.
*   **CHEAT-010 (Gate Jurisdiction Theater)**: The scanner literally *reads its own source code text* to verify that its search roots have not been tampered with to ignore critical production crates like `crates/bcinr-logic` or `crates/bcinr-cmca`.
*   **CHEAT-014 (Reachable Dependency Branch)**: Evaluates the project workspace via `cargo metadata` to ensure that external transitive dependencies do not introduce conditional branches into the production call-graph.

### 4. Boilerplate and Padding
*   **CHEAT-004 (Artificial File Inflation)**: Scans file text for padding strings (e.g., `PADDING ENSURING FILE LENGTH REQUIREMENT`) or blocks of consecutive dummy numbered comments added to meet line-count metrics.
*   **CHEAT-005 (Boilerplate Verification Claims)**: Scans for repetitive, mock "Hoare-logic Verification" comments that lack real proof linkages.
*   **CHEAT-021 (Rejection State Drift)**: Uses text analysis to ensure that any test suite containing `case_studies.rs` also includes a `test_rejection_invariance` harness. This proves that any transaction yielding a typed refusal leaves the persistent state bit-for-bit unchanged.
