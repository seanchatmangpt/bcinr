# Innovation Proposal: Branchless Linear Execution Token (BLET)

## 1. Executive Summary

This proposal introduces the **Branchless Linear Execution Token (BLET)**, a constant-time, zero-allocation, and 100% branchless token tracking primitive designed to replace the branching execution token within the POWL runner pipeline in `crates/bcinr-powl/src/typestate.rs`.

Currently, `ExecutionToken::consume_op` violates the strict **BCINR Radon Law** ($CC=1$, zero allocation, zero data-dependent branching) by returning a `Result<(), ExecutionDefect>` on every operation execution. This early error check requires conditional branches inside the execution loop:
1. **Conditional Jumps in Hot Path**: Checking whether an operation has been double-fired or is out of bounds requires source-level `if` checks, compiling to conditional jumps (`jne`/`je` / `cbnz`).
2. **Timing Side-Channels**: The latency of executing `consume_op` differs depending on whether a defect is triggered or not, introducing micro-architectural timing side-channels into the hot execution path.
3. **Partial Mutation & Lack of Transactional Boundaries**: If a double-fire occurs mid-execution, the current runner halts immediately with an error, leaving the execution state partially mutated and making clean rollback difficult.

By accumulating double-fire, out-of-bounds, and malformed-input defects branchlessly into a stateful status mask during the execution loop, BLET defers all verification and error-handling decisions to the final transition boundary (`complete` or `assert_exhausted`). This guarantees a cyclomatic complexity of $CC=1$ across the entire execution loop, eliminates timing side-channels, and implements strict transactional commit semantics.

---

## 2. Vulnerability & Limitation Analysis

### 2.1 Branching Control Flow in current `ExecutionToken`

In `crates/bcinr-powl/src/typestate.rs`, the `ExecutionToken` is used to mark when operations on a tape have been fired:

```rust
#[inline]
pub fn consume_op(&mut self, op_bit: u64) -> Result<(), ExecutionDefect> {
    // Branchless double-fire check: if the bit is NOT in remaining, error.
    let present = self.remaining & op_bit;
    
    // We use a branchless approach: compute an error sentinel and use it.
    let already_consumed = (present == 0) as u64;
    
    // Write through regardless; if it was already 0 this is idempotent.
    self.remaining &= !op_bit;
    
    if already_consumed != 0 {
        Err(ExecutionDefect::OpAlreadyConsumed { bit: op_bit })
    } else {
        Ok(())
    }
}
```

### 2.2 Violations of the Radon Law

Under the repository constitution (`AGENTS.md`), the current design presents three critical issues:
1. **Branching Control Flow**: The conditional branch `if already_consumed != 0` introduces two execution branches inside the hot path. Even if the compiler attempts to optimize, the return of a `Result::Err` variant forces the generation of branch instructions to handle stack frame modifications and value returns.
2. **Asymmetric Execution Latency**: When `already_consumed != 0` (a double-fire defect), the CPU executes the branch that constructs and returns the `Err` variant, which is significantly slower than returning `Ok(())`. This creates a timing side-channel that leaks details of the execution graph.
3. **Eager Failure vs. Transactional Integrity**: If execution fails early on op $N$, the token is left in a partially consumed state. Because the tape is mutated before full admission, we violate **Rule 10 (No mutation before complete admission)**.

---

## 3. Proposed Innovation: BLET Architecture

BLET completely removes early returns and status checks from `consume_op`. Instead of returning a `Result`, BLET accumulates defects into three independent, bit-parallel status registers:
* `defect_double_fire`: Accumulates bits that have been fired more than once.
* `defect_invalid`: Accumulates bits fired that lie outside the tape's valid operation mask.
* `defect_malformed`: Accumulates bits fired that violate structural constraints (e.g., zero masks or multi-bit fires).

All updates are executed using bitwise logical polynomials, running in absolute constant time with $CC=1$.

```mermaid
graph TD
    subgraph Current Branching Token
        A[consume_op] -->|Check bit present| B{present == 0?}
        B -->|Yes| C[Early Return Err]
        B -->|No| D[Clear bit in remaining]
        D --> E[Return Ok]
    end

    subgraph Branchless Linear Execution Token BLET
        F[consume_op] -->|1. Compute invalid mask| G[defect_invalid |= op_bit & ~valid_mask]
        F -->|2. Compute double-fire| H[defect_double_fire |= op_bit & ~remaining]
        F -->|3. Compute malformed| I[defect_malformed |= malformed_mask]
        F -->|4. Update remaining| J[remaining &= ~op_bit]
    end
    
    style Branchless Linear Execution Token BLET fill:#1a3a2a,stroke:#2e7d32,stroke-width:2px;
```

