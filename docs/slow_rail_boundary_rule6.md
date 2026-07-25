# Rule 6 (Slow Rail Boundary) Enforcement in BCINR

The BCINR deterministic substrate relies on Rule 6 to ensure that "Slow rail" operations—such as RDF parsing, SHACL validation, floating-point arithmetic, code generation, and dynamic allocation—are completely isolated from the authoritative `#![no_std]` hot path. Linkage boundary enforcement is rigorously mechanized throughout the project via the following techniques:

### 1. Cargo Dependency Isolation and `#![no_std]`
- **Authoritative Crates**: Crates on the hot path (like `bcinr-cmca` and `bcinr-core`) enforce `#![cfg_attr(not(feature = "std"), no_std)]` at their crate root. The `alloc` and `std` features are strictly opt-in and forbidden in production builds.
- **Dependency Segregation**: Slow rail tools (e.g., `tools/ggen`, `tools/bcinr-reporter`, and RDF producers) are strictly isolated into separate workspace binaries or external repositories. They are *never* declared in the `[dependencies]` array of the authoritative runtime crates, physically preventing Cargo from linking them.

### 2. Disassembly and Object Code Verification (`@turing_machine`)
Source-level `#![no_std]` isolation is considered necessary but insufficient by the project's rules. Rule 20 mandates a structural audit of the compiled output.
- **`audit-object-code` Gate**: The `Makefile.toml` runs a strict `audit-object-code` gate which shells out to system disassemblers (`objdump` or `otool`) over the final compiled artifacts (e.g., `libbcinr_cmca.rlib`, `.dylib`, or `.a`).
- **Mechanical Proof**: This disassembles the object code to verify that the release artifact contains no dynamic allocator symbols, panic handlers, indirect calls, floating-point instructions, loop backedges, or unexpected conditional jumps. Any slow rail code that accidentally gets linked would instantly fail this audit.

### 3. Execution Profiling (`perf-branch-gate`)
- The workspace enforces a `perf-branch-gate` task (in `Makefile.toml`) which runs Linux `perf stat -e instructions,branch-misses` against compiled release binaries.
- This asserts that dynamic branch mispredictions represent `< 0.1%` (10 basis points) of retired instructions. If slow rail branching logic were somehow invoked from the hot path, the empirical execution profile would exceed this threshold and fail the gate.

### 4. Asynchronous Data Handoff (The Rule of Discovery)
- **Derivation vs. Verification**: Rather than being linked directly into the hot path to parse or validate data at runtime, the slow rail discovers and validates models (RDF/SHACL) asynchronously or ahead-of-time (AOT).
- **Static IR Generation**: The slow rail produces static, deterministic Rust IR (e.g., `cmca_generated.rs`) or bounded numeric parameters which are ingested by the hot path strictly for *verification* over packed bit-parallel values. This separates the logic in time and space, bypassing the need for runtime linkage entirely.
