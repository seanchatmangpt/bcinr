Here is the documentation for `BumpArena` and `LockFreeSlab` based on the codebase search:

```markdown
# Implementation of `BumpArena` and `LockFreeSlab`

Both `BumpArena` and `LockFreeSlab` are implemented in the `bcinr-logic` crate, strictly adhering to the `bcinr` branchless execution and deterministic zero-allocation requirements.

## 1. `BumpArena`

There are two related structs for the bump arena logic.

### `BumpArenaState`
Located in `/Users/sac/bcinr/crates/bcinr-logic/src/abstractions/bump_arena.rs`:
```rust
#[derive(Debug, Clone, Copy, Default)]
pub struct BumpArenaState {
    pub offset: u32,
    pub capacity: u32,
}
```
This is a purely arithmetic state abstraction that tracks the offset and capacity for branchless allocation using bitwise masking logic.

### `BumpArena`
Located in `/Users/sac/bcinr/crates/bcinr-logic/src/mem.rs`:
```rust
#[cfg(feature = "alloc")]
pub struct BumpArena {
    pub data: Vec<u8>,
    pub offset: usize,
}
```
Although it manages a `Vec<u8>` backing store (gated by the `alloc` feature for initialization), its `alloc` method uses strictly branchless arithmetic (masking) and constant-time execution to verify bounds and advance the offset deterministically.

## 2. `LockFreeSlab`

Located in `/Users/sac/bcinr/crates/bcinr-logic/src/abstractions/lock_free_slab.rs`:
```rust
use core::sync::atomic::AtomicU32;

pub struct LockFreeSlab<const N: usize> {
    pub freelist: AtomicU32,
    pub next_indices: [u32; N],
}
```
This provides a deterministic $O(1)$ allocation/deallocation mechanism using an atomic freelist. Its state transitions and index selections are executed through branchless pointer selection masks without using conditional control flow blocks.
```
