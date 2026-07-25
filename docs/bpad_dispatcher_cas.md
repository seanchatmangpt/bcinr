# BpadDispatcher: 8-Lane CAS and fanout_pair Mechanism

The `BpadDispatcher` is a constant-time, branchless, and zero-allocation dispatcher used in the POWL v2 runtime. It is designed to strictly adhere to the project's BCINR Radon Law ($CC=1$, zero allocations, no loops or branches), meaning it avoids any use of `std::sync::Mutex`, heap-allocated collections like `Vec` or queues, and even branching logic (like `if`, `match`, or `while`).

Here is how the dispatcher physically executes the `fanout_pair` operation concurrently without locks or heap-backed structures:

## 1. Unified Bit-Parallel State (`AtomicU8`)
Instead of managing independent locks or occupancy flags for each worker slot, the dispatcher tracks the entire 8-lane array using a single atomic byte (`AtomicU8` named `occupancy`). 
- Each bit (0 to 7) corresponds to a slot in the 64-byte aligned `slots` array. 
- A `1` means the slot is occupied, and a `0` means it is free.

## 2. Branchless Slot Discovery
When `fanout_pair(left, right)` is called to schedule two concurrent operations, it must find two free slots without looping or branching:
- It atomically loads the current `occupancy` byte and bitwise negates it to get the `free_mask`.
- It counts the free slots using the hardware-accelerated `count_ones()` popcount instruction.
- It finds the two lowest free slots strictly using bitwise arithmetic:
  - `first = free_mask.trailing_zeros()`
  - It clears the lowest set bit using `free_mask & (free_mask.wrapping_sub(1))`
  - `second = temp.trailing_zeros()`

## 3. All-or-Nothing Single CAS
Traditional lock-free structures might try to claim one slot, then the next, and if the second fails, rollback the first. This violates the $CC=1$ rule because it requires conditional logic. 
Instead, `BpadDispatcher` performs a single, all-or-nothing Compare-And-Swap (CAS):
- It constructs a `target_bits` mask by shifting `1` into the `first` and `second` bit positions.
- Using a branchless boolean mask (`0u8.wrapping_sub(has_two_slots as u8)`), it zeros out the `target_bits` if there aren't at least 2 slots available.
- It proposes the new state: `old | acquire_mask`.
- A single `compare_exchange` instruction attempts to apply the mask. If it succeeds, both slots are claimed atomically without any intermediate partially-claimed state.

## 4. Branchless Write via the "Garbage Lane"
If the ring doesn't have two free slots or if the CAS fails due to contention, the code must still execute a constant-time path without using an `if` statement to skip the memory writes.
- The `BpadDispatcher` physically holds 9 slots: 8 active lanes and a 9th "garbage/sink lane" at index 8.
- A branchless `select` helper evaluates the success of the CAS. 
- If successful, it points the destination indices to `first` and `second`. 
- If it fails, it points the destination indices to `8` (the garbage lane).
- The operation indices (`left` and `right`) are then blindly written to the selected destinations using `Ordering::Release`. A failed request safely overwrites the garbage lane without affecting the active slots.

By relying purely on bitwise arithmetic, atomic CAS instructions, and a dummy sink for failed writes, the dispatcher completely eliminates race conditions, timing side-channels, and standard locking overheads.
