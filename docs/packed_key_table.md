# PackedKeyTable in BCINR

The `PackedKeyTable` is a core data structure used within the MAPE-K Autonomic Loop of the BCINR project, functioning as the deterministic, bounded repository for historic system telemetry and known metrics (the "Knowledge" component of MAPE-K).

## Zero-Allocation and Deterministic Bounds

In adherence to BCINR's Zero-Allocation Boundary and Radon Law ($CC=1$), the `PackedKeyTable` is engineered to bound historic telemetry deterministically without requiring heap allocation. It achieves this through several mechanisms:

1.  **Const-Generic Bounded Capacity (`const N: usize`)**:
    The table is parameterized at compile time with a maximum capacity `N`. All internal storage is backed by fixed-size arrays:
    ```rust
    pub struct PackedKeyTable<K, V, const N: usize> {
        pub hashes: [u64; N],
        pub keys: [K; N],
        pub values: [V; N],
        pub len: usize,
    }
    ```
    This completely eliminates the need for dynamic memory allocation, allowing the table to exist entirely on the stack or in a fixed memory region like a `BumpArena`.

2.  **Branchless Linear Search**:
    The structure employs a cache-friendly, branchless linear search for operations like `get` and `insert`. Instead of utilizing conditional jumps (`if`, `match`) which violate the project's complexity rules, it evaluates all elements in the table using deterministic evaluation and bitwise masks.
    
    For instance, retrieval computes an `is_match` mask and accumulates results using array indexing rather than branching logic:
    ```rust
    let is_match = (i < self.len && self.hashes[i] == hash) as usize;
    result = [result, self.values[i]][is_match];
    ```

3.  **Deterministic Hashing**:
    Keys are mapped to hashes using a deterministic `fnv1a_64` hash function over the type's byte representation. This relies on type-safe byte reinterpretation via pointer casting. The bounds explicitly require `K: Copy` to ensure the type has a stable, memory-safe bit representation, allowing the hash to be derived with zero side effects.

4.  **Constant-Time Execution**:
    Both lookups and insertions scan across the entire generic space `0..N`, regardless of the actual `len` of the collection. Because the loop bounds are compile-time constants and the operations inside the loop are strictly arithmetic or bitwise (such as calculating insertion positions through masks), the execution time and instruction path are completely deterministic and independent of the input data size or contents.

By utilizing fixed-size buffers, mask-based state selection, and deterministic type-safe hashing, `PackedKeyTable` guarantees strict compliance with BCINR's requirements for bounded, allocation-free, and side-channel-resistant execution.
