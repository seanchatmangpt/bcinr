# Innovation Proposal: Bit-Parallel Concurrency Guard Gating (BP-CGG)

## 1. Executive Summary

This proposal introduces the **Bit-Parallel Concurrency Guard Gating (BP-CGG)** mechanism, a constant-time, branch-free, and allocation-free algorithm for checking process concurrency guards and performing greedy stable-maximal concurrency selection.

By transposing the representation of minimal nonfaces from a list of heap-allocated event sets to a compact, cache-aligned event-to-nonface membership matrix, BP-CGG eliminates the need for:
1. Short-circuiting `.any()` iteration over nonfaces in `ConcurrencyGuardTable::admits`.
2. Variable data-dependent loops over ready candidates in `StableMaximalSelector::select`.
3. Speculative heap allocations (`Vec`) in the hot-path scheduler.

The resulting implementation satisfies the strict **BCINR Radon Law** ($CC=1$, zero allocation, zero data-dependent branches, zero data-dependent loop termination) with mathematical determinism and timing-channel immunity.

---

## 2. Problem Statement & Current Limitations

Concurrency-aware scheduling restricts which ready operations can fire simultaneously based on minimal nonfaces. The current implementation in `crates/bcinr-powl/src/tape.rs` defines:

```rust
pub struct ConcurrencyGuardTable {
    pub nonfaces: Vec<CompiledNonFace>,
}

impl ConcurrencyGuardTable {
    pub fn admits(&self, candidate: &bcinr_mfw_ir::EventSet) -> bool {
        !self
            .nonfaces
            .iter()
            .any(|nf| nf.members.is_subset_of(candidate))
    }
}
```

And in `crates/bcinr-powl/src/scheduler.rs`:

```rust
impl ConcurrencySelector for StableMaximalSelector {
    fn select(&mut self, ready: &EventSet, guards: &ConcurrencyGuardTable) -> EventSet {
        let mut selected = EventSet::empty();
        for id in ready.iter_stable() {
            let candidate = selected.with(id);
            if guards.admits(&candidate) {
                selected = candidate;
            }
        }
        selected
    }
}
```

This architecture introduces four critical violations of the **BCINR Deterministic Substrate Constitution**:
1. **Heap Allocation**: `ConcurrencyGuardTable` relies on `Vec<CompiledNonFace>`, requiring dynamic memory management in an environment where `#![no_std]` and `no alloc` are transitively mandatory.
2. **Data-Dependent Branching**: `guards.admits` uses `.any(...)` which terminates early (short-circuits) based on the input candidate, introducing severe timing variance and conditional jumps in assembly.
3. **Data-Dependent Loop Termination**: `StableMaximalSelector::select` iterates over `ready.iter_stable()`, whose iteration count depends on the number of set bits (popcount) of the ready set.
4. **Logic Branching**: The `if guards.admits(&candidate)` block in `StableMaximalSelector::select` generates a conditional jump instruction, directly violating the Radon Law ($CC=1$).

---

## 3. Proposed Innovation: Bit-Parallel Concurrency Guard Gating (BP-CGG)

The BP-CGG mechanism transposes the relation between tape slots (events) and minimal nonfaces, representing it as a compact, bit-sliced matrix.

### 3.1 Transposed Bitwise Matrix
Since a POWL v2 tape contains at most 64 slots, the index of any ready or firing operation is bounded in $[0, 64)$. Consequently:
- Any minimal nonface can be represented as a single `u64` bitmask.
- The guard table can store up to 64 active nonfaces.

We compile this complex into a transposed matrix $T$ of size $64 \times 64$ bits, represented as an array of 64 `u64` values:
```rust
#[repr(C, align(64))]
pub struct ConcurrencyGuardTable {
    /// Columns of the transposed membership matrix.
    /// cols[e] has bit j set iff tape slot e is a member of nonface j.
    pub cols: [u64; 64],
    /// Bitmask of active nonfaces. Bit j is set iff nonface j is defined.
    pub active_mask: u64,
}
```
This structure occupies exactly 520 bytes, fits on the stack or inline in the tape, and requires **zero heap allocation**.

