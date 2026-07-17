# Innovation Proposal: Branchless Saga Rollback Engine (BSRE) for Constant-Time Transactional Recovery

## 1. Executive Summary

This proposal introduces a **Branchless Saga Rollback Engine (BSRE)** design to optimize and harden transaction compensation scheduling in the Partially Ordered Workflow Language (POWL) runtime (`crates/bcinr-powl/src/enterprise.rs`) under the strict BCINR Radon Law ($CC=1$, zero allocation, zero branching).

In distributed and high-performance transactional workflows, saga patterns handle failures by executing compensating operations in reverse order (LIFO). The baseline saga stack (`SagaStack`) relies on option-based branching pop operations and conditional rollback trigger logic. Under the BSRE architecture, these branches are completely eliminated. BSRE introduces:
1. **Branchless Gated Packed-Integer Stack Pops**: Stack state is popped branchlessly, packing the validity status and compensation index into a single `u32` integer. Pop operations are branchlessly gated by the global rollback state, ensuring stack integrity during normal forward execution.
2. **Mask-Based Scheduler Commit Operations**: Instead of looping and branching to schedule compensations, the engine maps the popped indices to execution bitmasks and merges them into the scheduler's check mask using bitwise masks and sign-extension operations, while simultaneously suppressing forward progress.

This architecture guarantees constant execution time, eliminates timing side-channels during error recovery, and ensures compliance with the $CC=1$ complexity constraint.

---

## 2. Problem Statement & Current Limitations

### 2.1 Branching in Baseline Rollback Scheduling
In traditional workflow engines, rollback trigger and recovery loops are heavily branched. A typical rollback loop resembles:

```rust
// Traditional Branching Rollback Engine
fn process_failure(&mut self, step_failed: bool) {
    if step_failed {
        self.state.rollback_active = true;
        while let Some(comp_op) = self.saga_stack.pop() {
            self.scheduler.mark_ready(comp_op);
        }
        self.scheduler.halt_forward_progress();
    }
}
```

This pattern violates several core principles of the BCINR Radon Law (Section 3 and Section 8 of `AGENTS.md`):
1. **Conditional Branches (`if` and `while let`)**: Emits CPU conditional jumps (`jne`/`je`), altering the execution path depending on whether a failure occurs and how many steps succeeded.
2. **Dynamic/Data-Dependent Loop Termination**: Loop iterations depend on the runtime depth of the saga stack, leading to variable execution times.
3. **Timing Side-Channels**: Processor speculative execution and cache line accesses vary based on the rollback depth. An external observer measuring recovery latency can deduce how many operations succeeded before failure, leaking transactional metadata.

### 2.2 Inadequate Gating
If the rollback loop is decoupled from the stack, nominal forward progress might accidentally drain or corrupt the stack. In a branchless substrate, we cannot use conditional branches to prevent pops when rollback is inactive. Gating must be achieved entirely through bitwise masking.

---

## 3. Proposed BSRE Architecture

The BSRE achieves $CC=1$ complexity and constant-time execution by combining three hardware-friendly, bit-parallel techniques:

### 3.1 Branchless Gated Packed-Integer Pops
The saga stack backing representation is extended to support a packed representation. We define a `PackedPop` as a single `u32` value:
* **Bits 16..31**: Validity mask (`0xFFFF` if valid, `0x0000` if empty or gated).
* **Bits 0..15**: Compensation operation index (or the index of the garbage sink, `32`).

To prevent the stack from being drained during nominal forward execution, the pop operation is gated by the engine's `rollback_active_mask` ($M_{\text{active}} \in \{0, 2^{64}-1\}$). The entire pop process is branchless:

```text
Let top_val = self.top
Let is_empty_bit = (top_val - 1) >> 63
Let is_valid_bit = 1 - is_empty_bit

// Gating step: valid and active (gate_mask is u64::MAX if active, 0 if inactive)
Let gated_valid_mask = gate_mask & (0 - is_valid_bit)
Let gated_valid_bit = is_valid_bit & (gate_mask & 1)

// Decrement pointer only if gated valid
self.top = self.top - gated_valid_bit

// Read index: top if gated valid, 32 if empty/inactive
Let read_idx = (self.top & gated_valid_mask) | (32 & !gated_valid_mask)
Let value = self.frames[read_idx]
```

