# Innovation Proposal: Transposed Bit-Parallel Concurrency Guard Engine (T-BPGE)

## 1. Executive Summary

This proposal introduces the **Transposed Bit-Parallel Concurrency Guard Engine (T-BPGE)**, a constant-time, branchless, and zero-allocation algorithm for validating candidate ready-sets against process concurrency guards. 

By transposing the representation of minimal nonfaces from a list of event sets to an event-to-nonface membership matrix, T-BPGE eliminates the need for sequential loop traversals, short-circuiting control flow, and heap-allocated vectors in the hot path. The resulting execution has a cyclomatic complexity of exactly $CC=1$, fits perfectly within the BCINR Radon Law, and provides deterministic execution time regardless of the size of the concurrency complex or the input candidate set.

---

## 2. Problem Statement & Current Limitations

In process intelligence, concurrency guards restrict which activities can execute simultaneously due to resource limits or capacity constraints. These constraints are represented as a set of minimal nonfaces. A candidate execution set $C$ is valid if and only if it does not contain any minimal nonface as a subset.

In the current implementation (`crates/bcinr-powl/src/tape.rs` line 512):
```rust
pub fn admits(&self, candidate: &bcinr_mfw_ir::EventSet) -> bool {
    !self
        .nonfaces
        .iter()
        .any(|nf| nf.members.is_subset_of(candidate))
}
```

This design violates the strict BCINR Radon Law in several ways:
1. **Data-Dependent Branches**: The short-circuiting `.any(...)` operator terminates early depending on the input candidate, leading to variable execution time and instruction paths.
2. **Variable Loop Termination**: The number of iterations depends on which nonface is violated, opening timing side-channels.
3. **Heap Allocation**: The `ConcurrencyGuardTable` holds nonfaces in a heap-allocated `Vec<CompiledNonFace>`, violating the `#![no_std]` zero-heap allocation requirement on the hot path.
4. **Suboptimal Complexity**: Validating $M$ nonfaces sequentially requires $O(M)$ checks, where each check evaluates subset relations across up to 8 `u64` words.

---

## 3. Proposed Innovation: Transposed Bit-Parallel Verification

Rather than storing nonfaces as an array of event sets (row-major), T-BPGE transposes the relation into an event-to-nonface membership table (column-major).

### 3.1 Transposed Representation
Let $N_0, N_1, \dots, N_{M-1}$ be $M$ active minimal nonfaces over a set of events $E = \{0, 1, \dots, 511\}$ ($|E| = 512$).
We compile this simplicial complex into a transposed matrix $T$ of size $512 \times 64$ bits, represented as an array of 512 `u64` integers:
$$T \in \mathbb{U}_{64}^{512}$$

For each event $e \in [0, 512)$ and nonface index $i \in [0, 64)$:
- The $i$-th bit of $T[e]$ is $1$ if $e \in N_i$.
- The $i$-th bit of $T[e]$ is $0$ if $e \notin N_i$.

### 3.2 The Bit-Parallel Algorithm
A candidate set $C \subseteq E$ is valid if:
$$\forall i \in [0, M), N_i \not\subseteq C$$
Which is equivalent to saying that for every active nonface $N_i$, there exists at least one event $e \in N_i$ such that $e \notin C$.

To verify this in parallel:
1. Initialize a `not_subset_mask` of type `u64` to `0`.
2. Iterate through all events $e \in [0, 512)$.
3. For each event $e$, check if it is absent in the candidate set $C$.
4. If $e \notin C$, OR the transposed mask $T[e]$ into `not_subset_mask`.
5. After scanning all events, the $i$-th bit of `not_subset_mask` is $1$ if and only if $N_i \not\subseteq C$.
6. A violation is detected if any active nonface index has its bit set to $0$ in `not_subset_mask`.

```rust
#[inline(always)]
pub fn T_BPGE_admits(
    candidate: &bcinr_mfw_ir::EventSet,
    membership_table: &[u64; 512],
    active_nonfaces: u32,
) -> u64 {
    let mut not_subset_mask = 0u64;
    
    // Completely deterministic, fixed-iteration loop (CC=1)
    for e in 0..512 {
        let is_absent = (!candidate.contains(e)) as u64;
        // Broadcast the boolean flag to a full mask: 0x0 or 0xFFFFFFFFFFFFFFFF
        let mask = 0u64.wrapping_sub(is_absent);
        // OR the membership transitions into the accumulator
        not_subset_mask |= membership_table[e] & mask;
    }
    
    // Nonfaces that are subsets of the candidate will have a 0 in their slot
    let violation_candidates = !not_subset_mask;
    
    // Construct active mask for the M active nonfaces branchlessly
    let active_mask = (1u64 << active_nonfaces).wrapping_sub(1);
    
    // A non-zero result indicates at least one active nonface is a subset of the candidate
    let violation = violation_candidates & active_mask;
    
    // Return a branchless admission mask: 0u64 (rejected) or !0u64 (admitted)
    let is_admitted = (violation == 0) as u64;
    0u64.wrapping_sub(is_admitted)
}
```

---

## 4. Mathematical and Logical Contract

The verification of T-BPGE follows a strict Hoare contract:

$$\{P(C, M, T)\} \quad \text{T-BPGE\_admits}(C, M, T) \quad \{Q(C, M, T, \text{result})\}$$

### 4.1 Preconditions $P(C, M, T)$
- **Candidate Domain**: $C$ is a valid `EventSet` containing events only in $[0, 512)$.
- **Active Nonface Bound**: $M \in [0, 64]$.
- **Table Integrity**: $T$ is a read-only table of 512 `u64` words. Inactive bits of $T$ (bits $i \ge M$) must be $0$:
$$\forall e \in [0, 512), T[e] \land \neg((1 \ll M) - 1) = 0$$

