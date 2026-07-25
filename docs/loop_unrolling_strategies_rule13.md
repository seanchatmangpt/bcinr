# Rule 13 Enforcement: Unbounded Execution Unrolling Strategies

In the `bcinr` (BranchlessCInRust) deterministic substrate, Rule 13 strictly prohibits unbounded execution, including `while value > 0`, `for item in variable_slice`, and any loops with data-dependent termination conditions. The runtime must enforce bounded $O(1)$ cyclic complexity ($CC=1$). The final machine code must contain absolutely no loop backedges in authoritative symbols.

To achieve this, the codebase employs four primary architectural strategies to guarantee entirely straight-line, branchless iteration loops:

### 1. Static Macro-Unrolling
For multidimensional logic blocks requiring complex iteration, loops are manually replaced by static unrolling macros that explicitly duplicate the block body for each step index.
- **Location:** `crates/bcinr-cmca/src/allocator.rs`
- **Implementation:** Explicit macros like `unroll_8_static!`, `unroll_9_static!`, and `unroll_32_static!` are used. For example, `unroll_8_static!` accepts an identifier and a block of code, declaring `const $var: usize = N` inside separate scopes for each manual step:
  ```rust
  macro_rules! unroll_8_static {
      ($var:ident, $body:expr) => {{
          { const $var: usize = 0; $body }
          { const $var: usize = 1; $body }
          // ... up to 7
      }};
  }
  ```
- **Result:** Expanding the block purely via Rust macros guarantees the compiler will generate perfectly straight-line instructions, entirely preventing unbounded loop behavior or dynamic branching.

### 2. Const-Generic Bounded Iteration
Where generic structures require configurable boundaries without sacrificing determinism, they rely on `const` sizing generics that can be statically evaluated by LLVM.
- **Location:** `crates/bcinr-logic/src/autonomic/packed_key_table.rs`
- **Implementation:** Data structures leverage signatures like `pub struct PackedKeyTable<K, V, const N: usize>`. The iteration happens strictly using `(0..N).for_each(|i| { ... })`. Inside the block, all operations use branchless masking arrays rather than dynamic branching.
  ```rust
  (0..N).for_each(|i| {
      let is_match = (i < self.len && self.hashes[i] == hash) as usize;
      result = [result, self.values[i]][is_match];
      found |= is_match;
  });
  ```
- **Result:** Since `N` is an absolute compile-time constant, LLVM can unroll the logic entirely statically, without backedges.

### 3. Generated Domain Constants (DIM-based loops)
In matrices and algorithmic domains, sizes are generated and explicitly injected as static constants into the code.
- **Location:** `crates/bcinr-cmca/src/stability.rs` and `crates/bcinr-cmca/src/generated/case_studies.rs`
- **Implementation:** Bounded constants like `pub const DIM: usize = 2;` or `pub const N: usize = 8;` are baked in via external generation scripts. Fixed arrays bound by these sizes (e.g., `[[i64; DIM]; DIM]`) are then iterated.
- **Result:** Rigidly bounding sizes at generation time ensures that array iterations compile strictly to straight-line sequences.

### 4. Domain-Specific Straight-Line Macros
Complex iterative algorithms natively replace standard loops with predefined deterministic step-by-step structures resembling hardware logic gates.
- **Location:** `crates/bcinr-logic/src/algorithms/optimal_sort_5_u32.rs`
- **Implementation:** Nested iteration is omitted entirely in favor of predefined execution graphs, sequentially applied via `cas!(i, j)` macro steps mimicking hardware comparators.
- **Result:** The static execution sequence acts as data-oblivious arithmetic, sidestepping cyclical execution bounds completely.
