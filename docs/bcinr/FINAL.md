# bcinr to unibit: The Final Transition

This document outlines the final state of the `bcinr` project and the remaining tasks to fully transition the deterministic branchless substrate to its successor, `unibit`.

## 1. The Strategic Shift

`bcinr` (Branchless Calculus in Rust) proved that complex process intelligence, conformance checking, and reinforcement learning could be executed inside a strict branchless, zero-allocation boundary. It established the $CC=1$ Radon Law, the Zero-Allocation Boundary, the K-Tier timing constitution ($T_1 \le 200 \text{ns}$), and the `AutonomicSubstrate` loop.

`unibit` takes the physics proven by `bcinr` and turns it into a formal operating substrate.

### The Core Inversions:
*   **From Engine to Substrate:** `bcinr` was a deterministic process-discovery engine. `unibit` is the state physics that *manufactures* the hot path for process intelligence.
*   **From Byte-Centric to Bit-Native:** `bcinr` still used $K$-tier bitsets as a process modeling tool. `unibit` formalizes the universe as $U_{1,n} = B^n$, where one bit is one truth atom.
*   **From "No OS" to "OS Disappears":** `bcinr` minimized OS overhead. `unibit` formally separates the *visible* OS (planning, admission, capability binding) from the *invisible* OS (hot-path execution over `words + masks + scratch`).
*   **From `dteam` Monolith to Layered Contract:**
    *   `unibit`: sealed lawful bit-motion substrate.
    *   `unios`: private admission, proof, projection, and operating contract layer.
    *   `AtomVM`: real-world disorder boundary membrane.
    *   `dteam`: deterministic process intelligence over canonical facts.

## 2. What Remains to Complete `unibit`

The `unibit` proof-of-concept repository has been initialized (`unibit-nightly-smoke`). The `unrdf` code-manufacturing pipeline generates the workspace topology from ontology definitions.

To complete the transition from `bcinr` to a fully operational `unibit` substrate, the following must be implemented:

### Phase 0: The Skeleton (Completed)
*   [x] Nightly toolchain pinning.
*   [x] Ontology-driven workspace generation (`unrdf`).
*   [x] Lexicon enforcement (forbidding ordinary storage nouns like "b*te").
*   [x] `L1Region` validation (64 KiB truth + scratch geometry).
*   [x] Inline assembly smoke tests.

### Phase 1: The Sealed Substrate (`unibit`)
This phase implements the raw physics.
*   [ ] **OS Memory Locking (`mlock`):** The `unibit-smoke` binary must attempt to physically lock the `L1Region` pages in RAM and emit a position receipt.
*   [ ] **The `xtask` Enforcement Spine:**
    *   `cargo xtask prove`: The ultimate CI gate (lexicon, format, clippy, tests, compile-fail, `no_std`, dependency boundaries).
    *   `cargo xtask asm`: Assembly emission and inspection (proving no allocator, no vtable, no panic).
    *   `cargo xtask bench`: Tiered benchmark reports ($8^1$, $8^2$, $8^3$, $8^4$, $8^5$).
    *   `cargo xtask doctor`: Local machine readiness checks.
*   [ ] **Compile-Fail Tests:** `trybuild` tests to prove that `Vec`, `Box`, `String`, `dyn Trait`, `unwrap()`, and short-circuiting logic are rejected from hot paths.
*   [ ] **The `#[ubit_hot]` Attribute Macro:** To enforce timing targets, `no_alloc`, `no_panic`, and assembly-admission requirements at compile time.
*   [ ] **Substrate Crate Implementations:**
    *   `unibit-l1`: `UTruthBlock`, `UScratchpad`, `UActiveWordSet`, `USparseDelta`.
    *   `unibit-kernel`: Unsafe raw branchless kernels (scalar, SIMD, and assembly targets).
    *   `unibit-motion`: Typestate wrappers (`Raw -> Planned -> Admitted -> Hot`).
    *   `unibit-receipt`: Rolling `UReceiptFragment` implementation.

### Phase 2: The Operating Layer (`unios`)
This phase wraps `unibit` in a safe, plan-oriented interface.
*   [ ] **`UBitScopePlanner`:** Compiles process semantics into the smallest sufficient kinetic footprint ($8^n$ work tier inside a $64^m$ memory tier).
*   [ ] **`UBitCapability` & `UBitLaw`:** Admission masks for authority and policy.
*   [ ] **`UBitField`:** The 2 MiB ($64^4$) L2 meaning field for boot-time allocation and context caching.
*   [ ] **`UBitSupervisor`:** Off-hot-path observation and recovery.
*   [ ] **`UBitImage` & `DeltaTape`:** Checkpointing, replay, and exact state motion histories.

### Phase 3: The Boundary & The Engine (`AtomVM` & `dteam`)
*   [ ] **`AtomVM` Boundary Adapter:** Canonicalizes real-world disorder (lateness, missingness, retries, crashes) into explicit `CanonicalFact` objects.
*   [ ] **`dteam` Core:** Process intelligence (discovery, conformance, RL) rewritten to consume *only* `CanonicalFact` objects and delegate execution through the `ProcessSubstrate` trait to `unios`.
*   [ ] **`dteam-globe`:** The human-readable projection layer (rendering $64^n$ memory states as geographic/semantic projections).

## 3. Preparing `bcinr` 26.4.21 for Publishing

Before fully shifting focus to the `unibit` workspace, `bcinr` version 26.4.21 must be published to crates.io as the final, definitive release of the engine-era architecture.

### Publishing Checklist
1.  **Changelog Update:** Ensure `CHANGELOG.md` reflects the completion of the 359+ modules, the 100/100 Substrate Integrity Score (SIS), and the final transition architecture (UniverseOS / Vision 2030).
2.  **Documentation Synchronization:** Verify that `README.md` and the `docs/diataxis/` index accurately point to the final state of the `bcinr` API and the theoretical handoff to `unibit` (e.g., `rust-substrate-mapping.md`).
3.  **Manifest Versioning:** Ensure all `Cargo.toml` files in the workspace (`bcinr`, `bcinr-core`, `bcinr-bench`, `crates/bcinr-api`, `crates/bcinr-logic`, etc.) are bumped to `0.26.4` (or `26.4.21` if using date-based versioning) and depend on each other correctly.
4.  **License & Metadata:** Confirm `license`, `repository`, `description`, and `readme` fields are correct in all public crate manifests.
5.  **Dry Run:** Execute `cargo publish --dry-run` in the required crates to catch any packaged payload issues.
6.  **Git Tag:** Create and push a Git tag for the release (e.g., `v26.4.21`).
7.  **Publish:** Execute `cargo publish` for the appropriate crates in the correct dependency order.