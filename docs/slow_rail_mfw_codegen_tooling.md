# Slow Rail `mfw-codegen` Tooling in BCINR

In the `bcinr` architecture, the `mfw-codegen` tooling (part of the `mfw-meaning` / `mfw-shacl` suite) acts as the Ahead-of-Time (AOT) producer on the **Slow Rail**. Its role is to bridge the gap between unbounded, dynamic Semantic Web data (RDF/Turtle) and the heavily restricted Authoritative Hot Path governed by the Radon Law ($CC=1$).

To output strict, allocation-free C-ABI Rust logic (e.g., `cmca_generated.rs`), the generator enforces several uncompromising offline constraints to safely cross the deterministic `Gamma_CMCA` boundary:

## 1. Topological Flattening and Hardware Bitmasks
Because the Hot Path mathematically forbids variable-length queue allocations or loop-based graph traversals, all topological complexity is flattened out-of-band:
* **AOT Kahn's Algorithm:** The generator runs Kahn's Topological Sort offline to ensure the dependency graph (`cmca:dependsOn`) is purely acyclic. 
* **SWAR-Compatible Masks:** Instead of pointers or node structures, relationships are flattened into fixed-width C-ABI hardware bitmasks (e.g., `u64`). Entities are mapped to strict `pred_mask` (execution prerequisites) and `succ_mask` (downstream consequences). The runtime resolves execution blindly via $O(1)$ SIMD-Within-A-Register (SWAR) bitwise operations.

## 2. Zero-Allocation Mapping and Struct Enforcement
To guarantee zero heap allocation (`#![no_std]`) and constant-time behavior:
* **Index-Sorting:** Ontological entities are deterministically mapped to zero-indexed array offsets using explicit properties (e.g., `cmca:measureIndex`), establishing fixed sequence bounds (like `K` and `Q`).
* **Fixed-Point Arithmetic:** Floating-point parsing is explicitly banned. All numeric semantic values are verified via exact decimal arithmetic and serialized purely as exact `Q16.16` fixed-point integers.
* **C-ABI Struct Generation:** The flattened arrays, consequences tables, and static registries are emitted as static IR (`pub const` arrays) inside `#[repr(C, align(64))]` C-ABI Rust structs to ensure precise cache alignment and predictability.

## 3. `CC=1` Branchless Logic Generation
The generated logic itself is bound by the Substrate Constitution (Rule 21):
* **Loop Unrolling:** Dynamic iteration is eliminated. The generator either emits purely straight-line sequential state transitions or generates macros (e.g., `unroll_n_static!`) so the Rust compiler drops all loop backedges.
* **Strict Typed Refusals:** Missing SHACL shape conformance is refused out-of-band during the `validate` step. The generated struct perfectly maps the domain without needing fallback initialization logic or `unwrap()` calls in the hot path.

## 4. Substrate Integrity and Source Graph Binding
Finally, to prove compliance, the `mfw-codegen` tooling subjects the emitted Rust to a rigorous integrity check matrix:
* **Cryptographic Digests:** The output structurally embeds exact cryptographic hashes (`RDF_INPUT_DIGEST`, `GENERATOR_SOURCE_DIGEST`, etc.) into `cmca_generated.rs`, sealing the static output to the verified SHACL admission state.
* **AST-Level Scanner Verification:** The output is passed through the `bcinr-cheat-scanner`, parsing the generated AST to guarantee that no hidden branches, `match` statements, `?` operators, magic constants, or scanner evasion (`CHEAT-001` through `CHEAT-031`) exist in the expanded structures.
* **Object-Code Audits:** The compilation of the generated logic must physically demonstrate a Cyclomatic Complexity of 1 (`CC=1`) and absolute absence of conditional jumps or panic paths in the final release assembly.
