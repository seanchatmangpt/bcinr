# Branchless Error Handling in BCINR

The `bcinr` (BranchlessCInRust) codebase is a deterministic, allocation-free execution substrate. Under its strict constitutional mandates, standard Rust error-handling paradigms are fundamentally incompatible with the authoritative hot path. 

## 1. The Ban on `Result`-Based Control Flow (Rule 8: Absolute CC=1)

Under the **Radon Law ($CC=1$)**, the authoritative hot path must execute in exactly the same number of CPU instructions regardless of input data. The full transitive call graph must contain zero input-dependent jumps.

Standard Rust error-handling constructs violate this rule because they generate hidden JCC (Jump if Condition Is Met) branches in the compiled machine code:
- **`?` (Early Return)**: Expands to a `match` statement that introduces a branch to exit immediately upon encountering an `Err`.
- **`unwrap()` / `expect()`**: Introduce conditional branches to check variants, branching into bounds-check panic or unwinding paths upon failure.
- **`Option`/`Result` matching**: Explicitly translates into conditional assembly jumps.

Instead of aborting early, `bcinr` enforces **exhaustive speculative execution**. Authoritative functions fully compute an operation as if the input were valid, yielding an outcome structure that contains a "speculative candidate" alongside any accrued faults.

## 2. Division by Zero

Hardware integer division instructions are banned because they have variable cycle latencies. Instead, BCINR replaces division with a **Branchless Reciprocal Approximation** (minimax constants combined with Newton-Raphson refinement).

To handle division by zero without panicking or returning early:
1. **Branchless Substitution**: The divisor is checked for zero. If it is zero, a branchless select mathematically maps the divisor to `1` (e.g., `let d = den_is_zero.select_u32(1, other.val)`). This prevents undefined behavior or CPU hardware faults before normalization begins.
2. **Pipeline Execution**: The entire pipeline (Newton-Raphson iterations and quotient computation) runs as normal on the substituted value.
3. **Fault Accumulation**: The `den_is_zero` predicate is expanded to a mask, and a `DIVIDE_BY_ZERO` / `INVALID_DOMAIN` fault is branchlessly selected and unioned into the operation's `NumericFaultSet`.

## 3. Out-of-Bounds Indexing

Dynamic arrays (`Vec<T>`) are entirely prohibited; all memory is mapped over strictly bounded compile-time arrays (e.g., `[T; N]`). However, even with fixed arrays, standard indexing (`array[index]`) generates bounds-check panic branches in Rust.

To eliminate these hidden branches, BCINR coerces indices into guaranteed valid ranges using branchless bitwise logic:
- **Power-of-Two Masking**: For capacities that are powers of two, indexing uses bitwise AND (e.g., `REFUSALS[(err_code as usize) & 31]`).
- **Branchless Clamping**: For non-power-of-two boundaries, a safe fallback index is chosen using branchless selection (e.g., `let idx = const_select_u32(in_bounds, val, max_safe_idx)`).

Because the compiler can statically prove that the resulting index will never exceed the array length, it entirely optimizes away the bounds-check panic path, ensuring a clean disassembly audit.

## 4. Invalid Inputs and Typed Refusals

Instead of returning errors, anomalous conditions are accumulated into a **Typed Refusal** (`RefusalSet` or `NumericFaultSet`), which stores faults as bits in a `u32` mask. 

Mathematical predicates are evaluated into `1` (true) or `0` (false). These boolean values are then transformed into full-width bitmasks (e.g., `0xFFFFFFFF` or `0x00000000`). If an invalid input check fails, the corresponding error flag is merged into the refusal set using bitwise operators (`|`, `&`) rather than a conditional `if` block:

```rust
// Accumulates refusal branchlessly without jumping
#[inline(always)]
pub const fn masked(self, condition: u32) -> Self {
    Self(self.0 & 0u32.wrapping_sub(condition & 1))
}
```

## 5. Mask-Based Selection & State Commitment

According to Rule 10, persistent state must never be mutated speculatively before complete admission. Once the exhaustive computation completes, the runtime reduces the accumulated Typed Refusals into a single **admission mask**: $m \in \{0, 2^w-1\}$.

If there are any faults, the mask evaluates to exactly `0`. If perfectly valid, it evaluates to all `1`s. State mutation is then executed via fieldwise, fixed-width bitwise selection:

$$ \operatorname{select}(m, a, b) = (m \land a) \lor (\neg m \land b) $$

```rust
let m_admitted = valid_mask(outcome.refusals);
let next_state = State::select(m_admitted, outcome.candidate, current_state);
```

If an error occurred (such as division by zero or invalid input), the `m_admitted` mask zeroes out the candidate completely. The bitwise selection seamlessly leaves the original state bit-for-bit unchanged, gracefully handling the erroneous state in mathematically constant time without a single control-flow jump.
