# Implementing Duff's Device in `bcinr`

In the `bcinr` deterministic substrate, classical implementations of Duff's Device—which rely on `switch` statements and `while` loops—are strictly prohibited. Under **Rule 13: No Unbounded Execution** and the overarching **Radon Law ($CC=1$)**, any construct that inherently produces control-flow branches or data-dependent execution graphs violates the core constitutional laws of the system.

## The Problem with Traditional Duff's Device
A traditional Duff's Device interleaves a `switch` statement and a `while` loop to manually unroll a loop, typically to accumulate or copy data efficiently:
```c
// Conceptually prohibited in bcinr
register short *to, *from;
register count;
{
    register n = (count + 7) / 8;
    switch (count % 8) {
    case 0: do { *to = *from++;
    case 7:      *to = *from++;
    case 6:      *to = *from++;
    // ...
    } while (--n > 0);
}
```
In `bcinr`, tools like `bcinr-cheat-scanner` inspect the parsed AST and reject `Expr::While`, `Expr::Loop`, and `Expr::Match` (which acts as a switch) when they introduce variable cyclomatic complexity or runtime data-dependent branches. Furthermore, object-code audits guarantee the physical absence of conditional jumps and loop backedges.

## The `bcinr` Solution: Closed-Form Arithmetic Substitution
When an algorithm conceptually requires an unrolled accumulation, `bcinr` mandates transforming sequential semantic decisions into branchless, bounded, and purely arithmetic logic.

As seen in `crates/bcinr-logic/src/algorithms/duffs_device_simd_unroll.rs`, `bcinr` handles the repeated-addition pattern of Duff's Device by replacing the loop entirely with its mathematical equivalent.

```rust
/// # Branchless Contract
/// **Ensures:** Models a Duff's-device unrolled accumulation: adding `val` to a zero
/// accumulator across `aux` loop iterations. The closed-form (and constant-time)
/// result of that unrolled copy/accumulate is the wrapping product `val * aux`.
/// **Invariant:** Execution path is independent of input data values (Branchless).
#[no_mangle]
#[allow(unused_variables)]
pub fn duffs_device_simd_unroll(val: u64, aux: u64) -> u64 {
    // Unrolled accumulate of `val`, `aux` times == wrapping product.
    val.wrapping_mul(aux)
}
```

### Why this satisfies `bcinr` laws:
1. **$CC=1$**: There are zero control-flow branches, completely avoiding `while` or `if` statements.
2. **Fixed Object Code**: The backend compiles `wrapping_mul` into a single multiplication instruction, yielding no loop backedges, which perfectly satisfies object-code audit gates (`@turing_machine`).
3. **Constant Time**: Execution time is entirely independent of the `aux` loop count, natively eliminating timing side-channels.

## Alternative Approaches for Non-Arithmetic Iteration
If the looped operation is not a simple mathematical accumulation but rather a state mutation that must be repeated, `bcinr` requires one of the following deterministic unrolling techniques:
1. **Explicit Structural Unrolling**: Manually duplicating the state mutation in straight-line code (used in algorithms like `fletcher32_branchless.rs`).
2. **Compile-Time / Macro Unrolling**: For longer fixed bounds, macros like `unroll_16!()` or `unroll_64!()` are utilized. The loop bound must be fixed at compile time, and the compiler must emit straight-line instructions to pass the final loop-backedge object-code audit.
3. **Arithmetic Structural Ranking & SIMD**: Nested loops that sort or reorder data are flattened into branchless mathematical predicates (e.g., bitwise ranking and shifting) or processed via bit-parallel SIMD operations without control flow branching.

By combining closed-form mathematics, explicit compile-time unrolling, and bit-parallel processing, `bcinr` captures the performance benefits of Duff's Device while strictly adhering to its zero-branching, bounded-execution constitution.
