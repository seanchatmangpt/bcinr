# Bounded Iteration Mechanics and Unrolling in BCINR

## Rule 13: No Unbounded Execution

In the `bcinr` project, **Rule 13 of AGENTS.md (No unbounded execution)** strictly prohibits any runtime loops that rely on dynamic conditions. The rule explicitly bans constructs like `while value > 0`, `for item in variable_slice`, conditional `loop { break; }`, and iterator short-circuiting (`take_while`). 

According to the constitution, all authoritative iteration must be:
- Compile-time fixed
- Generated
- Macro-unrolled
- Proven to be fully unrolled in the release object code

## Why a Fixed Rust Source Loop is Insufficient

A common misconception is that a bounded, fixed-length `for` loop in Rust source code (e.g., `for i in 0..8`) satisfies the requirement. However, Rule 13 states: *"A fixed Rust source loop is not automatically accepted. The final machine code must contain no loop backedge in authoritative symbols."*

This is because:
1. **Loop Backedges in Object Code:** Standard loops compile into conditional backward jumps (backedges) in the machine code. The presence of a loop backedge means the runtime instruction shape depends on execution state, violating the mandate that authoritative instruction shapes must not depend on semantic input (Rule 3).
2. **Bounds-Check Panic Paths:** Traditional loops may force the compiler to insert bounds-checking panic paths to ensure memory safety, which violates the strict $CC=1$ cyclomatic complexity requirement (Rule 8).

If the LLVM compiler decides not to unroll a fixed source loop (e.g., due to optimization settings or heuristics), the resulting binary will contain conditional branches, failing the disassembly audit required by the Turing Machine Enforcer (`@turing_machine`).

## Achieving Safe Bounded Iteration via Macro Unrolling

To guarantee branchless assembly and perfectly bounded execution work, `bcinr` relies heavily on structural macro unrolling techniques (e.g., `unroll_8_static!`, `unroll_n_static!`).

Instead of writing a `for` loop, the developer uses a macro that manually expands the loop body multiple times. For example:

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

This structural expansion achieves several critical goals:
- **Sequential Execution:** The loop body is copied sequentially (`$body, $body, $body...`), meaning the execution flows strictly downwards. There is no backward jump generated in the LLVM IR.
- **Physical Eradication of Backedges:** Because there is no conditional jump to evaluate termination, loop backedges are physically impossible in the compiled binary.
- **Bit-Parallel Mechanics over Byte-Sequential Flow:** This transforms sequential semantic decisions into straight-line, branchless arithmetic operations.

## The Role of Const-Generics

A key mechanism within these macros is the use of `const-generics` (or statically scoped `const` variables). By injecting `const $var: usize = N` into each localized scope of the unrolled body:

1. **Counters Become Compile-Time Constants:** The dynamic loop iterator variable (`let mut i = 0;`) is entirely removed. The iteration index is statically known at compile time for each unrolled block.
2. **Constant Folding & Bounds Elision:** When array indexing occurs (e.g., `res[i & 7]`), the compiler resolves the index statically. Since the array dimensions and index are both known at compile time, LLVM can completely optimize away bounds checks. This eliminates all hidden panic paths and branching in the generated assembly.

By synthesizing macro unrolling with static consts, `bcinr` strictly adheres to Rule 13, completely eliminating control-flow branching in favor of arithmetic masks and bounded, deterministic hardware execution.
