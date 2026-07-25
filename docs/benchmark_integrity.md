# Benchmark Theater Prevention (CHEAT-008)

## Overview
In the BCINR deterministic substrate, **Rule 16 (`CHEAT-008` Benchmark theater)** defines Benchmark Theater as:
> *"Benchmarking a stub, constant-folded path, dead result, or reduced problem not equivalent to production."*

Because BCINR enforces strict branchless algorithms ($CC=1$) and zero-allocation hot paths, the LLVM optimizer can aggressively constant-fold or dead-code-eliminate these operations if inputs are known or outputs are unconsumed. Benchmarking these optimized-away paths provides false metrics, bypassing the mandatory requirement for genuine evidence.

## Why Benchmark Theater is a Constitutional Violation
1. **Fabrication of Evidence (Rule 24)**: Producing fabricated verification evidence is an absolute failure. It forces the Substrate Integrity Score (SIS) to 0 and triggers the strict `MaturityScrutiny` protocol.
2. **Performance Cannot Override the Constitution (Rule 2)**: Benchmarks that rely on compiler-optimized dead paths artificially inflate performance metrics at the expense of verifying the rigorously required branchless logic.
3. **Subverting Object-Code Behavior (Rule 4)**: The codebase strictly mandates fixed bounded execution work. Benchmarking an artificial stub evaluates the object code of the stub, not the true production path.

## Benchmark Integrity Rules & Enforcement

The project prevents the compiler from optimizing away hot paths and ensures structural compliance using the following mechanisms:

### 1. Mandatory use of `core::hint::black_box`
All benchmarks evaluating authoritative runtime paths (such as within Criterion harnesses) **must** wrap their inputs and outputs in `core::hint::black_box`. 

Example required pattern:
```rust
c.bench_function("shuffle_fisher_yates_branchless_avg", |b| {
    b.iter(|| shuffle_fisher_yates_branchless(black_box(42), black_box(1337)))
});
```
This serves as a strict optimization barrier. It forces the LLVM backend to evaluate the full branchless arithmetic polynomial as if its output were genuinely utilized, preventing constant folding and dead-code elimination.

### 2. Strict AST Enforcement (`bcinr-cheat-scanner`)
To prevent developers from accidentally or maliciously forgetting `black_box`, the project enforces this rule structurally through the `bcinr-cheat-scanner` (located at `tools/bcinr-cheat-scanner/src/main.rs`).

The scanner performs AST parsing to intercept method calls to `bench_function` or `iter`. If it detects authoritative logic (functions containing `branchless` or `allocate`) being benchmarked without a `black_box` wrapping, the scanner immediately fails the build with a fatal `CHEAT-008` violation:
```rust
// CHEAT-008: BENCHMARK_THEATER (AST check for criterion bench calling functions without black_box)
if (arg_str.contains("branchless") || arg_str.contains("allocate")) && !arg_str.contains("black_box") {
    self.findings.push(format!(
        "CHEAT[CHEAT-008]: {} — benchmark theater: return value of branchless call not consumed via black_box",
        self.path.display()
    ));
}
```
Any `CHEAT[CHEAT-008]` finding instantly blocks the merge process.

### 3. Identical Production Workloads
Benchmarks must test the exact implementations used in production workloads. The inputs must reflect the actual domain bounds defined in mathematical contracts, utilizing identical feature sets, data boundaries, and parameter scopes as the authoritative hot path.
