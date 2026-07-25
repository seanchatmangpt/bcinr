### Finding `SupportMismatch`

Using `grep_search`, `SupportMismatch` was located in `crates/bcinr-cmca/src/fixed.rs` (line 24) where it is defined as a constant branchless bitflag within the `NumericFaultSet` struct:

```rust
// crates/bcinr-cmca/src/fixed.rs (Line 24)
pub const SUPPORT_MISMATCH: Self = Self(1 << 5);
```

It exists as a discrete bit inside a fault accumulator (e.g., `NumericFaultSet`), strictly adhering to the project's **Radon Law ($CC=1$)** which forbids any cyclomatic complexity or early returns inside the authoritative hot path.

### Structural and Mathematical Check

Based on the architectural laws and repository design documentation (`docs/support_mismatch_refusal.md`), the `SupportMismatch` typed refusal is mathematically produced through a strict branchless mechanism:

1. **Bit-Parallel Bounds Checking:**
   The bounds constraint or structural requirement (such as evaluating bounded data structures or admitting topological workflow nodes) is checked via a bitwise polynomial rather than a comparison. A mismatch happens mathematically when requested support bits fall outside the pre-certified mask:
   ```rust
   let out_of_bounds = requested_nodes & !admitted_support_mask;
   let has_mismatch = (out_of_bounds != 0) as u64;
   ```

2. **Full-Width SWAR Mask Derivation:**
   To avoid `if !valid { ... }`, the resulting boolean is translated into a full-width Canonical Mask (all-zeros for valid `0x0000000000000000`, all-ones for invalid `0xFFFFFFFFFFFFFFFF`):
   ```rust
   let mismatch_mask = 0u64.wrapping_sub(has_mismatch);
   ```

3. **Continuous Execution & Sticky Accumulation (Join-Semilattice):**
   The computational pipeline continues uninhibited regardless of the mismatch. A branchless multiplexer (`select(mask, fallback, active)`) routes processing to a safe fallback to prevent hardware faults, while the `mismatch_mask` concurrently selects the `SUPPORT_MISMATCH` bit (e.g. `1 << 5`) to logically `OR` (`.union()`) it into the transaction's running fault state:
   ```rust
   let e = CanonicalMask::select_faults(
       mismatch_mask,
       NumericFaultSet::SUPPORT_MISMATCH,
       NumericFaultSet::EMPTY,
   );
   faults = faults.union(e); // Sticky accumulation, never first-error-wins
   ```

4. **Masked Commit & API Boundary Translation:**
   At the absolute edge of the deterministic boundary, the entire state commit is predicated on an overarching `m_admitted` mask. The state mutation is rejected field-wise using $x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t)$. Once safely outside the $CC=1$ hot-path boundary, the accumulated `1 << 5` fault bit is unpacked and translated into the strongly typed `Err(StabilityRefusal::SupportMismatch)` enum variant mandated by Rule 18 of the BCINR constitution.
