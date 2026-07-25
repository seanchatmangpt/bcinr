# PackedKeyTable (PKT) Analysis

The `PackedKeyTable<K, V, const N: usize>` is a deterministic, cache-friendly key-value table designed specifically to comply with the BCINR constitutional requirements (zero heap allocation and strict `CC=1` branchless execution). 

## 1. Zero Heap Allocation Mechanism

The struct guarantees zero heap allocation by enforcing bounded, fixed-capacity storage natively on the stack.

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

Instead of dynamic containers like `Vec` or `HashMap`, the table stores data in contiguous, fixed-sized arrays `[T; N]`. The type bounds `Copy + Default` ensure that element creation and manipulation rely strictly on stack memory and bitwise operations without invoking heap allocation or `Drop` semantics.

## 2. Branchless Hashing 

Keys are hashed using a type-safe, deterministic byte reinterpretation logic. Because of the `Copy` bound, memory is bit-stable, allowing the system to cast the key directly to raw bytes. This ensures no dynamic dispatch or type-specific trait branching occurs during hash computation.

```rust
let key_size = core::mem::size_of::<K>();
let key_bytes = unsafe { core::slice::from_raw_parts(key as *const K as *const u8, key_size) };
fnv1a_64(key_bytes)
```

## 3. Branchless Lookup (`get`)

Lookups must check the entire structure without early returns or conditional blocks. To ensure constant-time bounds and strict `CC=1`, `get` evaluates over the full array capacity and replaces conditional logic with integer casting and array selection.

```rust
pub fn get(&self, key: K) -> Option<V> {
    let hash = hash_key(&key);
    let mut result = V::default();
    let mut found = 0usize;
    
    (0..N).for_each(|i| {
        // Evaluate predicate to 0 or 1
        let is_match = (i < self.len && self.hashes[i] == hash) as usize;
        
        // Array selection instead of `if is_match { result = self.values[i]; }`
        result = [result, self.values[i]][is_match];
        
        found |= is_match;
    });
    
    // Return selection based on the combined `found` mask (0 or 1)
    [None, Some(result)][found]
}
```

## 4. Branchless Insertion (`insert`)

Similar to lookups, insertions evaluate the full sequence. To update state or locate insertion indices, the table leverages Two's Complement arithmetic to generate full-width bit masks, avoiding `if/else` assignments. 

```rust
let is_greater = (i < self.len && self.hashes[i] > hash) as usize;
let is_first_greater = (is_greater != 0 && pos == self.len) as usize;

// Mask Generation: 0usize.wrapping_sub(0) -> 0x00...00
// Mask Generation: 0usize.wrapping_sub(1) -> 0xFF...FF
let p_mask = 0usize.wrapping_sub(is_first_greater);

// Bitwise state selection using the generated mask
pos = (i & p_mask) | (pos & !p_mask);
```

By substituting `if` conditions with polynomial and bitwise logic across a statically sized `(0..N).for_each` loop, `PackedKeyTable` comprehensively maps logic to arithmetic operations, conforming precisely to BCINR constraints.
