# The Macro Unrolling Law in BCINR

In the `bcinr` project, the **Macro Unrolling Law** is a fundamental principle enforcing deterministic, branchless execution on the authoritative Hot Path. This document outlines why runtime loops are strictly prohibited and how structural macros physically guarantee branchless assembly.

## Why Runtime `for` and `while` Loops Are Strictly Banned

According to the **Radon Law ($CC=1$)** and **AGENTS.md Rule 8 & 13**, the authoritative runtime path must preserve deterministic execution time and perfectly bounded operational bounds. Traditional Rust loops (`for`, `while`, `loop`) are banned because they inherently violate these invariants:

1. **Conditional Control-Flow Branches:** Traditional loops intrinsically require conditional branching to evaluate bounds and termination conditions. This violates the absolute $CC=1$ cyclomatic complexity requirement.
2. **Loop Backedges:** In object code, loops are typically implemented using conditional backward jumps ("backedges") that disrupt straight-line execution and introduce unpredictable execution times based on data-dependent loop termination.
3. **Panic Paths:** Variable loop boundaries often require the compiler to insert bounds-checking panic paths to ensure memory safety, which violates the strict zero-panic mandate. 
4. **Unbounded Execution Work:** A compile-time bound implemented as a runtime variable loop implies non-constant operations, which is forbidden unless structurally mitigated.

## Enforcing Branchlessness via `unroll_*_static!` Macros

To strictly comply with these mandates, `bcinr` utilizes const-generic macro unrolling techniques, most notably `unroll_8_static!` (along with variants like `unroll_4_static!` and `unroll_32_static!`).

### Macro Architecture
The unrolling macros forcefully replace traditional iterations with a sequential, repeated expansion of the loop body, manually scoping a statically defined `const` variable for each unrolled step:

```rust
macro_rules! unroll_8_static {
    ($var:ident, $body:expr) => {{
        { const $var: usize = 0; $body }
        { const $var: usize = 1; $body }
        // ... continues through 7
        { const $var: usize = 7; $body }
    }};
}
```

### Physical Eradication of Loop Backedges in Assembly

This architectural pattern physically guarantees the absence of loop backedges in the compiled machine code through three compounding compiler optimizations:

1. **Loop Counters as Compile-Time Constants:** 
   The traditional runtime iterator variable (e.g., `let mut i = 0`) is replaced by isolated `const i: usize = N` definitions across separate block scopes. The loop index is no longer a runtime variable stored in a register; it is statically known at compile time for each serialized block.

2. **Complete Elision of Backedges:**
   Because the macro expands the block sequentially into straight-line code (`$body, $body, $body...`), execution flows strictly downwards. The LLVM IR generation has no backward jumps to track or emit, making the resulting assembly physically devoid of loop backedges.

3. **Total Constant Folding & Bounds Check Elimination:**
   With the iterator available as a static constant, the compiler can directly evaluate and fold indexing expressions (e.g., `array[x & 7]`). Because array bounds are statically matched, LLVM optimizes away all bounds-checking logic, completely removing potential panic paths from the object code.

By combining `unroll_*_static!` macros with generated straight-line code, the `bcinr` substrate successfully shapes the Rust compiler's output, achieving the absolute requirement of **bit-parallel mechanics over byte-sequential control flow**.
