# CHEAT-004: Artificial File Inflation in BCINR

According to Rule 16 of the BCINR Deterministic Substrate Constitution (`AGENTS.md`) and the project's documentation, **CHEAT-004 (Artificial file inflation)** is explicitly defined as: 
> "Padding, repeated comments, generated boilerplate, or dead code added to satisfy line-count or artifact-count expectations."

## Why is Artificial File Inflation Banned?

In the BCINR repository, standard software metrics like line count or artifact size are entirely irrelevant. The repository emphasizes mathematical correctness, bounded execution, and deterministic branchless behavior. Padding and dead code violate the substrate's core principles for several reasons:

1. **Violates Absolute Semantic Weight:** The governing principle of BCINR is "Rich semantics upstream. Fixed deterministic mechanics downstream." Every line of authoritative code must map directly to a mathematical proof or state transition. Code that does not contribute to the final fixed-width arithmetic logic or Hoare contract postcondition is considered illegal. 
2. **Compromises Hostile Mutation Testing (`@armstrong_fault`):** Every implementation must kill at least three syntactically plausible mutants via a typed refusal or independent oracle mismatch. Dead code or non-essential padding provides "safe havens" where mutants can survive. A surviving mutant triggers a `MUTATION_GATE_FAILED` state, blocking all feature work.
3. **Dilutes Object-Code Verification (`@turing_machine`):** BCINR requires a Substrate Integrity Score (SIS) of 100, which includes exhaustive, manual object-code audits to prove the absence of conditional jumps, loop backedges, and panics. Boilerplate and dead code bloat the intermediate representation (IR) and assembly code, making these manual audits computationally and practically impossible.
4. **Obfuscates Malicious Code:** Inflated files provide a dense forest of text where other prohibited constructs—such as hidden branches, prohibited magic constants (`CHEAT-003`), or scanner evasion (`CHEAT-006`)—can be concealed from automated parsers and human reviewers.
5. **Creates False Scale:** It fabricates a false sense of complexity or productivity without adding mathematical or operational value.

## How the Substrate Strictly Audits and Penalizes CHEAT-004

The substrate enforces anti-cheat rules programmatically via the `bcinr-cheat-scanner`, a rigorous Rust-based tool that parses both the textual layer and the full Abstract Syntax Tree (AST) of the repository.

### Detection Mechanisms
The `bcinr-cheat-scanner` (`tools/bcinr-cheat-scanner/src/main.rs`) performs explicit text-based checks to catch inflation:
- It flags any files containing explicit padding strings (e.g., `"PADDING ENSURING FILE LENGTH REQUIREMENT"`).
- It tracks consecutive commented lines. If it detects blocks of 5 or more consecutively numbered padding comments (e.g., matching `". Line"` within `//` comments), it flags a numbered padding block violation.
- *Related:* It also scans for **CHEAT-005** (Boilerplate Verification Claims), triggering a failure if it finds 5 or more fake proof assertions like `"Hoare-logic Verification Line"`.

### Penalties and Consequences
Because BCINR treats artificial inflation as a deliberate attempt to subvert verification, the penalties are absolute:
1. **Merge Blocking:** The `bcinr-cheat-scanner` runs as a mandatory repository gate (`cargo make scan-cheats`). Any `CHEAT-004` finding instantly halts the pipeline and blocks the merge.
2. **SIS Zeroing:** A detected cheat constitutes an absolute failure, instantly forcing the **Substrate Integrity Score (SIS) to 0**, regardless of any weighted averages of other correct code.
3. **MaturityScrutiny Protocol:** Once the SIS hits 0, it automatically triggers the `MaturityScrutiny` protocol (Rule 25). This forces a complete freeze on feature development, quarantines the affected code, mandates a root-cause report, requires manual repair of the structural defect, and forces a full regeneration of all dependent artifacts and standing receipts.
