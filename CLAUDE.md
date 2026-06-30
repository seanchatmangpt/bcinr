# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**bcinr** (BranchlessCInRust v26.6.24) is a performance-first, research-grade systems library providing a principled calculus for branchless algorithmics. It is designed for high-performance, deterministic systems where predictable latency, memory-safety, and side-channel resilience are critical requirements.

### Core Principles

- **Deterministic Latency:** All primitives are branchless ($O(1)$ constant time), eliminating pipeline stalls
- **Branchless Calculus Formalism:** Each primitive maps to a formal framework ($\mathcal{B}$-Calculus) ensuring invariant-preserving state transitions
- **Hardware-Agnostic SIMD:** SSE4.2 with portable fallbacks for ARM Neon and WebAssembly
- **Zero-Dependency Core:** The logic layer is strictly `no_std` with zero external dependencies
- **Panic-Free Design:** Memory arenas and `Result`-based contracts for numerical stability

## Workspace Structure

```
bcinr/
├── bcinr/                  # Main crate (re-exports public API)
├── bcinr-core/             # Public facade and ergonomic wrappers
├── crates/bcinr-api/       # Additional API layer
├── crates/bcinr-logic/     # Core algorithmic calculus (Mask, Int, Fix, Network, Bitset, Scan, UTF-8, Parse, DFA, Reduce, Sketch)
├── crates/bcinr-mcp/       # MCP server: 23 tools across 6 groups (PDDL, POWL, algorithms, receipts)
├── crates/bcinr-pddl/      # PDDL 3.1 planner: parsing, grounding, BFS planning
├── crates/bcinr-pddl-lsp/  # Language server for PDDL (planning as protocol)
├── crates/bcinr-powl/      # POWL runtime: workflow compilation, O(1) admission, execution
├── crates/bcinr-powl-receipt/ # Receipt verification & inspection (BLAKE3 proofs)
├── tools/                  # Utility and analysis tools
├── bcinr-bench/            # Benchmarks and performance testing
└── docs/                   # Diátaxis documentation (Tutorials, How-To, Explanations, References)
```

### Core Modules (in `crates/bcinr-logic/src/`)

| Module | Purpose |
|--------|---------|
| `abstractions/` | Memory arenas, epoch reclamation, lock-free structures, fiber management |
| `algorithms/` | 300+ optimized branchless implementations indexed by difficulty (1-300) |
| `autonomic/` | Self-monitoring and adaptive runtime behavior |
| `patterns/` | Control flow patterns and composition operators |
| `mask.rs` | Branchless conditional selection and masking primitives |
| `int.rs` | Integer operations (abs, min, max, saturating arithmetic) |
| `fix.rs` | Fixed-point arithmetic |
| `network.rs` | Benes networks and permutation networks |
| `bitset.rs` | Bit manipulation and population counting |
| `scan.rs` | Parallel prefix (scan) operations |
| `reduce.rs` | Reduction operations |
| `parse.rs` | Parsing primitives |
| `dfa.rs` | Deterministic finite automata |
| `sketch.rs` | Probabilistic sketches (HyperLogLog, Bloom filters) |
| `utf8.rs` | UTF-8 validation and classification |

## bcinr-mcp: Model Context Protocol Server

**bcinr-mcp** exposes the entire bcinr ecosystem as an MCP server with **23 tools** for use in Claude Code.

### Tool Inventory (23 total)

| Group | Tools | Purpose |
|-------|-------|---------|
| **PDDL** (7) | `pddl_parse_domain`, `pddl_parse_problem`, `pddl_plan`, `pddl_admit_domain`, `pddl_domain_info`, `pddl_temporal_plan_info`, `manufacture_world` | Planning via PDDL 3.1 (parsing, grounding, BFS search, Prolog8 admission gate, atomic manufacturing loop with BLAKE3 receipt) |
| **POWL** (5) | `powl_compile_sequence`, `powl_compile_choice`, `powl_admit_context`, `powl_capability_check`, `powl_plan_to_tape` | Workflow orchestration (AST → tape, O(1) context LUT, bitset permission checks, PDDL→POWL bridge) |
| **bcinr-core** (3) | `bcinr_library_info`, `bcinr_mask_ops`, `bcinr_powl_info` | System introspection (library overview, branchless bitset algebra, POWL runtime description) |
| **Algorithms** (6) | `utf8_validate`, `bitset_operations`, `dfa_info`, `scan_patterns`, `reduce_sequence`, `simd_string_info` | Branchless algorithms (UTF-8 validation, O(1) bitset ops, DFA capabilities, pattern scanning, reduction, SIMD throughput info) |
| **Receipts** (1) | `receipt_inspect` | Verification & inspection of POWL execution receipts (BLAKE3 proofs, goal verification) |
| **Cross-crate** (1) | `system_capabilities` | Unified capability report (all crates, tool counts, pipeline diagram) |

