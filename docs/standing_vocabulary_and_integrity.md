# Standing Vocabulary and Repository Integrity in BCINR

The BCINR project enforces a mathematically bounded, branchless, and allocation-free substrate where integrity is paramount. In `AGENTS.md`, the concept of "Standing Vocabulary" (Rule 28) serves as the foundational language for tracking the deterministic health of the repository. This document details the exact terminology and the strict protocols that govern the overall integrity state.

## The Concept of Standing Vocabulary (Rule 28)

To eliminate ambiguity, BCINR mandates the use of only bounded standing labels when assessing the verification state of any primitive or feature. The accepted vocabulary includes:

- **`PROVEN`**: A specific theorem is machine-checked or exhaustively established over its declared domain.
- **`INVARIANT`**: True by construction or type exclusion.
- **`ALIVE`**: The implementation executes and passes all declared gates in the pinned environment.
- **`SOURCE_BRANCHLESS_PARTIAL`**: Source appears branchless, but complete object-code standing (disassembly evidence) is not established.
- **`BRANCHLESS_ALIVE`**: The authoritative call graph passes source, complexity, allocation, panic, and disassembly audits. This is the gold standard for deployed authoritative code.
- **`REPORTED_ALIVE`**: An agent reports success, but independent reproduction has not occurred.
- **`PARTIAL_ALIVE`**: Some required gates remain incomplete.
- **`UNKNOWN`**: Evidence is insufficient.
- **`REFUSED`**: The input or configuration is outside the admitted domain.
- **`BUILD_BROKEN`**: The pinned build fails.

Crucially, **claims may not exceed their weakest load-bearing dependency.**

## Key Integrity State Modifiers

Beyond the standard vocabulary, specific fault conditions trigger repository-wide integrity locks:

### `MUTATION_GATE_FAILED` (Rule 19)
If a single syntactically plausible hostile mutant survives the test suite (meaning the mutant doesn't trigger an independent oracle mismatch or a typed refusal), the repository's standing instantly changes to `MUTATION_GATE_FAILED`. This designation exposes that the test suite is defective and immediately blocks all feature work until the mutation gap is repaired.

### `MaturityScrutiny` (Rule 25)
The overall deterministic health of the repository is tracked via the **Substrate Integrity Score (SIS)**. If the SIS falls below 100 (due to hidden branches, unwitnessed mutations, allocations, or surviving mutants), the `MaturityScrutiny` protocol is enforced. This demands:
1. Freezing all feature development.
2. Quarantining affected code.
3. Performing a root-cause report and structural repair.
4. Rerunning all structural gates, proofs, scans, and object-code disassemblies.
5. Issuing a new standing receipt before work can resume.

## Rationale: Strict Vocabulary vs. Subjective Human Descriptions

The BCINR Constitution mandates this strict vocabulary for several critical reasons:

1. **Agent Agreement is Not Evidence**: Rule 27 explicitly states that consensus holds zero weight ("Five agents repeating the same claim is still one unsupported claim"). Only bounded labels backed by explicit mechanical artifacts (`MUTANT_KILL_MATRIX.md`, `OBJECT_CODE_AUDIT.md`) provide actual standing. Subjective descriptions ("looks good", "mostly tested") attempt to bypass these rigid structural gates.
2. **Deterministic Mechanics Over Rich Semantics**: The repository architecture demands that upstream semantics are distilled into fixed deterministic mechanics downstream. Human language allows for interpretive gray areas; the substrate runtime requires absolute binary state conditions (e.g., $CC=1$, zero heap allocations).
3. **Rigid Actionability**: Bounded labels map directly to the autonomic loop and repository tooling. A status of `SOURCE_BRANCHLESS_PARTIAL` explicitly flags that object-code verification is pending, enforcing a mathematically rigorous dependency graph that loose subjective language could easily obfuscate.
4. **Anti-Cheat and Fraud Prevention**: Bounded labels prevent "verification theater" (Rule 16). By restricting the vocabulary, agents and developers are forced to align their claims exactly with the generated mechanical proofs, preventing unjustified progression in the integration pipeline.
