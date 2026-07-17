# bcinr-cheat-scanner Rule Specification

This document defines the comprehensive rule specification for the `bcinr-cheat-scanner` and `bcinr-contract-gate` tools on the BCINR deterministic systems substrate.

## 1. Differentiating Source-Level and Machine-Code Branchlessness

Branchlessness must be enforced and verified at two distinct boundaries:
1. **Source-Level Branchlessness**: The absence of semantic control-flow constructs in the source code (AST), such as `if`, `match`, `while`, `loop`, `for`, `?`, `unwrap`, `expect`, `unwrap_or_else`, etc. This is verified using syntax tree analysis (AST traversal) on production code.
2. **Machine-Code Branchlessness**: The physical absence of data-dependent conditional branches (e.g. `b.eq`, `b.ne` on ARM64, `je`, `jne` on x86_64) in the compiled object code. 

### LLVM Re-branching Correction
No source-level Rust construct (including `core::hint::black_box`, bitwise operations, or mathematical masks) can guarantee that LLVM will not emit branch-based object code. LLVM's optimization passes (such as loop vectorization, branch-induction, or layout simplification) may rewrite branchless bitwise logic back into branches if it determines that branching is cheaper.
Therefore, any documentation or code comment asserting that `core::hint::black_box` or any source-level construct *guarantees* machine-level branchlessness is false and is flagged under `CHEAT-031`.

## 2. Policy Selection: Policy B
BCINR adopts **Policy B (Audited intrinsic island with portable oracles)**:
- High-performance architecture-specific primitives (e.g., SIMD shuffles, `PDEP`/`PEXT`) are isolated within audited platform-specific modules.
- Every intrinsic block must be accompanied by a portable, standard-compliant fallback oracle that is mathematically equivalent.
- The runtime automatically selects the platform-specific intrinsic or standard fallback via constant-time selection mechanisms.

## 3. Authoritative Reachability
The runtime branchlessness claim applies strictly to symbols that are transitively reachable from the designated `AUTHORITATIVE_ROOTS` (such as the `allocate` function in `bcinr-cmca`). Symbols in non-authoritative components (such as display logic, JSON serializing, and test harnesses) are part of the "slow rail" and are excluded from Radon Law object-code audits.

