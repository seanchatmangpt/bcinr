I have investigated `RefusalSet::NO_LEAVES` in `crates/bcinr-cmca/src/allocator.rs`.

### Structural Condition
`RefusalSet::NO_LEAVES` represents a structural property of the input candidate forest where there are **no valid leaf nodes** to allocate resource flow to (i.e., a leafless forest topology, likely due to cycles or missing leaf structures).

### Branchless Implementation
The `NO_LEAVES` bitmask is set entirely without conditional branches (`if`/`match`), in strict adherence to the project's Radon Law ($CC=1$). Here is the exact mechanical breakdown of how it's handled in the `allocate` function:

1. **Identifying Leaves:** 
   The allocator starts by assuming all nodes are leaves (`[true; N]`). It determines if a node is a leaf by comparing every node's parent index. This is done via nested static unrolling (`unroll_8_static!`). It branchlessly clears the `is_leaf[i]` flag if any node `j` declares `i` as its parent:
   ```rust
   let mut is_leaf = [true; N];
   unroll_8_static!(i, {
       unroll_8_static!(j, {
           let is_match = parent[j & 7] == i as i32;
           is_leaf[i & 7] &= !is_match;
       });
   });
   ```

2. **Summing the Leaf Count (`nl`):**
   The total number of leaves is accumulated by casting the boolean values directly to `u32` inside another unrolled loop, avoiding any branching iterators:
   ```rust
   let mut nl = 0u32;
   unroll_8_static!(i, {
       nl += is_leaf[i & 7] as u32;
   });
   ```

3. **Branchless Equality Check:**
   Checking if `nl == 0` is done using the project's custom `const_eq_u32` polynomial bitwise helper:
   ```rust
   let nl_is_zero = const_eq_u32(nl, 0);
   ```

4. **Applying the Bitmask:**
   The `NO_LEAVES` bit is applied to `final_refusals` unconditionally using `union()` and `masked()`. 
   ```rust
   let final_refusals = gated_refusals.union(RefusalSet::NO_LEAVES.masked(nl_is_zero));
   ```
   The `masked` function avoids a conditional branch by using wrapping subtraction to create a bitmask of all 1s (`0xFFFFFFFF`) if `condition == 1` and all 0s if `condition == 0`:
   ```rust
   pub const fn masked(self, condition: u32) -> Self {
       Self(self.0 & 0u32.wrapping_sub(condition & 1))
   }
   ```

*(Reference: [allocator.rs](file:///Users/sac/bcinr/crates/bcinr-cmca/src/allocator.rs))*
