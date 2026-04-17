# Reference: Full API Catalog

## 1. Bitset Algebra
- `set_bit_u64(x, pos)`: Sets bit at `pos`.
- `clear_bit_u64(x, pos)`: Clears bit at `pos`.
- `rank_u64(x, pos)`: Count set bits up to `pos`.
- `select_bit_u64(x, nth)`: Finds index of `nth` set bit.
- `parity_u64_slice(a)`: Calculates parity of slice.
- `jaccard_u64_slices(a, b)`: Computes Jaccard index.
- `hamming_u64_slices(a, b)`: Computes Hamming distance.
- `intersect_u64_slices(a, b, out)`: In-place intersection.
- `union_u64_slices(a, b, out)`: In-place union.
- `any_bit_set_u64_slice(a)`: Checks if any bit is set.

## 2. Saturation Arithmetic
- `add_sat_u8(a, b)`, `add_sat_u32(a, b)`: Clamped addition.
- `sub_sat_u8(a, b)`, `sub_sat_u32(a, b)`: Clamped subtraction.
- `mul_sat_u32(a, b)`: Clamped multiplication.
- `inc_sat_u32(x)`, `dec_sat_u32(x)`: Monotonic bounded counters.
- `clamp_u32(x, min, max)`: Clamps value to range.
- `bucketize_u32<const N>(x, buckets)`: Maps value to index.

## 3. SWAR & SIMD
- `splat_u8x16(val)`: Broadcast value.
- `shuffle_u8x16(a, b, mask)`: Permutes vector elements.
- `movemask_u8x16(a)`: Sign-bit scalar projection.
- `has_zero_byte_u64(x)`: Null-byte detection.
- `has_less_than_byte_u64(x, val)`: Byte threshold filtering.

## 4. DFA & Parsing
- `dfa_advance(state, input, table, size)`: Single step.
- `dfa_run(input, table, size, start)`: Bulk execution.
- `dfa_is_accepting(state, accepts)`: State validation.
- `skip_whitespace(bytes)`: Fast-forward ASCII.
- `parse_hex_u32(bytes)`: Hex-to-int conversion.
- `parse_decimal_u64(bytes)`: Dec-to-int conversion.
- `skip_spaces(bytes)`: Advanced whitespace skipping.

## 5. Network & Sorting
- `compare_exchange(a, b)`: Atomic sort-step.
- `bitonic_sort_32u32(arr)`: Fixed-depth network sort.
- `sort3_u32(arr)`, `sort4_u32(arr)`, `sort5_u32(arr)`: Micro-sorting.

## 6. Integer & Mask
- `popcount_u64/u32(x)`: Bit population count.
- `leading_zeros_u64/u32(x)`, `trailing_zeros_u64/u32(x)`: Bit position.
- `reverse_bits_u64/u32(x)`: Bit reversal.
- `saturating_add/sub/mul_i64(a, b)`: Signed saturation.
- `select_u32/u64(mask, a, b)`: Branchless multiplexer.
- `eq_mask_u32(a, b)`, `is_zero_mask_u32(x)`: Mask generation.
- `min/max_u32(a, b)`: Branchless extremum.
- `abs_i32(x)`: Branchless magnitude.
