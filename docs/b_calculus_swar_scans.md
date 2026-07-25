# B-Calculus Primitives: SWAR and Parallel Prefix Scans

The core requirement of `bcinr` (B-Calculus) is the Radon Law ($CC=1$), meaning no data-dependent branches (`if`, `match`, or variable loops). The parallel prefix primitives achieve this by utilizing fixed-width fully unrolled networks and bitwise arithmetic.

Here is how these B-Calculus primitives operate:

### 1. Fully-Unrolled Parallel Prefix Sum (`prefix_sum_u32x16`)
This primitive computes the inclusive prefix sum of 16 `u32` values. Instead of a variable loop, it implements a fully unrolled Hillis-Steele parallel prefix network with an up-sweep and down-sweep.
- It operates in exactly $\log_2(16) = 4$ passes.
- For each pass (with strides of 1, 2, 4, and 8), it uses `.wrapping_add()` between statically known indices.
- Because the dependencies are fixed at compile time, the execution is a straight line of instructions with zero conditional branches.

### 2. Exclusive Blelloch Scan (`exclusive_scan_u32x16`)
Built directly on top of the inclusive sum, this returns an exclusive prefix sum where `out[0] = 0` and `out[i] = arr[0] + ... + arr[i-1]`.
- It executes `prefix_sum_u32x16` and then statically shifts the result array right by one index, inserting `0` at the beginning. This avoids any runtime conditional logic.

### 3. Segmented Prefix Sum (`segmented_prefix_sum_u32x8`)
This computes an inclusive prefix sum for 8 elements but resets the accumulator based on an array of `bool` flags indicating segment boundaries.
- **Branchless Reset Trick**: Instead of `if flag { acc = 0 }`, it computes a reset mask: `let reset = (flags[i] as u32).wrapping_neg();`
- If `flags[i]` is `true` (1), `reset` becomes `0xFFFF_FFFF`. If `false` (0), `reset` becomes `0`.
- It then masks the accumulator: `acc &= !reset;` before adding the next value, successfully wiping the accumulator branchlessly at segment boundaries.

### 4. Prefix Maximum (`prefix_max_u32x16`)
Computes the running maximum of an array.
- Instead of using `if current > max`, it leverages `u32::max(prev, current)` which the Rust compiler explicitly lowers to the branchless `CMOV` (conditional move) instruction on x86 architectures. This guarantees $CC=1$.

### 5. Gray-Code Prefix XOR (`prefix_xor_u64x8`)
Computes the running XOR of 8 `u64` values (`out[i] = arr[0] ^ ... ^ arr[i]`).
- It statically unrolls 7 sequential XOR operations (`a[1] ^= a[0]`, etc.), acting as a branchless Gray-code transformer.

### 6. SWAR Byte Position Scans (`swar_find_all_positions` & `count_leading_eq_u8`)
These operations scan arrays for specific bytes (like searching for a character) without branching on each byte.
- **Broadcast & XOR**: The target byte is multiplied by `0x0101_0101_0101_0101` to broadcast it across an entire 64-bit register. XORing a block of 8 bytes with this register turns any matching bytes into `0x00`.
- **Zero-Byte Detection**: It then applies the standard SWAR formula: 
  `zero_bytes = xored.wrapping_sub(0x0101_0101_0101_0101) & !xored & 0x8080_8080_8080_8080`.
- This strictly isolated arithmetic sequence sets the high bit of any byte lane that was `0x00` (meaning it matched the target byte), completely eliminating loop-based scanning.
