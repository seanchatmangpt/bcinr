# Research on CHEAT-004: Artificial File Inflation

Based on Rule 16 of the BCINR Deterministic Substrate Constitution (`AGENTS.md`), **CHEAT-004 (Artificial file inflation)** strictly prohibits the addition of padding, repeated comments, generated boilerplate, or dead code intended to satisfy line-count or artifact-count expectations.

## What Constitutes Artificial File Inflation?

### 1. Padding
Padding involves inserting mathematically meaningless spacing, structure, or syntactic tokens that do not contribute to the deterministic output. In BCINR, every instruction must have a fixed structural shape directly tied to a mathematical contract. Any code inserted merely to "bulk up" the file without performing necessary branchless arithmetic or fixed-width state transitions is considered padding.

### 2. Repeated Comments
These are comments that restate the same information multiple times, or re-explain boilerplate details without adding semantic value. Since the true specification lies in the Hoare contract (`@hoare_oracle`) and the executable mathematics, redundant commentary is noise. **CHEAT-005** also specifically warns against "Boilerplate verification claims"—repeated comments asserting verification without a linked proof or receipt.

### 3. Dead Code
Dead code consists of variables, functions, or operations that are unreachable or whose outcomes are never consumed. In a substrate where every operation must be accounted for and mathematically bounded, dead code violates the deterministic constraints. It introduces unverified syntax tree nodes that perform no useful work.

## Why is Artificial Inflation a Constitutional Violation?

In the BCINR framework, "Rich semantics upstream. Fixed deterministic mechanics downstream." Adding code merely to hit an arbitrary metric like line count violates multiple absolute runtime laws:

1. **Subverts Hostile Verification (`@armstrong_fault`)**: Rule 19 dictates that every implementation file must have at least three syntactically plausible mutants, and every mutant must be killed by a typed refusal or independent oracle mismatch. Dead code or padding creates safe havens where mutants might survive. A surviving mutant triggers `MUTATION_GATE_FAILED` and blocks all work.

2. **Obfuscates Structural Enforcement (`@turing_machine`)**: The `bcinr-cheat-scanner` and the object-code auditor must prove that there are no conditional branches, panic paths, or dynamic dispatches. Artificial inflation clutters the abstract syntax tree and the disassembly. It makes manual and automated verification significantly harder and can be used to mask scanner evasion attempts (CHEAT-006).

3. **Lacks Proof Obligations (`@hoare_oracle`)**: Every piece of code in BCINR must have a mathematical contract dictating its preconditions, invariants, and output boundaries. Dead code or padding has no valid mathematical contract because it makes no contractual contribution to the output (similar to CHEAT-001: Self-canceling operations).

4. **Triggers MaturityScrutiny (Rule 25)**: Any detected cheat is an absolute failure that forces the Substrate Integrity Score (SIS) to `0`. It triggers `MaturityScrutiny`, completely freezing feature development and quarantining the affected code until it is structurally repaired and recertified. 

By banning CHEAT-004, BCINR ensures that every line of code is strictly load-bearing, fully verified, and functionally indispensable to the substrate's branchless deterministic mandate.
