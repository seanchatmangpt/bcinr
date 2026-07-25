# BCINR Rule 7: Transitive Audit Process for Traits and Generics

According to the BCINR Deterministic Substrate Constitution (`AGENTS.md`), **Rule 7 (Whole-call-graph branchlessness)** mandates that the Radon Law ($CC=1$) applies to the entire transitive call graph, not just the public entry point. Because Rust’s zero-cost abstractions can obscure control flow, the substrate enforces a rigorous audit process to prevent trait implementations and generic monomorphizations from sneaking dynamic dispatch or hidden branches into the authoritative hot path.

## The Core Problem: Hidden Branches in Abstractions
When a developer writes generic code (`fn process<T>(item: T)`) or uses traits, the source code may appear perfectly linear and branchless (e.g., no `if`, `match`, or `loop`). 

However, this source-level purity is an illusion:
* **Monomorphization:** Rust stamps out a distinct machine-code copy for every concrete type `T`. A generic function might be perfectly branchless for integers but compile into conditional jumps if instantiated with an `Option`, an `Enum` (requiring discriminant checks), or a type whose trait implementation includes overflow/bounds checks.
* **Dynamic Dispatch:** Using `&dyn Trait` introduces runtime virtual method table (vtable) lookups, fundamentally relying on unpredictable control flow.

To combat this, the constitution explicitly rejects the claim: *"The function contains no `if`, therefore it is branchless."*

## The Transitive Audit Process
Because source-level scanners (`bcinr-cheat-scanner`) cannot reliably resolve the final executable instructions of monomorphized generics or dependencies, BCINR shifts enforcement to **Rule 20: The Object-Code Audit**. The structural auditor (`@turing_machine`) executes the following transitive audit process:

### 1. Transitive Call Graph Enumeration
The audit does not merely inspect the root function. It forces the complete enumeration of the transitive call graph, explicitly including:
* Direct and transitive callees (even inside dependencies).
* Generic monomorphizations.
* Trait methods.
* Macros and generated modules.
* Compiler intrinsics and language-generated panic paths.

### 2. Auditing Every Monomorphized Symbol
Standing is never granted to a generic function definition. Instead, for every generic type instantiated in the release build, the auditor tracks its specific, monomorphized symbol in the final binary (e.g., `process::<ComplexType>`). Each unique machine-code call graph is inspected individually to ensure no conditional jumps (e.g., `je`, `jne`) were introduced by the substituted concrete type.

### 3. Blocking Dynamic Dispatch (Indirect Calls)
Rule 3 explicitly outlaws "dynamic dispatch" and "indirect calls." The object code audit inspects the exact release-profile disassembly (via `objdump -d` or `cargo asm`). If the auditor encounters any symbol containing an **indirect call** instruction (e.g., `call *%rax`), it is immediately flagged as a violation, as it indicates vtable resolution.

### 4. Verification and Classification
Every identified symbol in the final binary must be classified in an audit table proving it has:
* $CC=1$
* $0$ Conditional jumps
* $0$ Loop backedges
* $0$ Panic paths
* $0$ Allocator calls

Any symbol that fails this classification (e.g., showing a hidden branch from a dependency's trait implementation) results in the failure of the object code gate and blocks the merge.

## Conclusion
In BCINR, generic functions and traits are treated merely as templates. True verification is physically enforced on the **release profile disassembly** of every instantiated type parameter across the full transitive call graph. This ensures that no hidden conditional logic or dynamic dispatch can bypass the substrate's deterministic constraints.
