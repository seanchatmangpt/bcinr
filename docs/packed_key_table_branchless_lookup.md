# PackedKeyTable Branchless Lookup in BCINR

The `PackedKeyTable` is a fundamental building block of the BCINR autonomic substrate, primarily acting as the deterministic, bounded repository for historic system telemetry and known metrics within the MAPE-K (Monitor-Analyze-Plan-Execute over a shared Knowledge base) Autonomic Loop.

## Zero-Allocation Knowledge Storage

In strict adherence to BCINR's core architectural laws—namely the Zero-Allocation Boundary and the Radon Law ($CC=1$)—the `PackedKeyTable` avoids all dynamic heap allocation. It achieves this by storing knowledge in fixed-size arrays parameterized by a compile-time const generic `N`:

```rust
pub struct PackedKeyTable<K, V, const N: usize>
where
    K: Copy + Default,
    V: Copy + Default,
{
    pub hashes: [u64; N],
    pub keys: [K; N],
    pub values: [V; N],
    pub len: usize,
}
```
All keys, values, and pre-computed deterministic `fnv1a_64` hashes reside strictly on the stack or within fixed memory regions like a `BumpArena`. The requirements `K: Copy + Default` and `V: Copy + Default` ensure that data representation is stable and no `Drop` semantics are triggered, keeping execution bit-safe.

## Fixed-Size Limitations

Because `PackedKeyTable` relies exclusively on fixed-size arrays `[T; N]`, its capacity `N` is completely bounded at compile-time (e.g., `PackedKeyTable<u64, u8, 16>`). 
- **Absolute Upper Bound**: The table can never grow beyond `N` items.
- **Fixed Work Execution**: Any operation iterating over the table will always evaluate precisely `N` iterations. This design eliminates the risk of unbounded execution or out-of-memory panics, at the trade-off of requiring a known, finite domain of telemetry knowledge during the system design phase.

## Mechanics of Branchless Lookup (Infer Phase)

During the Infer phase, when the `AutonomicSubstrate` needs to retrieve historic context from the `PackedKeyTable`, it must perform a lookup without violating the rule against data-dependent branching (no `if`, `match`, or early-return loops). 

The `get` method implements this by scanning the entire table of size `N` deterministically:

```rust
#[must_use]
#[inline(always)]
pub fn get(&self, key: K) -> Option<V> {
    let hash = hash_key(&key);
    let mut result = V::default();
    let mut found = 0usize;
    
    (0..N).for_each(|i| {
        // 1. Evaluate match branchlessly: generates 0 or 1
        let is_match = (i < self.len && self.hashes[i] == hash) as usize;
        
        // 2. Select value branchlessly: array indexing based on match mask
        result = [result, self.values[i]][is_match];
        
        // 3. Accumulate found state unconditionally
        found |= is_match;
    });
    
    // 4. Return Option branchlessly
    [None, Some(result)][found]
}
```

### Breakdown of the Mechanics:
1. **Exhaustive Iteration**: The `(0..N).for_each` loop unconditionally processes every possible slot up to the static capacity `N`, guaranteeing fixed bounded execution work. No `break` or early return is possible.
2. **Mask Generation**: The boolean condition `(i < self.len && self.hashes[i] == hash)` is evaluated and cast to a `usize` mask (`0` for false, `1` for true).
3. **Array-Indexed State Selection**: By wrapping the `current_result` and the `candidate_value` in an inline array `[result, self.values[i]]`, the system uses the `is_match` value (`0` or `1`) as an index. This effectively selects the new value if a match occurs, or preserves the existing result otherwise, all without any CPU branching.
4. **Final Resolution**: The boolean accumulator `found` tracks if any match occurred, allowing the function to select between `None` and `Some(result)` using the exact same array-indexing technique at the end.
