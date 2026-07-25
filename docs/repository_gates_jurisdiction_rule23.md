# Rule 23 (Required repository gates) Research

According to Rule 23 in `AGENTS.md`, any change must be subjected to a strict matrix of automated gates to verify the integrity of the deterministic substrate. A "green" (successful) command is invalid unless the reporter explicitly proves that the gate had **full jurisdiction** over the changed files. To prove jurisdiction, the report must include the exact command, exit status, files inspected, features inspected, targets inspected, findings, and an artifact digest.

Here is the breakdown of how the two requested gates operate and satisfy these rules:

## 1. `cargo make contract-gate`

### Execution Mechanism
Running `cargo make contract-gate` triggers:
```bash
cargo run --manifest-path tools/bcinr-contract-gate/Cargo.toml --release --quiet
```

### Inner Workings
The `bcinr-contract-gate` tool acts as the automated Enforcer (`@turing_machine`). It parses the Rust AST of all source files in the target directories (by default, `crates/bcinr-logic` and `crates/bcinr-cmca`) using the `syn` crate. 
1. **Reachability Graph**: It identifies the `AUTHORITATIVE_ROOTS` (e.g., `allocate`, `evaluate_calibration`) and computes the full transitive call graph of reachable functions.
2. **Rule Enforcement**: For each reachable function, it enforces the Radon Law by checking that Cyclomatic Complexity ($CC$) equals exactly 1. It scans for hidden branches, forbidden operations (e.g., `unwrap`, `?`, basic math operators in bitwise contexts), and panics.
3. **U64 Contract**: It ensures every public reachable function has an explicit mathematical contract declared in its attributes.

### Proving Full Jurisdiction
Because `bcinr-contract-gate` limits its checks to the computed reachability graph stemming from predefined authoritative roots within specific directories, a changed file might successfully compile but remain uninspected if it is unreachable. 
To prove jurisdiction per Rule 23, you must explicitly cross-reference the changed files against the files actually parsed and reached by the tool. The report must demonstrate that the modifications were traversed in the call graph and evaluated for $CC=1$, proving the tool had full jurisdiction over the new logic.

---

## 2. `cargo make verify-generated`

### Execution Mechanism
Running `cargo make verify-generated` executes an inline bash script defined in `Makefile.toml`. 

### Inner Workings
Historically, this gate verified reproducibility by running a Python RDF generator and diffing the output. However, to respect a strict artifact boundary (`bcinr-cmca` is now a consumer-only crate), the generator has been quarantined.
The modernized gate now performs a consumer-side digest verification:
1. It loops over specifically hardcoded artifact directories: `crates/bcinr-cmca/generated-artifact/case-studies` and `generalization`.
2. It parses the `cmca_generation_manifest.json` file in each directory to extract the `schema_version` and the `generated_payload_digest`.
3. It validates that the schema version is supported (e.g., version `'1'`).
4. It recomputes the BLAKE3 hash of the local `cmca_generated.rs` payload using `b3sum`.
5. It fails the gate if the recomputed hash does not perfectly match the manifest's declared digest.

### Proving Full Jurisdiction
Because this task's bash script hardcodes the paths it checks, any newly added generated artifact directories would silently bypass this gate. 
To prove full jurisdiction, the report must enumerate the specific `.rs` and `.json` files that were evaluated by the script. If your changes involve generating a new module, you must prove that the new directory was added to the bash script's loop, ensuring the gate actively inspected the newly introduced generated files.
