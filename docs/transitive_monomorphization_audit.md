# Transitive Monomorphization Audit

Under **Rule 7 (Whole-call-graph branchlessness)**, the BCINR deterministic substrate mandates that no data-dependent branches exist anywhere within the complete authoritative call graph. This applies transitively and specifically prohibits relying on source-level definitions—such as claiming "the function contains no `if`, therefore it is branchless."

### How the Disassembly Audit Handles Generic Monomorphizations

In Rust, generic functions undergo **monomorphization**, where the compiler generates unique machine code for every concrete type instantiated. A generic function that appears perfectly linear and branchless in source code can become a branching liability depending on the type it is instantiated with.

The disassembly audit (governed by the `@turing_machine` role) handles generic monomorphizations with the following strict requirements:

1. **Object Code Over Source Claims:**
   The generic definition itself is never granted standing. Standing is only granted after analyzing the **audited release object code** for the specific target. The auditor must verify the final machine code of the transitive call graph for *every* monomorphized instance.

2. **Every Instantiated Type is Audited:**
   Because each concrete type substitution produces distinct machine code, every single instantiation of a generic parameter is treated as a separate function. If a generic function is instantiated with three different types, all three resulting machine-code call graphs must be independently audited to ensure a Cyclomatic Complexity ($CC$) of 1.

3. **Detection of Hidden Branches in Transitive Callees:**
   The audit strictly inspects monomorphizations for hidden branches introduced by concrete types, such as:
   * **Enums & `Option<T>`:** Automatically generated discriminant checks (e.g., dropping an `Option<T>` requires checking if it is `Some` or `None`, introducing a hidden `match`).
   * **Derived Traits:** Traits like `#[derive(Clone)]` or `#[derive(PartialEq)]` on enums generate branching code.
   * **Iterators:** Complex iterators might inject state-machine branches, bounds checks, or early returns.
   * **Panics & Bounds Checks:** Trait implementations that introduce overflow checks or slice indexing without masks can result in compiler-generated panic paths.

### Conclusion

The permitted claim for branchlessness in the presence of generics is:
> *"The full authoritative call graph contains no input-dependent conditional branch in the audited release object code for the declared target."*

All private functions, trait methods, macros, and **generic monomorphizations** must pass this strict exact production-profile disassembly audit to satisfy Rule 7.
