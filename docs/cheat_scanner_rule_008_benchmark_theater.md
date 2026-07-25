Here is the documentation on how `bcinr-cheat-scanner` detects `CHEAT-008` (Benchmark Theater), based on the source code in `tools/bcinr-cheat-scanner/src/main.rs`:

### CHEAT-008: Benchmark Theater Detection

According to Rule 16 of the BCINR Constitution (`AGENTS.md`), **CHEAT-008 (Benchmark Theater)** prohibits benchmarking a stub, constant-folded path, dead result, or reduced problem not equivalent to production. 

The primary vector for this cheat is allowing the compiler (LLVM) to optimize away branchless or allocating algorithms during a Criterion benchmark because the outputs are not consumed, resulting in artificially fast but dead/invalid measurements.

#### How It Works in the AST Scanner

The `bcinr-cheat-scanner` enforces this by structurally parsing the Rust Abstract Syntax Tree (AST) using the `syn` crate. Specifically, it inspects method calls to ensure optimization barriers are used:

1.  **Target Method Calls**: In the `visit_expr` method, it specifically looks for method calls (`Expr::MethodCall`) named `bench_function` or `iter`. These are the standard methods used by the `criterion` benchmarking library.
2.  **Stringified Inspection**: It converts the arguments of these method calls into a string representation using `quote::quote!(#mc).to_string()`.
3.  **Heuristic Match**: It checks if the benchmarked block is testing authoritative code by looking for the keywords `"branchless"` or `"allocate"` within the stringified arguments.
4.  **Black Box Requirement**: If it finds those keywords, it strictly requires the presence of `"black_box"` (which refers to `core::hint::black_box`) in the same expression.
5.  **Violation Trigger**: If authoritative code is called but not consumed by `black_box`, the scanner triggers a fatal violation:
    ```
    CHEAT[CHEAT-008]: <file> — benchmark theater: return value of branchless call not consumed via black_box
    ```

#### Implementation Extract

```rust
// In tools/bcinr-cheat-scanner/src/main.rs

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

**Remediation for CHEAT-008**: 
As stated in the tool's metadata, the required remediation is to "Feed benchmark outputs into `core::hint::black_box` to prevent compiler optimization," which guarantees the benchmark tests the actual production object code rather than a constant-folded dead result.
