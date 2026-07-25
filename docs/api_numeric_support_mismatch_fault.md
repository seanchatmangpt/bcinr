Based on the research within `crates/bcinr-cmca/src/fixed.rs` and the related design documents (`docs/api_support_mismatch_refusal.md` and `docs/support_mismatch_refusal.md`), here is the documentation for `NumericFaultSet::SUPPORT_MISMATCH`.

### Definition Location

The `SUPPORT_MISMATCH` bitmask is defined in `crates/bcinr-cmca/src/fixed.rs` at line 24 as a constant within the `NumericFaultSet` struct:

```rust
pub const SUPPORT_MISMATCH: Self = Self(1 << 5);
```

### Mathematical Condition

The `SUPPORT_MISMATCH` fault represents a violation of pre-certified topological support or operational bounds. It surfaces when an incoming transition, allocation request, or bounded data structure access attempts to utilize resources or graph nodes that fall outside the statically admitted limits (the "admitted support mask"). 

Mathematically, it represents the condition where the intersection of requested support and the complement of admitted support is non-zero:
`requested_nodes ∩ ¬admitted_support_mask ≠ ∅`

### Branchless Setting Mechanism

Because the `bcinr` substrate strictly adheres to the Radon Law ($CC=1$) which forbids data-dependent branching (no `if`, `match`, or early returns), the fault is derived and set structurally using bitwise polynomials and SWAR (SIMD Within A Register) masking:

1. **Bit-Parallel Derivation:**
   The boolean existence of a mismatch is calculated bitwise without branching:
   ```rust
   let out_of_bounds = requested_nodes & !admitted_support_mask;
   let has_mismatch = (out_of_bounds != 0) as u64;
   ```

2. **Canonical Mask Translation:**
   The boolean is transformed into a full-width SWAR mask (e.g., `0xFFFFFFFFFFFFFFFF` if true, `0x0000000000000000` if false) using a wrapping subtraction:
   ```rust
   let mismatch_mask = 0u64.wrapping_sub(has_mismatch);
   ```

3. **Sticky Fault Accumulation:**
   The execution pipeline does not halt on error. Instead, a branchless multiplexer routes execution to a safe fallback state, while simultaneously using the `mismatch_mask` to select the `SUPPORT_MISMATCH` bit (`1 << 5`). This bit is logically `OR`ed into the transaction's running fault state. This behaves as a join-semilattice under bitwise union, ensuring strict constant-time accumulation with no "first-error-wins" short-circuiting:
   ```rust
   let e = CanonicalMask::select_faults(
       mismatch_mask,
       NumericFaultSet::SUPPORT_MISMATCH,
       NumericFaultSet::EMPTY,
   );
   
   faults = faults.union(e);
   ```

At the outer boundary of the authoritative hot path, the persistent state is finally updated using a field-wise masked commit. If the `SUPPORT_MISMATCH` bit was accumulated into the fault set, the admission mask evaluates to false and the state update is branchlessly discarded ($x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t)$), leaving the state bit-for-bit unchanged.
