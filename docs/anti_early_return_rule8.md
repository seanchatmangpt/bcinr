# BCINR Rule 8: Anti-Early-Return and Branchless Result Semantics

The BCINR substrate constitution strictly enforces Rule 8 (the "Absolute `CC=1` Law"). This law prohibits data-dependent branches, explicitly banning control-flow semantics such as `if`, `match`, `while`, early `return`, `unwrap`, and the `?` (try) operator. 

Because LLVM cannot be guaranteed to optimize away source-level branches (even with constructs like `core::hint::black_box`), BCINR enforces a cyclomatic complexity of `CC=1` directly on the Abstract Syntax Tree (AST) using syntax parsers to guarantee perfectly linear code.

## 1. AST Interception of Early Returns (`?`, `unwrap`, `return`)

To rigorously enforce branchlessness, custom tools like `bcinr-contract-gate` and `bcinr-cheat-scanner` parse the Rust syntax tree using the `syn` crate. They utilize a `syn::visit::Visit` trait implementation to traverse the AST and intercept branching structures:

* **Intercepting `?` (Try Operator):** 
  The scanner explicitly matches against `syn::Expr::Try(_)`. When encountered, it instantly increments the function's complexity counter, flagging a violation.
* **Intercepting `unwrap()` and `expect()`:** 
  These are caught by matching `syn::Expr::MethodCall(mc)`. The scanner extracts the method identifier string and explicitly flags `"unwrap"`, `"expect"`, `"unwrap_or"`, and `"unwrap_or_else"` as branching violations (because they implicitly branch to a panic/fallback path).
* **Intercepting Explicit Control Flow (`return`, `break`, `if`, etc.):** 
  Constructs facilitating early returns are intercepted via AST node matching on variants like `Expr::If`, `Expr::Match`, `Expr::Loop`, `Expr::While`, and `Expr::ForLoop`. Finding any of these increments the complexity counter above `CC=1` and triggers an automated merge block.

## 2. Branchless Error Propagation (Result-like Semantics)

To comply with the strict branchless architecture, BCINR cannot "short-circuit" when encountering an error. Instead, all operations execute unconditionally, and errors are handled purely via bitwise arithmetic and constant-time array indexing.

### A. Bitwise Error Accumulation
Instead of early returning on an invalid state, functions calculate error conditions as boolean masks (0 or 1), up-casting them to `u32` and accumulating them using bitwise OR (`|`). 
```rust
let mut err = (len == 0 || len > 8) as u32;
(0..8).for_each(|i| {
    // Branchlessly accumulate faults bit-by-bit:
    err |= (!(is_digit | is_upper | is_lower) & (i < len) as u32) & 1;
});
```
For more complex domains (e.g., `RefusalSet`), methods like `.union()` or `.masked(condition)` are used to subtract or accumulate fault bits without any conditional jumps.

### B. Array Indexing for Result Construction
To return a `Result<T, E>` without invoking an `if` or `match`, BCINR constructs an array containing both possible variants (the `Err` case and the `Ok` case). It then indexes into this array using the evaluated error bitmask (`0` or `1`).
```rust
// If err == 0 (no error), index 1 (Ok) is returned.
// If err != 0, index 0 (Err) is returned.
[Err(fault_code), Ok(res)][(err == 0) as usize]
```

### C. Mask-Based State Selection
When errors are encountered mid-operation and a state rollback or fallback is necessary, the substrate never branches using `if err { ... } else { ... }`. Instead, it uses canonical masks to perform fieldwise selection across state variables:
```rust
let mask = CanonicalMask::from_lsb(condition);
NonNegativeFixed::from_parts(
    mask.select_u32(a.value_bits(), b.value_bits()),
    mask.select_faults(a.faults(), b.faults()), // Picks faults for the selected branch
)
```
This deferral to bitwise arithmetic guarantees that the processor executes both the success and error logic sequentially, finalizing the decision exclusively through fixed-width mathematical selection.
