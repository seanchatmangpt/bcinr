# Standing Vocabulary in BCINR: `ALIVE` and `BRANCHLESS_ALIVE`

According to the BCINR deterministic substrate constitution defined in `AGENTS.md`, standing vocabulary labels are strictly bounded terms used to represent the objectively verified state of an implementation.

## Requirements for Standing

### `ALIVE`
**Definition:** *The implementation executes and passes all declared gates in the pinned environment.*
**Requirements:**
- The code must successfully compile and pass all automated repository gates (e.g., `contract-gate`, `ci`, `test-mutants`, `verify-generated`).
- It represents functional completeness—the implementation works, satisfies its mathematical oracle, and kills plausible mutants in the pinned build environment.

### `BRANCHLESS_ALIVE`
**Definition:** *The authoritative call graph passes source, complexity, allocation, panic, and disassembly audits.*
**Requirements:**
To achieve this ultimate standing, an implementation must not only work (`ALIVE`), but definitively prove its structural compliance across its *entire transitive call graph*:
1. **Source-level Enforcement (Checkpoint 5):**
   - AST scanners verify there is no hidden control flow (e.g., no `unwrap`, `?`, hidden trait branches, or early returns).
   - Absolute `CC=1` (Cyclomatic Complexity of 1) across the authoritative function.
   - Zero heap allocation and absolutely no panic paths.
2. **Object-code Verification (Checkpoint 7 & Rule 20):**
   - Source-level compliance is considered "necessary but insufficient."
   - The compiled release artifact must undergo an exact production-profile disassembly audit.
   - The final disassembly evidence (receipt) must definitively prove: zero conditional jumps, zero loop backedges, no indirect calls, and no unexpected runtime library calls.

## Why is this Strict Vocabulary Mandated?

1. **The Core Deterministic Mission (Rule 1):** 
   BCINR is built to be a deterministic computational substrate. The authoritative runtime must absolutely preserve the pipeline: `admitted input → fixed instruction shape → deterministic output`. 

2. **Eradicating Subjectivity & Apparent Correctness (Rules 31 & 33):**
   The constitution bans subjective, unverified claims like *"looks correct"*, *"should be branchless"*, *"likely optimized"*, or *"appears safe"*. As Rule 33 states: *"No agent may trade structural truth for apparent progress."* A passing test suite alone is not sufficient to claim a function is branchless.

3. **Mechanical Evidence over Consensus (Rule 27):**
   *"Five agents repeating the same claim is still one unsupported claim."* Bounded labels prevent self-certification. A standing like `BRANCHLESS_ALIVE` can only be claimed if there is a linked mechanical artifact (such as an `OBJECT_CODE_AUDIT.md` receipt containing the exact disassembly matrix) backing it up.

4. **Weakest-Dependency Bound (Rule 28):**
   *"Claims may not exceed their weakest load-bearing dependency."* By forcing strict vocabulary, the system can objectively compute the integrity of a full call graph by finding the lowest standing label among its dependencies. If a single private helper is only `ALIVE` and not `BRANCHLESS_ALIVE`, the parent authoritative function cannot claim branchless standing.
