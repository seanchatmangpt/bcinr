Here is the documentation detailing how `RefusalSet` is used in `bcinr` to comply with the Rule 8 `CC=1` mandate:

```markdown
# How `RefusalSet` Replaces `Result<T, E>` and `?` for CC=1 Compliance

In the `bcinr` (BranchlessCInRust) substrate, **Rule 8 (Absolute CC=1 law)** explicitly prohibits control-flow branching constructs in authoritative code, including `Result`-based control flow, early returns, and the `?` operator. A standard Rust `Result<T, E>` implicitly requires conditional jumps to bubble up errors or short-circuit logic, which violates the strict constant-time ($CC=1$) mandate.

Based on an inspection of `crates/bcinr-cmca/src/allocator.rs`, the codebase achieves compliance by replacing `Result` with **`RefusalSet`** and **`AllocationOutcome`**.

## 1. `RefusalSet`: Bitwise Error Tracking
Instead of a single `Err` variant that terminates execution, errors (refusals) are encoded as bits inside a wrapped `u32` (the `RefusalSet`). This allows multiple errors to co-occur and be collected branchlessly.

*   **Branchless Accumulation**: Operations unconditionally produce error bitflags. Instead of `if err { return err; }`, the state is accumulated using a bitwise OR operation.
    ```rust
    #[inline(always)]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    ```
*   **Masked Selection**: To conditionally apply an error based on an algorithmic state, `RefusalSet` provides a `masked` method that avoids `if` blocks completely using arithmetic bitwise logic:
    ```rust
    #[inline(always)]
    pub const fn masked(self, condition: u32) -> Self {
        Self(self.0 & 0u32.wrapping_sub(condition & 1))
    }
    ```

## 2. `AllocationOutcome`: Guaranteed Totality
Instead of returning `Result<T, E>`, authoritative entry points (like `allocate`) unconditionally return an `AllocationOutcome` struct. This struct packages:
1.  The computed `candidate` data (valid or invalid).
2.  `numeric_faults` (e.g., overflow anomalies).
3.  `refusals` (the `RefusalSet`).

Because `AllocationOutcome` is *always* constructible for any input, the authoritative root never early-returns, panics, or halts computation midway. Both the success state and error flags are evaluated in parallel, ensuring a fixed execution path and complexity for all inputs.

## 3. Slow-Rail Bridging
While the hot path evaluates strictly without branches, the slow rail (non-authoritative code) can safely bridge back to idiomatic Rust control flow. `AllocationOutcome` provides an `into_result` adapter:
```rust
pub fn into_result(self) -> Result<[NonNegativeFixed; N], StabilityRefusal> {
    if self.refusals.is_empty() {
        Ok(self.candidate)
    } else {
        Err(self.refusals.primary_reason())
    }
}
```
This is only called outside the strict $CC=1$ bounds, fulfilling the "typed refusal" requirement without corrupting the branchless execution phase.
```