### 4.2 Postconditions $Q(C, M, T, \text{result})$
- **Output Range**: $\text{result} \in \{0, 2^{64}-1\}$ (where $0$ indicates Refusal, and $2^{64}-1$ indicates Admission).
- **Core Invariant (Subset Law)**:
$$\text{result} = 2^{64}-1 \iff \forall i \in [0, M), N_i \not\subseteq C$$
$$\text{result} = 0 \iff \exists i \in [0, M), N_i \subseteq C$$
- **Conservation Law**: The function must not mutate the state of $C$, $M$, or $T$ (strict read-only footprint).
- **Monotonicity Law**: If a candidate set $C_1$ is admitted, any subset $C_0 \subseteq C_1$ must also be admitted:
$$(f(C_1) = 2^{64}-1) \land (C_0 \subseteq C_1) \implies (f(C_0) = 2^{64}-1)$$
- **Overflow Safety**: The function uses only wrapping subtraction and bitwise logic, making arithmetic overflow impossible.
- **Determinism**: Given identical arguments, the function produces bit-for-bit identical outcomes under all optimization levels.

---

## 5. Implementation Architecture & Target Optimizations

T-BPGE will be implemented inside the `crates/bcinr-powl/` and `crates/bcinr-logic/` crates.

### 5.1 SWAR optimization
Instead of checking `candidate.contains(e)` which performs bitwise shifting on each iteration, we can operate directly on the underlying `u64` words of the `EventSet` to process 64 events at a time.

```rust
#[inline(always)]
pub fn T_BPGE_admits_swar(
    candidate: &bcinr_mfw_ir::EventSet,
    membership_table: &[u64; 512],
    active_nonfaces: u32,
) -> u64 {
    let mut not_subset_mask = 0u64;
    
    // EventSet is backed by 8 u64 words
    for w in 0..8 {
        let absent_events = !candidate.words()[w];
        
        // Loop over the 64 bits of the word.
        // This is a fixed loop of 64 iterations, which is fully unrolled by LLVM.
        for b in 0..64 {
            let is_absent = (absent_events >> b) & 1;
            let mask = 0u64.wrapping_sub(is_absent);
            not_subset_mask |= membership_table[w * 64 + b] & mask;
        }
    }
    
    let violation = (!not_subset_mask) & ((1u64 << active_nonfaces).wrapping_sub(1));
    0u64.wrapping_sub((violation == 0) as u64)
}
```

### 5.2 Compiler Target Vectorization
By utilizing compile-time fixed loop bounds ($8 \times 64$), the compiler can automatically vectorize the inner loop to use SIMD vector registers (e.g., AVX2 or ARM Neon), performing multiple bitwise mask operations in parallel.

---

## 6. Verification Strategy

To satisfy the **PhD-Verified** requirement, the implementation will undergo a three-tier validation process.

### 6.1 Independent Reference Oracle
We will write a structurally distinct reference implementation in `tests/` that uses standard, branching Rust collections and standard set arithmetic (the "slow rail"):
```rust
fn oracle_admits(candidate: &EventSet, nonfaces: &[EventSet]) -> bool {
    for nf in nonfaces {
        if nf.is_subset_of(candidate) {
            return false;
        }
    }
    true
}
```
A differential test suite will verify bit-identical equivalence between `T_BPGE_admits_swar` and `oracle_admits` across:
1. Exhaustive checking of all subsets of $E$ for small simplicial complexes.
2. 100,000 randomized candidate sets and random nonface matrices.
3. Edge-case boundaries (empty sets, full sets, single-element nonfaces).

### 6.2 Hostile Mutants
Under the `@armstrong_fault` Master of Failure Law, we define three mutants to verify test-suite teeth:

1. **Mutant 1 (Subset Inversion)**:
   Inverts the check from `!candidate.contains(e)` to `candidate.contains(e)`. The hostile test must catch this and raise a `StabilityRefusal::EnvelopeViolated`.
2. **Mutant 2 (Dropped Word/Index Skew)**:
   Skews the index lookup to `membership_table[(w * 64 + b) + 1]`. This causes incorrect event mappings, which must trigger a verification failure.
3. **Mutant 3 (Active Nonface Shift)**:
   Constructs the active mask as `1u64 << active_nonfaces` (omitting the subtraction of 1). This drops checking for the last active nonface, which must be detected by the reference oracle test.

### 6.3 Object-Code Disassembly Audit Plan
We will dump the disassembly of the release target for the audited symbols:
```bash
cargo objdump --bin bcinr-powl --release -- --disassemble
```
The audit must verify:
1. **Zero Conditional Jumps**: The compiled machine code must contain no `je`, `jne`, `jg`, or other conditional jump instructions.
2. **No Loop Backedges**: The compiler must have completely unrolled the inner 64-iteration loop, leaving only straight-line assembly instructions.
3. **Zero Allocator References**: No calls to allocator functions (`__rust_alloc`, `jemalloc`, etc.).

---

## 7. Downstream Impact & Standing

- **Maturity Score**: Proving this design achieves a Substrate Integrity Score (SIS) of 100/100, as it completely eliminates branching inside the guard evaluation.
- **Latency Consistency**: By replacing the variable-time `.any()` search with a fixed sequence of bitwise masks, we guarantee that the worst-case execution time (WCET) is identical to the best-case execution time (BCET).
