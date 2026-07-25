# The StateCommitGate Pattern in BCINR

In the BCINR architecture, the **StateCommitGate** pattern is the structural realization of the absolute runtime law: **"No mutation before complete admission."** 

To satisfy the deterministic, branchless `CC=1` mandate, persistent memory cannot be updated speculatively or through traditional control-flow branches (e.g., `if valid { state = candidate; }`). Instead, the hot path strictly segregates the **evaluation phase** (off-target computation and validation) from the **mutation phase** (the physical overwrite of persistent memory), using bitwise selection as the impenetrable mathematical boundary.

## 1. Phase Isolation: Evaluation vs. Mutation

### The Evaluation Phase
During the evaluation phase, the hot path operates entirely independent of the persistent state memory. Because the authoritative runtime is allocation-free (`#![no_std]` and 0 heap allocations), it prepares the transition using fixed-size scratch structures or stack values.

The sequence follows a strict transaction shape:
1. **Current Immutable State**: The system reads the current state $x_t$.
2. **Fixed-Size Candidate State**: The system computes the proposed candidate state $x_{candidate}$ entirely off-target.
3. **Verify All Predicates**: All contractual predicates, constraints, and bounds are evaluated unconditionally. 
4. **Derive Admission Mask**: Any invalidation or refusal conditions are mathematically reduced to a boolean predicate, which is then expanded into a full-width bitmask ($m_{admitted}$), where:
   - Complete admission (`true`) $\rightarrow$ `0xFFFFFFFFFFFFFFFF`
   - Any refusal (`false`) $\rightarrow$ `0x0000000000000000`

### The Mutation Phase
The mutation phase is completely devoid of semantic logic or evaluation. It blindly performs a SWAR (SIMD Within A Register) bitwise selection across the fields of the state structure. There is no `if` check; the state is unconditionally overwritten every single cycle.

## 2. Mathematical State Commit via `select`

The sole mechanism for writing to persistent state is the bitwise polynomial:

$$ x_{t+1} = \operatorname{select}(m_{admitted}, x_{candidate}, x_t) $$

Where the physical commit expands to:

$$ \operatorname{select}(m, a, b) = (m \land a) \lor (\neg m \land b) $$

### Fieldwise Masked Commit
The runtime iterates over the fields or words of the persistent structure (often statically unrolled over an array of `u64` words) and assigns the next state using the selection polynomial:

```rust
// SWAR selection unconditionally applied to every word
next.words[i] = (candidate.words[i] & mask) | (current.words[i] & !mask);
```

### Unconditional Memory Overwrite
Because the assignment occurs unconditionally, there are no timing side-channels, hidden branches, or execution paths that vary based on the data:
- If the transaction is fully admitted ($m_{admitted}$ = `0xFFFFFF...`), the state is entirely updated to $x_{candidate}$.
- If the operation is rejected ($m_{admitted}$ = `0x000000...`), the mask mathematically zeroes out the candidate state, and the bitwise `OR` restores $x_t$. The persistent state is unconditionally overwritten with a bit-for-bit identical copy of itself.

## Conclusion

The `StateCommitGate` pattern ensures that rejected operations leave persistent state physically unmutated without using control flow to avoid the mutation. By forcing all semantic decisions into a bitmask ($m_{admitted}$) and applying that mask via the $\operatorname{select}$ polynomial, BCINR guarantees that bounded execution work, memory access, and mathematical correctness are maintained across all inputs, satisfying the rigid requirements of the deterministic substrate.
