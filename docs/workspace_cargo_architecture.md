# BCINR Workspace Feature & Dependency Architecture

Based on the analysis of `Cargo.toml` files within the `bcinr` workspace (e.g., `crates/bcinr-cmca/Cargo.toml`), the project strictly adheres to the architectural laws defined in `AGENTS.md` and `GEMINI.md`.

## 1. Feature Flags for Isolating Hostile Mutations
The `bcinr-cmca` crate employs an adversarial testing protocol driven by the `@armstrong_fault` role (Master of Failure Law). To enforce this, the `Cargo.toml` defines a series of explicit feature flags:
```toml
[features]
mutant_1 = []
mutant_2 = []
# ...
mutant_11 = []
```
- **Isolation of Faults**: These feature flags are used to deliberately inject structural faults (mutants) into the codebase to evaluate test suite adequacy.
- **Opt-in Only**: Because they are explicitly modeled as Cargo features and not enabled by `default`, the main production build path remains pristine and unaffected.
- **Test Matrix Validation**: The CI/CD pipeline selectively compiles the crate with individual mutant features enabled and verifies that the test suite mathematically fails (returning specific typed refusals as governed by the constitution). 

## 2. Strict Dependency Partitioning (Allocation-Free Hot Path)
To satisfy the "Zero-Allocation Boundary" and Radon Law (`CC=1`) which mandates `#![no_std]` and 0 heap allocations in the hot path, dependencies are fiercely partitioned.

### Runtime Dependencies
```toml
[dependencies]
bcinr-logic = { path = "../bcinr-logic", version = "26.7.17" }
```
- The production `[dependencies]` block for `bcinr-cmca` only includes `bcinr-logic`, an academic-grade branchless algorithm library. 
- This guarantees that no implicit dependencies with heap-allocation or hidden panics can infect the deterministic runtime logic.

### Development and Verification Dependencies
```toml
[dev-dependencies]
trybuild = "1.0"
proptest = "1.2.0"
blake3 = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chicago-tdd-tools = { version = "26.7.1", features = ["ocel-generation"] }
```
- Crates requiring dynamic memory, serialization (`serde`), hashing (`blake3`), or property testing (`proptest`) are strictly confined to `[dev-dependencies]`.
- This ensures non-test builds carry zero additional dependencies beyond `bcinr-logic`.

### Memory Auditing Features
```toml
[features]
alloc-gate = []
```
- The `alloc-gate` feature (dev-only and default-off) gates `src/alloc_counter.rs`, introducing a `CountingAlloc` global allocator during specific tests.
- This provides infrastructural enforcement to explicitly assert zero heap allocations across authoritative calls without tainting non-gated test binaries.

## Conclusion
The repository achieves its deterministic and bounded execution mandate by weaponizing Cargo's feature resolution to perform negative proof testing (mutants) and aggressively leveraging `dev-dependencies` to decouple the testing/allocation overhead from the `#![no_std]` core execution path.
