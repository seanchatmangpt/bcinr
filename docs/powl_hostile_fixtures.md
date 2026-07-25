# `@armstrong_fault` POWL Hostile Fixtures and Adversarial Mutants

Based on the adversarial test requirements and negative domain testing protocols in `bcinr`, here is how `@armstrong_fault` generates and manages adversarial input files aimed at breaking the POWL tape parser and compiler.

**Owner:** `@armstrong_fault` (Master of Failure Law)
**Domain:** `bcinr-powl`, `bcinr-powl-receipt`, `bcinr-pddl`
**Mandate:** A test suite that cannot find a bug in a broken implementation is itself defective. Every authoritative implementation must have at least three syntactically plausible mutants, and they must trigger bounded typed refusals rather than generic inequalities.

## 1. Methodology: The Counterfactual Mutant Protocol

`@armstrong_fault` manages adversarial files not through purely randomized fuzzing, but via structured **Counterfactual Mutants**. For the POWL compiler and scheduler, these mutants intentionally alter load-bearing mathematical laws to prove that the deterministic execution limits and concurrency constraints cannot be bypassed. 

When a mutant is introduced, the test suite must not panic or rely on `assert_ne!`. Instead, it must catch the structural failure using a Bounded Typed Refusal (e.g., `Err(StabilityRefusal::ContractionMarginInsufficient)` or `ExecutionIntegrityError`).

### Specific POWL Mutants Injected
1. **The "Guard Bypass" Mutant:** Skips the `ConcurrencyGuardTable.admits(&fired)` check. It consults the V2 Tape correctly but fires all ready operations blindly, producing an invalid state transition.
2. **The "Single-Step Reversion" Mutant:** Artificially restricts the scheduler to firing a maximum of one operation per tick (`fired.count_ones() <= 1`). This neutralizes structural concurrency while passing all sequential logic tests.
3. **The "Index Skew" Mutant:** Misaligns the `PowlNodeId` lookup by an offset of 1 during translation from V2 bitmasks to tape execution.
4. **The "Subset Inversion" Mutant (Receipts):** Modifies receipt validation to check `ready.is_subset_of(&fired)`, meaning it only passes if the scheduler fired *everything* that was ready, which artificially outlaws valid partial-scheduling strategies (where `fired ⊆ ready`).
5. **The "Self-Fulfilling Prophecy" Mutant (Receipts):** Modifies `verify_execution_receipt` to dynamically redefine `ready = fired`, trivially passing invariants.

## 2. Hostile Negative Fixtures

To ensure the POWL graph verification engines and schedulers catch illegal graphs, `@armstrong_fault` utilizes **Hostile Negative Fixtures**. These are structured test cases meticulously designed to expose edge cases in the compiler phase (`compile_powl`) and execution phase (`scheduler_tick_guarded`).

### Fixture A: The Capacity-2 Nonface (`mfw_capacity2_fixture.rs`)
Since the classical STRIPS analyzer cannot produce a genuine 3-element minimal nonface without numeric fluents, `@armstrong_fault` hand-builds `ExecutableConcurrencyComplex` fixtures (e.g., `capacity2-abc-resource-conflict`). 
- **The Setup:** A capacity-2 resource with three independent actions {A, B, C}. Every pair is jointly executable, but the full triple is not.
- **The Execution (Link 6):** The fixture feeds this into `scheduler_tick_guarded`. A preview confirms the ready set is `{A, B, C}`. The test rigorously asserts that the real scheduler + selector never fires the full triple, but rather emits a mask corresponding to exactly one of the valid pairs (`0b011`, `0b101`, or `0b110`).

### Fixture B: The Unconstrained Selector (`release_mode_fireset_gap.rs`)
A custom adversarial implementation of `ConcurrencySelector` called `AlwaysFireEverything`.
- **The Threat:** It intentionally ignores the `ConcurrencyGuardTable` entirely and returns the full `ready` set.
- **The Verification:** It verifies that `scheduler_tick_guarded` asserts the postcondition (`guards.admits(&fired)`) unconditionally. This fixture explicitly runs in **release mode** to ensure that safety gates are not conditionally compiled out by `debug_assert!`.

### Fixture C: Predecessor Constraint Violations (`chicago_tdd_integration.rs`)
To ensure OCEL 2.0 artifact receipts are tamper-evident:
- **The Setup:** A 2-operation sequence where Op 1 depends on Op 0.
- **The Adversarial Injection:** A forged execution log where Op 1 fires without Op 0 having fired.
- **The Verification:** The verifier catches the predecessor violation, proving the graph wiring (successor/predecessor masks on the POWL tape) holds up against maliciously sealed receipts.

### Fixture D: Empty Precedence & Independence Mismatches
`@armstrong_fault` feeds $N$ completely independent actions into the causal analyzer to verify the resulting `precedes` graph. If the system erroneously inserts order dependencies between pairwise-independent nodes, it is flagged as violating the rule that vector ordering alone must never create precedence without a valid witness.

## 3. Tape Parser and Graph Compiler Validation
During `compile_powl`, hostile AST inputs are supplied to trigger structural refusals.
- **XorInsideLoop Fixtures:** ASTs where an `XorChoice` node is placed inside a `Loop` body. The compiler must reject this with `CompileError::XorInsideLoop` because loop iterations could re-enable unchosen XOR branches, violating determinism.
- **Kahn Cycle & Reachability Fixtures:** ASTs with circular dependency edges in a `PartialOrder`. The two-phase validation algorithm (Kahn's Topological Sort + Bit-Parallel Transitive Closure Reachability Validation) must emit `CompileError::Cycle` or `CompileError::Unreachable`, guaranteeing that cyclic or unreachable operations cannot be allocated onto the flat 64-slot `PowlTape`.
