Based on the `AGENTS.md` and `GEMINI.md` repository constitution, Rule 10's strict requirements for state cloning are directly tied to BCINR's mission as a deterministic, branchless, allocation-free computational substrate.

### Why "clone the state" must use fixed-size stack/scratch structures
1. **Allocation-Free Execution Context**: The authoritative runtime operates strictly under `#![no_std]` with an absolute "zero heap allocation" law (Rule 3). Any speculative state (the `candidate`) must be constructed using statically known, fixed-bounds memory.
2. **Deterministic Mask-Based Selection**: To achieve a Cyclomatic Complexity of 1 ($CC=1$) and completely branchless execution, a transaction must use the structural shape `select(mask, candidate, current)`. Fixed-size stack values or scratch structures ensure that the `candidate` and `current` states have identical, fixed widths. This allows fieldwise masked commits to be executed purely via bitwise arithmetic logic (as mandated by `@von_neumann_bypass` in Rule 4) rather than `if/else` control-flow branches.
3. **Bounded Memory & Work Laws**: Fixed-size stack or scratch structures guarantee "fixed bounded memory access" and "fixed bounded execution work" (Rule 3). The computational cost of creating the candidate state is statically guaranteed and strictly bounded at compile-time.

### Why heap-backed cloning is absolutely forbidden
1. **Violates the Zero-Allocation Boundary**: Heap allocation is an explicit constitutional violation of the `no alloc` and `zero heap allocation` laws that govern the authoritative hot path.
2. **Introduces Non-Determinism and Variable Work**: Dynamic memory allocation inherently involves variable execution time (breaking the "fixed bounded execution work" law) and hidden runtime logic to locate free memory blocks.
3. **Creates Hidden Branches and Panic Paths**: Heap allocation can dynamically fail (e.g., Out-Of-Memory). Handling this failure introduces data-dependent branches, unwinding, or panic paths—all of which are categorical violations of the absolute $CC=1$ runtime laws (Rules 3 and 8).

In short, heap-backed cloning relies on unpredictable, branching dynamic systems, whereas fixed-size stack cloning allows the substrate state machine to remain an axiomatic, constant-time, structurally lawful mathematical construct.
