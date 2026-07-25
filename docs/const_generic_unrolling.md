# Const-Generic Loop Unrolling in BCINR

In the BCINR determinism substrate, authoritative runtime paths must enforce an absolute mathematical guarantee of branchless execution, allocation-free execution, and structurally bounded runtime behavior. Traditional Rust loops (`for`, `while`, `loop`) intrinsically introduce conditional control-flow branches (for loop bounds and termination conditions) and create backward jumps ("backedges") in the compiled machine code. 

To strictly comply with **AGENTS.md Rule 8 (Absolute CC=1 Law)** and **Rule 13 (No Unbounded Execution)**, the codebase leverages const-generic macro unrolling techniques, primarily via macros like `unroll_n_static!`, `unroll_8_static!`, `unroll_k_static!`, and `unroll_q_static!`.

## Macro Anatomy

The unrolling macros work by generating a sequential, repeated expansion of the loop body. Crucially, they scope a statically defined `const` variable for each unrolled step, effectively converting the loop counter into a compile-time constant.

Here is the definition of `unroll_8_static!` from `crates/bcinr-cmca/src/allocator.rs`:

```rust
#[macro_export]
macro_rules! unroll_8_static {
    ($var:ident, $body:expr) => {{
        {
            const $var: usize = 0;
            $body
        }
        {
            const $var: usize = 1;
            $body
        }
        // ... continues through 7
        {
            const $var: usize = 7;
            $body
        }
    }};
}
```

## How It Eradicates Counters and Backedges

These macros physically eradicate loop counters and backedges in the release object code through several compounding compiler optimizations:

1. **Loop Counters Become Compile-Time Constants:**
   The traditional loop iterator variable (e.g., `let mut i = 0;`) is replaced by isolated `const i: usize = N;` definitions across separate block scopes. The loop index is no longer a runtime variable stored in a register; it is statically known for each serialized block.

2. **Complete Elision of Backedges:**
   In object code, a loop is typically represented by a forward block that conditionally jumps back up the execution stream (a loop backedge) if the termination condition hasn't been met. Because the macro expands the body in a straight line `$body, $body, $body...`, the execution flows continuously downwards. The LLVM IR has no backward jumps to track.

3. **Total Constant Folding and Bounds Check Elimination:**
   By knowing the exact value of the iterator `const $var` at every step, the compiler can fold indexing expressions directly.
   
   Take this example from `allocator.rs`:
   ```rust
   unroll_8_static!(x, res[x & 7] = flat_alloc[x & 7] + alloc_flow[x & 7]);
   ```
   During compilation, the compiler parses `res[0 & 7]`, `res[1 & 7]`, up to `res[7 & 7]`. Since the array bounds are statically matched (e.g., arrays of fixed length `N=8`), LLVM can entirely optimize away all bounds-checking panic paths. This ensures compliance with the ban on `panic` code paths.

## Nested Unrolling for Multidimensional Arrays

The macro allows for effortless Cartesian expansion of multi-dimensional unrolling without losing the `CC=1` guarantee. 

For instance, computing a boolean matrix matching operation in `allocator.rs` looks like this:

```rust
let mut is_leaf = [true; N];
unroll_8_static!(i, {
    unroll_8_static!(j, {
        let is_match = parent[j & 7] == i as i32;
        is_leaf[i & 7] &= !is_match;
    });
});
```

Here, the nested loop generates 64 distinct blocks (8x8). There are zero `if` statements and zero `for` loops. The variables `i` and `j` are purely structural guides to the compiler. The compiler flattens this perfectly into 64 straight-line assignments with hardcoded offsets, executing strictly in $O(1)$ time with a static operation count.

## Summary

Through code-generation scripts and structural Rust macros, `unroll_*_static!` enforces deterministic boundaries by reshaping how the Rust compiler digests iterative math. It acts as the ultimate enforcer of the substrate's requirement for bit-parallel mechanics over byte-sequential control flow.
