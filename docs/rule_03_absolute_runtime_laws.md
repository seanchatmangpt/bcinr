# Rule 3: Absolute Runtime Laws

According to `AGENTS.md`, the authoritative runtime must preserve a fixed deterministic execution trace without any conditional logic or dynamic resource use. 

## Transitive Constraints

The complete authoritative call graph must satisfy the following constraints:

- `#![no_std]`
- `no alloc`
- `zero heap allocation`
- `CC = 1 per authoritative function`
- `no data-dependent branches`
- `no data-dependent loop termination`
- `no panic paths`
- `no unwinding`
- `no floating-point operations`
- `no dynamic dispatch`
- `no indirect calls`
- `no runtime parsing`
- `no variable graph traversal`
- `no runtime algorithm search`
- `no runtime stability discovery`
- `fixed-width inputs`
- `fixed-width outputs`
- `fixed bounded memory access`
- `fixed bounded execution work`

## Why a Branching Private Helper is a Violation

These runtime laws apply **transitively**. A branchless public function calling a branching private helper is a violation because branchlessness applies to the *whole transitive call graph*, not merely the public entry point. 

If any part of the execution path (including private functions, trait methods, macros, generic monomorphizations, or compiler intrinsics) introduces a branch, the final machine code will contain input-dependent jumps. This breaks the fundamental mandate of the deterministic substrate, which requires execution work to be invariant and strictly composed of bit-parallel mechanics across all layers of the authoritative path.
