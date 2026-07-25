# Absolute CC=1 Law: Prohibition on Idiomatic Rust Control Flow

In the BCINR repository, the **Radon Law ($CC=1$)** mandates a purely deterministic, branchless execution model for all authoritative runtime logic. Rule 8 of the `AGENTS.md` constitution strictly forbids traditional control-flow constructs, including standard idiomatic Rust patterns like `Option`/`Result`-based control flow, `unwrap_or_else`, and iterator short-circuiting. 

This document explores *why* these constructs inherently violate the $CC=1$ rule and *how* the `@turing_machine` enforcer detects them, even when heavily abstracted.

## Why Idiomatic Rust Violates $CC=1$

In standard Rust development, constructs like `unwrap_or_else`, `?`, and short-circuiting iterators are considered best practices for safe, expressive code. However, in BCINR's "hard substrate," they represent unacceptable timing side-channels and state variability.

### 1. `Option`- and `Result`-based Control Flow (and `?`)
Under the hood, `Option` and `Result` types are enums driven by discriminant tags (e.g., `Some` vs. `None`). 
- **The `?` Operator**: Expanding the `?` operator generates a `match` statement that checks the tag. If the value is an `Err` or `None`, it triggers an `early return`. An early return is an explicit control flow branch that terminates execution prematurely, violating the law of "fixed bounded execution work."
- **Pattern Matching (`match`, `if let`)**: Any check of an enum discriminant intrinsically generates conditional jump instructions (e.g., `je`, `jne` in x86 assembly). These branches create execution paths of varying lengths, breaking constant-time guarantees.

### 2. `unwrap_or_else` (and similar closures)
While `unwrap_or(default_value)` might sometimes be optimized by the compiler into a branchless `cmov` (conditional move), `unwrap_or_else(|| ...)` takes a closure that is evaluated *lazily*.
- **Lazy Evaluation as a Branch**: The compiler must insert a conditional branch to decide whether or not to execute the code inside the closure. 
- **The Mask Alternative**: Rule 9 requires that conditional execution must instead be structured as eagerly evaluated bitwise polynomials and masks: `(mask & a) | (~mask & b)`. Both sides must be computed uniformly, with the final state selected via constant-time bitwise logic, not lazy evaluation.

### 3. Iterator Short-Circuiting (`take_while`, `find`, `any`, etc.)
Standard iterators often rely on dynamic termination (e.g., stopping when a condition is met).
- **Data-Dependent Loop Termination**: Short-circuiting introduces a dynamic backedge or a conditional break based on the semantic input. 
- **Violation of Bounded Bounds**: Rule 13 states that all loops must be "compile-time fixed, generated, macro-unrolled, or demonstrated as fully unrolled." If an iterator can stop at element 3 or element 100 depending on the data, it fails the mandate for fixed execution work and determinism.

---

## How `@turing_machine` Detects Hidden Violations

Developers often attempt to encapsulate branches inside helper functions, macros, or trait implementations (what Rule 16 calls "Scanner evasion" or "CHEAT-006"). The `@turing_machine` role (Enforcer of Determinism) utilizes a defense-in-depth auditing approach to ensure no branch escapes detection.

### 1. Abstract Syntax Tree (AST) Parsing
The `bcinr-cheat-scanner` (Rule 17) does not rely on naive regex or line-by-line grepping. It parses the full Rust **Syntax Tree**. This means standard text formatting, splitting operators across lines, or inserting comments inside tokens cannot hide a `match` or an `if`. 

### 2. Macro Expansion Inspection
Macros are frequently used to hide boilerplate—and inadvertently, control flow. The `@turing_machine` scanner explicitly inspects **macro definitions and their expanded output**. If a macro generates a `?` or a `match` underneath the surface, the AST scanner catches the expanded syntax, flagging the `CC>1` violation.

### 3. Transitive Call Graph Analysis
Rule 7 dictates "Whole-call-graph branchlessness." It is not enough for the public entry point to be branchless. The audit recursively resolves:
- Private wrapper functions.
- Generic monomorphizations.
- Trait methods (branches hidden in trait `impl` blocks count).
- Linked runtime symbols and compiler intrinsics.

If a developer calls a seemingly harmless dependency method that internally contains a branch or panic path (e.g., bounds checking), the scanner flags the entire authoritative path as unlawful.

### 4. The Ultimate Gate: Production-Profile Object Code Audit
Source-level checks are necessary but insufficient, as the Rust compiler itself can inject branches (e.g., for bounds checks or implicit panics). Rule 20 enforces an exact **disassembly audit** of the final machine code.
- `@turing_machine` analyzes the output assembly for all release targets.
- It scans for *any* conditional jumps, loop backedges, indirect calls, or allocator symbols.
- If the compiler transforms an idiomatic Rust construct into a conditional jump instruction, the source-level intent is irrelevant—the build is rejected.

In BCINR, if a property cannot be stated in fixed-width bitwise arithmetic without branching, it is mathematically rejected. The `@turing_machine` guarantees this invariant from the source code down to the bare silicon.
