# CHEAT-004: Artificial File Inflation

## Definition

According to Rule 16 of the BCINR Deterministic Substrate Constitution (`AGENTS.md`), **CHEAT-004 (Artificial file inflation)** is explicitly defined as:
> "Padding, repeated comments, generated boilerplate, or dead code added to satisfy line-count or artifact-count expectations."

Any attempt to artificially bloat the codebase through these methods is categorized as a severe violation (Cheat) of the repository's rules.

## Why is Artificial File Inflation Explicitly Banned?

In the BCINR repository, code metrics such as line-count or artifact counts are entirely irrelevant compared to mathematical correctness, bounded execution, and deterministic behavior. These inflationary practices are explicitly banned for the following reasons:

### 1. The "Contract with Teeth" and Absolute Semantic Weight
The governing principle of BCINR is "Rich semantics upstream. Fixed deterministic mechanics downstream." Every line of authoritative code must map directly to a mathematical proof or state transition (as defined by `@hoare_oracle`). Boilerplate, padding, and dead code have no formal proof obligations, lack any contractual contribution to the output, and therefore violate the core requirement that every operation must be load-bearing. 

### 2. Compromising Hostile Mutation Testing (`@armstrong_fault`)
Under Rule 19, every implementation file must have at least three syntactically plausible mutants, and every mutant must be killed by a typed refusal or independent oracle mismatch. If dead code or non-essential padding is added, mutants injected into these areas will survive. A surviving mutant changes the project standing to `MUTATION_GATE_FAILED` and blocks all feature work.

### 3. Diluting Object-Code Verification and Auditing (`@turing_machine`)
To achieve a Substrate Integrity Score (SIS) of 100, the final object code must be manually verified to contain zero conditional jumps, loop backedges, or panics (Rule 20). Dead code and boilerplate force the structural auditor to scan irrelevant syntax trees, bloating the generated intermediate representation and assembly. This makes exhaustive object-code audits practically impossible.

### 4. Masking Scanner Evasion
Inflated files provide a dense forest of text where other prohibited constructs (such as hidden branches, prohibited magic constants `CHEAT-003`, or `CHEAT-006` scanner evasion) can be concealed from reviewers and automated syntax tree parsers. 

### 5. Absolute Failure Consequence
Under Rule 24, attempting to artificially inflate files is a form of cheating. A detection by the `bcinr-cheat-scanner` (which parses the full syntax tree and normalizes comments) constitutes an absolute failure. This forces the Substrate Integrity Score (SIS) to `0`, triggering the `MaturityScrutiny` protocol that freezes feature development, quarantines the affected code, and mandates a root-cause report.

By enforcing `CHEAT-004`, the BCINR substrate ensures that every byte of source code is load-bearing, deeply audited, mathematically proven, and strictly essential to branchless deterministic execution.