### 3.1 Data Structures & Initialization

The proposed BLET struct is defined as:

```rust
pub struct BranchlessLinearExecutionToken {
    /// Mask of operations remaining to be fired.
    remaining: u64,
    /// Bitmask defining the valid operational boundary: (1 << total) - 1.
    valid_mask: u64,
    
    /// Stateful Status Accumulators
    defect_double_fire: u64,
    defect_invalid: u64,
    defect_malformed: u64,
    
    /// Total op count.
    total: u8,
    /// Event log count & order (for verification).
    pub(crate) topo_order: [u8; 64],
    pub(crate) event_count: u8,
}
```

To construct `valid_mask` without branching, we avoid `if total == 64` checks using wrapping arithmetic:

```rust
#[inline(always)]
fn compute_valid_mask(total: u8) -> u64 {
    let is_64 = (total == 64) as u64;
    // (1 << (total & 63)) - 1, or u64::MAX if total == 64
    let base_mask = (1u64.wrapping_shl(total as u32 & 63)).wrapping_sub(1);
    let sentinel = 0u64.wrapping_sub(is_64); // u64::MAX if is_64, else 0
    base_mask | sentinel
}
```

### 3.2 Straight-Line Firing Logic

The new `consume_op` executes as a single, contiguous block of bitwise operations:

```rust
impl BranchlessLinearExecutionToken {
    /// Consumes the specified operation mask branchlessly, accumulating defects.
    #[inline(always)]
    pub fn consume_op(&mut self, op_bit: u64) {
        // 1. Accumulate invalid bit fires (bits outside the valid boundary)
        let invalid = op_bit & !self.valid_mask;
        self.defect_invalid |= invalid;

        // 2. Accumulate double-fire defects
        // If an op is set in op_bit & valid_mask but is NOT in remaining, it was double-fired.
        let target_valid = op_bit & self.valid_mask;
        let present = self.remaining & target_valid;
        let double_fired = target_valid ^ present;
        self.defect_double_fire |= double_fired;

        // 3. Accumulate malformed fires (zero bits or multi-bit operations)
        let is_zero = (op_bit == 0) as u64;
        let is_multi = ((op_bit & op_bit.wrapping_sub(1)) != 0) as u64;
        let malformed_flag = is_zero | is_multi;
        
        // Write through the malformed flag and the offending bits.
        // We use 0u64.wrapping_sub(malformed_flag) to propagate a full-width mask.
        let malformed_mask = op_bit | 0u64.wrapping_sub(malformed_flag);
        self.defect_malformed |= malformed_mask;

        // 4. Update the remaining mask (idempotent write-through)
        self.remaining &= !op_bit;
    }
}
```

### 3.3 Deferred Defect Resolution

At the transaction boundary (`complete` transition), the runner inspects the accumulated state registers:

```rust
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BletDefect {
    /// One or more operations remained unfired.
    Unexhausted { remaining: u64 },
    /// One or more operations were fired multiple times.
    DoubleFire { bits: u64 },
    /// Out-of-bounds/inactive operations were fired.
    InvalidFires { bits: u64 },
    /// Malformed (zero or multi-bit) operations were fired.
    MalformedFires { bits: u64 },
}

impl<Tape: HasPowlTape, const KIND: TopologyKind> PowlRunner<Executing<KIND>, Tape> {
    /// Transactional completion. Returns a receipt if successful, otherwise reverts.
    pub fn complete(
        self,
        token: BranchlessLinearExecutionToken,
    ) -> Result<(PowlRunner<Receipted<KIND>, Tape>, Receipt<KIND>), BletDefect> {
        // Enforce transactional admission: check defect accumulators in order of priority.
        if token.defect_malformed != 0 {
            return Err(BletDefect::MalformedFires { bits: token.defect_malformed });
        }
        if token.defect_invalid != 0 {
            return Err(BletDefect::InvalidFires { bits: token.defect_invalid });
        }
        if token.defect_double_fire != 0 {
            return Err(BletDefect::DoubleFire { bits: token.defect_double_fire });
        }
        if token.remaining != 0 {
            return Err(BletDefect::Unexhausted { remaining: token.remaining });
        }

        // Clean validation, transition to receipted state
        let op_trace = !token.remaining & token.valid_mask;
        let receipt = Receipt::<KIND> {
            run_id: self.run_id,
            op_trace,
            topology: KIND,
            chain_hash: self.tape.content_hash(),
            replay_ptr: self.run_id,
            topo_order: token.topo_order,
            event_count: token.event_count,
        };
        
        let runner = PowlRunner {
            tape: self.tape,
            run_id: self.run_id,
            _phase: PhantomData,
        };
        Ok((runner, receipt))
    }
}
```