### Registration & Usage

**Binary:** `/Users/sac/bcinr/target/debug/bcinr-mcp` (23 tools ready)

**Registration:** Configured in `~/.claude/settings.json`
```json
{
  "mcpServers": {
    "bcinr": {
      "command": "/Users/sac/bcinr/target/debug/bcinr-mcp"
    }
  }
}
```

**How to Use:**
1. The MCP is automatically available in Claude Code sessions
2. Call any of the 23 tools to access PDDL planning, POWL workflows, and algorithms
3. Example: Use `manufacture_world` for end-to-end admission → plan → execute → receipt loop

### Integration Tests

**Location:** `crates/bcinr-mcp/tests/integration_tests.rs` (18 dynamic tests, 100% pass)

**What They Verify:**
- ✓ All 23 tools extracted from source (no hardcoding)
- ✓ Tool names, groups, and definitions consistent
- ✓ PDDL planning pipeline complete
- ✓ POWL orchestration pipeline complete
- ✓ Algorithm tools accessible
- ✓ End-to-end flow coverage
- ✓ Documentation completeness

**Run Tests:**
```bash
cargo test -p bcinr-mcp --test integration_tests
# Output: running 18 tests / test result: ok. 18 passed; 0 failed
```

### Architecture: Vision 2030 (BRCE Loop)

```
PDDL Domain + Problem
    ↓ (parse & validate)
Prolog8 Admission Gate (R ⊢ A)
    ↓ (BFS planner)
Temporal Plan
    ↓ (POWL compilation)
POWL Tape (Sequence/Choice/PartialOrder)
    ↓ (context admission)
Execution Context (Priority/Standard/Background/Quarantine)
    ↓ (capability check: bitset AND)
Branchless Execution (O(1/log n))
    ├→ utf8_validate (text input)
    ├→ bitset_operations (permission algebra)
    ├→ dfa_info (state machine)
    ├→ scan_patterns (search)
    ├→ reduce_sequence (aggregation)
    └→ simd_string_info (throughput)
    ↓ (receipt generation)
BLAKE3 Receipt (proof of work)
    ↓ (verification)
receipt_inspect → goal_reached ✓
```

**Key Properties:**
- **Zero-trust**: Admission gates + bitset capability checks
- **Deterministic**: All paths O(1) or O(log n), branchless
- **Cryptographic**: BLAKE3-chained receipts
- **Real-time**: Priority queue scheduling with SLA tokens
- **Auditable**: Full execution trace in receipts

### Changelog

See `BCINR_MCP_CHANGELOG.md` for:
- Tool definitions (v0.2.0)
- Integration history (v0.1.0 legacy)
- Quality assurance results
- Known issues and roadmap

## Build & Development Commands

All commands use `cargo make` (defined in `Makefile.toml`). The `Makefile` delegates to cargo-make.

### Workspace-Wide Commands

```bash
# Check compilation (fast, all targets)
make check          # or: cargo make check

# Build release binary (optimized)
make build          # or: cargo make build

# Run all tests
make test           # or: cargo make test

# Run library tests only (subset)
make test-lib       # or: cargo make test-lib

# Execute benchmarks
make bench          # or: cargo make bench

# Generate performance report (requires jq)
make bench-report   # or: cargo make bench-report

# Lint (clippy with -D warnings)
make clippy         # or: cargo make clippy

# Check formatting
make fmt            # or: cargo make fmt

# Security audits
cargo make audit    # cargo-audit for CVEs
cargo make deny     # license and supply chain check

# Documentation
cargo make docs     # Build HTML docs

# Clean artifacts
make clean          # or: cargo make clean
```

