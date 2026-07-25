# Research Report: Rule 13 (No Unbounded Execution) & Loop Unrolling Architecture

Under the `bcinr` determinism rules, **Rule 13** strictly prohibits any form of unbounded or data-dependent execution (e.g., `while value > 0`, `for x in variable_slice`). The runtime must enforce a bounded $O(1)$ cyclic complexity ($CC=1$), and the final release object code must contain absolutely **no loop backedges** in authoritative symbols. 

To achieve this, the codebase completely replaces dynamic iteration loops with static, straight-line evaluation strategies.

## How `unroll_8_static!` Guarantees Strictly Unrolled Execution

In `crates/bcinr-cmca/src/allocator.rs`, `unroll_8_static!` is defined as a static macro that explicitly duplicates the provided block of code 8 times, assigning a `const` step index for each scope. 

```rust
macro_rules! unroll_8_static {
    ($var:ident, $body:expr) => {{
        { const $var: usize = 0; $body }
        { const $var: usize = 1; $body }
        // ... up to 7
    }};
}
```

**How this eliminates loop backedges:**
Because this expands at the AST layer (Rust macros), the compiler never sees a `loop` or a `for` construct. Instead, it parses 8 distinct sequential blocks of code. Since `$var` is a compile-time `const`, the LLVM optimizer can perfectly fold array indices and evaluation steps into straight-line sequences. There is no dynamic bounds checking, no branch condition, and consequently, no loop backedge emitted in the release object code.

## The Loop Unrolling Architecture (4 Strategies)

According to `docs/bounded_execution_unrolling.md` and `docs/loop_unrolling_strategies_rule13.md`, the loop unrolling architecture is composed of 4 key strategies:

### 1. Static Macro-Unrolling
- **Use Case**: Multidimensional logic blocks requiring complex iteration blocks.
- **Mechanism**: Explicit macros like `unroll_8_static!`, `unroll_9_static!`, and `unroll_32_static!` manually duplicate the block body for each step index.
- **Example**: Found extensively in `crates/bcinr-cmca/src/allocator.rs`.

### 2. Const-Generic Bounded Iteration
- **Use Case**: Generic structures requiring configurable boundaries without sacrificing determinism.
- **Mechanism**: Data structures leverage `const` sizing generics (e.g., `struct PackedKeyTable<K, V, const N: usize>`). Iteration happens strictly using `(0..N).for_each(|i| { ... })`. Because `N` is an absolute compile-time constant, LLVM easily resolves the iteration space completely statically, unrolling the logic without backedges.

### 3. Generated Domain Constants (DIM-based loops)
- **Use Case**: Matrices and algorithms where dimensions are known before compilation.
- **Mechanism**: Bounded sizes are baked in via external generation scripts as constants (e.g., `pub const DIM: usize = 2;`). Fixed arrays bound by these sizes are then iterated over. Because the sizes are rigidly bounded, array iterations compile exactly to straight-line instruction sequences.
- **Example**: Found in `crates/bcinr-cmca/src/stability.rs` and generated artifacts.

### 4. Domain-Specific Straight-Line Macros
- **Use Case**: Complex algorithms that inherently rely on looping, like sorting.
- **Mechanism**: Traditional loops are sidestepped entirely by replacing them with predefined deterministic execution graphs resembling hardware logic gates. Sequential steps are manually orchestrated via step-by-step macros (e.g., `cas!(i, j)` steps for optimal sorting algorithms).
- **Example**: Found in `crates/bcinr-logic/src/algorithms/optimal_sort_5_u32.rs`.
