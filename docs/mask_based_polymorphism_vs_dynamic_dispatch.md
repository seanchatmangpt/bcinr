# Mask-Based Polymorphism vs. Dynamic Dispatch in BCINR

In standard Rust (and object-oriented languages), runtime polymorphism is typically achieved using dynamic dispatch (`dyn Trait`). This involves virtual method tables (vtables), where function pointers are resolved at runtime. A polymorphic call translates into an indirect jump at the machine level, allowing different types to execute completely different code paths while exposing the same interface.

Under the BCINR constitution, Rule 3 explicitly forbids dynamic dispatch and indirect calls. These operations introduce data-dependent branching, variable execution timing, and hardware-level control-flow hazards, violating the core principle that "the authoritative instruction shape must not depend on semantic input."

To solve this, BCINR replaces traditional control-flow-based polymorphism with **mask-based data-flow polymorphism**, governed by Rule 9 (Mask-based execution law). Instead of dispatching execution to different functions based on runtime types or states, BCINR unifies all possible semantic behaviors into a single, branchless execution path using full-width masks, arithmetic selection, and fixed-width states.

## 1. Unified Fixed-Width States Instead of Subclasses
In a standard polymorphic system, different behaviors are often encapsulated in different types of varying sizes (e.g., a `ValidUpdate` struct versus an `InvalidUpdate` struct). In BCINR, these heterogeneous types are replaced by a single, comprehensive **fixed-width state structure**. Every possible attribute or field required by any semantic variant is included in this unified state. Because the structure is fixed-size, it requires zero heap allocation (Rule 3) and can be processed predictably in the hot path.

## 2. Full-Width Masks Instead of Type Identifiers
Rather than checking the runtime type of an object to decide which behavior to execute (e.g., using `match` on an enum or vtable dispatch), BCINR evaluates structural predicates into **full-width bitmasks**. 

A mask in BCINR must be strictly:
`m ∈ {0, 2^w - 1}`

For instance, `0xFFFF_FFFF` for true and `0x0000_0000` for false. These masks act as the mathematical equivalent of type or state identification.

## 3. Arithmetic Selection Instead of Vtable Dispatch
When a program needs to execute behavior `A` or behavior `B` based on a condition, traditional polymorphism uses indirect calls via a vtable to jump to the correct instruction sequence.

BCINR, however, computes the candidate outcomes of *both* (or *all*) behaviors unconditionally. It then uses **arithmetic selection** to blend the results back into the fixed-width state, avoiding any jumps. The core operation is defined mathematically as:

`select(m, a, b) = (m ∧ a) ∨ (¬m ∧ b)`

In practice, an operation that would normally dispatch dynamically is replaced by computing the candidate states and selecting the correct outcome using the mask:

```rust
// Compute the mask representing the "type" or "state"
let is_valid_mask = valid_mask(...);

// Compute candidate state transitions unconditionally
let candidate_state = compute_candidate(&current_state);

// Unconditionally select the correct candidate for every field
let next_state = State::select(
    is_valid_mask, 
    candidate_state, 
    current_state
);
```

## 4. Fieldwise Selection for Structured State
For complex data structures, Rule 9 dictates that "selection must be fieldwise and fixed-width." Instead of replacing an entire allocated object reference, BCINR applies the `select` function to every individual field of the fixed-width state struct. This ensures that the state transition remains completely flat, branchless, and fully unrolled by the compiler. A rejected operation leaves the persistent state bit-for-bit unchanged without ever executing an `if` statement.

## Conclusion
In BCINR, semantic polymorphism is not achieved by altering the control flow (vtables, indirect jumps, `if`/`match` blocks). Instead, it is achieved by running a unified, branchless pipeline where all potential state transitions are computed simultaneously, and the "correct" polymorphic behavior is mathematically selected via bit-parallel mask operations. This guarantees `O(1)` cycle predictability, perfect determinism, and full compliance with the Radon Law (`CC=1`).