### Single Crate / Single Test

**Run tests in specific crate:**
```bash
cd crates/bcinr-logic
cargo test          # All tests in this crate
cargo test --lib    # Library tests only
cargo test --test name_of_test  # Specific integration test
```

**Run specific test by name:**
```bash
cargo test -p bcinr-logic test_name -- --nocapture  # with output
```

**Test bcinr-mcp (MCP server):**
```bash
cargo test -p bcinr-mcp                    # All tests
cargo test -p bcinr-mcp --test integration_tests  # MCP tools verification
cargo build -p bcinr-mcp                   # Build MCP binary
/Users/sac/bcinr/target/debug/bcinr-mcp    # Run MCP server
```

**Run benchmarks for specific algorithm family:**
```bash
cd bcinr-bench
cargo bench --bench bcinr_bench -- algorithm_family
```

### Strict Build (CI-equivalent)

```bash
RUSTFLAGS="-D warnings" cargo make all
```

This enforces:
- All targets compile
- Zero compiler warnings
- All tests pass
- Code formatted per `cargo fmt`

## Architecture & Design Patterns

### Key Design Decisions

1. **Branchless Calculus:** All primitives avoid conditional branches. Conditions become arithmetic/bitwise operations or lookup tables.
   - **Example:** `select_u32(mask, val_if_true, val_if_false)` uses a mask (0xFFFFFFFF or 0x0) to select without `if`
   - **Why:** Branch misprediction kills pipeline efficiency on modern CPUs

2. **Memory Safety:** No `unsafe` in algorithm implementations except where SIMD requires it (clearly marked, audited).

3. **Result-Based Contracts:** Functions return `Result<T, E>` for operations that may fail (clamping, saturation boundaries), never panicking in normal flow.

4. **no_std Core:** The logic layer compiles without `std` or allocator (critical for embedded and defense-in-depth use).

### Module Layers

- **Abstractions:** Low-level memory and execution primitives (arenas, epoch-based reclamation, resumable fibers)
- **Algorithms:** 300+ indexed by difficulty level (1-100, 101-200, 201-300)
- **Autonomic:** Self-monitoring (failure detection, runtime adaptation)
- **Patterns:** Composition and control flow (high-level orchestration)

### Algorithm Indexing

Algorithms are organized by difficulty level:
- **1-100:** Basic primitives (select, min/max, abs, arithmetic)
- **101-200:** SIMD operations, string processing, encoding
- **201-300:** Complex networks (Benes), specialized structures (HyperLogLog, DFA)

This allows testing and benchmarking by complexity tier.

## Test Organization

Tests are co-located with implementations (in modules, not separate test files). Key test patterns:

- **Unit tests:** In-module `#[cfg(test)]` blocks
- **Property-based tests:** Using `proptest` (see `Cargo.toml` dev-dependencies)
- **Benchmarks:** In `bcinr-bench/benches/` with Criterion harness

### Running Tests

```bash
# All tests (including benchmarks with --test mode)
make test

# Fast unit tests only
cargo test --lib --all-features

# With logging/output
RUST_LOG=debug cargo test --lib -- --nocapture

# Benchmark-mode tests (long-running)
cargo test --bench bcinr_bench
```

## Performance & Profiling

### Benchmarking

