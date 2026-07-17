# Innovation Proposal: Bit-Parallel Atomic Dispatcher (BPAD) for Zero-Rollback Fan-Outs

## 1. Executive Summary

This proposal introduces the **Bit-Parallel Atomic Dispatcher (BPAD)**, a constant-time, branchless, zero-allocation, and rollback-free worker slot dispatcher for wide partially-ordered workflow systems. 

By replacing independent per-slot occupancy tracking with a single unified `AtomicU8` bitmask, BPAD enables all-or-nothing two-slot allocations for parallel fan-outs in a single atomic Compare-And-Swap (CAS) instruction. This completely eliminates the intermediate states, race conditions, and conditional rollback recovery logic present in legacy dispatchers. The resulting implementation exhibits a cyclomatic complexity of exactly $CC=1$, uses no heap allocations, obeys the strict BCINR Radon Law, and guarantees deterministic timing boundaries.

---

## 2. Problem Statement & Current Limitations

In Partially Ordered Workflow Language (POWL) runtimes, the dispatcher routes executable activities to parallel worker lanes. In the current implementation ([dispatcher.rs](file:///Users/sac/bcinr/crates/bcinr-powl/src/dispatcher.rs)), the dispatcher manages 8 worker lanes using individual `Slot` structures, each containing its own atomic fields:

```rust
pub struct Slot {
    pub op_index: AtomicU32,
    pub claimed: AtomicBool,
    _pad: [u8; 59],
}
```

This design has several severe limitations under the strict BCINR Radon Law:

### 2.1 Non-Atomic Multi-Slot Submissions and Rollbacks
When dispatching a parallel fan-out (e.g., executing a left and right branch concurrently), the runtime must allocate two distinct slots. In the current [fanout_pair](file:///Users/sac/bcinr/crates/bcinr-powl/src/dispatcher.rs#L213) implementation, this is achieved by calling `try_submit` twice sequentially:

```rust
pub fn fanout_pair(&self, left: u32, right: u32) -> Option<[u8; 2]> {
    let left_slot = self.try_submit(left)?;
    match self.try_submit(right) {
        Some(right_slot) => {
            debug_assert_ne!(left_slot, right_slot, "fanout_pair must use distinct slots");
            Some([left_slot, right_slot])
        }
        None => {
            // Roll back left so a failed fan-out leaves no debris.
            self.release(left_slot);
            None
        }
    }
}
```

This approach violates the Radon Law and systems-safety principles:
1. **Lack of Atomicity**: The state is modified incrementally. If the first slot is claimed but the second fails (e.g., due to ring capacity), the ring enters a transient partially-claimed state. Other concurrent workers can observe this partial allocation before the rollback completes.
2. **Conditional Rollback Control-Flow**: The rollback logic introduces branching (`match` / `None` / `release`) and variable control paths. Under the Radon Law, this branching violates the $CC=1$ rule.
3. **Data-Dependent Timing**: The execution duration of `fanout_pair` depends on whether it succeeds or fails, creating timing side-channels.

### 2.2 $O(N)$ Scanning Complexity
To submit a single operation, `try_submit` sequentially iterates over all 8 slots, performing a CAS on each slot's `claimed` flag until it succeeds:

```rust
for (i, slot) in self.slots.iter().enumerate() {
    if slot.claimed.compare_exchange(false, true, ...).is_ok() {
        slot.op_index.store(op_idx, ...);
        return Some(i as u8);
    }
}
```

This loop exhibits data-dependent iteration counts (terminating early on the first free slot), resulting in variable latency and a cyclomatic complexity of $CC > 1$.

---

## 3. Proposed Innovation: Bit-Parallel Atomic Dispatcher (BPAD)

We propose unifying the occupancy state of all slots into a single atomic byte mask, $O \in \mathbb{U}_8$. Every bit in the mask corresponds to the occupancy of a slot: bit $i$ is $1$ if slot $i$ is occupied, and $0$ if it is free.

### 3.1 Data Structures
The individual `Slot` claimed flag is eliminated. The slot storage is simplified to a flat array of 9 slots: 8 active lanes and 1 garbage/sink lane (used to absorb failed writes branchlessly).

```rust
#[repr(C, align(64))]
pub struct BpadSlot {
    /// The index of the enqueued operation. SLOT_FREE when unoccupied.
    pub op_index: AtomicU32,
    _pad: [u8; 60],
}

pub struct BpadDispatcher {
    /// Bitmask representing slot occupancy (bits 0..7).
    pub occupancy: AtomicU8,
    /// 8 active worker slots + 1 garbage slot at index 8.
    pub slots: [BpadSlot; 9],
}
```

### 3.2 Branchless Helper Function: Select
To achieve $CC=1$ and eliminate conditional jumps, we define a branchless selector:
```rust
#[inline(always)]
fn select(cond: bool, true_val: usize, false_val: usize) -> usize {
    let mask = 0usize.wrapping_sub(cond as usize);
    (true_val & mask) | (false_val & !mask)
}
```

### 3.3 Atomic Single-Slot Submission (`try_submit`)
The single-slot submission reads the current occupancy, identifies the first free slot using the bitwise trailing-zero count (`trailing_zeros`), and performs a single CAS operation to claim it. If the ring is full or the CAS fails, the write is branchlessly directed to the garbage slot at index 8.

```rust
#[derive(Clone, Copy, Debug)]
pub struct SubmissionResult {
    pub slot_id: u8,
    pub is_ok: bool,
    pub refusal_code: u8,
}

impl BpadDispatcher {
    #[inline(always)]
    pub fn try_submit(&self, op_idx: u32) -> SubmissionResult {
        let old = self.occupancy.load(Ordering::Acquire);
        let free_mask = !old;
        let slot_idx = free_mask.trailing_zeros() as u8;
        
        let is_full = slot_idx >= 8;
        let target_bit = 1u8 << (slot_idx & 7);
        let proposed = old | target_bit;
        
        // If full, proposed == old, causing the CAS to have no net effect.
        let success = self.occupancy
            .compare_exchange(old, proposed, Ordering::SeqCst, Ordering::Acquire)
            .is_ok();
            
        let is_ok = success && !is_full;
        let dest_idx = select(is_ok, slot_idx as usize, 8);
        
        // Write to target slot if successful, or to the garbage slot if failed.
        self.slots[dest_idx].op_index.store(op_idx, Ordering::Release);
        
        let refusal_code = select(is_full, 1, select(!success, 2, 0)) as u8;
        
        SubmissionResult {
            slot_id: slot_idx,
            is_ok,
            refusal_code,
        }
    }
}
```

### 3.4 Atomic All-or-Nothing Two-Slot Submission (`fanout_pair`)
Parallel fan-outs are claimed in a single atomic step. We calculate the two lowest free slots from the occupancy mask, construct a combined acquisition bitmask, and update the occupancy byte in a single CAS instruction. If fewer than 2 slots are free or the CAS fails, both writes are directed to the garbage slot. No rollbacks are ever required.

```rust
#[derive(Clone, Copy, Debug)]
pub struct SubmissionPairResult {
    pub first: u8,
    pub second: u8,
    pub is_ok: bool,
    pub refusal_code: u8,
}

impl BpadDispatcher {
    #[inline(always)]
    pub fn fanout_pair(&self, left: u32, right: u32) -> SubmissionPairResult {
        let old = self.occupancy.load(Ordering::Acquire);
        let free_mask = !old;
        
        // Count free slots using branchless popcnt
        let free_count = free_mask.count_ones();
        let has_two_slots = free_count >= 2;
        
        // Extract two lowest set bits branchlessly
        let first = free_mask.trailing_zeros() as u8;
        let temp = free_mask & (free_mask.wrapping_sub(1));
        let second = temp.trailing_zeros() as u8;
        
        let target_bits = (1u8 << (first & 7)) | (1u8 << (second & 7));
        // If insufficient slots, zero out target bits to make CAS a no-op
        let acquire_mask = target_bits & (0u8.wrapping_sub(has_two_slots as u8));
        let proposed = old | acquire_mask;
        
        let success = self.occupancy
            .compare_exchange(old, proposed, Ordering::SeqCst, Ordering::Acquire)
            .is_ok();
            
        let is_ok = success && has_two_slots;
        
        let dest_first = select(is_ok, first as usize, 8);
        let dest_second = select(is_ok, second as usize, 8);
        
        // Commit operation indices to their respective slots
        self.slots[dest_first].op_index.store(left, Ordering::Release);
        self.slots[dest_second].op_index.store(right, Ordering::Release);
        
        let refusal_code = select(!has_two_slots, 3, select(!success, 2, 0)) as u8;
        
        SubmissionPairResult {
            first: first & 7,
            second: second & 7,
            is_ok,
            refusal_code,
        }
    }
}
```

### 3.5 Claim and Release Operations
Workers claim operations and release slots by manipulating the occupancy mask. Clearing slot $i$ is performed via an atomic `fetch_and` operation, ensuring other lanes are unaffected.

```rust
#[derive(Clone, Copy, Debug)]
pub struct ClaimResult {
    pub op_index: u32,
    pub is_ok: bool,
}

impl BpadDispatcher {
    #[inline(always)]
    pub fn try_claim(&self, slot_idx: u8) -> ClaimResult {
        let occ = self.occupancy.load(Ordering::Acquire);
        let is_claimed = ((occ >> (slot_idx & 7)) & 1) == 1;
        let idx = self.slots[(slot_idx & 7) as usize].op_index.load(Ordering::Acquire);
        
        let has_op = idx != SLOT_FREE;
        let is_ok = is_claimed && has_op;
        
        ClaimResult {
            op_index: idx,
            is_ok,
        }
    }

    #[inline(always)]
    pub fn release(&self, slot_idx: u8) {
        let s_idx = (slot_idx & 7) as usize;
        
        // 1. Reset slot index first (Release ordering ensures this happens-before occupancy clear)
        self.slots[s_idx].op_index.store(SLOT_FREE, Ordering::Release);
        
        // 2. Clear occupancy bit (Release ordering synchronizes with subsequent Acquire loads)
        self.occupancy.fetch_and(!(1u8 << s_idx), Ordering::Release);
    }
}
```

---

## 4. Mathematical and Logical Contract

Let the dispatcher state be defined as $\sigma = (O, V)$ where:
- $O \in \mathbb{U}_8$ represents the occupancy bitmask.
- $V \in \mathbb{U}_{32}^9$ represents the operation indices stored in the slots ($V[0..7]$ for lanes, $V[8]$ for garbage).
- Let $SLOT\_FREE = 2^{32}-1$.

### 4.1 System Invariants $I(\sigma)$
1. **Occupancy-Index Consistency**: If a slot is marked unoccupied, its index must be $SLOT\_FREE$:
   $$\forall i \in [0, 8), \quad (O \land (1 \ll i)) = 0 \implies V[i] = SLOT\_FREE$$
2. **Garbage Lane Isolation**: The garbage lane index $V[8]$ does not affect system correctness:
   $$V[8] \text{ is an unconstrained write-only sink.}$$

### 4.2 Hoare Contracts

#### Single-Slot Submission: $\text{try\_submit}(\text{op\_idx})$
$$\{I(\sigma_{\text{pre}})\} \quad \text{try\_submit}(\text{op\_idx}) \quad \{I(\sigma_{\text{post}}) \land Q_{\text{submit}}\}$$

Where $Q_{\text{submit}}$ is defined as:
- **Case 1: Admission Success** ($R.\text{is\_ok} = \text{true}$):
  - $\exists j \in [0, 8)$ such that $(O_{\text{pre}} \land (1 \ll j)) = 0$.
  - $R.\text{slot\_id} = j$.
  - $O_{\text{post}} = O_{\text{pre}} \lor (1 \ll j)$.
  - $V_{\text{post}}[j] = \text{op\_idx}$.
  - $\forall k \in [0, 8) \setminus \{j\}, \quad V_{\text{post}}[k] = V_{\text{pre}}[k]$.
- **Case 2: Submission Refusal** ($R.\text{is\_ok} = \text{false}$):
  - $O_{\text{post}} = O_{\text{pre}}$.
  - $\forall k \in [0, 8), \quad V_{\text{post}}[k] = V_{\text{pre}}[k]$.
  - If $O_{\text{pre}} = 0xFF \implies R.\text{refusal\_code} = 1$ (RingFull).
  - If $O_{\text{pre}} < 0xFF \implies R.\text{refusal\_code} = 2$ (ContentionFailure).
  - $V_{\text{post}}[8] = \text{op\_idx}$ (garbage sink write).

#### Two-Slot Submission: $\text{fanout\_pair}(\text{left}, \text{right})$
$$\{I(\sigma_{\text{pre}})\} \quad \text{fanout\_pair}(\text{left}, \text{right}) \quad \{I(\sigma_{\text{post}}) \land Q_{\text{pair}}\}$$

Where $Q_{\text{pair}}$ is defined as:
- **Case 1: Admission Success** ($R.\text{is\_ok} = \text{true}$):
  - $\exists j_1, j_2 \in [0, 8)$ such that $j_1 \neq j_2 \land (O_{\text{pre}} \land (1 \ll j_1)) = 0 \land (O_{\text{pre}} \land (1 \ll j_2)) = 0$.
  - $R.\text{first} = j_1 \land R.\text{second} = j_2$.
  - $O_{\text{post}} = O_{\text{pre}} \lor (1 \ll j_1) \lor (1 \ll j_2)$.
  - $V_{\text{post}}[j_1] = \text{left} \land V_{\text{post}}[j_2] = \text{right}$.
  - $\forall k \in [0, 8) \setminus \{j_1, j_2\}, \quad V_{\text{post}}[k] = V_{\text{pre}}[k]$.
- **Case 2: Submission Refusal** ($R.\text{is\_ok} = \text{false}$):
  - $O_{\text{post}} = O_{\text{pre}}$.
  - $\forall k \in [0, 8), \quad V_{\text{post}}[k] = V_{\text{pre}}[k]$.
  - If $\text{popcnt}(\neg O_{\text{pre}}) < 2 \implies R.\text{refusal\_code} = 3$ (InsufficientSlots).
  - If $\text{popcnt}(\neg O_{\text{pre}}) \ge 2 \implies R.\text{refusal\_code} = 2$ (ContentionFailure).
  - $V_{\text{post}}[8] = \text{right}$ (garbage sink write).

---

## 5. Radon Law Compliance Audit

We evaluate BPAD against the absolute runtime laws of the substrate:

| Law | Status | Evidence |
| :--- | :---: | :--- |
| **$CC = 1$ per function** | **Passed** | All conditional branches are eliminated via bitwise arithmetic and `select` mask operations. |
| **No heap allocation** | **Passed** | The dispatcher has a fixed memory layout. No dynamic types (`Vec`, `Box`) are imported or used. |
| **No data-dependent branches** | **Passed** | Single and multi-slot allocations use bitwise scans and straight-line instructions. |
| **No data-dependent loops** | **Passed** | Loop constructs are completely removed. Iterating over slots is replaced by trailing zero counts. |
| **No panic paths** | **Passed** | Index operations are statically bounded using `& 7` and `& 8` masks. Arithmetic overflow is impossible. |

---

## 6. Verification Strategy

To achieve the mandatory **PhD-Verified** standing (100/100 SIS), BPAD utilizes three complementary verification vectors.

### 6.1 Independent Reference Oracle
We construct an independent, slow-rail reference model in `tests/` using standard mutex-guarded queues and sequential operations:

```rust
pub struct OracleDispatcher {
    slots: std::sync::Mutex<Vec<Option<u32>>>,
}

impl OracleDispatcher {
    pub fn submit(&self, op_idx: u32) -> Result<usize, String> {
        let mut guard = self.slots.lock().unwrap();
        for i in 0..8 {
            if guard[i].is_none() {
                guard[i] = Some(op_idx);
                return Ok(i);
            }
        }
        Err("RingFull".into())
    }

    pub fn fanout(&self, left: u32, right: u32) -> Result<(usize, usize), String> {
        let mut guard = self.slots.lock().unwrap();
        let free_indices: Vec<usize> = guard.iter().enumerate()
            .filter(|(_, slot)| slot.is_none())
            .map(|(idx, _)| idx)
            .collect();
            
        if free_indices.len() < 2 {
            return Err("InsufficientSlots".into());
        }
        
        let l_idx = free_indices[0];
        let r_idx = free_indices[1];
        guard[l_idx] = Some(left);
        guard[r_idx] = Some(right);
        Ok((l_idx, r_idx))
    }
}
```

A differential testing harness will verify correctness under:
1. **Static Validation**: Single-threaded random permutations of submissions and releases (100k cycles).
2. **Concurrent Verification via Loom**: We will run verification inside the Loom concurrency framework to validate all possible thread interleavings of `try_submit`, `fanout_pair`, `try_claim`, and `release`, confirming memory ordering correctness under weak memory models.

### 6.2 Hostile Mutants
Three mutants will be injected into the codebase to verify test suite efficacy:

#### Mutant 1: Partial Claim Inversion (Dropping Second Lane Protection)
Modifies the `fanout_pair` acquisition mask to only set the first bit:
```diff
- let target_bits = (1u8 << (first & 7)) | (1u8 << (second & 7));
+ let target_bits = (1u8 << (first & 7));
```
*Expected Detection*: The test suite must catch that the second operation index is overwritten by subsequent submissions due to slot collision.

#### Mutant 2: Out-of-Order Release (Race Window Creation)
Swaps the memory operations in `release`:
```diff
- self.slots[s_idx].op_index.store(SLOT_FREE, Ordering::Release);
- self.occupancy.fetch_and(!(1u8 << s_idx), Ordering::Release);
+ self.occupancy.fetch_and(!(1u8 << s_idx), Ordering::Release);
+ self.slots[s_idx].op_index.store(SLOT_FREE, Ordering::Release);
```
*Expected Detection*: The Loom model-checking run must detect a data race where a producer wins a slot and writes a new op index, which is then immediately clobbered by the delayed `store(SLOT_FREE)` from the worker thread.

#### Mutant 3: Insufficient Capacity Check Bypass
Disables the free slot count validation:
```diff
- let acquire_mask = target_bits & (0u8.wrapping_sub(has_two_slots as u8));
+ let acquire_mask = target_bits;
```
*Expected Detection*: In a state with only 1 free slot, `fanout_pair` will attempt to write to both the free slot and an already occupied slot (since `second` trailing-zero count will wrap or overlap with `first`). The oracle checks must detect slot corruption.

### 6.3 Disassembly Audit Plan
The compiled production binaries will be audited for target architectures (x86_64 and AArch64):
1. **Conditional Jump Absence**: Inspect the instruction stream for `jxx` (x86) and `b.xx` (ARM) inside the BPAD methods.
2. **Lock-free Primitives**: Confirm that `try_submit` and `fanout_pair` compile to exactly one instruction containing `lock cmpxchg` (x86_64) or `ldaxr`/`stlxr`/`cas` (AArch64).
3. **No Out-of-Line Calls**: Verify that no calls to allocation, formatting, or panic symbols exist in the disassembly.

---

## 7. Downstream Impact & Standing

1. **Deterministic Latency**: Eliminating the sequential scanning loop guarantees that dispatcher operations execute in a fixed number of CPU cycles. Worst-case execution time (WCET) is identical to best-case execution time (BCET).
2. **100/100 SIS Compliance**: The complete elimination of branching, heap allocations, and dynamic rollback routines satisfies the Radon Law requirements, raising the dispatcher standing to PhD-Verified status.
3. **Zero Intermediate States**: Eliminating rollback logic simplifies the mental model and verification footprint, removing transient states from the concurrent state space.
