# Mask-Based Normalization Enforcements in `bcinr`

In accordance with **Rule 14 (Numeric Laws)** and the invariants defined in `.claude/rules/cmca/numeric-hot-path.md`, `bcinr` mandates strict constant-time, branchless ($CC=1$), and deterministic numeric evaluation. The resource allocator implements these principles for fixed-point `Q16.16` normalization across elements (where exactly `65536` equals `1.0`).

## 1. Exact Budget Conservation (Invariant 4)
Independent, per-item rounding (such as truncation from division) violates Invariant 4 because the rounded shares might sum to slightly more or less than the whole unit. The substrate enforces exact budget conservation branchlessly by splitting the unit budget into a base quotient and a remainder:
* It computes the number of valid elements safely: `nl_safe = const_select_u32(nl_is_zero, 1, nl)`.
* It calculates `q_floor = 65536 / nl_safe` and `r_floor = 65536 - q_floor * nl_safe`.
* Using a branchless canonical rank `leaf_rank`, it checks `const_lt_u32(leaf_rank[x], r_floor)`. The first `r_floor` leaves receive `q_floor + 1`, and the remaining leaves receive `q_floor`.

## 2. Correcting mixed truncation losses branchlessly
When uniform distributions are mixed with price-normalized distributions (`p_mu`), truncation in `saturating_div` and `saturating_mul` can yield a sum that falls short of exactly `1.0`. The allocator branchlessly repairs the entire sum using a gap-distribution technique:
* **Observe the gap:** It computes the actual sum of the shares (`leaf_sum`) and detects deficits or excesses branchlessly:
  ```rust
  let is_deficit = const_lt_u32(leaf_sum, target_bits);
  let is_excess = const_lt_u32(target_bits, leaf_sum);
  ```
* **Distribute the gap:** It computes `gap_safe = abs(target_bits - leaf_sum)` branchlessly and splits it into `gap_q` and `gap_r`.
* **Apply the correction:** It calculates a `bump` and bumps the values up or down depending on the masks:
  ```rust
  let corrected = select_nnf(
      is_deficit,
      bumped_up,
      select_nnf(is_excess, bumped_down, pi_res[x & 7]),
  );
  ```

## 3. Preservation of Numeric Faults (Invariant 2)
When correcting the shares, the system must not "silently erase" accumulated numeric faults (e.g., `SATURATION` or `RANGE_VIOLATION`). To satisfy Invariant 2, the substrate uses the `from_parts` constructor rather than `from_value_bits`:
```rust
let bumped_up = NonNegativeFixed::from_parts(
    pi_res[x & 7].value_bits().wrapping_add(bump),
    pi_res[x & 7].faults(), // Explicit preservation of faults
);
```

## 4. Branchless Handling of Out-of-Bounds Normalization (Invariant 6)
When an out-of-bounds scenario occurs, such as a denominator sum of `0` (`priced_sum == 0`), the substrate prohibits panics, early returns, or control-flow branches (Invariant 6 requires totality). It handles this branchlessly by:
1. **Detecting the Zero:** `let priced_sum_is_zero = const_eq_u32(priced_sum.value_bits(), 0);`
2. **Accumulating the Fault:** It injects `INVALID_NORMALIZATION` into the join-semilattice of numeric faults using a canonical mask:
   ```rust
   local_numeric_faults = local_numeric_faults.union(
       CanonicalMask::from_lsb(priced_sum_is_zero).select_faults(
           NumericFaultSet::INVALID_NORMALIZATION,
           NumericFaultSet::EMPTY,
       )
   );
   ```
3. **Substituting a Safe Denominator:** It substitutes `1.0` (i.e. `NonNegativeFixed::ONE`) for `0` to prevent division-by-zero, allowing the authoritative root to safely finish computation and return a total result alongside the accumulated faults:
   ```rust
   let psd = select_nnf(priced_sum_is_zero, NonNegativeFixed::ONE, priced_sum);
   ```
