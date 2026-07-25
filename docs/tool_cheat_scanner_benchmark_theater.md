Based on the source code in `tools/bcinr-cheat-scanner/src/main.rs`, here is how the cheat scanner detects `CHEAT-008` (Benchmark theater):

### Detection Mechanism

The scanner uses an AST-based approach (via the `syn` crate) to inspect the code. Specifically, it looks for method calls commonly used in benchmarking frameworks (like `criterion`) to ensure that outputs of algorithmic functions are being consumed properly to prevent the compiler from optimizing them away.

It triggers a violation when all of the following conditions are met within a method call expression:
1. The method being called is either `bench_function` or `iter`.
2. The stringified arguments of the method call contain the keywords `"branchless"` or `"allocate"`.
3. The stringified arguments **do not** contain the keyword `"black_box"`.

### Source Code Snippet

The logic is implemented in the `visit_expr` method of `SynCheatVisitor` (lines 289-303):

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

### Remediation

As defined in the rule metadata, the required remediation is to feed benchmark outputs into `core::hint::black_box` to prevent compiler optimizations.