Benchmarks use [Criterion.rs](https://bheisler.github.io/criterion.rs/book/) for statistical rigor.

```bash
# Run all benchmarks
make bench

# Run specific benchmark
cd bcinr-bench && cargo bench --bench all_300_bench

# Generate human-readable report
make bench-report
```

Results stored in `target/criterion/` with HTML reports.

### Profiling Individual Kernels

```bash
# Compile with perf symbols
RUSTFLAGS="-g" cargo build --release -p bcinr-logic

# Profile with perf (Linux)
perf record --call-graph=dwarf ./target/release/algorithm_binary
perf report

# Profile with instruments (macOS)
instruments -t "System Trace" ./target/release/algorithm_binary
```

## Code Style & Standards

### Rust Edition & MSRV

- **Edition:** 2021
- **MSRV (Minimum Supported Rust Version):** 1.70
- **Linter:** Clippy with `-D warnings` (all warnings are errors)
- **Formatter:** `cargo fmt` (enforced in CI)

### Naming Conventions

- **Types:** `PascalCase` (e.g., `BumpArena`, `EpochReclamation`)
- **Functions:** `snake_case` (e.g., `select_u32`, `add_sat_u8`)
- **Constants:** `UPPER_SNAKE_CASE` (e.g., `MAX_CAPACITY`)
- **Modules:** `snake_case`

### Documentation

- **Inline Comments:** Only for WHY, not WHAT (code is self-documenting via branchless patterns)
- **Doc Comments:** Required for public APIs (`/// ...`)
- **Examples in Docs:** Provide working examples in `/// # Examples`

**Example:**
```rust
/// Branchless minimum of two u32 values.
/// Avoids branch misprediction by using bitwise arithmetic.
///
/// # Examples
/// ```
/// use bcinr_logic::int::min_u32;
/// assert_eq!(min_u32(10, 5), 5);
/// ```
pub fn min_u32(a: u32, b: u32) -> u32 {
    // ...
}
```

## Git & Commit Standards

### Branch Naming

- `feat/*` — New feature or algorithm
- `fix/*` — Bug fix
- `refactor/*` — Code improvement (no behavior change)
- `bench/*` — Benchmark or performance work
- `docs/*` — Documentation updates

### Commit Format

Conventional commits:
```
type(scope): description

type: feat, fix, refactor, docs, bench, test
scope: module name (e.g., "mask", "algorithms", "abstractions")
description: imperative, lowercase (e.g., "add branchless min for u64")
```

### Before Merging

- [ ] `make check` passes (compilation, all targets)
- [ ] `make test` passes (all tests)
- [ ] `make clippy` passes (zero warnings)
- [ ] `make fmt` passes (formatted)
- [ ] New algorithms have benchmarks in `bcinr-bench/`
- [ ] Public APIs documented with examples
- [ ] Commit messages follow conventional format

## Unsafe Code Policy

**CRITICAL: All algorithm modules (308 files) have `#![forbid(unsafe_code)]` enforced.**

Unsafe code is **strictly limited** to three files with formal verification:

| File | Unsafe Blocks | Justification |
|------|---------------|---------------|
| `src/mem.rs` | 1 | Memory arena bounds checks (proven safe) |
| `src/autonomic/packed_key_table.rs` | 1 | Type-safe byte reinterpretation (Copy trait enforces safety) |
| `src/patterns/deterministic_mpmc.rs` | 2 | Lock-free MPMC with CAS linearization (proven safe) |

**Process for adding unsafe code:**
1. Write **Hoare-logic proof** of safety (prose or formal notation)
2. Implement **precondition verification** before the unsafe block
3. Add **`// SAFETY:` comment** explaining invariants
4. Add **PhD Gate** (`// Hoare-logic Verification Line N: ...`)
5. Include **test oracle** (reference implementation or bounded model check)
6. Request formal peer review before merge

**Compile-time enforcement:**
```bash
RUSTFLAGS="-D warnings" cargo build  # Will fail if unsafe appears outside permitted files
```

See `crates/bcinr-logic/src/SAFETY.md` for full audit trail of all unsafe blocks with preconditions and proofs.

## Formal Verification & Claims

This project includes formal mathematical proofs (see `thesis.pdf` and `docs/diataxis/explanation/`).

**Key Verification Artifacts:**

- **Hoare-Logic Annotations:** Lines marked `// Hoare-logic Verification Line N: Radon Law verified.` document formal proofs
- **Mathematical Formalism:** $\mathcal{B}$-Calculus framework in thesis and documentation
- **Proof Gates:** Academic-grade validation of primitive correctness
- **PhD Gates Documentation:** `docs/diataxis/reference/phd_gates.md` — Explains gates as **formal verification anchors, not stubs**

When modifying algorithms:
1. Ensure changes preserve the formal invariants
2. Re-verify with proof tool if claim changes
3. Update documentation to reflect new proofs
4. Include verification evidence in PR

**PhD Gates are NOT stubs.** If a gate is present, it represents a **completed formal verification** via Hoare-logic + proptest oracle matching. See `phd_gates.md` for details.

## Documentation & Resources

### Diátaxis Structure

Documentation organized by purpose:
- **Tutorials:** Step-by-step implementation guides (e.g., `docs/diataxis/tutorials/`)
- **How-To:** Practical recipes (e.g., hardening, WCET bounding)
- **Explanations:** Deep dives (e.g., Branchless Calculus, architecture)
- **References:** API catalog and specifications

**Index:** `docs/diataxis/INDEX.md`

### Key Documents

- `ARCHITECTURE.md` — Design philosophy and module taxonomy
- `docs/BENCHMARKS.md` — Performance targets by algorithm family
- `README.md` — Quick start and feature overview
- `thesis.pdf` — Formal mathematical foundation

### Building Docs Locally

```bash
cargo make docs
open target/doc/bcinr_logic/index.html  # macOS
xdg-open target/doc/bcinr_logic/index.html  # Linux
```

## Common Development Tasks

### Adding a New Algorithm

1. **Create module:** `crates/bcinr-logic/src/algorithms/new_algo.rs`
2. **Write failing test first:** Unit tests in the module with `#[cfg(test)]`
3. **Implement branchlessly:** Avoid conditionals; use masks, arithmetic, or tables
4. **Add benchmark:** `bcinr-bench/benches/` with Criterion harness
5. **Document:** Public functions with examples, explain WHY branchless choice
6. **Verify formally:** Add Hoare-logic annotation if claim is safety-critical
7. **Run checks:**
   ```bash
   make check && make test && make clippy && make fmt
   ```
8. **Commit:** `feat(algorithms): add branchless implementation for ...`

### Optimizing an Existing Algorithm

1. **Profile:** Run benchmarks to establish baseline (`make bench`)
2. **Identify bottleneck:** Use perf/instruments to find cache misses, branches
3. **Implement:** Test with `cargo test --lib` after changes
4. **Verify:** Run `make bench` again, compare criterion results
5. **Commit:** `refactor(algorithms): optimize ... for X%  improvement`

### Running Tests in Different Modes

```bash
# Unit tests (fast, no benchmarks)
cargo test --lib --all-features

# All tests including integration and benches
cargo test --all-features

# With verbose output
cargo test --lib -- --nocapture --test-threads=1

# Specific test
cargo test -p bcinr-logic my_test_name

# Benchmark mode (statistical analysis)
cargo bench --bench bcinr_bench -- --baseline my_baseline
```

## Troubleshooting

### Build Fails with Warnings
The project enforces `-D warnings`. Fix warnings, don't suppress them:
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### Benchmark Runs Slowly
Benchmarks use Criterion with statistical sampling (can take minutes). To speed up development:
```bash
# Quick bench without full Criterion sampling
cargo build --release && time ./target/release/binary
```

### Tests Fail on Different Architecture
Branchless implementations are architecture-specific (SSE4.2 vs ARM). Verify:
- [ ] `#[cfg(target_arch)]` guards platform-specific code
- [ ] Fallback paths compile for all targets
- [ ] Tests pass on target platform

### Documentation Build Fails
Ensure all examples in doc comments compile:
```bash
cargo test --doc  # Tests all examples in /// comments
```

## Dependencies & Supply Chain

- **Zero runtime dependencies** in `crates/bcinr-logic/` (security-critical)
- **Dev dependencies:** criterion, proptest (testing only, not in release)
- **Supply chain check:** `cargo make deny` verifies licenses and audits

Audit policy:
```bash
cargo make audit  # Check for known CVEs
```

## Performance Targets

See `docs/BENCHMARKS.md` for:
- Latency targets (ns, μs, ms per algorithm family)
- Memory footprint limits
- Throughput expectations
- Regression testing thresholds

## Contact & Contribution

- **Repository:** GitHub (see `README.md` for URL)
- **Issues:** Bug reports and feature requests via GitHub Issues
- **License:** MIT OR Apache-2.0 (dual-licensed)

---

**Last Updated:** 2026-06-30 (bcinr-mcp v0.2.0 integrated)
**Version:** 26.6.30
**MCP Tools:** 23 across 6 groups (PDDL, POWL, core, algorithms, receipts, cross-crate)
**Test Status:** 18/18 integration tests passing (100%)
