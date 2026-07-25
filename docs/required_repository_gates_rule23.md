# Rule 23: Required Repository Gates

The `cargo make` CI pipeline strictly sequences six required repository gates to uphold the deterministic substrate laws. Below is a breakdown of how they are structured in `Makefile.toml` and how they verify jurisdiction.

## Pipeline Structure

1. **`cargo make scan-cheats`**
   - **Structure:** Executes the internal cheat scanner (`cargo run --manifest-path tools/bcinr-cheat-scanner/Cargo.toml --release`).
   - **Purpose:** Scans the AST to detect algorithmic cheats, prohibited operations, scanner evasion, or violation of substrate anti-patterns.

2. **`cargo make contract-gate`**
   - **Structure:** Executes the branchless contract enforcer (`cargo run --manifest-path tools/bcinr-contract-gate/Cargo.toml --release`).
   - **Purpose:** Verifies that functions strictly adhere to branchless execution (e.g., the Radon Law $CC=1$), ensuring no data-dependent conditionals exist in the authoritative code.

3. **`cargo make test-mutants`**
   - **Structure:** Iterates over all 11 recognized hostile mutation features (`mutant_1` to `mutant_11`) and runs dedicated oracle tests in isolation (`cargo test -p bcinr-cmca --features "mutant_$n" --test hostile_mutants "$t" -- --exact`).
   - **Purpose:** Ensures the implementation fails expectedly when perturbed, explicitly checking typed refusal codes or oracle mismatches without failing out on collateral damage from shared baseline tests.

4. **`cargo make audit-object-code`**
   - **Structure:** Builds a release version of the `bcinr-cmca` library and fully disassembles the artifact (`.rlib`, `.dylib`, or `.a`) into a raw dump at `target/audit/bcinr-cmca-object-audit.txt` via `otool -tv` (macOS) or `objdump -d` (Linux).
   - **Purpose:** Ensures source-level compliance results in genuinely branchless, zero-allocation object code before per-symbol classification.

5. **`cargo make verify-generated`**
   - **Structure:** Bypasses legacy generator scripts, using a bash script to directly parse `cmca_generation_manifest.json` and compute a BLAKE3 (`b3sum`) digest over `cmca_generated.rs`.
   - **Purpose:** Ensures that generated artifacts remain byte-for-byte identical to their declared cryptographic digests without inadvertently invoking network, RDF parsing, or generation scripts.

6. **`cargo make ci`**
   - **Structure:** Serves as the overarching umbrella task.
   - **Dependencies Sequence:** `fmt`, `check`, `clippy`, `scan-cheats`, `contract-gate`, `factory-verify`, `test`, `test-mutants`, `audit`, `deny`.
   - **Purpose:** Runs the comprehensive suite for formatting, linting, branchless compliance, mutation testing, and security auditing in a structured order.

## Proving Jurisdiction

To successfully satisfy Rule 23, simply running the gates and obtaining a successful exit code (green command) is **insufficient**. A gate is not fully verified unless it is proven that the test's **jurisdiction** covered the exact changed files.

Before reporting results, you must affirmatively prove that:
- The command inspected the relevant features.
- The changed files were within the scanner or gate's purview.

To complete this requirement, any final evidence report must explicitly include:
- `command`
- `exit status`
- `files inspected`
- `features inspected`
- `targets inspected`
- `findings`
- `artifact digest`
