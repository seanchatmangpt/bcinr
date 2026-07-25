### Overview of Rule 21 Requirements
According to `AGENTS.md` and `docs/rule_21_generated_code_law.md`, generated code must bridge high-level semantics to down-stream fixed mechanics completely deterministically. 

The rule enforces a **4-step Reproducibility Process**:
`clean generation → digest output → regenerate → verify byte-identical output`

Additionally, generated authoritative code must:
1. Contain no fixture-specific identifiers or hidden branches.
2. Pass the cheat scanner (`scan-cheats`).
3. Pass `CC=1` enforcement (`contract-gate`).
4. Pass disassembly inspection.
5. Bind to source graph and certificate digests.
6. Strictly prohibit **hand-editing**.

### How Code Generation is Implemented and Verified

Based on the `Makefile.toml` tasks, `PRODUCER_REPRODUCTION.md`, and project documentation, Rule 21 is strictly enforced through several specialized CI gates:

#### 1. Byte-Identical Output and Immutable Artifacts (`verify-generated`)
For the `bcinr-cmca` crate, the code generator script (Python) has been deliberately quarantined to preserve an exact architectural boundary (the crate is consumer-only). 
To satisfy the rule, the **`verify-generated`** task in `Makefile.toml` enforces consumer-side reproducibility. 
It computes the BLAKE3 hash of the checked-in `crates/bcinr-cmca/generated-artifact/case-studies/cmca_generated.rs` (and the `generalization` variant) and compares it against the `generated_payload_digest` declared in `cmca_generation_manifest.json`. If an engineer hand-edits the generated `.rs` file, this gate instantly fails due to mismatched hashes, adhering to the "Hand-editing generated output is prohibited" law.

#### 2. Independent Reproduction Checks
In `crates/bcinr-cmca/generated-artifact/PRODUCER_REPRODUCTION.md`, independent reproduction runs prove deterministic alignment. Running the MFW CMCA producer outputs exactly byte-identical digests across independent runs for:
- `rdf_digest`
- `admission_digest`
- `generator_digest`
- `numeric_profile_digest`
- `formula_registry_digest`
- `generated_payload_digest`

This explicitly satisfies the rule's requirement to **"bind to source graph and certificate digests."**

#### 3. Continuous Integration Gates (`scan-cheats`, `contract-gate`, `audit-object-code`)
Generated code is not exempt from core project strictures. `Makefile.toml` wires generated artifacts directly into the substrate's verification loop:
- **`scan-cheats`:** Runs `bcinr-cheat-scanner` over the workspace, ensuring no generated code evades scrutiny via macro-indirection or prohibited magic constants.
- **`factory-verify`:** Runs `bcinr-contract-gate` specifically over `crates/chess-factory/src/stations` to enforce `CC=1` (cyclomatic complexity = 1, Radon Law) over the `ggen sync` (code factory) generated output.
- **`audit-object-code`:** Assembles a raw disassembly dump (`bcinr-cmca-object-audit.txt`) via `otool`/`objdump`. While manual follow-up is needed for symbol classification, it exposes the generated machine instructions so they can be audited for hidden conditional jumps, allocator calls, and loop backedges, ensuring the final artifact meets the branchless execution mandate.

### Conclusion
The project adheres to Rule 21 by shifting from arbitrary code-generation scripts running locally to a **cryptographically-bound, strict consumer verification model**. Tasks like `verify-generated` and `factory-verify` automate the detection of drift, hand-edits, or branching violations before they can merge into the `bcinr` hot path.