## 4. Rejection Invariance and Harness
For any rejected operation, the persistent state must remain bit-for-bit unchanged:
$$\operatorname{Rejected}(x) \implies \text{Bytes}(S') == \text{Bytes}(S)$$
A rejection invariance harness must test this property by calling the allocator with invalid certificates or out-of-bounds parameters and asserting that the state memory is unmodified. Any state mutation occurring during a rejected transaction is flagged under `CHEAT-021: REJECTION_STATE_DRIFT`.

## 5. Benchmark Integrity Checks
To prevent benchmark theater (`CHEAT-008`), benchmarks must:
- Consume the results of the benchmarked function via `core::hint::black_box` to prevent LLVM from constant-folding the operation.
- Use identical feature sets and parameter scopes as those defined in production.

---

## 6. Schema and Conformance Layers

### Conformance Layers
1. **Source AST Gates**: Verifies syntax compliance and scans for forbidden keywords/expressions in code.
2. **Generated-Source Gates**: Verifies that generated code remains reproducible and passes all AST scans.
3. **MIR and Call-Graph Gates**: Audits compile-time MIR output for unreachable blocks and calls to panic symbols.
4. **Behavioral Hostile Gates**: Runs counterfactual tests with hostile mutants to confirm the test suite kills them.
5. **Object-Code Gates**: Disassembles target binaries and verifies zero conditional jumps or loop backedges.

### CheatRule Schema
```rust
pub struct CheatRule {
    pub id: String,
    pub title: String,
    pub constitutional_clause: String,
    pub severity: String,
    pub layers: Vec<String>,
    pub authoritative_only: bool,
    pub detection_contract: String,
    pub required_fixture_ids: Vec<String>,
    pub required_mutant_ids: Vec<String>,
    pub remediation_code: String,
}
```

---

## 7. Constitutional Rules CHEAT-001 through CHEAT-031

### CHEAT-001: SELF_CANCELING_OPERATIONS
- **Constitutional Clause**: Rule 16 (Anti-cheat manifesto: CHEAT-001)
- **Layer**: AST, Text
- **Description**: Forbidden self-canceling expressions, e.g. `a.wrapping_add(b) ^ a` or `(x) ^ (x)`, introduced to artificially inflate code complexity.
- **Detection**: Parse AST and detect binary expressions where LHS and RHS operands are structurally identical and canceled out by the operator.

### CHEAT-002: CIRCULAR_ORACLE
- **Constitutional Clause**: Rule 16 (Anti-cheat manifesto: CHEAT-002)
- **Layer**: AST
- **Description**: Reference oracles copied from the production implementation.
- **Detection**: Check if reference functions ending in `_reference` or `_oracle` have bodies structurally identical to their production equivalents.

### CHEAT-003: MAGIC_CONSTANTS
- **Constitutional Clause**: Rule 16 (Anti-cheat manifesto: CHEAT-003)
- **Layer**: Text, AST
- **Description**: Forbidden literals (e.g. `0xDEADBEEF`, `0xCAFEBABE`) controlling production logic.
- **Detection**: Scan source and check for occurrences of magic constants outside test modules or public documentation comments.

### CHEAT-004: ARTIFICIAL_FILE_INFLATION
- **Constitutional Clause**: Rule 16 (Anti-cheat manifesto: CHEAT-004)
- **Layer**: Text
- **Description**: Dead code, comments, or boilerplate added to inflate file lines.
- **Detection**: Scan for lines containing "PADDING ENSURING FILE LENGTH" or 5+ consecutive lines matching `// N. Line N`.

### CHEAT-005: BOILERPLATE_VERIFICATION_CLAIMS
- **Constitutional Clause**: Rule 16 (Anti-cheat manifesto: CHEAT-005)
- **Layer**: Text
- **Description**: Repeated comments asserting verification without a linked proof.
- **Detection**: Scan for 5+ identical comments asserting "Hoare-logic Verification".

### CHEAT-006: SCANNER_EVASION
- **Constitutional Clause**: Rule 16 (Anti-cheat manifesto: CHEAT-006)
- **Layer**: AST, Text
- **Description**: Use of macros or formatting to hide branching or cheating patterns.
- **Detection**: Flag nested macro definitions or operators split across lines in a suspicious manner.

### CHEAT-007: DEAD_PATH_COMPLIANCE
- **Constitutional Clause**: Rule 16 (Anti-cheat manifesto: CHEAT-007)
- **Layer**: AST, MIR
- **Description**: Compliant code placed in unreachable dead paths.
- **Detection**: Scan for unused functions or blocks containing compliant stubs while the active code branches.

### CHEAT-008: BENCHMARK_THEATER
- **Constitutional Clause**: Rule 16 (Anti-cheat manifesto: CHEAT-008)
- **Layer**: AST
- **Description**: Benchmarking constant-folded paths or stubs.
- **Detection**: Check that all benchmark functions pass outputs to `black_box`.

### CHEAT-009: MUTANT_THEATER
- **Constitutional Clause**: Rule 16 (Anti-cheat manifesto: CHEAT-009)
- **Layer**: AST
- **Description**: Trivial mutants or mutants checked only with weak assertions.
- **Detection**: Verify that mutant test cases assert specific typed refusals rather than just `assert_ne!`.

### CHEAT-010: GATE_JURISDICTION_THEATER
- **Constitutional Clause**: Rule 16 (Anti-cheat manifesto: CHEAT-010)
- **Layer**: Text
- **Description**: Scanner configured to ignore target crates or source files.
- **Detection**: Verify scanner configurations target `crates/bcinr-logic` and `crates/bcinr-cmca`.

### CHEAT-014: REACHABLE_DEPENDENCY_BRANCH
- **Constitutional Clause**: Rule 7 (Whole-call-graph branchlessness)
- **Layer**: Call-graph, Object-code
- **Description**: Reachable dependencies containing conditional branches.
- **Detection**: Check deep transitive dependencies (via `cargo metadata`) and flag any reachable dependency containing branches.

### CHEAT-020: MUTATION_BEFORE_ADMISSION
- **Constitutional Clause**: Rule 10 (No mutation before complete admission)
- **Layer**: AST
- **Description**: Specs modifying persistent state before checking admission validity.
- **Detection**: Check functions to ensure state field assignments occur after all validity checks.

### CHEAT-021: REJECTION_STATE_DRIFT
- **Constitutional Clause**: Rule 10, Rule 18 (Typed refusals)
- **Layer**: Behavioral hostile
- **Description**: Operation rejection causing state modification.
- **Detection**: Ensure that any transaction yielding a typed refusal does not modify the persistent state bytes.

### CHEAT-031: BLACK_BOX_BRANCHLESSNESS_CLAIM
- **Constitutional Clause**: Rule 3, Rule 7
- **Layer**: Text
- **Description**: Documentation claiming `core::hint::black_box` guarantees machine-level branchlessness.
- **Detection**: Search for text/comments claiming `black_box` guarantees that LLVM will not branch.

---

## 8. Execution Sequence

The validation pipeline executes sequentially as follows:
1. `cargo make scan-cheats`:
   - Runs the AST/Text analysis on source files and generated code to check rules `CHEAT-001` through `CHEAT-031`.
   - Runs deep dependency scanning to check `CHEAT-014`.
2. `cargo make contract-gate`:
   - Verifies the Radon Law (`CC=1`) across all target modules using AST complexity checks.
   - Audits the release object code for conditional jumps and loop backedges.
   - Validates proof reproducibility and checks mutants.