### 3.2 Branch-Free Bit-Parallel Reduction Sweep
Let $C \in \mathbb{U}_{64}$ be the candidate mask (where bit $e$ is set if slot $e$ is in the candidate set).
Let $U = \sim C$ be the mask of slots *absent* from the candidate set.

A candidate $C$ is admitted if and only if no active nonface is a subset of $C$. A nonface $N_j$ is not a subset of $C$ if there is at least one slot $e \in N_j$ that is absent from $C$.

BP-CGG computes this condition for all 64 nonfaces in parallel:
1. For each slot $e$ absent from $C$, OR the transposed column `cols[e]` into an accumulator $S$.
2. The $j$-th bit of $S$ is 1 if and only if nonface $N_j \not\subseteq C$.
3. Check if all active nonfaces are not subsets: `unmet = active_mask & !S`.
4. If `unmet == 0`, the candidate is admitted.

```rust
impl ConcurrencyGuardTable {
    /// Determines branchlessly if the candidate set is admitted.
    /// Returns `u64::MAX` if admitted, `0` if rejected.
    #[inline(always)]
    pub fn admits_mask(&self, candidate: u64) -> u64 {
        let u = !candidate;
        let mut s = 0u64;
        
        // Constant-time loop (64 iterations), fully unrolled by the compiler.
        // No data-dependent termination, CC=1.
        for e in 0..64 {
            let bit = (u >> e) & 1;
            let mask = 0u64.wrapping_sub(bit); // 0x0 or 0xFFFFFFFFFFFFFFFF
            s |= mask & self.cols[e];
        }
        
        let unmet = self.active_mask & !s;
        0u64.wrapping_sub((unmet == 0) as u64)
    }
}
```

### 3.3 Branchless Concurrency Selector
With `admits_mask` returning a full-width mask, the greedy concurrency selection in `StableMaximalSelector` can be rewritten to eliminate all conditional branches and data-dependent loops:

```rust
impl StableMaximalSelector {
    /// Greedy choice of maximal subset of `ready_mask` branchlessly.
    /// Operates entirely on u64 masks in O(1) time.
    #[inline(always)]
    pub fn select_branchless(&mut self, ready_mask: u64, guards: &ConcurrencyGuardTable) -> u64 {
        let mut selected = 0u64;
        
        // Loop runs exactly 64 times under all inputs.
        // No branching or short-circuiting.
        for e in 0..64 {
            let bit = 1u64 << e;
            let is_ready = (ready_mask >> e) & 1;
            let is_ready_mask = 0u64.wrapping_sub(is_ready);
            
            // Speculatively construct candidate set containing e
            let candidate_next = selected | bit;
            
            // Query guard table branchlessly
            let admitted = guards.admits_mask(candidate_next);
            
            // Commit slot e iff e was ready AND candidate_next is admitted
            selected |= bit & is_ready_mask & admitted;
        }
        
        selected
    }
}
```

---

## 4. Mathematical and Logical Contract

The BP-CGG implementation satisfies a strict Hoare logic contract.

### 4.1 Hoare Contract Representation
$$\{P(C, T)\} \quad \text{admits\_mask}(C, T) \quad \{Q(C, T, \text{result})\}$$

### 4.2 Preconditions $P(C, T)$
- **Candidate Set Boundaries**: $C \in [0, 2^{64}-1]$ (well-defined 64-bit integer representation of candidate events).
- **Matrix Consistency**: The transposed matrix $T = \text{self.cols}$ consists of 64 `u64` words. Inactive bits of columns (bits $j$ where $\text{active\_mask}_j = 0$) must be zero:
  $$\forall e \in [0, 64), T[e] \land \neg \text{active\_mask} = 0$$

### 4.3 Postconditions $Q(C, T, \text{result})$
- **Output Range**: $\text{result} \in \{0, 2^{64}-1\}$ (where $0$ indicates Refusal, and $2^{64}-1$ indicates Admission).
- **Correctness Law (Sub-complex Preservation)**:
  $$\text{result} = 2^{64}-1 \iff \forall j \in [0, 64) \text{ s.t. } \text{active\_mask}_j = 1, \exists e \in [0, 64) \text{ s.t. } (T[e] \ \& \ (1 \ll j) \neq 0) \land (C \ \& \ (1 \ll e) = 0)$$
  This guarantees that the candidate set $C$ contains no active minimal nonface as a subset.
