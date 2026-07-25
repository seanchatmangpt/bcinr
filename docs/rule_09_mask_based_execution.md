# Rule 9: Mask-Based Execution Law

In the `bcinr` deterministic computational substrate, Rule 9 dictates how runtime conditional logic must be transformed into arithmetic operations. This is a foundational pillar for achieving the project's goal of $CC=1$ (Cyclomatic Complexity of 1) and entirely branchless algorithmics.

## The Mathematics of Full-Width Masks

The law mandates that runtime predicates must be evaluated into **full-width masks**:

$$ m \in \{0, 2^w - 1\} $$

Where $w$ is the bit-width of the target data type (e.g., 8, 32, or 64 bits). 

In traditional programming, a boolean predicate evaluates to a 1-bit value (`1` for true, `0` for false). In mask-based execution, a predicate evaluates to a mask that fills the entire data width:
* **False:** All bits are `0` (mathematically $0$, or `0x0000...0000`).
* **True:** All bits are `1` (mathematically $2^w - 1$, which is equivalent to `!0` or `0xFFFF...FFFF` in two's complement).

By extending the predicate's truth value across all bits of the word, we can use bitwise operations to route data without invoking the CPU's branch predictor.

## The Selection Formula

Selection must take a pure bitwise algebraic form equivalent to:

$$ \operatorname{select}(m, a, b) = (m \land a) \lor (\neg m \land b) $$

This formula evaluates the choice between candidate $a$ and fallback $b$ in constant time using parallel logic gates:

1. **If $m$ is True (All `1`s):**
   * $m \land a$ preserves all bits of $a$ (since `1 & x = x`).
   * $\neg m$ becomes all `0`s.
   * $\neg m \land b$ clears all bits of $b$ (since `0 & x = 0`).
   * The result is $a \lor 0$, which simplifies to **$a$**.

2. **If $m$ is False (All `0`s):**
   * $m \land a$ clears all bits of $a$ (since `0 & x = 0`).
   * $\neg m$ becomes all `1`s.
   * $\neg m \land b$ preserves all bits of $b$.
   * The result is $0 \lor b$, which simplifies to **$b$**.

This guarantees that both inputs $a$ and $b$ are unconditionally computed and merged, producing the exact required value based on the mask $m$.

## Why Branching State Assignments are Prohibited

The constitution strictly prohibits branching constructs for state transitions.

**Prohibited Shape:**
```rust
if valid {
    candidate
} else {
    current
}
```

**Required Shape:**
```rust
let mask = valid_mask(...);
let next = State::select(mask, candidate, current);
```

### 1. Protection Against Timing Side-Channels
A conditional branch (`if / else`) can take a variable amount of time depending on whether the branch is taken or whether the CPU correctly predicts it. In an AGI-grade hard substrate, timing side-channels are treated as physically impossible. Mask-based execution ensures the instruction latency is identical for both true and false paths.

### 2. The Radon Law ($CC=1$)
`bcinr` mandates a Cyclomatic Complexity of 1 per authoritative function. The authoritative instruction shape must never depend on semantic input. An `if` statement forks the control-flow graph, violating the $CC=1$ rule. Masking reduces sequential semantic decisions into straight-line data-flow polynomials.

### 3. Structural Determinism
When code relies on branching, unexpected conditions or host environment artifacts can cause divergent logic execution. Mask-based execution forces all paths to be computed and explicitly unified. This means the CPU processes a fixed set of instructions (a fixed execution work bound), regardless of the runtime payload, guaranteeing mathematically deterministic output. 

For structured state, this logic extends fieldwise and fixed-width, ensuring that state transitions are treated as atomic, allocation-free, bit-parallel transitions across the entire execution frame.
