### Fixed-Width Wrapping Logic for OOM Bounds

In `crates/bcinr-logic/src/mem.rs`, the `BumpArena::alloc` function performs bounds checking and computes the new offset without using `if` branches:

```rust
#[inline(always)]
pub fn alloc(&mut self, size: usize) -> Option<&mut [u8]> {
    let current_offset = self.offset;
    // 1. Calculate the next offset, catching any potential fixed-width integer overflow.
    let (next_offset, overflow) = current_offset.overflowing_add(size);
    
    // 2. Evaluate bounds strictly mathematically: next offset must be within capacity AND not overflowed.
    let can_alloc = ((next_offset <= self.data.len()) & !overflow) as usize;
    
    // 3. Create a branchless mask. If can_alloc is 1, mask becomes all 1s (0xFF...). If 0, mask is all 0s.
    let mask = 0usize.wrapping_sub(can_alloc);

    // 4. Update the offset via branchless selection:
    //    If mask is 11...1: self.offset becomes next_offset
    //    If mask is 00...0: self.offset remains current_offset
    self.offset = (next_offset & mask) | (current_offset & !mask);

    (can_alloc != 0).then(|| {
        let slice = &mut self.data[current_offset..];
        let ptr = slice.as_mut_ptr();
        unsafe { core::slice::from_raw_parts_mut(ptr, size) }
    })
}
```

Similarly, in `crates/bcinr-logic/src/abstractions/bump_arena.rs` (a slightly different state representation), `try_alloc` behaves branchlessly using `u32`:

```rust
#[must_use]
#[inline(always)]
pub fn try_alloc(&mut self, size: u32) -> (u32, u32) {
    let current_offset = self.offset;
    let next_offset = current_offset.wrapping_add(size);
    
    // Evaluate if the new bounds are within the capacity limit.
    let success = (next_offset <= self.capacity) as u32;
    
    // Generate a mask (either 0x00000000 or 0xFFFFFFFF) using wrapping subtraction.
    let mask = 0u32.wrapping_sub(success);

    // Conditionally commit the new offset or keep the current one branchlessly.
    self.offset = (next_offset & mask) | (current_offset & !mask);
    
    (current_offset & mask, mask)
}
```

**Key Mechanisms:**
1. **Arithmetic Bounds Check**: Bounds evaluation produces a boolean translated to `1` or `0` (`can_alloc` / `success`). 
2. **Wrapping Subtraction for Masking**: `0usize.wrapping_sub(can_alloc)` (or `0u32`) converts a `1` to `!0` (all bits set) and `0` to `0`. 
3. **Bitwise Selection**: The update step `(next_offset & mask) | (current_offset & !mask)` acts as a branchless `select`, satisfying the constitutional requirement of $CC=1$ (Cyclomatic Complexity of 1) with no conditional jumps.
