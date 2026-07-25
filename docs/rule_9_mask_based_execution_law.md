Based on `AGENTS.md`, here are the details regarding the **Mask-based execution law** under Rule 9:

### Mask-Based Execution Law

**Full-Width Masks**
Runtime predicates must not use standard boolean values for branching. Instead, they must be converted into **full-width masks**, where the mask $m$ consists of all 0s or all 1s for a given bit-width $w$:
$$m \in \{0, 2^w-1\}$$

**Required Logical Structure for Selection**
Control flow decisions must be executed using bit-parallel mechanics. The selection must take a branchless form using bitwise operations equivalent to:
$$\operatorname{select}(m, a, b) = (m \land a) \lor (\neg m \land b)$$

For structured state, this selection must be performed **fieldwise** and be **fixed-width**.

**Prohibited Pattern**
Using traditional control flow constructs for selection is strictly forbidden:
```rust
if valid {
    candidate
} else {
    current
}
```

**Required Shape**
Instead, you must derive a mask and use a state selection method:
```rust
let mask = valid_mask(...);
let next = State::select(mask, candidate, current);
```

Lastly, the mask implementation itself is subject to strict verification and must pass object-code inspection to ensure no hidden branches exist in the compiled output.
