# cargo-make Build and Verification Workflows

The `bcinr` workspace uses `cargo-make` (configured via `Makefile.toml`) to orchestrate its build, test, verification, and factory pipelines. This document details the key workflow steps ensuring the deterministic substrate requirements (like CC=1 branchless execution, artifact reproducibility, and hostile mutant resistance).

## Core CI and Build Tasks

*   **`ci` (default):** The comprehensive Continuous Integration pipeline. It runs the full suite in this order: `fmt` -> `check` -> `clippy` -> `scan-cheats` -> `contract-gate` -> `factory-verify` -> `test` -> `test-mutants` -> `audit` -> `deny`.
*   **`check` & `build`:** Uses `--workspace --all-targets --all-features` to verify and build the entire workspace (including MFW retrofit crates) centrally.
*   **`test`:** Runs tests across the workspace with `--no-fail-fast` ensuring failures in one crate (like `bcinr` E2E tests) do not prevent testing the remaining crates.
*   **`bench` & `bench-report`:** Executes performance benchmarks and generates structured markdown tables using `jq` to parse `criterion` estimates.
*   **`docs`, `audit`, `deny`, `clean`:** Standard lifecycle tasks for documentation, security vulnerabilities check, license/supply chain verification, and target cleanup.

## Linting, Contracts, and Anti-Cheat

These tasks enforce the strict determinism and project constitution (`AGENTS.md` rules):

*   **`scan-cheats`:** Runs the `bcinr-cheat-scanner` tool against the codebase to detect algorithmic cheats, evasion tactics, and anti-patterns.
*   **`contract-gate`:** Validates branchless contract compliance using the `bcinr-contract-gate` tool. Ensures public primitives are mathematically proven and adhere to $CC=1$.
*   **`lint-anti-llm` / `clippy`:** Scans `bcinr-mcp` specifically for LLM-generated stubs, hedge comments, and unverified claims using `anti-llm-cheat-lsp`. This is wired into the `clippy` task to ensure CI catches these natively, applying pedantic linting (`-D warnings`) in the process.

## GGEN Chess Factory (Manufacturing Cell)

A dedicated pipeline for manufacturing `chess-factory` stations from law (via `ontology/chess.ttl`). Do not hand-edit generated source; rely on this cell.

*   **`chess-factory-sync`:** Manufactures chess-factory stations from the law using `ggen sync`.
*   **`factory-build`:** Builds the `chess-factory` crate (`lib` + `bins`).
*   **`factory-verify`:** Runs the `contract-gate` over the generated stations and motifs to ensure CC=1 on the generated output.
*   **`factory-bench`:** A smoke benchmark that tests one `sanity_random` cell against the Elo curve.
*   **`factory-kaizen`:** Emits one `DemoteStationWeight` Kaizen recommendation derived from the benchmark curve.
*   **`factory`:** A meta-task that chains the full manufacturing lifecycle (`sync` -> `build` -> `verify` -> `bench` -> `kaizen`).

## Verification & Hostile Mutations

Deep systemic verification steps to ensure algorithmic law and architectural determinism.

*   **`test-mutants`:** Validates mutant detection via compile-time feature injection (features `mutant_1` to `mutant_11`). 
    *   *Gating Pass:* Runs each mutant strictly against its dedicated oracle test function (e.g., `kill_mutant_9_false_drift`) to avoid collateral breakage on shared baseline tests.
    *   *Diagnostic Pass:* Runs the whole `hostile_mutants.rs` suite defensively per feature to log any collateral failures.
*   **`audit-object-code`:** Disassembles the release `bcinr-cmca` artifact (`libbcinr_cmca*`) to a raw dump (`target/audit/bcinr-cmca-object-audit.txt`) using `objdump` or `otool`. This is the skeleton preparation for rigorous object-code auditing (detecting conditional jumps, loop backedges, etc.).
*   **`perf-branch-gate`:** (Linux Only) Benchmarks `bcinr-powl` using `perf stat` and fails the pipeline if the branch misprediction rate exceeds 10 basis points (0.1% of retired instructions).
*   **`verify-generated`:** Validates committed `Gamma_CMCA` artifact digests (`cmca_generated.rs` vs. `cmca_generation_manifest.json`) without invoking the quarantined Python/RDF generator. Ensures payload schema version validity and BLAKE3 hash equivalence.
*   **`package-reality-check`:** A replayable gate that tests `cargo package` limits (especially `bcinr-logic` sequencer blockers). Logs its run logic into a freshly regenerated `PACKAGE_REALITY_RECEIPT.md` artifact to provide replayable evidence without performing a destructive filesystem modification or `cargo publish`.
