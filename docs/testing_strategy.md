# BCINR Testing Strategy

The `bcinr` project employs a highly rigorous, requirement-driven testing infrastructure designed to enforce its deterministic, allocation-free, and branchless constraints. The testing strategy prioritizes strict verification of the codebase against interface contracts at all layers (Petri, YAWL, POWL, WASM).

The ultimate goal of the test suite is to maintain a **Substrate Integrity Score (SIS) of 100/100**, denoting a "PhD-Verified" standing. The strategy rests on four tiers:
1. **Tier 1 (Unit & Component Tests)**: Isolated correctness and strict type assertions.
2. **Tier 2 (Boundary & Corner Cases)**: Extreme inputs and graceful failures (no panics).
3. **Tier 3 (Cross-Feature & Differential)**: Behavioral parity against a reference model.
4. **Tier 4 (Real-World & Mutation)**: End-to-end integration and adversarial mutant survival.

---

## 1. Unit, Boundary, and Differential Testing

### Unit and Boundary Verification
Standard unit tests verify individual modules (such as Petri structures and YAWL state masks). Boundary testing focuses on extreme inputs (e.g., maximum bounds, empty nets). A critical invariant enforced here is that edge inputs must fail gracefully with structured error codes—unwinding or panicking is strictly prohibited in the hot path.

### Differential Testing
Because the production code relies on heavily optimized, bitwise, branchless execution paths, it is verified against a clean, readable, *branching* reference implementation (an "Oracle"). 
- The test runner feeds generated traces and event logs to both the production codebase and the reference models. 
- Any discrepancy between the models (e.g., in token counts or state marking updates) constitutes a test failure. 
- The reference model must be mathematically equivalent but structurally independent to prevent "circular oracles" (`CHEAT-002`).

---

## 2. Adversarial Mutants (Armstrong Fault)

Adversarial mutation testing is overseen by the `@armstrong_fault` protocol (Master of Failure Law). It ensures that the test suite is actively capable of catching syntactically plausible bugs.

- **Mandatory Minimums**: Every authoritative implementation file must include at least three independent, plausible mutants (e.g., dropping a factor, bypassing a refusal mask, or using a point-estimate instead of an uncertainty bound).
- **Typed Refusal Requirement**: Tests cannot lazily check `assert_ne!(baseline, mutant)`. The test suite must assert the precise violated contract or the specific typed refusal code returned. For instance, if a mutant ignores numeric error bounds, the test must verify that a specific bitflag (like `ObservatoryFlag::NumericallyUncertain`) is modified.
- **Zero-Tolerance Protocol**: A single surviving mutant results in `MUTATION_GATE_FAILED` standing, halting all feature development until the defect in the test suite or implementation is resolved.

---

## 3. Cheat Scanning (`bcinr-cheat-scanner`)

Cheat scanning is enforced by the `@turing_machine` protocol to guarantee strict determinism, cyclomatic complexity of exactly 1 (`CC=1`), and the absence of LLVM-induced branching. It operates on two crucial boundaries:

### Source-Level vs. Machine-Code Branchlessness
1. **Source AST Gates**: Syntax tree analysis traverses the production codebase to ensure no semantic control-flow constructs exist (`if`, `match`, `while`, `?`, `unwrap`, etc.). 
2. **Object-Code Audits**: Because LLVM optimizations can rewrite branchless source logic into branches, `bcinr` inspects the *disassembled binary object code* for specific targets. It strictly checks for the physical absence of conditional jumps (e.g., `je`, `jne`) and loop backedges. Documentation claiming that `core::hint::black_box` guarantees branchlessness is considered a violation (`CHEAT-031`).

### Anti-Cheat Manifesto (The Rules)
The `bcinr-cheat-scanner` scans for specific rules (`CHEAT-001` through `CHEAT-031`), including:
- **`CHEAT-001` (Self-Canceling Operations)**: Operations that artificially inflate code complexity but cancel out (e.g., `a ^ a`).
- **`CHEAT-002` (Circular Oracle)**: Reference models that are copy-pasted from the production implementation.
- **`CHEAT-006` (Scanner Evasion)**: Using macros or line-splits to hide forbidden control flow from the AST scanner.
- **`CHEAT-008` (Benchmark Theater)**: Benchmarking dead or constant-folded paths.
- **`CHEAT-010` (Gate-Jurisdiction Theater)**: Misconfiguring the scanner to ignore authoritative target directories.
- **`CHEAT-021` (Rejection State Drift)**: Allowing persistent state to modify its bytes during a rejected transaction.

**Execution Pipeline**: 
1. `cargo make scan-cheats` runs the AST/text analysis.
2. `cargo make contract-gate` verifies `CC=1` across targets and audits the final release object code.
