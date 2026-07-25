# Rule 23: Contract Gate Architecture

Under **Rule 23 (Required repository gates)**, `cargo make contract-gate` acts as the structural enforcer (the `@turing_machine` role) for the Radon Law ($CC=1$) and branchless constraints. It verifies mathematical Hoare contracts and the absence of control flow in the authoritative computational substrate.

The command delegates to a dedicated Rust tool located in the workspace at `tools/bcinr-contract-gate`. 

Here is how the Contract Gate architecture is structurally implemented to verify these constraints:

## Enforcement Mechanism

The `bcinr-contract-gate` **parses the Abstract Syntax Tree (AST)** directly using the `syn` crate. It **does not** hook into MIR (Mid-level Intermediate Representation) or LLVM passes. It evaluates the codebase completely at the source-code level by implementing a custom AST visitor (`syn::visit::Visit`) that scans every function block and expression.

## Architectural Capabilities

### 1. AST Node Visitation (`CalleeVisitor`)
The tool iterates through all expressions in the source and manually bumps the cyclomatic complexity counter for any syntax that implies a branch or a potential panic path, strictly enforcing $CC=1$. Specifically, it penalizes:
- **Control flow constructs:** `Expr::If`, `Expr::Match`, `Expr::Loop`, `Expr::While`, `Expr::ForLoop`
- **Early returns / error propagation:** `Expr::Try` (the `?` operator)
- **Panic-inducing method calls:** `unwrap`, `expect`, `unwrap_or`, `unwrap_or_else`

### 2. Reachability Graph Analysis
The tool only applies the strict $CC=1$ check to functions that are part of the hot path.
- It builds a call graph starting from defined `AUTHORITATIVE_ROOTS` (specifically `"allocate"` and `"evaluate_calibration"`).
- It traces reachability to prevent the tool from unnecessarily failing on non-authoritative logic.

### 3. Verification of Hoare Contracts
The tool verifies mathematical contracts by parsing docstrings and attributes for public reachable functions. 
- It enforces that all public reachable primitives (unless explicitly excluded like benchmarks or tests) contain specific contractual declarations in their documentation or attributes.
- Valid strings include `"Branchless Contract"`, `"BRANCHLESS CONTRACT"`, or the `"u64_contract!"` macro. 
- If a reachable function lacks a mathematical specification, it raises a `MISSING_U64_CONTRACT` error.

### 4. Rejection of "Bluffs"
The visitor captures binary operators (`+`, `-`, `*`, `/`). If a function claims to be branchless or bitwise (e.g. `add_bitwise` or `sub_bitwise`) but uses standard arithmetic operators instead of actual bitwise masks, it detects the violation and raises a `"Bluff detected!"` error.

## Summary

By statically enforcing rules at the AST level, the `contract-gate` mathematically guarantees that the authoritative path correctly applies formal specification contracts, expresses logic purely as bitwise polynomials, and remains completely free of branching control flow and panic paths.
