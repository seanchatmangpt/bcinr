Here are the findings from inspecting `hostile_mutants.rs` regarding `mutant_3`:

### Mathematical Law Broken
**The flat-share normalization law.**
Under this canonical law, each leaf's flat share must be correctly normalized by the true sum of sibling leaf weights. The implementation of `mutant_3` intentionally breaks this law by forcing the leaf-weight-sum denominator (`lw_denom`) to a constant `1` (ONE), completely ignoring the actual calculated sum (`lw_sum`). 

### Expected Outcome / Refusal
Instead of triggering a typed refusal or failure, `mutant_3` deterministically produces a corrupted allocation output. 

The test suite asserts that running the allocation baseline (`run_alloc_tree`) with `mutant_3` active **must** yield the exact following corrupted allocation array:
```rust
const WRONG_M3_BROKEN_NORMALIZATION: [u32; N] = [0, 9805, 7211, 7938, 7938, 7937, 9099, 15608];
```

It must also successfully assert deviation from the uncorrupted baseline:
```rust
const CORRECT_TREE: [u32; N] = [0, 9392, 6624, 8067, 8067, 8067, 9276, 16043];
```

*(Note: The test for `mutant_3` is skipped if `mutant_7` is concurrently active, because `mutant_7`'s division check corruption saturates the `allocate` path to `u32::MAX`, effectively masking `mutant_3`'s normalization signature.)*