- **State Preservation**: The execution has zero side-effects on the inputs:
  $$\text{self.cols}_{t+1} = \text{self.cols}_t \quad \land \quad C_{t+1} = C_t$$
- **Monotonicity Law**: If a candidate set $C_1$ is admitted, any subset $C_0 \subseteq C_1$ (meaning $C_0 \& !C_1 == 0$) must also be admitted:
  $$(\text{admits\_mask}(C_1, T) == \text{MAX}) \land (C_0 \subseteq C_1) \implies (\text{admits\_mask}(C_0, T) == \text{MAX})$$
- **Timing Invariance**: The execution time and instruction footprint are bit-for-bit invariant with respect to the values of $C$ and $T$.

---

## 5. Verification Strategy

Following the mandatory decomposition protocol in `AGENTS.md`, the implementation is verified using independent validation tiers.

### 5.1 Independent Reference Oracle
A structurally distinct specification will be written in `tests/` using standard, branching Rust collections and standard set arithmetic (the slow rail):

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

A differential testing harness will generate 50,000 randomized candidate masks and active nonface matrices to assert bit-identical equivalence:
$$\text{oracle\_admits}(\text{mask\_to\_set}(C), \text{active\_nonfaces}(T)) \iff \text{guards.admits\_mask}(C) == \text{MAX}$$

### 5.2 Hostile Mutants
Under the `@armstrong_fault` Master of Failure Law, three mutants will be injected into the implementation to verify test adequacy:

1. **Mutant 1 (Incorrect Absent Mask)**:
   Change `let u = !candidate` to `let u = candidate`. This tests whether the logic fails when verifying present instead of absent slots.
   *Expected result*: Differential test catches output discrepancy and triggers `StabilityRefusal::EnvelopeViolated`.
   
2. **Mutant 2 (Off-by-one Column Shift)**:
   Change `cols[e]` to `cols[(e + 1) & 63]`. This shifts the event-to-nonface mapping by 1.
   *Expected result*: Refusal behavior mismatch against the reference oracle.
   
3. **Mutant 3 (Symmetric Selection Committal)**:
   Change the branchless commit in `select_branchless` from:
   `selected |= bit & is_ready_mask & admitted;`
   to:
   `selected |= bit & admitted;` (dropping the ready check).
   *Expected result*: Selected set contains non-ready slots, violating the subset postcondition check:
   `selected.is_subset_of(ready)`.

### 5.3 Object-Code Disassembly Audit Plan
We will analyze the compiled assembly for target platforms (x86_64, aarch64) in the release profile:
```bash
cargo objdump --bin bcinr-powl --release -- --disassemble --symbol="admits_mask"
```
The disassembly must show:
1. **Zero Conditional Jumps**: Total count of conditional jump instructions (`je`, `jne`, `jg`, etc.) in the audited symbols must be exactly $0$.
2. **No Loop Backedges**: The 64-iteration loops in both `admits_mask` and `select_branchless` must be fully unrolled by the compiler, leaving only straight-line logic.
3. **No Unwind / Panic Blocks**: The assembly must contain no references to core panic handlers or unwinding tables, proving all bounds checks are optimized away.
4. **No Allocator Links**: Total absence of malloc/alloc symbols in the transitive call graph.

---

## 6. Implementation Architecture

The BP-CGG structures will be integrated in:
1. [tape.rs](file:///Users/sac/bcinr/crates/bcinr-powl/src/tape.rs):
   Replace the heap-allocated `Vec<CompiledNonFace>` inside `ConcurrencyGuardTable` with the transposed matrix representation.
2. [scheduler.rs](file:///Users/sac/bcinr/crates/bcinr-powl/src/scheduler.rs):
   Modify `scheduler_tick_guarded` to accept `u64` masks, and update `StableMaximalSelector` to use `select_branchless`.
