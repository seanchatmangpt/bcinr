# Branchless Endian Conversions (SWAR `bswap`)

In the `bcinr` (BranchlessCInRust) deterministic substrate, operations must adhere to a strict $CC=1$ (cyclomatic complexity of 1) mandate. Relying on compiler-inserted branching or variable-latency hardware intrinsics for endian conversions violates the "Zero-Branching" and constant-time execution axioms.

To achieve a purely branchless, architecture-agnostic 64-bit byte swap (`bswap`), `bcinr` utilizes **SWAR** (SIMD Within A Register) techniques. Instead of operating on bytes sequentially, it treats the 64-bit register as an array of independent byte lanes and permutes them in parallel using only bitwise masks and shifts (`>>`, `<<`, `&`, `|`).

## The SWAR Byte-Swap Algorithm

A 64-bit byte swap reverses the order of the 8 bytes. In SWAR, this is accomplished via a divide-and-conquer approach in $\log_2(8) = 3$ steps. This technique is often referred to as "delta swapping."

Here is the branchless 64-bit byte swap using purely arithmetic and bitwise logic:

```rust
#[inline(always)]
pub const fn swar_bswap_u64(mut v: u64) -> u64 {
    // Step 1: Swap adjacent bytes (8-bit delta)
    // Mask: 0x00FF00FF00FF00FF
    v = ((v >> 8) & 0x00FF_00FF_00FF_00FF) | ((v & 0x00FF_00FF_00FF_00FF) << 8);

    // Step 2: Swap adjacent 2-byte pairs (16-bit delta)
    // Mask: 0x0000FFFF0000FFFF
    v = ((v >> 16) & 0x0000_FFFF_0000_FFFF) | ((v & 0x0000_FFFF_0000_FFFF) << 16);

    // Step 3: Swap adjacent 4-byte halves (32-bit delta)
    // No mask needed for the final step since the shift clears the bits
    v = (v >> 32) | (v << 32);

    v
}
```

### How It Works

1. **Step 1 (8-bit shift):** 
   - `v >> 8` shifts the odd bytes into the even byte positions. The mask `0x00FF_00FF_00FF_00FF` zeroes out the odd positions.
   - `v << 8` shifts the even bytes into the odd byte positions. The mask zeroes out the even positions before shifting.
   - The bitwise OR `|` combines them, effectively swapping bytes `(0,1)`, `(2,3)`, `(4,5)`, and `(6,7)`.

2. **Step 2 (16-bit shift):** 
   - The same logic applies, but with a 16-bit shift and a mask of `0x0000_FFFF_0000_FFFF`.
   - This swaps the 2-byte chunks: `(0-1, 2-3)` and `(4-5, 6-7)`.

3. **Step 3 (32-bit shift):**
   - The final step swaps the upper 32 bits with the lower 32 bits. 
   - Because the shift amount matches half the register size, masking is redundant—the shift inherently clears the vacated bits.

## Compliance with `bcinr` Mandates

This implementation is mathematically proven to be $CC=1$ and executes in deterministic, constant time across all platforms. 

- **No Data-Dependent Branches:** There are zero `if` statements or conditional jumps. 
- **No Hardware Intrinsics:** It relies entirely on standard boolean logic and shifts, making it perfectly portable without falling back to architecture-specific `bswap` instructions (which can sometimes compile to branching logic on unsupported targets).
- **Zero Allocation:** The entire operation stays within the CPU's general-purpose registers (`#![no_std]` compliant).

This approach natively maps to the `delta_swap_u64` primitive found in the `bcinr-logic` crate, guaranteeing identical instruction shapes and latency regardless of the input data.