### 3.2 Rollback Trigger Logic
Failures are captured as a boolean flag `failed` and converted to a mask without branching:
$$ M_{\text{failure}} = 0 - \text{failed} $$
The global rollback state is updated by accumulating this mask:
$$ M_{\text{active}} = M_{\text{active}} \lor M_{\text{failure}} $$

### 3.3 Mask-Based Scheduler Commit
During recovery, the engine executes a fixed sequence of exactly 32 steps (matching stack capacity). For each step:
1. Call the gated packed pop function using $M_{\text{active}}$ as the gate.
2. Sign-extend the popped validity mask `valid_mask` to a full 64-bit mask:
   $$ M_{\text{op\_valid}} = \operatorname{sign\_extend}(\text{valid\_mask}) $$
3. Construct the scheduler slot activation bitmask for the compensation op:
   $$ B_{\text{op}} = 1 \ll \text{comp\_op\_idx} $$
4. Combine the global active mask and op validity mask to form the commit mask:
   $$ M_{\text{commit}} = M_{\text{active}} \land M_{\text{op\_valid}} $$
5. Commit the activation bit to the scheduler's check mask:
   $$ \text{scheduler.check\_mask} = \text{scheduler.check\_mask} \lor (B_{\text{op}} \land M_{\text{commit}}) $$
6. Suppress forward execution progress:
   $$ \text{scheduler.fire\_mask} = \text{scheduler.fire\_mask} \land \neg M_{\text{active}} $$

---

## 4. Mathematical and Logical Contract

We formalize the behavioral contracts for the BSRE using Hoare triples. Let $R$ denote the BSRE state, $S$ the stack state, and $C$ the scheduler state.