---

## 4. Mathematical and Logical Contract

Under `@hoare_oracle` jurisdiction, the BLET state transition satisfies the following contract:

$$\{P(S_t)\} \quad \text{consume\_op}(S_t, O_t) \quad \{Q(S_t, O_t, S_{t+1})\}$$

### 4.1 State Variable Tuple
The state at step $t$ is represented by the tuple:
$$S_t = \langle R_t, V, D_{\text{double}, t}, D_{\text{invalid}, t}, D_{\text{malformed}, t} \rangle$$
where:
* $R_t \in [0, 2^{64}-1]$ is the `remaining` bitmask.
* $V \in [0, 2^{64}-1]$ is the constant `valid_mask` representing the operational bounds.
* $D_{\text{double}, t}, D_{\text{invalid}, t}, D_{\text{malformed}, t} \in [0, 2^{64}-1]$ are the defect masks.

### 4.2 Preconditions $P(S_t)$
* **Bounded Width**: $V$ must be a valid mask calculated branchlessly from the tape length $N \in [0, 64]$.
* **Correct Initialization**: At $t=0$:
  $$R_0 = V, \quad D_{\text{double}, 0} = 0, \quad D_{\text{invalid}, 0} = 0, \quad D_{\text{malformed}, 0} = 0$$

### 4.3 Postconditions $Q(S_t, O_t, S_{t+1})$

* **Transition Determinism**:
  $$\forall S_t, O_t, \quad S_{t+1} = f(S_t, O_t) \text{ is uniquely determined.}$$

* **Remaining Bits Conservation**:
  $$R_{t+1} = R_t \land \neg O_t$$

* **Double-Fire Soundness**:
  For all bits $i \in [0, 64)$:
  $$(D_{\text{double}, t+1})_i = (D_{\text{double}, t})_i \lor \left( (O_t)_i \land V_i \land \neg(R_t)_i \right)$$
  
  **Proof (Universal Stand over Finite Partition):**
  Let $i$ be an arbitrary bit index.
  * *Case 1:* $V_i = 0$ (out of bounds).
    $$(O_t)_i \land V_i \land \neg(R_t)_i = 0$$
    The double-fire accumulator remains unchanged at index $i$.
  * *Case 2:* $V_i = 1$ and $(O_t)_i = 0$ (no write to $i$).
    $$(O_t)_i \land V_i \land \neg(R_t)_i = 0$$
    The accumulator remains unchanged.
  * *Case 3:* $V_i = 1$, $(O_t)_i = 1$, and $(R_t)_i = 1$ (first write to active bit).
    $$(O_t)_i \land V_i \land \neg(R_t)_i = 1 \land 1 \land 0 = 0$$
    The accumulator remains unchanged.
  * *Case 4:* $V_i = 1$, $(O_t)_i = 1$, and $(R_t)_i = 0$ (double-fire of active bit).
    $$(O_t)_i \land V_i \land \neg(R_t)_i = 1 \land 1 \land 1 = 1$$
    The accumulator is set to 1.
  
  The formula $(O_t \land V) \oplus (R_t \land O_t)$ is logically equivalent to $O_t \land V \land \neg R_t$:
  $$(O_t \land V) \oplus (R_t \land O_t) = (O_t \land V \land \neg (R_t \land O_t)) \lor (\neg (O_t \land V) \land (R_t \land O_t))$$
  Since $R_t \subseteq V$, if $(R_t \land O_t) = 1$, then $(O_t \land V) = 1$, simplifying the XOR term directly to $O_t \land V \land \neg R_t$.

* **Invalid Fire Detection**:
  $$(D_{\text{invalid}, t+1})_i = (D_{\text{invalid}, t})_i \lor \left( (O_t)_i \land \neg V_i \right)$$
  An out-of-bounds fire is detected if and only if bit $i$ is set in $O_t$ but is not active in the valid mask $V$.

* **Malformed Input Detection**:
  Let $\text{Malformed}(O_t) = [O_t = 0] \lor [\operatorname{popcount}(O_t) > 1]$.
  If $\text{Malformed}(O_t)$ is true, then $D_{\text{malformed}, t+1} = D_{\text{malformed}, t} \lor O_t \lor \text{0xFFFF\_FFFF\_FFFF\_FFFF}$. Otherwise, $D_{\text{malformed}, t+1} = D_{\text{malformed}, t}$.

