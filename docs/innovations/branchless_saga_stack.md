# Innovation Proposal: Branchless Saga Stack (BSS) for Constant-Time Compensation Recovery

## 1. Executive Summary

This proposal introduces a **Branchless Saga Stack (BSS)** design to optimize and harden saga compensation indexing in `crates/bcinr-powl/src/enterprise.rs` under the strict BCINR Radon Law ($CC=1$, zero allocation).

The primary objective is to eliminate all conditional branches and data-dependent execution paths from [`SagaStack::push`] and [`SagaStack::pop`] operations. In the baseline implementation, these functions contain `if` conditionals to handle capacity overflow and stack underflow. In the proposed BSS architecture, these branches are eliminated by utilizing a **multiplexed index layout** backed by a **dedicated garbage sink slot** (array size 33 instead of 32) and a **branchless pop status return**. This guarantees constant execution time, eliminates timing side-channels during saga recovery, and ensures compliance with the $CC=1$ mandate of the Radon Law.

---

## 2. Theoretical Analysis & Algorithmic Flaws

### 2.1 Branching in Baseline Recovery
The current implementation of [`SagaStack`] contains two explicit control-flow branches:
1. **Push Overflow Gate** ([enterprise.rs:L117-122](file:///Users/sac/bcinr/crates/bcinr-powl/src/enterprise.rs#L117-L122)):
   ```rust
   pub fn push(&mut self, comp_op_idx: u16) {
       if (self.top as usize) < 32 {
           self.frames[self.top as usize] = comp_op_idx;
           self.top = self.top.saturating_add(1);
       }
   }
   ```
2. **Pop Underflow Gate** ([enterprise.rs:L128-134](file:///Users/sac/bcinr/crates/bcinr-powl/src/enterprise.rs#L128-L134)):
   ```rust
   pub fn pop(&mut self) -> Option<u16> {
       if self.top == 0 {
           return None;
       }
       self.top -= 1;
       Some(self.frames[self.top as usize])
   }
   ```

### 2.2 Violation of the Radon Law
Under Section 3 and Section 8 of the BCINR Constitution (`AGENTS.md`), these conditional branches represent structural violations. Specifically:
- **Control-Flow Dependency**: The runtime execution path differs based on whether the stack is empty or full.
- **Timing Side-Channels**: Processor branch predictors will speculatively execute paths, causing variable execution times based on stack depth. In transactional rollbacks, timing variations can expose details about the internal saga structure.
- **Assembly Branching**: The compiler emits conditional jumps (e.g., `cmp` followed by `jne`/`je`), breaking the absolute requirement that the disassembly has zero conditional jumps.

---

## 3. The Branchless Saga Stack (BSS) Architecture

The BSS design achieves $CC=1$ complexity by applying three core structural principles:

### 3.1 Multiplexed Indexing & Garbage Sink
Instead of a 32-element array, BSS defines the backing array with size **33**:
```rust
pub struct BranchlessSagaStack {
    frames: [u16; 33],
    top: u8,
}
```
* **Indices `0..32`**: Valid saga compensation frame storage.
* **Index `32`**: The designated garbage sink. Any push that occurs when the stack is full ($top \ge 32$) is written to this index, and any pop that occurs when the stack is empty ($top = 0$) reads from this index.

### 3.2 Branchless Push Algorithm
For a push operation, the target index is computed branchlessly by comparing the current stack pointer `top` against the capacity limit:
1. Compute the difference: $diff = 32 - \text{top} - 1$.
2. Extract the sign bit to determine if the stack is full: $is\_full\_bit = diff \gg 63$.
   - If $top < 32$, then $32 - top - 1 \ge 0$, yielding a sign bit of $0$.
   - If $top \ge 32$ (saturating at 32), then $32 - 32 - 1 = -1$ (or `u64::MAX`), yielding a sign bit of $1$.
3. Compute the write mask: $mask = 0 - is\_full\_bit$ (all-ones if full, all-zeros if not).
4. Multiplex the write index:
   $$write\_idx = (top \land \neg mask) \lor (32 \land mask)$$
5. Commit the write: `frames[write_idx] = comp_op_idx`.
6. Update the pointer: $top = top + (1 - is\_full\_bit)$.

### 3.3 Branchless Pop Algorithm
For a pop operation, we must decrement `top` only if it is greater than $0$, and read the value at index `top` (if valid) or the garbage sink (if empty):
1. Compute the empty condition sign bit:
   $$is\_empty\_bit = (top - 1) \gg 63$$
   - If $top = 0$, $top - 1 = -1$, yielding $1$.
   - If $top > 0$, $top - 1 \ge 0$, yielding $0$.
2. Compute the valid bit: $is\_valid\_bit = 1 - is\_empty\_bit$.
3. Decrement `top` branchlessly: $top = top - is\_valid\_bit$.
4. Multiplex the read index:
   - If valid, read from the updated `top`.
   - If empty, read from index `32`.
   $$read\_idx = (top \land \text{mask}_{valid}) \lor (32 \land \text{mask}_{empty})$$
   where $\text{mask}_{valid} = 0 - is\_valid\_bit$ and $\text{mask}_{empty} = 0 - is\_empty\_bit$.
5. Return the value along with a branchless status return:
   ```rust
   pub struct BranchlessPop {
       pub value: u16,
       pub valid_mask: u16,
   }
   ```
   where `valid_mask` is `0xFFFF` if the pop succeeded, and `0x0000` if the stack was empty.

---

## 4. Mathematical and Logical Contract

Using Hoare logic, we specify the contracts for BSS operations. Let $S$ denote the stack state, $S.\text{frames}$ the backing array, and $S.\text{top}$ the pointer.

### 4.1 Push Contract
$$\{P_{\text{push}}(S, x)\} \quad \text{push}(S, x) \quad \{Q_{\text{push}}(S, x, S')\}$$

* **Preconditions $P_{\text{push}}$**:
  - $S.\text{top} \in [0, 32]$
  - $x \in [0, 2^{16}-1]$
* **Postconditions $Q_{\text{push}}$**:
  - **Not Full Case ($S.\text{top} < 32$)**:
    - $S'.\text{frames}[S.\text{top}] = x$
    - $S'.\text{top} = S.\text{top} + 1$
    - $\forall i \ne S.\text{top}, \ S'.\text{frames}[i] = S.\text{frames}[i]$
  - **Full Case ($S.\text{top} = 32$)**:
    - $S'.\text{frames}[32] = x$ (sink updated)
    - $S'.\text{top} = 32$
    - $\forall i < 32, \ S'.\text{frames}[i] = S.\text{frames}[i]$ (prior valid elements unchanged)

### 4.2 Pop Contract
$$\{P_{\text{pop}}(S)\} \quad \text{pop}(S) \quad \{Q_{\text{pop}}(S, S', \text{result})\}$$

* **Preconditions $P_{\text{pop}}$**:
  - $S.\text{top} \in [0, 32]$
* **Postconditions $Q_{\text{pop}}$**:
  - **Not Empty Case ($S.\text{top} > 0$)**:
    - $S'.\text{top} = S.\text{top} - 1$
    - $\text{result.value} = S.\text{frames}[S'.\text{top}]$
    - $\text{result.valid\_mask} = 0xFFFF$
    - $\forall i, \ S'.\text{frames}[i] = S.\text{frames}[i]$
  - **Empty Case ($S.\text{top} = 0$)**:
    - $S'.\text{top} = 0$
    - $\text{result.value} = S.\text{frames}[32]$
    - $\text{result.valid\_mask} = 0x0000$
    - $\forall i, \ S'.\text{frames}[i] = S.\text{frames}[i]$

---

## 5. Branchless Rust Implementation

Below is the implementation details proposed to replace `SagaStack` in `crates/bcinr-powl/src/enterprise.rs`:

```rust
/// The return type of branchless pop operations.
///
/// Contains the popped value and an execution status mask (`0xFFFF` if valid, `0` if empty).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BranchlessPop {
    /// The compensation op index popped, or a garbage value if the stack was empty.
    pub value: u16,
    /// Status mask: `0xFFFF` for successful pop, `0` for empty stack.
    pub valid_mask: u16,
}

/// A fixed-capacity, branchless LIFO stack for saga compensation operation indices.
///
/// Under the BCINR Radon Law, all operations have a cyclomatic complexity of CC=1
/// and execute with zero conditional branches.
#[derive(Debug)]
pub struct BranchlessSagaStack {
    /// Storage for 32 stack frames + 1 garbage sink slot.
    frames: [u16; 33],
    /// Current stack depth (0..=32).
    top: u8,
}

impl BranchlessSagaStack {
    /// Create a new empty [`BranchlessSagaStack`].
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            frames: [0u16; 33],
            top: 0,
        }
    }

    /// Push a compensation op index branchlessly.
    ///
    /// If the stack is full, the write is multiplexed to the garbage sink slot (index 32)
    /// and the pointer does not increment, providing saturating behavior without branching.
    #[inline(always)]
    pub fn push(&mut self, comp_op_idx: u16) {
        let top_val = self.top as u64;
        
        // diff is non-negative (sign bit 0) when top_val < 32, and negative (sign bit 1) when top_val >= 32.
        let diff = 32u64.wrapping_sub(top_val).wrapping_sub(1);
        let is_full_bit = diff >> 63;
        
        // Write index: top if not full, 32 if full.
        let mask = 0u64.wrapping_sub(is_full_bit);
        let write_idx = (top_val & !mask) | (32 & mask);
        
        self.frames[write_idx as usize] = comp_op_idx;
        
        // Increment top only if not full
        self.top = self.top.wrapping_add((1u64.wrapping_sub(is_full_bit)) as u8);
    }

    /// Pop the most-recently-pushed compensation op index branchlessly.
    ///
    /// Returns a [`BranchlessPop`] struct containing the status mask and value.
    #[inline(always)]
    pub fn pop(&mut self) -> BranchlessPop {
        let top_val = self.top as u64;
        
        // If top is 0, wrapping_sub(1) has sign bit set (1). If top > 0, sign bit is 0.
        let is_empty_bit = (top_val.wrapping_sub(1)) >> 63;
        let is_valid_bit = 1u64.wrapping_sub(is_empty_bit);
        
        // Decrement top only if valid
        self.top = self.top.wrapping_sub(is_valid_bit as u8);
        
        // Read index: new top if valid, 32 if empty
        let valid_mask_u64 = 0u64.wrapping_sub(is_valid_bit);
        let empty_mask_u64 = 0u64.wrapping_sub(is_empty_bit);
        let read_idx = ((self.top as u64) & valid_mask_u64) | (32 & empty_mask_u64);
        
        let value = self.frames[read_idx as usize];
        let valid_mask = valid_mask_u64 as u16;
        
        BranchlessPop { value, valid_mask }
    }
}
```

---

## 6. Verification Strategy

Following Section 19 and Section 20 of `AGENTS.md`, verification must follow a strict independent audit path before merge.

### 6.1 Reference Oracle
We construct an independent mathematical oracle (`SlowSagaStack`) inside the test suite. This oracle may use standard branching and vectors to serve as an axiomatic reference:
```rust
struct SlowSagaStack {
    inner: Vec<u16>,
}

impl SlowSagaStack {
    fn new() -> Self {
        Self { inner: Vec::new() }
    }
    fn push(&mut self, val: u16) {
        if self.inner.len() < 32 {
            self.inner.push(val);
        }
    }
    fn pop(&mut self) -> Option<u16> {
        self.inner.pop()
    }
}
```
A differential test suite will verify all possible permutations of 100 random operations (mix of push/pop) across $2^{16}$ runs. For each state transition, it verifies:
1. `BSS.top == SlowSagaStack.len()`
2. `BSS.pop().value == SlowSagaStack.pop().unwrap()` (when `valid_mask` is `0xFFFF`).
3. `BSS.pop().valid_mask == 0` when `SlowSagaStack.pop() == None`.

### 6.2 Hostile Mutants
Under the authority of `@armstrong_fault`, we define three independent mutants to verify the testing framework's sensitivity:
1. **Mutant 1 (Pointer Creep)**:
   Change pointer increment to `self.top = self.top.wrapping_add(1)` regardless of `is_full_bit`.
   - *Expectation*: Stack pointer exceeds 32 on full push, overwriting random stack memory or triggering panic, immediately caught by bounds check or oracle mismatch.
2. **Mutant 2 (Underflow Saturation Bypass)**:
   Change pop decrement to `self.top = self.top.wrapping_sub(1)` regardless of `is_valid_bit`.
   - *Expectation*: Stack pointer wraps to `255` on empty pop, causing out-of-bounds panics, immediately killed.
3. **Mutant 3 (Stale Read Index)**:
   Change read index to `let read_idx = self.top as u64;` without empty/valid multiplexing.
   - *Expectation*: Empty pop reads index 0 instead of the garbage sink at index 32, which violates state isolation, caught by differential assertions.

### 6.3 Disassembly Audit
Compile the target using the `--release` profile and execute a binary audit:
- Confirm **0 conditional jump instructions** (`je`, `jne`, `jg`, `js`, etc.) exist within the compiled symbols for `push` and `pop`.
- Confirm `frames` writes/reads use compiler-generated conditional moves (`cmov`) or bitwise logic instead of jumps.

---

## 7. Downstream Impact

1. **Radon Law Compliance**: Achieves a perfect cyclomatic complexity of $CC=1$ for saga frames management.
2. **Side-Channel Immunity**: Every call to `push` and `pop` executes in exactly the same sequence of instructions, shielding saga state transitions from microarchitectural latency leaks.
3. **Rust Standard Compatibility**: Provides a wrapper conversion to `Option<u16>` for slow rail compatibility while preserving raw branchless operations in the hot path.
