# Transitive Enforcement of Absolute Runtime Laws in BCINR

In the BCINR deterministic substrate, the "Radon Law" ($CC=1$) and the absolute runtime laws are not merely top-level suggestions; they are **transitive, whole-call-graph requirements**. As defined in **Rule 3** and **Rule 7** of the BCINR Constitution (`AGENTS.md`), a branchless public function is immediately invalidated if it calls a branching private helper or compiles into input-dependent jumps.

This document explores how these constitutional mandates apply to imported `core` methods, standard abstractions, and third-party dependencies, and outlines how `@turing_machine` systematically enforces these laws.

---

## 1. The Mandate: Rules 3 and 7

**Rule 3 (Absolute runtime laws)** dictates that the complete authoritative call graph must satisfy strict zero-branch, zero-allocation, and bounded-execution constraints. Crucially, it asserts: *"These laws apply transitively."*

**Rule 7 (Whole-call-graph branchlessness)** expands on this, stating that branchlessness applies to the transitive call graph, not merely the public entry point. This includes:
* Private functions and trait methods.
* Generic monomorphizations.
* Macros and generated modules.
* Compiler intrinsics and linked runtime symbols.
* Language-generated panic paths.

Any dependency linked into the authoritative hot path inherits the exact same constitutional burden as `bcinr` itself.

---

## 2. The Danger of Standard Library Abstractions

Rust's `core` library and common `no_std` crates are designed for safety and idiomatic developer experience, relying heavily on hidden control flow. Abstractions that seem harmless at the source level frequently violate the structural determinism required by BCINR.

### Examples of Deceptive `core` Primitives

1. **`Iterator::take_while` and Variable Iteration**
   * **The Illusion:** Functional, chainable, and seemingly declarative.
   * **The Violation:** `take_while` relies on a data-dependent conditional check for loop termination. Under the hood, it evaluates an `if let` or `match` on every iteration and breaks the loop when the predicate fails. This violates Rule 3 ("no data-dependent loop termination") and Rule 13 ("no unbounded execution"). Authoritative loops must be fully unrolled or compile-time fixed.

2. **Formatting via `core::fmt::Display`**
   * **The Illusion:** A standard way to stringify values without heap allocation.
   * **The Violation:** The formatting machinery includes loops for padding strings, conditional branches for alignment, and variable-length string traversals. It violates $CC=1$ immediately. In BCINR, formatting belongs strictly to the **Slow Rail** (Rule 6).

3. **Array Indexing (`slice[i]`)**
   * **The Illusion:** An essential, fundamental language operation.
   * **The Violation:** Array indexing in Rust inserts an implicit bounds check. If the bounds check fails, it branches to a panic path (`core::panicking::panic_bounds_check`). This is a hidden conditional branch and a language-generated panic path. Authoritative code must instead rely on fixed-width masking and bitwise selections without invoking bounds checks.

4. **Checked Arithmetic (`checked_add`, `checked_mul`)**
   * **The Illusion:** Safe mathematics that prevents wrapping.
   * **The Violation:** These methods return an `Option<T>`. To handle the `Option`, the caller must use an `if let`, `match`, or `?` operator. This constitutes branch-bearing handling. Instead, BCINR requires saturating, wrapping, or masked SWAR (SIMD Within A Register) operations designed to avoid control-flow divergence.

5. **The `?` Operator and `unwrap()`**
   * **The Illusion:** Idiomatic error propagation.
   * **The Violation:** `?` expands to a `match` statement, creating an early return branch. `unwrap()` contains a hidden panic path. Both explicitly violate Rule 8.

---

## 3. Enforcement: The `@turing_machine` Audit Protocol

Because source code claims cannot substitute for execution reality ("The function contains no `if`, therefore it is branchless" is a prohibited claim), the **`@turing_machine`** role (Enforcer of Determinism) utilizes a multi-layered audit protocol to verify transitive compliance.

### Phase 1: Deep Source Inspection (`bcinr-cheat-scanner`)
`@turing_machine` does not merely parse line-by-line text; it parses the full syntax tree.
* **Macro Expansion:** The scanner expands all macros (including those from dependencies) to ensure no hidden `match` or `if` statements are emitted.
* **Trait Implementations:** It traces trait method calls to verify that generic monomorphizations don't inject branching behavior.

### Phase 2: Hostile Mutants (`@armstrong_fault`)
To prove that no hidden fallback branches exist, the mutation matrix injects faults that alter mathematical laws. The implementation must yield a deterministic, bounded `TypedRefusal` rather than panicking or traversing a branch-based fallback algorithm. 

### Phase 3: Object-Code Disassembly (Rule 20)
The ultimate source of truth is the release object code. `@turing_machine` demands an exact production-profile disassembly audit. The transitive call graph is rejected if the final compiled artifact contains:
* **Conditional Jumps** (e.g., `je`, `jne`, `cmov` exceptions depending on the architecture).
* **Loop Backedges** (proving that loops were not completely unrolled).
* **Panic/Allocator Symbols** (proving that indexing or unwrapping slipped through).
* **Indirect Calls** (proving that dynamic dispatch was utilized).

If `Iterator::take_while` or an implicit array bounds check makes it into the authoritative build, the compiler will emit a conditional jump to a panic handler or loop backedge. The object-code audit catches this immediately, reducing the Substrate Integrity Score (SIS) to 0 and triggering `MaturityScrutiny`.

---

## Conclusion

In BCINR, the semantic boundary does not stop at the edge of the crate. Every imported trait, macro, and standard library function is subjected to the same mathematical and structural scrutiny as local code. Control flow hidden by convenient abstractions must be manually unpacked and transformed into branchless, mask-based arithmetic to survive `@turing_machine`'s rigorous disassembly audit.
