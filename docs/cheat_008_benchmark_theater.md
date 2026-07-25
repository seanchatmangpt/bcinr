# CHEAT-008: Benchmark Theater

## Overview

In the `bcinr` deterministic substrate, **CHEAT-008 (Benchmark Theater)** is a strict anti-cheat violation defined as the practice of benchmarking a stub, a constant-folded path, a dead result, or a reduced problem that is not equivalent to the true production workload. 

Benchmark theater creates the illusion of exceptional performance by measuring code paths that the compiler has optimized away entirely, rather than measuring the actual computational cost of the complex, branchless arithmetic polynomials required by the substrate.

### What Constitutes Benchmark Theater?
- **Constant-Folding / Dead Results**: Benchmarking an operation without explicitly consuming its result. Because the substrate's authoritative logic operates purely on side-effect-free bitwise arithmetic, an unconsumed result allows LLVM optimization passes to entirely strip out the operations as dead code, yielding artificial benchmark times of nearly 0ns.
- **Stubs and Reduced Problems**: Benchmarking an artificially simplified version of an algorithm, or a subset of the state parameters, instead of the full mathematical domain handled in the production hot-path.

## Why is it an Evasion of Authoritative Runtime Laws?

The `bcinr` architecture is governed by strict laws, including the **Radon Law ($CC=1$)** and **zero-allocation** hot paths. All control flow must be replaced by fixed-width bitwise polynomials, masks, and fixed table lookups. This transformation inherently carries a baseline computational cost, as the code must perform all work required to evaluate every state outcome logically before applying the selection mask.

Benchmarking an artificial or dead path subverts these core integrity laws for the following reasons:
1. **Evades Execution Constraints Verification**: It allows an implementation to illegitimately claim it meets performance and execution bounds, when in reality, the benchmark is simply measuring the compiler's ability to delete the unutilized code. 
2. **Hides True Object Code Behavior**: The authoritative runtime laws mandate "fixed bounded execution work" and "object-code verification". If you benchmark a folded path or stub, you are verifying the object code of the stub, not the production path.
3. **Corrupts Substrate Integrity Score (SIS)**: Valid performance profiling is part of the verification matrix. Benchmark theater falsifies reproducible evidence, violating the Anti-Cheat Manifesto.

## Structuring Authoritative Benchmarks

To prove actual hot-path metrics and pass the mandatory `bcinr-cheat-scanner` checks, authoritative benchmarks must be structured according to the following rules:

### 1. Mandatory use of `core::hint::black_box`
The return values of all authoritative operations inside a Criterion benchmark closure (e.g., `.bench_function(...)` or `.iter(...)`) **must** be passed into `core::hint::black_box`. 

This acts as a strict optimization barrier. It forces the LLVM backend to evaluate the full branchless arithmetic polynomial as if its output were genuinely utilized by external state, preventing constant folding and dead-code elimination. 

The `bcinr-cheat-scanner` enforces this by structurally parsing the AST. If it detects authoritative algorithms (like `branchless` or `allocate`) being benchmarked without a wrapping `black_box`, it immediately throws a `CHEAT[CHEAT-008]` violation and blocks the merge process.

### 2. Identical Production Workloads
Benchmarks must test the exact implementation used in production. The inputs must reflect the actual domain of the mathematical contracts, and the benchmarks must be run using the identical feature sets, data boundaries, and parameter scopes as the authoritative runtime.
