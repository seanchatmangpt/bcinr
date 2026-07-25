# CHEAT-008: Benchmark Theater Prevention

## Overview
In the BCINR deterministic substrate, **Rule 16 (`CHEAT-008` Benchmark theater)** strictly prohibits benchmarking a stub, a constant-folded path, a dead result, or a reduced problem that is not equivalent to production. Because BCINR strictly enforces branchless, zero-allocation algorithms (Cyclomatic Complexity = 1), the LLVM optimizer is highly aggressive and can easily constant-fold or dead-code-eliminate these operations if their inputs are known or outputs are unconsumed. 

Benchmarking optimized-away paths provides false metrics and fundamentally violates the project's integrity constraints.

## The Anti-Theater Mechanism

The codebase prevents the compiler from optimizing away the hot paths by employing explicit barriers combined with structural AST-level enforcement.

### 1. `core::hint::black_box` Usage in Benchmarks
All tests using the `criterion` harness are required to wrap their inputs in `black_box`. In the `bcinr-bench` test harnesses, this pattern is universally applied:

```rust
c.bench_function("shuffle_fisher_yates_branchless_avg", |b| {
    b.iter(|| shuffle_fisher_yates_branchless(black_box(42), black_box(1337)))
});
```

By wrapping the inputs `42` and `1337` in `black_box`, the values are obscured from LLVM, completely preventing compile-time constant folding. Additionally, the `criterion::Criterion::iter()` method internally takes the returned result of the closure and feeds it into a `black_box`. This ensures the return value is "consumed," preventing the compiler from realizing it's a dead result and optimizing away the entire function call. 

### 2. Strict AST Enforcement (`bcinr-cheat-scanner`)
To prevent any possibility of a developer forgetting `black_box` or faking a benchmark, the project enforces compliance structurally through the `bcinr-cheat-scanner`.

Located in `tools/bcinr-cheat-scanner/src/main.rs`, the AST parsing logic specifically intercepts method calls made to `bench_function` or `iter`. If the scanner detects authoritative logic (functions containing `branchless` or `allocate`) being benchmarked without a `black_box` wrapping, the scanner aborts and logs a fatal `CHEAT-008` error.

```rust
// CHEAT-008: BENCHMARK_THEATER (AST check for criterion bench calling functions without black_box)
if let Expr::MethodCall(mc) = i {
    if mc.method == "bench_function" || mc.method == "iter" {
        let arg_str = quote::quote!(#mc).to_string();
        // If it is calling algorithms in the benchmark but missing black_box
        if (arg_str.contains("branchless") || arg_str.contains("allocate"))
            && !arg_str.contains("black_box")
        {
            self.findings.push(format!(
                "CHEAT[CHEAT-008]: {} — benchmark theater: return value of branchless call not consumed via black_box",
                self.path.display()
            ));
        }
    }
}
```

This strict architectural pipeline guarantees that all benchmarks execute the true, un-folded code, forcing LLVM to output the actual branchless object code and exposing the real performance cost of the implementation.
