# CHEAT-008: Benchmark Theater Prevention in BCINR

## Overview
In the BCINR architecture, **CHEAT-008 (Benchmark Theater)** is defined as the prohibited practice of benchmarking a stub, a constant-folded path, a dead result, or a reduced problem that is not equivalent to the true production workload. Because the BCINR deterministic substrate relies strictly on execution constraints like $CC=1$ (The Radon Law) and zero-allocation hot paths, benchmarking artificial or dead paths undermines the project's integrity laws.

## Active Detection & Prevention Mechanisms
To enforce this law, the architecture employs the **`bcinr-cheat-scanner`**, a mandatory structural AST (Abstract Syntax Tree) verification tool that detects benchmark evasion strategies.

According to `docs/bcinr-cheat-scanner-spec.md` and the scanner's source code, the system actively detects benchmark theater as follows:

1. **AST Traversal**: The scanner uses the `syn` crate to parse the full syntax tree of the repository, looking specifically at `Expr::MethodCall` expressions.
2. **Criterion Framework Hooking**: It flags instances where standard Criterion benchmarking methods, specifically `.bench_function(...)` or `.iter(...)`, are invoked.
3. **Black Box Enforcement**: If the benchmark closure invokes authoritative operations (by detecting identifiers like `branchless` or `allocate`), the scanner mandates that the result must be passed into `core::hint::black_box`. 
4. **Validation Failure**: If `black_box` is missing, LLVM optimization passes might constant-fold the branchless math or eliminate it entirely as dead code. The scanner flags this failure and strictly blocks the commit/merge process.

### Implementation Logic
The exact detection block residing in `tools/bcinr-cheat-scanner/src/main.rs` demonstrates this enforcement:

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

## Remediation
When `CHEAT[CHEAT-008]` is detected, the pipeline fails immediately. The mandated remediation requires the developer to:
> "Feed benchmark outputs into `core::hint::black_box` to prevent compiler optimization."

Additionally, the overarching specifications in `docs/bcinr-cheat-scanner-spec.md` demand that benchmarks use the identical feature sets and parameter scopes defined in production, preventing benchmark subsets from bypassing full rigorous workloads.
