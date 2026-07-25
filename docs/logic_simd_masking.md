# BCINR Vectorization and Masking Approach

BCINR's architecture scales branchless logic using two distinct layers for SIMD vectorization, strictly adhering to the $CC=1$ "Radon Law" (zero conditional branches).

## 1. Portable Auto-Vectorization (`simd.rs`)
The base `simd.rs` module models 128-bit vector operations entirely using fixed-size Rust arrays (`[u8; 16]`). 
- Instead of using architecture-specific SIMD intrinsics (like `__m128i`), operations like `splat_u8x16`, `shuffle_u8x16`, and `movemask_u8x16` are written using pure scalar loops and bitwise math.
- **Why?** It ensures perfect portability across targets (including `no_std` WebAssembly) without requiring conditional compilation or `unsafe` blocks. 
- It relies on LLVM's auto-vectorizer to compile the simple loop unrolling into native SIMD instructions (e.g. `PSHUFB`).

## 2. Compile-Time Hardware Dispatch (`simd_dispatch.rs`)
For targets where autovectorization might fall short or to guarantee specific hardware paths, BCINR uses `simd_dispatch.rs` (a SIMDe-style dispatch layer). It defines three paths routed at compile time:
- **x86_64 (SSE4.2/SSSE3)**: Explicit intrinsic calls like `_mm_set1_epi8`, `_mm_shuffle_epi8`, `_mm_movemask_epi8`, and `_mm_cmpeq_epi8`.
- **AArch64 (NEON)**: Explicit intrinsic calls like `vdupq_n_u8`, `vqtbl1q_u8`, etc.
- **Fallback**: The same pure-Rust scalar array implementations from `simd.rs`.

## 3. Interaction Between SIMD Vectors and Masks
Masks in BCINR follow a strict all-ones (`0xFF` / `0xFFFFFFFF`) or all-zeros (`0x00`) convention.
- **Condition Evaluation**: SIMD comparison operations (like `_mm_cmpeq_epi8` in x86_64 or `((a[i] == b[i]) as u8).wrapping_neg()` in scalar) generate masks directly inline without branching.
- **Branchless Selection**: Mask application is handled via bitwise arithmetic rather than conditional execution. For example, branchless `min_u8x16` selects minimum values across 16 lanes simultaneously using a mask-based XOR trick: `b ^ ((a ^ b) & mask)`.
- **Shuffle Masking**: A 16-byte mask drives the byte-shuffle logic (analogous to `pshufb`). By setting the high bit (`0x80`) of a mask byte, the specific lane is forcibly zeroed.
- **Movemask Operations**: To summarize vectors back to scalar conditions, `movemask_u8x16` extracts the most-significant bit (sign bit) from each of the 16 bytes, packing them into a single `u16` mask. This directly maps to `_mm_movemask_epi8`.
