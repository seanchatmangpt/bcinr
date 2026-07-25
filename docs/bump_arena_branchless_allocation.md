# Research Report: `BumpArena` Branchless Pointer Arithmetic

I've researched the implementation of `BumpArena` across the `bcinr-logic` crate to understand how it enforces boundary limits and updates the pointer without using conditional branches (enforcing Rule 3). 

Here are my findings:

## Core Implementation Mechanics

The branchless memory allocation relies heavily on arithmetic masking and bitwise operations instead of traditional `if (ptr + size > end)` conditional logic. 

There are two primary implementations found in the codebase:

### 1. High-Level State Allocation (`crates/bcinr-logic/src/abstractions/bump_arena.rs`)

The `BumpArenaState::try_alloc` method tracks the offset purely as a `u32` boundary:

```rust
pub fn try_alloc(&mut self, size: u32) -> (u32, u32) {
    let current_offset = self.offset;
    
    // 1. Calculate the tentative next offset with wrapping to prevent panics
    let next_offset = current_offset.wrapping_add(size);
    
    // 2. Boolean logic cast to u32 (1 if true, 0 if false)
    let success = (next_offset <= self.capacity) as u32;
    
    // 3. Create a bitmask using two's complement subtraction:
    // If success is 1: 0 - 1 = 0xFFFFFFFF (all 1s)
    // If success is 0: 0 - 0 = 0x00000000 (all 0s)
    let mask = 0u32.wrapping_sub(success);

    // 4. Bitwise selection to update the state
    self.offset = (next_offset & mask) | (current_offset & !mask);
    
    (current_offset & mask, mask)
}
```

### 2. Memory Substrate Allocation (`crates/bcinr-logic/src/mem.rs`)

The `BumpArena::alloc` method applies a similar branchless strategy but handles slicing, overflow protection (`usize::MAX`), and memory lifetimes:

```rust
pub fn alloc(&mut self, size: usize) -> Option<&mut [u8]> {
    let current_offset = self.offset;
    
    // 1. Calculate the tentative next offset, bubbling up overflow status
    let (next_offset, overflow) = current_offset.overflowing_add(size);
    
    // 2. Boolean condition ensures boundaries are respected and overflow is avoided
    let can_alloc = ((next_offset <= self.data.len()) & !overflow) as usize;
    
    // 3. Two's complement bitmask generation
    let mask = 0usize.wrapping_sub(can_alloc);

    // 4. Bitwise selection updates the state
    self.offset = (next_offset & mask) | (current_offset & !mask);

    // Note: Rust's `.then(|| ...)` creates an Option but relies on branchless selection 
    // inside the hot execution environment up to this boundary.
    (can_alloc != 0).then(|| {
        let slice = &mut self.data[current_offset..];
        let ptr = slice.as_mut_ptr();
        unsafe { core::slice::from_raw_parts_mut(ptr, size) }
    })
}
```

## How It Bypasses Branches (Radon Law, CC=1)

1. **Boolean coercion into a mask (`success as u32`)**: It evaluates `next_offset <= capacity`, casting it to a numeric `0` or `1`.
2. **Two's complement expansion**: `0u32.wrapping_sub(success)` turns a `1` into an all-ones bitmask (`0xFFFFFFFF`) and a `0` into an all-zeros bitmask (`0x00000000`).
3. **Bitwise Selection (Select):** `(next_offset & mask) | (current_offset & !mask)` seamlessly selects the newly bumped offset if allocation is valid, or clamps/retains the old offset if it exceeded the capacity, completely side-stepping jumps and branches.
4. **Overflow Protection**: `mem.rs` explicitly pairs `& !overflow` alongside the boundary check to prevent large `size` allocations from wrapping around and passing the bounds check falsely.