### 4.1 Trigger Contract
$$\{P_{\text{trigger}}(R, f)\} \quad \operatorname{trigger}(R, f) \quad \{Q_{\text{trigger}}(R, f, R')\}$$

* **Preconditions $P_{\text{trigger}}$**:
  - $R.\text{rollback\_active\_mask} \in \{0, \text{u64::MAX}\}$
  - $f \in \{\text{true}, \text{false}\}$
* **Postconditions $Q_{\text{trigger}}$**:
  - If $f = \text{true}$, then $R'.\text{rollback\_active\_mask} = \text{u64::MAX}$.
  - If $f = \text{false}$, then $R'.\text{rollback\_active\_mask} = R.\text{rollback\_active\_mask}$.
  - Latency is constant and independent of the value of $f$.

### 4.2 Gated Packed Pop Contract
$$\{P_{\text{pop}}(S, M)\} \quad \operatorname{pop\_packed\_gated}(S, M) \quad \{Q_{\text{pop}}(S, M, S', \text{result})\}$$

* **Preconditions $P_{\text{pop}}$**:
  - $S.\text{top} \in [0, 32]$
  - $M \in \{0, \text{u64::MAX}\}$
* **Postconditions $Q_{\text{pop}}$**:
  - **Inactive Case ($M = 0$)**:
    - $S'.\text{top} = S.\text{top}$ (no state mutation)
    - $\operatorname{upper16}(\text{result}) = 0x0000$ (marked invalid)
    - $\operatorname{lower16}(\text{result}) = S.\text{frames}[32]$
  - **Active & Empty Case ($M = \text{u64::MAX} \land S.\text{top} = 0$)**:
    - $S'.\text{top} = 0$
    - $\operatorname{upper16}(\text{result}) = 0x0000$
    - $\operatorname{lower16}(\text{result}) = S.\text{frames}[32]$
  - **Active & Non-Empty Case ($M = \text{u64::MAX} \land S.\text{top} > 0$)**:
    - $S'.\text{top} = S.\text{top} - 1$
    - $\operatorname{upper16}(\text{result}) = 0xFFFF$
    - $\operatorname{lower16}(\text{result}) = S.\text{frames}[S'.\text{top}]$

### 4.3 Scheduler Commit Contract
$$\{P_{\text{commit}}(R, S, C)\} \quad \operatorname{rollback\_step}(R, S, C) \quad \{Q_{\text{commit}}(R, S, C, R', S', C')\}$$

* **Preconditions $P_{\text{commit}}$**:
  - $R.\text{rollback\_active\_mask} \in \{0, \text{u64::MAX}\}$
  - $S.\text{top} \in [0, 32]$
  - $C.\text{check\_mask} \in [0, 2^{64}-1]$
  - $C.\text{fire\_mask} \in [0, 2^{64}-1]$
* **Postconditions $Q_{\text{commit}}$**:
  - $S'$ transitions according to $\operatorname{pop\_packed\_gated}(S, R.\text{rollback\_active\_mask})$.
  - Let $\text{valid} = (\operatorname{upper16}(\text{result}) == 0xFFFF)$ and $\text{op} = \operatorname{lower16}(\text{result})$.
  - If $R.\text{rollback\_active\_mask} = \text{u64::MAX}$ and $\text{valid} = \text{true}$:
    - $C'.\text{check\_mask} = C.\text{check\_mask} \lor (1 \ll \text{op})$
    - $C'.\text{fire\_mask} = 0$
  - Otherwise:
    - $C'.\text{check\_mask} = C.\text{check\_mask}$
    - If $R.\text{rollback\_active\_mask} = \text{u64::MAX}$:
      - $C'.\text{fire\_mask} = 0$
    - If $R.\text{rollback\_active\_mask} = 0$:
      - $C'.\text{fire\_mask} = C.\text{fire\_mask}$

---

## 5. Proposed Rust Implementation

Below is the complete branchless implementation of the BSRE:

```rust
/// A branchless packed pop representation containing the validity mask and op index.
pub type PackedPop = u32;

/// Gated branchless stack implementation.
#[derive(Debug)]
pub struct BranchlessSagaStack {
    /// Backing frames (32 valid + 1 garbage sink at index 32).
    frames: [u16; 33],
    /// Current pointer depth (0..=32).
    top: u8,
}

impl BranchlessSagaStack {
    /// Create a new stack.
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            frames: [0u16; 33],
            top: 0,
        }
    }

    /// Push a compensation operation.
    #[inline(always)]
    pub fn push(&mut self, comp_op_idx: u16) {
        let top_val = self.top as u64;
        let diff = 32u64.wrapping_sub(top_val).wrapping_sub(1);
        let is_full_bit = diff >> 63;
        
        let mask = 0u64.wrapping_sub(is_full_bit);
        let write_idx = (top_val & !mask) | (32 & mask);
        
        self.frames[write_idx as usize] = comp_op_idx;
        self.top = self.top.wrapping_add((1u64.wrapping_sub(is_full_bit)) as u8);
    }

    /// Perform a branchless gated pop, returning a packed u32.
    ///
    /// If gate_mask is 0 (inactive), the stack remains unmodified.
    /// If gate_mask is u64::MAX (active), a standard pop is performed.
    #[inline(always)]
    pub fn pop_packed_gated(&mut self, gate_mask: u64) -> PackedPop {
        let top_val = self.top as u64;
        
        let is_empty_bit = (top_val.wrapping_sub(1)) >> 63;
        let is_valid_bit = 1u64.wrapping_sub(is_empty_bit);
        
        // Gated validity
        let gated_valid_mask = gate_mask & (0u64.wrapping_sub(is_valid_bit));
        let gated_valid_bit = is_valid_bit & (gate_mask & 1);
        
        // Mutate state branchlessly
        self.top = self.top.wrapping_sub(gated_valid_bit as u8);
        
        // Select index: top if gated valid, 32 if empty/gated out
        let read_idx = ((self.top as u64) & gated_valid_mask) | (32 & !gated_valid_mask);
        
        let value = self.frames[read_idx as usize] as u32;
        let valid_mask_upper = (gated_valid_mask as u32) & 0xFFFF_0000;
        
        valid_mask_upper | value
    }
}

/// The Branchless Saga Rollback Engine (BSRE).
#[derive(Debug)]
pub struct BranchlessSagaRollbackEngine {
    /// Mask representing rollback activity status.
    pub rollback_active_mask: u64,
}

impl BranchlessSagaRollbackEngine {
    /// Initialize the engine.
    pub const fn new() -> Self {
        Self {
            rollback_active_mask: 0,
        }
    }

    /// Trigger rollback based on step success/failure status.
    #[inline(always)]
    pub fn trigger(&mut self, failed: bool) {
        let failure_mask = 0u64.wrapping_sub(failed as u64);
        self.rollback_active_mask |= failure_mask;
    }

    /// Execute a single rollback step.
    ///
    /// This updates the scheduler's check_mask and fire_mask branchlessly.
    #[inline(always)]
    pub fn rollback_step(
        &self,
        stack: &mut BranchlessSagaStack,
        check_mask: &mut u64,
        fire_mask: &mut u64,
    ) {
        let packed = stack.pop_packed_gated(self.rollback_active_mask);
        
        let valid_mask = (packed >> 16) as u16;
        let comp_op_idx = (packed & 0xFFFF) as u16;
        
        // Sign-extend u16 mask to u64 mask via signed cast logic
        let op_mask_64 = (valid_mask as i16 as i64) as u64;
        
        // Combine active and validity masks
        let commit_mask = self.rollback_active_mask & op_mask_64;
        
        // Compute activation bit for target compensation slot
        let op_bit = 1u64 << (comp_op_idx as u64);
        
        // Commit compensation bit to scheduler check mask
        *check_mask |= op_bit & commit_mask;
        
        // Suppress forward scheduler tick execution
        *fire_mask &= !self.rollback_active_mask;
    }

    /// Execute the complete 32-step rollback engine sweep branchlessly.
    ///
    /// Fully unrolled compile-time loop guarantees CC=1 and constant execution time.
    #[inline(always)]
    pub fn rollback_sweep(
        &self,
        stack: &mut BranchlessSagaStack,
        check_mask: &mut u64,
        fire_mask: &mut u64,
    ) {
        // Fully unrolled 32 steps
        self.rollback_step(stack, check_mask, fire_mask);
        self.rollback_step(stack, check_mask, fire_mask);
        self.rollback_step(stack, check_mask, fire_mask);
        self.rollback_step(stack, check_mask, fire_mask);
        self.rollback_step(stack, check_mask, fire_mask);
        self.rollback_step(stack, check_mask, fire_mask);
        self.rollback_step(stack, check_mask, fire_mask);
        self.rollback_step(stack, check_mask, fire_mask);
        self.rollback_step(stack, check_mask, fire_mask);
        self.rollback_step(stack, check_mask, fire_mask);
        self.rollback_step(stack, check_mask, fire_mask);
        self.rollback_step(stack, check_mask, fire_mask);
        self.rollback_step(stack, check_mask, fire_mask);
        self.rollback_step(stack, check_mask, fire_mask);
        self.rollback_step(stack, check_mask, fire_mask);
        self.rollback_step(stack, check_mask, fire_mask);
        self.rollback_step(stack, check_mask, fire_mask);
        self.rollback_step(stack, check_mask, fire_mask);
        self.rollback_step(stack, check_mask, fire_mask);
        self.rollback_step(stack, check_mask, fire_mask);
        self.rollback_step(stack, check_mask, fire_mask);
        self.rollback_step(stack, check_mask, fire_mask);
        self.rollback_step(stack, check_mask, fire_mask);
        self.rollback_step(stack, check_mask, fire_mask);
        self.rollback_step(stack, check_mask, fire_mask);
        self.rollback_step(stack, check_mask, fire_mask);
        self.rollback_step(stack, check_mask, fire_mask);
        self.rollback_step(stack, check_mask, fire_mask);
        self.rollback_step(stack, check_mask, fire_mask);
        self.rollback_step(stack, check_mask, fire_mask);
        self.rollback_step(stack, check_mask, fire_mask);
        self.rollback_step(stack, check_mask, fire_mask);
    }
}
```

---

## 6. Verification Strategy

To achieve 100/100 Substrate Integrity Score (SIS) standing, the implementation will undergo verification based on three independent gates.

### 6.1 Reference Oracle
We construct an independent branching reference oracle (`SlowSagaRollbackEngine`) for differential testing. The oracle uses standard vectors and branching constructs:

```rust
struct SlowSagaRollbackEngine {
    rollback_active: bool,
    stack: Vec<u16>,
}

impl SlowSagaRollbackEngine {
    fn new() -> Self {
        Self {
            rollback_active: false,
            stack: Vec::new(),
        }
    }
    
    fn push(&mut self, op: u16) {
        if self.stack.len() < 32 {
            self.stack.push(op);
        }
    }
    
    fn trigger(&mut self, failed: bool) {
        if failed {
            self.rollback_active = true;
        }
    }
    
    fn tick(&mut self, check_mask: &mut u64, fire_mask: &mut u64) {
        if self.rollback_active {
            *fire_mask = 0;
            while let Some(op) = self.stack.pop() {
                *check_mask |= 1u64 << op;
            }
        }
    }
}
```

#### Differential Test Scenarios
A fuzzing harness runs $2^{20}$ trials generating random operational sequences (pushes, failures, ticks). The test asserts:
1. State Equivalence: $C_{\text{check\_mask}} == C'_{\text{check\_mask}}$ after each tick.
2. Suppression Equivalence: $C_{\text{fire\_mask}} == C'_{\text{fire\_mask}}$.
3. Stack Pointer Equivalence: $S.\text{top} == S'.\text{inner.len()}$ when rollback completes.

### 6.2 Hostile Mutants
Under Section 19 of `AGENTS.md`, three hostile mutants are defined to verify test coverage:
1. **Mutant 1 (Trigger Mask Failure)**: Change `trigger` mask generation to `let failure_mask = failed as u64;`.
   * *Detection*: Fails to set active mask to `u64::MAX`, leaving it at `1`. Subsequent rollback steps fail to commit, causing a mismatch in the scheduler check mask.
2. **Mutant 2 (Unconditional Pop)**: In `pop_packed_gated`, replace `self.top.wrapping_sub(gated_valid_bit)` with `self.top.wrapping_sub(is_valid_bit)`.
   * *Detection*: Stack is popped during normal forward progress (inactive rollback), leading to stack exhaustion and data corruption before actual failure.
3. **Mutant 3 (Bypassed Suppression)**: In `rollback_step`, omit the line `*fire_mask &= !self.rollback_active_mask;`.
   * *Detection*: Forward progress continues during a rollback sweep, triggering standard execution concurrent with compensations, causing a state violation.

### 6.3 Disassembly Audit Plan
The generated machine code for `rollback_sweep` must be audited in the release profile:
1. **Zero Conditional Jumps**: Ensure the assembly contains no `jne`, `je`, `jg`, or similar control-flow jumps.
2. **Straight-Line execution**: The loop must be completely unrolled, resulting in a linear sequence of `ldr`, `str`, `and`, `orr`, and `bic` instructions.
3. **No Panic Targets**: Verify that no bounds-checks or panics are compiled into the hot path.

---

## 7. Conclusion & Downstream Impact

Integrating the BSRE into the POWL runtime guarantees:
* **Side-Channel Immunity**: Deterministic timing regardless of stack depth or error state.
* **$CC=1$ Absolute Compliance**: Replaces complex rollback branches with flat, bitwise pipelines.
* **Safety Under Stress**: Garbage sink and gated pop techniques prevent heap corruption and out-of-bounds execution.