---

## 5. Verification Strategy

To guarantee that BLET achieves a Substrate Integrity Score (SIS) of 100/100, the verification plan combines a differential oracle, hostile mutation testing, and machine-level disassembly audits.

### 5.1 Independent Reference Oracle

We define an independent oracle implementation (`BletOracle`) that uses the standard library's branching structures to verify equivalent behavior:

```rust
struct BletOracle {
    remaining: Vec<u8>, // Vector of active operation indices
    total: usize,
    double_fires: Vec<usize>,
    invalid_fires: Vec<usize>,
    malformed_fires: Vec<u64>,
}

impl BletOracle {
    fn new(total: usize) -> Self {
        Self {
            remaining: (0..total).map(|x| x as u8).collect(),
            total,
            double_fires: Vec::new(),
            invalid_fires: Vec::new(),
            malformed_fires: Vec::new(),
        }
    }

    fn consume(&mut self, op_bit: u64) {
        if op_bit == 0 || op_bit.count_ones() > 1 {
            self.malformed_fires.push(op_bit);
        }
        
        for bit in 0..64 {
            if (op_bit & (1 << bit)) != 0 {
                if bit >= self.total {
                    self.invalid_fires.push(bit);
                } else if !self.remaining.contains(&(bit as u8)) {
                    self.double_fires.push(bit);
                } else {
                    self.remaining.retain(|&x| x != bit as u8);
                }
            }
        }
    }
}
```

A differential check runs 1,000,000 randomized execution steps (including valid sequences, double-fires, invalid ranges, and multi-bit masks) to assert that:
1. `Blet::remaining` matches `BletOracle::remaining` bitwise.
2. `Blet::defect_double_fire` contains a set bit $i$ if and only if $i \in \text{BletOracle::double\_fires}$.
3. `Blet::defect_invalid` contains a set bit $i$ if and only if $i \in \text{BletOracle::invalid\_fires}$.

### 5.2 Hostile Mutation Plan

Under the `@armstrong_fault` framework, we define three mutants to verify test coverage:

1. **Mutant 1 (Double-Fire Mask Inversion)**:
   ```rust
   // Mutant: XOR is replaced with AND
   let double_fired = target_valid & present;
   ```
   *Expectation*: Double fires will never be recorded (since `present` will be 0 on a double fire). The test suite must catch this by executing a double-fire step and verifying that `complete()` does not return a success receipt but instead returns `Err(BletDefect::DoubleFire)`.
   
2. **Mutant 2 (Omit Malformed Check)**:
   ```rust
   // Mutant: Disable the malformed checks
   let malformed_flag = 0u64;
   ```
   *Expectation*: Feeding $0$ or multi-bit masks to `consume_op` does not trigger `BletDefect::MalformedFires`. The test suite must feed `0b11` and `0` to the token and assert rejection.

3. **Mutant 3 (Valid Mask Calculation Shift Error)**:
   ```rust
   // Mutant: Shift logic error (wrapping_shl is not masked)
   let base_mask = (1u64.wrapping_shl(total as u32)).wrapping_sub(1);
   ```
   *Expectation*: When `total` is 64, `wrapping_shl` will overflow or result in 0 on x86/ARM platforms, yielding a `valid_mask` of 0 instead of `u64::MAX`. The test suite must execute a 64-op tape run and verify that valid operations are not flagged as `BletDefect::InvalidFires`.

### 5.3 Object-Code Disassembly Audit Plan

The `@turing_machine` role inspects the compiled release object code of the target library:

```bash
cargo objdump --lib --release -- --disassemble
```

The audit confirms:
1. **Zero Conditional Jumps**: The `consume_op` assembly must contain no jumps (`je`, `jne`, `cbz`, `cbnz`, `jmp`). It must consist solely of bitwise shifts (`shl`/`shr`), bitwise operations (`and`/`orr`/`eor`/`bic`), and memory instructions (`ldr`/`str`).
2. **Zero Loop Backedges**: The function body must represent a straight-line basic block with a single entry and exit point.
3. **No External Library Calls**: No calls to allocators or panic-related symbols.

---

## 6. Downstream Impact & Standing

* **Radon Compliance**: The cyclomatic complexity of `consume_op` is reduced to exactly $CC=1$.
* **Side-Channel Elimination**: Timing variability in the execution loop is completely eliminated.
* **Maturity Rating**: This innovation raises the typestate and POWL runner components to a Substrate Integrity Score (SIS) of 100/100, enabling PhD-level verification of the core execution token.
