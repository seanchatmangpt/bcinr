# Branchless Error Handling in `bcinr`

The `bcinr` (BranchlessCInRust) codebase is a deterministic, allocation-free execution substrate. Under its strict constitutional mandates, standard Rust error-handling paradigms are strictly prohibited in the authoritative hot path. 

## 1. The Ban on `Result`-Based Control Flow

Under the **Radon Law ($CC=1$)**, the authoritative hot path must execute in exactly the same number of CPU instructions regardless of input data. The full transitive call graph must contain zero input-dependent jumps.

Standard Rust error-handling constructs fundamentally violate this rule because they generate hidden JCC (Jump if Condition Is Met) branches in the compiled machine code:

- **`?` (Early Return)**: Expands to a `match` statement that introduces a branch to exit the function immediately upon encountering an `Err`.
- **`unwrap()` / `expect()`**: Introduce conditional branches to check for valid variants, branching into panic/unwinding paths upon failure.
- **`unwrap_or()` / `unwrap_or_else()`**: Generate branches to evaluate and return either the `Some`/`Ok` inner value or the fallback path.
- **`Option`/`Result`-based `if let` / `match`**: Explicitly translate into conditional assembly jumps based on semantic input.

The constitution enforces this at the object-code level (`@turing_machine`). Any hidden branches—whether in standard library calls, macro expansions, or trait implementations—are mechanically detected and immediately block merges.

## 2. The Algebraic Alternative: Branchless Accumulation

To handle failures without branching, `bcinr` replaces control flow with boolean algebra, bitwise masks, and exhaustive computation. This is implemented through a combination of **Branchless Fault Accumulation** and **Masked State Commitment**.

### Exhaustive Speculative Execution
Instead of aborting early when an invalid condition is met, authoritative functions must completely compute an operation as if the input were valid. This generates a fixed-width "speculative candidate". 

Functions do not return standard `Result` types. Instead, they output exhaustive outcome structs that carry the speculative result *alongside* any accrued faults:
```rust
pub struct AllocationOutcome {
    candidate: [NonNegativeFixed; N],
    numeric_faults: NumericFaultSet,
    refusals: RefusalSet,
}
```

### Branchless Fault Accumulation (Typed Refusals)
Every anomalous condition is represented by a bounded, discrete enum or bitflag (a "Typed Refusal"). A `RefusalSet` uses a `u32` bitmask to aggregate errors concurrently. 

Mathematical predicates are evaluated into `1` (true) or `0` (false) and expanded into full-width bitmasks. If a validation check fails, the corresponding error flag is merged into the refusal set using bitwise logic (e.g., `|`, `&`) rather than a conditional jump:

```rust
// Accumulate refusal bitwise without jumping
#[inline(always)]
pub const fn masked(self, condition: u32) -> Self {
    // 0u32.wrapping_sub(1) produces all 1s (0xFFFFFFFF)
    // 0u32.wrapping_sub(0) produces all 0s (0x00000000)
    Self(self.0 & 0u32.wrapping_sub(condition & 1))
}
```

### Masked State Commitment
According to the `bcinr` constitution, persistent state must never be mutated speculatively before complete admission. When a state transition is requested, the runtime reduces the accumulated faults into a single admission mask: $m \in \{0, 2^w-1\}$.

State mutation is then executed via fixed-width bitwise selection, mathematically equivalent to:
$$ \operatorname{select}(m, a, b) = (m \land a) \lor (\neg m \land b) $$

```rust
// Derive full-width admission mask (all 1s if valid, all 0s if invalid)
let m_admitted = valid_mask(outcome.refusals);

// Field-wise masked commit over the entire structure
let next_state = State::select(m_admitted, outcome.candidate, current_state);
```

If an error occurred, the mask evaluates to `0`. The candidate state is completely wiped out by the bitwise operation, leaving the persistent state bit-for-bit unchanged. 

Only at the extreme outer edge of the system (the "slow rail" boundary), outside of the authoritative hot path, are these numeric typed refusals finally mapped into standard Rust `Result` adapters for logging and telemetry.
