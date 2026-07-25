Here is the documentation of the `cargo make` commands defined in `Makefile.toml` along with the exact scripts and cargo commands they execute:

### 🔍 Core Verification & Linting Tasks (Requested)

#### `cargo make scan-cheats`
- **Description:** Scans for algorithmic cheats and anti-patterns.
- **Under the hood:** Executes a standalone tool within the repository workspace via `cargo run`.
- **Command Executed:** 
  ```bash
  cargo run --manifest-path tools/bcinr-cheat-scanner/Cargo.toml --release --quiet
  ```

#### `cargo make contract-gate`
- **Description:** Validates branchless contract compliance (`CC=1` and missing panic paths).
- **Under the hood:** Executes the contract-gate tool.
- **Command Executed:**
  ```bash
  cargo run --manifest-path tools/bcinr-contract-gate/Cargo.toml --release --quiet
  ```

#### `cargo make audit-object-code`
- **Description:** Disassembles the release `bcinr-cmca` artifact to a raw dump (best-effort skeleton).
- **Under the hood:** A bash script that clears `RUSTFLAGS` (to ignore unrelated warnings) and builds `bcinr-cmca` in release mode. It then finds the built library artifact (e.g. `.rlib`, `.dylib`, `.so`) and runs an OS-specific disassembler to output a raw dump to `target/audit/bcinr-cmca-object-audit.txt`.
- **Script snippet executed:**
  ```bash
  cargo build --release -p bcinr-cmca
  # ... locates libbcinr_cmca*
  # Runs one of these depending on platform:
  otool -tv "$LIBFILE" >> "$OUT" 2>&1    # on macOS
  objdump -d "$LIBFILE" >> "$OUT" 2>&1   # on Linux
  ```

#### `cargo make verify-generated`
- **Description:** Verifies committed `Gamma_CMCA` artifact digests without invoking the Python RDF generator.
- **Under the hood:** A bash script that iterates through `case-studies` and `generalization` artifact directories in `crates/bcinr-cmca/generated-artifact/`. It uses inline Python to extract the `schema_version` and declared `generated_payload_digest` from `cmca_generation_manifest.json`. It then recomputes the BLAKE3 hash of `cmca_generated.rs` using `b3sum` and compares them to ensure integrity.
- **Script snippet executed:**
  ```bash
  # Extracts schema & digest:
  python3 -c "import json,sys; print(json.load(open(sys.argv[1]))['schema_version'])" "$MANIFEST"
  python3 -c "import json,sys; print(json.load(open(sys.argv[1]))['digests']['generated_payload_digest'])" "$MANIFEST"
  # Recomputes and compares hash:
  RECOMPUTED_DIGEST="blake3:$(b3sum --no-names "$PAYLOAD")"
  ```

### 🛡️ Other Notable Custom CI & Verification Tasks

*   **`cargo make test-mutants`**: Runs an exhaustive gating pass that selectively executes dedicated oracle tests against 11 hostile mutations. It loops over features like `mutant_1` to `mutant_11` using:
    `cargo test -p bcinr-cmca --features mutant_$n --test hostile_mutants <exact_test_name> -- --exact`
*   **`cargo make lint-anti-llm`**: Scans the `bcinr-mcp` crate for LLM-generated stubs and hedge comments by shelling out to a tool in an external repository:
    `cargo run --manifest-path /Users/sac/anti-llm-cheat-lsp/Cargo.toml --quiet --bin anti-llm-cheat-lsp -- scan --dir crates/bcinr-mcp/src`
*   **`cargo make perf-branch-gate`**: (Linux only) Compiles a benchmark for `bcinr-powl` and runs it under `perf stat` to assert that branch mispredictions account for less than 0.1% (10 basis points) of retired instructions.
*   **`cargo make package-reality-check`**: Runs a bash script (`scripts/gates/package-reality-check.sh`) and formats the output (checking packaging sequence blockers and working-tree cleanliness) into a regenerated `crates/bcinr-cmca/PACKAGE_REALITY_RECEIPT.md` artifact.
*   **`cargo make ci`** (also `cargo make default`): Runs the comprehensive pipeline: `fmt`, `check`, `clippy`, `scan-cheats`, `contract-gate`, `factory-verify`, `test`, `test-mutants`, `audit`, and `deny`.

### 🏗️ Standard Build & Test Tasks
These tasks have been explicitly configured with `workspace = false` to override `cargo-make`'s default directory-descent and enforce single, root-level invocations. They map directly to standard cargo commands:
*   **`cargo make check`**: `cargo check --workspace --all-targets --all-features`
*   **`cargo make build`**: `cargo build --workspace --release --all-features`
*   **`cargo make test`**: `cargo test --workspace --all-features --all-targets --no-fail-fast`
*   **`cargo make bench`**: `cargo bench --all-features`
*   **`cargo make fmt`**: `cargo fmt --all -- --check`
*   **`cargo make clippy`**: `cargo clippy --workspace --all-targets --all-features -- -D warnings` (runs `lint-anti-llm` as a dependency first).
*   **`cargo make docs`**: `cargo doc --workspace --all-features --no-deps --document-private-items`
