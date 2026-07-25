# Rule 9: Mask-Based Execution Law

The Mask-Based Execution Law is a fundamental pillar of the BCINR Deterministic Substrate Constitution, enforcing constant-time, branchless logic across the authoritative runtime.

## Transforming Predicates into Full-Width Masks

Under Rule 9, runtime predicates cannot be evaluated as simple boolean expressions that drive control flow. Instead, they must be transformed into **full-width masks**. 

Mathematically, a mask $m$ for a word of width $w$ must be:
$$m \in \{0, 2^w - 1\}$$

This means a "true" predicate must result in a bit-pattern of all 1s (e.g., `0xFFFFFFFF` for a 32-bit integer), and a "false" predicate must result in all 0s (`0x00000000`). This transformation must itself pass object-code inspection to ensure it contains no hidden branches.

## The `select(m, a, b)` Arithmetic Formulation

Once a predicate is converted to a mask, state transitions must be executed using bitwise arithmetic rather than jumps. Selection must take the following exact structural form:

$$ \text{select}(m, a, b) = (m \land a) \lor (\neg m \land b) $$

**How it works:**
- If the predicate is true ($m$ is all 1s), $m \land a$ yields $a$, and $\neg m \land b$ yields $0$. The result is $a$.
- If the predicate is false ($m$ is all 0s), $m \land a$ yields $0$, and $\neg m \land b$ yields $b$. The result is $b$.

For complex, structured states, this selection cannot be bypassed; it must be applied **fieldwise** and must be **fixed-width**. 

The required shape in Rust looks like this:
```rust
let mask = valid_mask(...);
let next = State::select(mask, candidate, current);
```

## Why `if valid { candidate } else { current }` is Prohibited

The traditional `if/else` control flow is explicitly prohibited in authoritative code for several critical, constitutional reasons:

1. **Violation of Absolute $CC=1$ Law (The Radon Law):** 
   Authoritative code is forbidden from having a cyclomatic complexity greater than 1. An `if/else` statement introduces a control-flow branch, directly violating the requirement that the complete call graph contain exactly one path.
2. **Introduction of Conditional Jumps:**
   An `if` statement compiles into conditional jump instructions in object code. This means the execution path depends on the input data, violating the principle of "fixed bounded execution work."
3. **Timing Side-Channels:**
   Because branches take different numbers of CPU cycles depending on branch prediction and which path is taken, they introduce timing side-channels. The `select` formulation ensures that the exact same operations (bitwise AND, OR, NOT) are executed regardless of whether the predicate is true or false, making timing side-channels physically impossible.
4. **The Von Neumann Bypass Mandate:**
   The constitution states: *"Sequential semantic decisions must be transformed into masks, arithmetic selection, fixed lookup tables, generated straight-line code."* The overarching philosophy is "Bit-parallel mechanics over byte-sequential control flow." 

Using `select` ensures that all state mutations occur through bitwise polynomials, completely eliminating the need for semantic branching in the machine code.
