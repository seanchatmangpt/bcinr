# Enforcing Branchless Execution Across Traits and Generics in BCINR

According to the BCINR Deterministic Substrate Constitution (`AGENTS.md`), the core requirement of deterministic, branchless execution extends far beyond the surface-level source code of a public function. Rule 8 and Rule 20 work in tandem to ensure that zero branches are hidden within Rust's abstraction mechanisms, specifically trait implementations and generic monomorphizations.

## Rule 8: The Absolute `CC=1` Law and Hidden Abstractions

Rule 8 explicitly states that **"Branches hidden in trait implementations count."** 

When a developer uses generic traits (e.g., `fn process<T: Computable>(item: T)`) or dynamic dispatch (e.g., `&dyn Computable`), the source code might look completely branchless:

```rust
// Looks branchless in source (CC=1 at the syntax level)
fn process_item(item: &dyn Computable) -> u64 {
    item.compute()
}
```

However, the constitution rejects the claim that "The function contains no `if`, therefore it is branchless." (Rule 7). If the underlying implementation of `compute()` for a specific type contains an `if`, `match`, or bounds-check, or if the compiler must generate an indirect call via a vtable to resolve it at runtime, the transitive call graph is no longer `CC=1`. Furthermore, Rule 3 explicitly outlaws "dynamic dispatch" and "indirect calls" for this exact reason.

## Rule 20: The Object-Code Audit as the Ultimate Enforcer

Because source-level scanners (`bcinr-cheat-scanner`) cannot easily resolve the final executable instructions of monomorphized generics or dynamic dispatch, BCINR relies on **Rule 20 (Object-code audit)** to catch what a syntax tree parser might miss. 

The object code audit catches these abstractions in the following ways:

### 1. Transitive Call Graph Enumeration
Per Rule 7 and Rule 20, the audit does not just look at the root function. It forces the enumeration of the entire transitive call graph, explicitly including:
* **Trait methods**
* **Generic monomorphizations**

For every generic type `T` instantiated in the release build, the auditor (`@turing_machine`) must track its specific, monomorphized symbol in the final binary.

### 2. Detection of Dynamic Dispatch (Indirect Calls)
If a developer attempts to use `dyn Trait`, the Rust compiler implements this using virtual method tables (vtables). The object code audit specifically inspects the disassembly (via `objdump -d` or `cargo asm`) for **indirect calls** (e.g., `call *%rax` or similar instructions). Since the runtime resolution of a function pointer fundamentally introduces unpredictable control flow, any symbol containing an indirect call immediately violates Rule 20 and is blocked from merging.

### 3. Detection of Monomorphized Conditionals (Conditional Jumps)
If a generic function is monomorphized with a type whose trait implementation contains hidden logic, the compiler will generate conditional jump instructions (e.g., `je`, `jne`, `jb`) for that specific instantiation. During the disassembly step, the auditor inspects the exact machine code for these jumps. Even if the generic wrapper function is perfectly clean in source, the specific compiled symbol for `process::<ComplexType>` will reveal the conditional jumps. The symbol will be flagged on the audit table with conditional jumps > 0, earning an `UNKNOWN` or `SOURCE_BRANCHLESS_PARTIAL` standing rather than the required `BRANCHLESS_ALIVE`.

## Conclusion

In the BCINR framework, Rust's high-level, zero-cost abstractions are not trusted blindly. While generics and traits are permitted, they are subjected to a post-compilation physical inspection. By shifting the final verification to the **release profile disassembly**, Rule 20 guarantees that any control-flow branch or vtable dispatch hidden behind a clean trait boundary is mathematically and physically caught before reaching the substrate.
