# Rule 8 (Absolute CC=1 law) and `unwrap_or_else`

Based on the `AGENTS.md` BCINR Deterministic Substrate Constitution, **Rule 8 (Absolute `CC=1` law)** strictly mandates that the entire authoritative call graph must have a Cyclomatic Complexity of exactly 1. This means no data-dependent conditional branches are permitted anywhere in the execution path.

### Why `unwrap_or_else` is Prohibited

`unwrap_or_else` (typically used with Rust's `Option` or `Result` types) is explicitly prohibited because it inherently introduces a control-flow branch at runtime. 

Here is exactly how it produces a branch:
1. **Conditional Evaluation**: When `unwrap_or_else` is called, the program evaluates the variant of the `Option` or `Result`.
2. **The Branching Logic**: 
   - **Path A (Success)**: If the value is `Some` or `Ok`, the execution directly returns the contained value.
   - **Path B (Fallback)**: If the value is `None` or `Err`, the execution jumps to a different path to invoke the provided closure and compute the fallback value.
3. **Data-Dependent Control Flow**: Because the decision to execute the closure depends entirely on the runtime data (the variant type), it creates a hidden conditional jump (equivalent to an `if/else`) in the compiled object code. This violates the rule that instruction shape must not depend on semantic input.

### The Lawful Alternative in BCINR

Under **Rule 9 (Mask-based execution law)**, all sequential semantic decisions and runtime predicates must be transformed into full-width bitmasks ($m \in \{0, 2^w-1\}$), where selection takes a bitwise, branchless form equivalent to:

```rust
(m & a) | (!m & b)
```

In the BCINR substrate, you cannot lazily or conditionally evaluate a fallback using a closure. Instead, an authoritative implementation must compute (or statically provide) the candidate values branchlessly, and then use a mathematical mask to select the final accepted state. Control-flow abstractions that hide conditional execution, including `unwrap_or_else`, `unwrap_or`, `unwrap`, `?`, and `match`, fundamentally violate the deterministic, constant-time guarantees of the runtime.
