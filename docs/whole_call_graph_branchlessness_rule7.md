# BCINR Rule 7: Whole-Call-Graph Branchlessness & Audit Strategy

In the BCINR framework, **Rule 7** dictates that the Radon Law ($CC=1$) applies to the *entire transitive call graph*, not merely the public entry point of a function. This requirement ensures that no data-dependent execution paths or timing side-channels can be hidden in deeper layers of the call stack.

## The Core Mandate

The constitution strictly rejects source-level purity claims. 
* **Prohibited Claim:** *"The function contains no `if`, therefore it is branchless."*
* **Permitted Claim:** *"The full authoritative call graph contains no input-dependent conditional branch in the audited release object code for the declared target."*

## The Problem: Hidden Branches in Zero-Cost Abstractions

Rust’s abstractions can easily obscure control flow, making source-level audits insufficient:

1. **Generic Monomorphizations:** A generic function like `fn process<T>` might be mathematically pure. However, Rust stamps out a distinct machine-code copy for every concrete type `T`. If `T` is instantiated as an `Option` or an `Enum`, seemingly branchless operations (like dropping a value or checking equality) compile into conditional jumps to inspect discriminants.
2. **Trait Methods:** Branches can be hidden inside trait implementations (whether local or from dependencies). Furthermore, dynamic dispatch (`&dyn Trait`) relies on runtime vtable lookups, which inherently use indirect calls and violate determinism laws.
3. **Macros & Private Helpers:** Macros can generate hidden panic paths, short-circuiting logic, or conditional checks beneath simple-looking invocations. Moving branches into private wrappers does not reduce the cyclomatic complexity of the overall operation.

## The Transitive Audit Strategy

To guarantee true fixed-width execution, the structural auditor (`@turing_machine`) shifts enforcement from the source code to **Rule 20 (The Object-Code Audit)**. 

### 1. Complete Call Graph Enumeration
The audit forces a complete enumeration of the call hierarchy, expanding out from the root:
`root function → direct callees → transitive callees → compiler intrinsics → linked runtime symbols`

This explicitly includes all private functions, trait methods, generic monomorphizations, macros, generated modules, indexing operations, fixed-point helpers, and language-generated panic paths.

### 2. Auditing Every Monomorphized Symbol
Standing is **never** granted to a generic function definition. Instead, for every generic type instantiated in the release build, the auditor tracks its specific, monomorphized symbol in the final binary. Each unique machine-code call graph is inspected individually to ensure the substituted concrete type did not introduce conditional jumps.

### 3. Release-Profile Disassembly Inspection
Because source scanners (`bcinr-cheat-scanner`) cannot reliably resolve the final executable instructions, the audit inspects the exact release-profile disassembly (via tools like `objdump -d` or `cargo asm`). 

The auditor looks for:
* **Conditional Jumps:** (e.g., `je`, `jne`) resulting from trait impls, generic types, or implicit bounds checks.
* **Indirect Calls:** (e.g., `call *%rax`) resulting from unapproved dynamic dispatch.
* **Loop Backedges:** Resulting from un-unrolled iteration.

### 4. Classification & Verification Matrix
Every identified symbol reachable from the authoritative root in the final binary must be classified in an audit table proving it has:
* $CC = 1$
* $0$ Conditional jumps
* $0$ Loop backedges
* $0$ Panic paths
* $0$ Allocator calls

If any symbol in the transitive call graph fails this classification (for example, showing a hidden branch from a dependency's trait implementation), the object code gate fails, the project's Substrate Integrity Score drops, and the merge is blocked.
