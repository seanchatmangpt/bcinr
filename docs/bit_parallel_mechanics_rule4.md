# Bit-Parallel Mechanics (Rule 4: `@von_neumann_bypass`)

Rule 4 dictates that sequential semantic decisions must be transformed into bit-parallel mechanics over byte-sequential control flow. In the BCINR codebase, this is exclusively achieved using **Mask Calculus** and **SWAR (SIMD Within A Register)** techniques to execute conditional logic securely and deterministically without branching.

Here is a breakdown of the bit-parallel strategies found in the codebase:

## 1. Mask-Based Selection (`mask.rs` & `ct.rs`)
Instead of using conditional branches (like `if / else`) which violate $CC=1$ and risk pipeline mispredictions, BCINR relies on generating full-width bit masks and binary selection.
- **The Core Selection Identity:** All conditionals apply formal mask calculus selection: `(mask & a) | (!mask & b)`.
- **Mask Generation (`mask.rs`):** Masks strictly adhere to the all-ones (`0xFFFFFFFF` for true) or all-zeros (`0x00000000` for false) convention.
  - For `<` comparisons (`lt_mask_u32`), instead of conditional jumps, the code uses `0u32.wrapping_sub((a < b) as u32)`. On x86-64, this compiles to a branchless `SETB` + `NEG` sequence.
  - For equality (`eq_mask_u32`), it isolates the sign bit of the two's complement to detect zero differences: `((x | x.wrapping_neg()) >> 31).wrapping_sub(1)`.

## 2. Constant-Time Comparisons and Arithmetic (`ct.rs`)
The `ct.rs` module ensures execution time remains $O(1)$ and is immune to timing side-channel attacks by operating at the bit level:
- **`ct_lt_u32` (Unsigned Less-Than)**: Uses the classic "borrow propagation trick" to isolate the borrow bit without relying on CPU flags conditionally:
  `((a ^ ((a ^ b) | (a.wrapping_sub(b) ^ b))) >> 31) & 1`
- **Branchless Min/Max**: `ct_min_u32(a, b)` generates the less-than mask and computes `b.wrapping_add(a.wrapping_sub(b) & mask)`. When `a < b`, the mask allows the difference to be added to `b` producing `a`; otherwise, it adds `0` leaving `b`.
- **Absolute Value (`abs_i32`)**: Uses an arithmetic right shift (`x >> 31`) to broadcast the sign bit across the whole register (yielding `-1` or `0`), XORs the value, and subtracts the mask to compute two's complement negation seamlessly: `(x ^ mask).wrapping_sub(mask)`.

## 3. SWAR Strings and Parallel Processing (`swar_str.rs`)
To avoid byte-sequential loops (e.g., iterating string characters), the framework leverages SWAR to process 8 bytes simultaneously inside a standard 64-bit register (`u64`):
- **Broadcast Patterns:** Code utilizes structural masks like `ONES` (`0x0101_0101_0101_0101`) and `HIGHS` (`0x8080_8080_8080_8080`) to affect all 8 byte lanes in parallel.
- **Zero-Byte Detection (`find_byte_in_word`)**: Leverages Hacker's Delight branchless technique to detect exact characters across the 8-byte lane simultaneously:
  `xored.wrapping_sub(ONES) & !xored & HIGHS`
  If a byte matches, the `0x80` bit is set for that specific lane.
- **Counting and Positioning:** Using the mask generated above, counting characters (`count_byte_in_word`) just calls `.count_ones()` (popcount). Finding a byte position uses `.trailing_zeros() / 8`.
- **Parallel Range Checks (`swar_is_in_range`)**: Subtraction and MSB checks calculate bounds matching mathematically to determine if all bytes in a lane fall within an arbitrary byte range `[lo, hi]`.
- **ASCII Transformation**: `to_lower_ascii_word` computes a mask for bytes falling in the `A..Z` range and uses bitwise `|` to set the `0x20` bit on matching lanes simultaneously, bypassing the need for character-by-character checks.
