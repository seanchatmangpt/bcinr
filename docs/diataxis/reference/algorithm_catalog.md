# Algorithm Catalog

Complete reference for all 300+ branchless algorithms in `crates/bcinr-logic/src/algorithms/`.
Organized by family; difficulty levels indicate relative algorithmic complexity (1 = simplest,
300 = most involved). All algorithms are `O(1)` in the branchless sense: no data-dependent
branches, constant instruction count per call.

---

## Bit Manipulation (Difficulty 1-50)

Core single-instruction-class operations on integer words. These are the atoms of
all higher-level algorithms.

| Algorithm | Function | I/O Type | Description |
|-----------|----------|----------|-------------|
| `blsi_u64` | `blsi_u64` | u64 → u64 | Isolate lowest set bit: `x & x.wrapping_neg()` |
| `blsr_u64` | `blsr_u64` | u64 → u64 | Reset lowest set bit: `x & (x - 1)` |
| `blsmsk_u64` | `blsmsk_u64` | u64 → u64 | Create mask up to and including lowest set bit |
| `bclr_u64` | `bclr_u64` | (u64, u32) → u64 | Clear bit at given position |
| `bset_u64` | `bset_u64` | (u64, u32) → u64 | Set bit at given position |
| `btst_u64` | `btst_u64` | (u64, u32) → bool | Test bit at given position |
| `bext_u64` | `bext_u64` | (u64, u32, u32) → u64 | Extract bitfield (BMI1 BEXTR analogue) |
| `compress_bits_u64` | `compress_bits_u64` | (u64, u64) → u64 | Compress bits selected by mask (BMI2 PEXT) |
| `expand_bits_u64` | `expand_bits_u64` | (u64, u64) → u64 | Expand bits into mask positions (BMI2 PDEP) |
| `parallel_bits_deposit_u64` | `parallel_bits_deposit_u64` | (u64, u64) → u64 | Alias for expand (PDEP) |
| `parallel_bits_extract_u64` | `parallel_bits_extract_u64` | (u64, u64) → u64 | Alias for compress (PEXT) |
| `gather_bits_u64` | `gather_bits_u64` | (u64, u64) → u64 | Gather bits from arbitrary positions |
| `scatter_bits_u64` | `scatter_bits_u64` | (u64, u64) → u64 | Scatter bits to arbitrary positions |
| `weight_u64` | `weight_u64` | u64 → u32 | Population count (Hamming weight) |
| `popcount_u128` | `popcount_u128` | u128 → u32 | Population count for 128-bit integers |
| `parity_check_u128` | `parity_check_u128` | u128 → bool | XOR parity of all bits |
| `count_consecutive_set_bits_u64` | `count_consecutive_set_bits_u64` | u64 → u32 | Length of leading run of 1s |
| `bit_swap_u64` | `bit_swap_u64` | (u64, u32, u32) → u64 | Swap two bits at given positions |
| `delta_swap_u64` | `delta_swap_u64` | (u64, u64, u32) → u64 | Swap interleaved bit groups |
| `bit_permute_step_u64` | `bit_permute_step_u64` | (u64, u64, u32) → u64 | One Benes network permutation step |
| `bit_permute_identity_64` | `bit_permute_identity_64` | u64 → u64 | Identity permutation (baseline / test) |
| `funnel_shift_left_u64` | `funnel_shift_left_u64` | (u64, u64, u32) → u64 | Shift across two words leftward |
| `funnel_shift_right_u64` | `funnel_shift_right_u64` | (u64, u64, u32) → u64 | Shift across two words rightward |
| `rotate_left_u64` | `rotate_left_u64` | (u64, u32) → u64 | Branchless left rotation |
| `rotate_right_u64` | `rotate_right_u64` | (u64, u32) → u64 | Branchless right rotation |
| `gray_encode_u64` | `gray_encode_u64` | u64 → u64 | Binary to Gray code |
| `gray_decode_u64` | `gray_decode_u64` | u64 → u64 | Gray code to binary |
| `reverse_bits_u128` | `reverse_bits_u128` | u128 → u128 | Reverse all bits |
| `get_mask_boundary_low_u64` | `get_mask_boundary_low_u64` | u32 → u64 | Mask of N lowest bits |
| `get_mask_boundary_high_u64` | `get_mask_boundary_high_u64` | u32 → u64 | Mask of N highest bits |
| `mask_range_u64` | `mask_range_u64` | (u32, u32) → u64 | Mask of bits lo..hi |
| `is_contiguous_mask_u64` | `is_contiguous_mask_u64` | u64 → bool | True if set bits form a contiguous run |
| `is_subset_mask_u64` | `is_subset_mask_u64` | (u64, u64) → bool | True if all bits of `a` are set in `b` |
| `mask_xor_reduce_u64` | `mask_xor_reduce_u64` | u64 → u64 | XOR-reduce under a bit mask |
| `t1mskc_u64` | `t1mskc_u64` | u64 → u64 | Trailing ones mask complement |
| `tzmsk_u64` | `tzmsk_u64` | u64 → u64 | Trailing zeros mask |
| `find_nth_set_bit_u128` | `find_nth_set_bit_u128` | (u128, u32) → u32 | Position of the N-th set bit |

---

## Saturation Arithmetic (Difficulty 1-30)

Operations that clamp to type boundaries instead of wrapping or panicking.

| Algorithm | Function | I/O Type | Description |
|-----------|----------|----------|-------------|
| `add_sat_i32` | `add_sat_i32` | (i32, i32) → i32 | Saturating addition for signed 32-bit |
| `sub_sat_i32` | `sub_sat_i32` | (i32, i32) → i32 | Saturating subtraction for signed 32-bit |
| `mul_sat_i32` | `mul_sat_i32` | (i32, i32) → i32 | Saturating multiplication for signed 32-bit |
| `mul_sat_u64` | `mul_sat_u64` | (u64, u64) → u64 | Saturating multiplication for unsigned 64-bit |
| `div_sat_u64` | `div_sat_u64` | (u64, u64) → u64 | Saturating division (avoids divide-by-zero) |
| `pow_sat_u64` | `pow_sat_u64` | (u64, u32) → u64 | Saturating exponentiation |
| `binom_sat_u32` | `binom_sat_u32` | (u32, u32) → u32 | Saturating binomial coefficient |
| `factorial_sat_u32` | `factorial_sat_u32` | u32 → u32 | Saturating factorial |
| `clamp_i64` | `clamp_i64` | (i64, i64, i64) → i64 | Clamp to `[lo, hi]` branchlessly |
| `clamp_slice_branchless` | `clamp_slice_branchless` | (&mut [i64], i64, i64) | In-place clamp every element |
| `clamped_scaling_u64` | `clamped_scaling_u64` | (u64, u64, u64) → u64 | Scale then saturate to bound |

---

## Arithmetic Utilities (Difficulty 1-40)

| Algorithm | Function | I/O Type | Description |
|-----------|----------|----------|-------------|
| `abs_diff_u64` | `abs_diff_u64` | (u64, u64) → u64 | Absolute difference without overflow |
| `abs_diff_i64` | `abs_diff_i64` | (i64, i64) → u64 | Absolute difference for signed integers |
| `branchless_signum_i64` | `branchless_signum_i64` | i64 → i32 | Sign (-1, 0, +1) without branches |
| `copy_sign_i64` | `copy_sign_i64` | (i64, i64) → i64 | Apply sign of one value to magnitude of another |
| `avg_u64` | `avg_u64` | (u64, u64) → u64 | Average without overflow |
| `avg_ceil_u64` | `avg_ceil_u64` | (u64, u64) → u64 | Ceiling average without overflow |
| `round_up_u32` | `round_up_u32` | (u32, u32) → u32 | Round up to next multiple |
| `round_down_u32` | `round_down_u32` | (u32, u32) → u32 | Round down to previous multiple |
| `round_to_nearest_u32` | `round_to_nearest_u32` | (u32, u32) → u32 | Round to nearest multiple |
| `gcd_u64_branchless` | `gcd_u64_branchless` | (u64, u64) → u64 | GCD via binary (Stein) algorithm |
| `lcm_u64_branchless` | `lcm_u64_branchless` | (u64, u64) → u64 | LCM via GCD |
| `modular_add_u64` | `modular_add_u64` | (u64, u64, u64) → u64 | `(a + b) % m` without overflow |
| `modular_sub_u64` | `modular_sub_u64` | (u64, u64, u64) → u64 | `(a - b) % m` without underflow |
| `modular_mul_u64` | `modular_mul_u64` | (u64, u64, u64) → u64 | `(a * b) % m` without overflow |
| `is_prime_u64_branchless` | `is_prime_u64_branchless` | u64 → bool | Miller-Rabin primality test |
| `norm_u32` | `norm_u32` | &[u32] → u64 | L1 norm of a slice |
| `weighted_avg_u32` | `weighted_avg_u32` | (&[u32], &[u32]) → u32 | Weighted average |
| `leaky_relu_u32` | `leaky_relu_u32` | (u32, u32) → u32 | Leaky ReLU activation |
| `relu_u32` | `relu_u32` | i32 → u32 | ReLU (branchless max-with-zero) |
| `sigmoid_sat_u32` | `sigmoid_sat_u32` | i32 → u32 | Saturating sigmoid approximation |
| `smoothstep_u32` | `smoothstep_u32` | (u32, u32, u32) → u32 | Hermite smoothstep interpolation |
| `lerp_sat_u32` | `lerp_sat_u32` | (u32, u32, u32) → u32 | Linear interpolation, clamped |
| `lerp_sat_u8` | `lerp_sat_u8` | (u8, u8, u8) → u8 | Linear interpolation in 8-bit space |
| `cubic_interpolate_u32` | `cubic_interpolate_u32` | (u32, u32, u32, u32, u32) → u32 | Cubic interpolation |

---

## Fixed-Point Math (Difficulty 30-60)

| Algorithm | Function | I/O Type | Description |
|-----------|----------|----------|-------------|
| `fp_mul_u32_q16` | `fp_mul_u32_q16` | (u32, u32) → u32 | Q16.16 fixed-point multiply |
| `fp_div_u32_q16` | `fp_div_u32_q16` | (u32, u32) → u32 | Q16.16 fixed-point divide |
| `fp_sin_u32_q16` | `fp_sin_u32_q16` | u32 → i32 | Sine via lookup + interpolation (Q16) |
| `fp_cos_u32_q16` | `fp_cos_u32_q16` | u32 → i32 | Cosine via lookup + interpolation (Q16) |
| `fp_atan2_u32_q16` | `fp_atan2_u32_q16` | (i32, i32) → i32 | Arctangent2 in fixed-point |
| `fp_sqrt_u32_q16` | `fp_sqrt_u32_q16` | u32 → u32 | Square root in fixed-point |
| `fast_inverse_sqrt_u32` | `fast_inverse_sqrt_u32` | f32 → f32 | Fast inverse square root (Quake method) |
| `exp2_u64_fixed` | `exp2_u64_fixed` | u64 → u64 | 2^x in fixed-point arithmetic |
| `log2_u64_fixed` | `log2_u64_fixed` | u64 → u64 | log2(x) in fixed-point arithmetic |
| `fixed_point_log2` | `fixed_point_log2` | u32 → u32 | Base-2 logarithm (Q16 output) |
| `dequantize_u32` | `dequantize_u32` | (u32, u32, u32) → u32 | Inverse uniform quantization |
| `quantize_u32` | `quantize_u32` | (u32, u32, u32) → u32 | Uniform quantization to N levels |

---

## Comparison, Selection, and Ordering (Difficulty 10-50)

| Algorithm | Function | I/O Type | Description |
|-----------|----------|----------|-------------|
| `median3_u32` | `median3_u32` | (u32, u32, u32) → u32 | Median of three values |
| `median5_u32` | `median5_u32` | 5× u32 → u32 | Median of five values |
| `median9_u32` | `median9_u32` | 9× u32 → u32 | Median of nine values |
| `minmax_element_branchless_u32` | `minmax_element_branchless_u32` | &[u32] → (u32, u32) | Simultaneous min and max |
| `min_element_branchless_u32` | `min_element_branchless_u32` | &[u32] → u32 | Branchless linear min scan |
| `max_element_branchless_u32` | `max_element_branchless_u32` | &[u32] → u32 | Branchless linear max scan |
| `is_sorted_branchless_u32` | `is_sorted_branchless_u32` | &[u32] → bool | Verify sorted order branchlessly |
| `is_permutation_branchless` | `is_permutation_branchless` | (&[u32], &[u32]) → bool | Test if two slices are permutations |

---

## Sorting Networks (Difficulty 51-120)

Deterministic, comparison-network-based sorting with fixed depth and comparator count.

| Algorithm | Elements | Comparators | Depth | Notes |
|-----------|----------|-------------|-------|-------|
| `sort_pairs_u32x4` | 4 | 5 | 3 | Optimal 4-element network |
| `optimal_sort_5_u32` | 5 | 9 | 5 | Optimal 5-element network |
| `optimal_sort_6_u32` | 6 | 12 | 6 | Optimal 6-element network |
| `optimal_sort_7_u32` | 7 | 16 | 6 | Near-optimal |
| `optimal_sort_8_u32` | 8 | 19 | 6 | Batcher odd-even |
| `bit_parallel_sort8_u32` | 8 | 19 | 6 | Bit-parallel Batcher sort |
| `insertion_sort_branchless_fixed` | N (small) | N(N-1)/2 | N-1 | For N ≤ 16; sentinel-free |
| `green_sorting_network_16` | 16 | 60 | 10 | Green's network, depth-optimal |
| `odd_even_merge_sort_16u32` | 16 | 60 | 10 | Batcher odd-even merge sort |
| `merge_sorted_u32x8` | 8+8 → 16 | 24 | 8 | Merge two sorted 8-element arrays |
| `bitonic_merge_u64x8` | 8 | 24 | 6 | Bitonic merge for u64 |
| `bitonic_sort_64u32` | 64 | ~2016 | 21 | Full bitonic sort |
| `sorting_network_verify_u32` | N | — | — | Verify a network sorts correctly |
| `shear_sort_bitonic_2d` | N×M | — | — | 2D shear sort with bitonic pass |

---

## Search and Lookup (Difficulty 30-90)

| Algorithm | Description |
|-----------|-------------|
| `binary_search_v_u32x4` | Vectorized binary search over sorted `[u32]` (4 probes/iter) |
| `unrolled_binary_search_u32` | Unrolled binary search with predictable branch pattern |
| `lower_bound_branchless_u32` | Branchless lower_bound (Eytzinger-style) |
| `upper_bound_branchless_u32` | Branchless upper_bound |
| `equal_range_branchless_u32` | Combined lower + upper bound in one pass |
| `search_eytzinger_u32` | Cache-friendly Eytzinger-layout binary search |
| `search_van_emde_boas` | Van Emde Boas tree lookup |
| `linear_search_simd_u8` | SWAR-accelerated linear search in byte slices |
| `find_first_of_branchless` | First position of any byte from a set |
| `find_last_of_branchless` | Last position of any byte from a set |
| `mismatch_branchless_u8` | First position where two slices differ |
| `simd_memchr_u8x16` | 16-byte SIMD `memchr` analogue |
| `simd_memrchr_u8x16` | 16-byte SIMD reverse `memchr` |
| `simd_strstr_branchless` | Substring search (branchless Horspool) |

---

## Hash Functions (Difficulty 50-150)

| Algorithm | Output | Quality | Relative Speed | Use Case |
|-----------|--------|---------|---------------|----------|
| `fnv1a_64_hash` | 64-bit | Non-crypto | Fast | HashMap, diagnostics |
| `murmur3_32_hash` | 32-bit | Non-crypto | Fast | General purpose |
| `murmur3_x64_128` | 128-bit | Non-crypto | Fast | Large key dedup |
| `xxhash64` | 64-bit | Non-crypto | Very fast | Checksums, indexing |
| `xxh3_64` | 64-bit | Non-crypto | Very fast | Preferred general-purpose |
| `wyhash_64` | 64-bit | Non-crypto | Very fast | Stream hashing |
| `metrohash64` | 64-bit | Non-crypto | Very fast | Latency-sensitive |
| `highwayhash_64` | 64-bit | Non-crypto | Fast | SIMD-friendly |
| `farmhash64` | 64-bit | Non-crypto | Fast | Google workloads |
| `cityhash64` | 64-bit | Non-crypto | Fast | String hashing |
| `clhash` | 64-bit | Non-crypto | Fast | CLMUL-accelerated |
| `spookyhash_v2_128` | 128-bit | Non-crypto | Fast | Large inputs |
| `siphash_2_4_branchless` | 64-bit | DoS-resistant | Moderate | Public-facing inputs |
| `adler32_branchless` | 32-bit | Checksum | Very fast | Data integrity |
| `bsd_checksum_u16` | 16-bit | Checksum | Very fast | Legacy compatibility |
| `crc32c_branchless` | 32-bit | Checksum | Fast | Storage integrity |
| `cyclic_redundancy_check_crc32c` | 32-bit | Checksum | Fast | Network/storage CRC |
| `cyclic_redundancy_check_crc64` | 64-bit | Checksum | Fast | Large file integrity |
| `internet_checksum_u16` | 16-bit | Checksum | Very fast | TCP/IP checksum |
| `fletcher32_branchless` | 32-bit | Checksum | Fast | Embedded diagnostics |
| `pearson_hash_u8` | 8-bit | Non-crypto | Very fast | Micro-lookup tables |
| `pearson_hash_16` | 16-bit | Non-crypto | Very fast | Larger lookup tables |
| `polynomial_hash_u64` | 64-bit | Non-crypto | Fast | Rolling hash base |
| `tabulation_hash_u64` | 64-bit | Non-crypto | Fast | 4-independent hashing |
| `k_independent_hash_gen` | 64-bit | Non-crypto | Moderate | k-independent families |
| `knuth_hash_u64` | 64-bit | Non-crypto | Very fast | Fibonacci hashing |
| `fibonacci_hash_u64` | 64-bit | Non-crypto | Very fast | Bucket selection |
| `zobrist_hash_64` | 64-bit | Non-crypto | Very fast | Board-game state hashing |
| `hashing_trick_u64` | 64-bit | Non-crypto | Fast | Feature hashing (ML) |

---

## Probabilistic Data Structures (Difficulty 100-200)

| Algorithm | Structure | Operation | False Positive Rate |
|-----------|-----------|-----------|-------------------|
| `bloom_filter_add_u64` | Bloom filter | Insert | — |
| `bloom_filter_query_u64` | Bloom filter | Membership | Configurable |
| `bloom_filter_union` | Bloom filter | Union | — |
| `bloom_filter_intersect` | Bloom filter | Intersection | — |
| `bloom_filter_graph_visited` | Bloom filter | Graph traversal | Configurable |
| `xor_filter_lookup` | XOR filter | Membership | Very low |
| `cuckoo_filter_add_u64` | Cuckoo filter | Insert | Low |
| `quotient_filter_add_u64` | Quotient filter | Insert | Low |
| `count_min_sketch_add` | Count-min sketch | Increment counter | — |
| `count_min_sketch_query` | Count-min sketch | Point query | Overestimate |
| `count_min_sketch_update` | Count-min sketch | Batch update | — |
| `hyperloglog_add_u64` | HyperLogLog | Add element | — |
| `hyperloglog_add_u64_registers` | HyperLogLog | Register-level add | — |
| `hyperloglog_merge` | HyperLogLog | Merge two sketches | — |
| `cardinality_linear_counting` | Linear counting | Cardinality estimate | Low |
| `minhash_u64_k` | MinHash | Jaccard estimate | Configurable |
| `simhash_cosine_u64` | SimHash | Cosine similarity | Configurable |
| `locality_sensitive_hash_cosine` | LSH | Near-duplicate detection | Configurable |
| `locality_sensitive_hash_euclidean` | LSH | Euclidean neighbor | Configurable |
| `heavy_hitter_update` | Heavy hitters | Update frequency estimate | — |
| `heavy_keepers_add` | Heavy Keepers | Conservative update | — |
| `misra_gries_add` | Misra-Gries | Majority / heavy hitter | — |
| `space_saving_add` | Space-Saving | Frequent items | — |
| `t_digest_add_u32` | t-Digest | Quantile estimation | — |

---

## String and Text Processing (Difficulty 50-150)

| Algorithm | Description |
|-----------|-------------|
| `ascii_to_lowercase_simd` | Branchless ASCII lowercase (8 bytes/op via SWAR) |
| `ascii_to_uppercase_simd` | Branchless ASCII uppercase (8 bytes/op via SWAR) |
| `hex_encode_chunk8` | Encode 8 bytes to hex without branches |
| `hex_encode_simd` | SIMD-accelerated hex encoding |
| `hex_decode_simd` | SIMD-accelerated hex decoding |
| `base64_encode_simd` | Branchless Base64 encoding |
| `base64_decode_simd` | Branchless Base64 decoding |
| `base64_decode_chunk4` | Decode one 4-byte Base64 block |
| `base32_encode_rfc4648` | RFC 4648 Base32 encoding |
| `base85_encode_ascii85` | ASCII85 / Ascii85 encoding |
| `url_encode_branchless` | Percent-encode URL characters |
| `url_decode_branchless` | Decode percent-encoded URLs |
| `punycode_encode_branchless` | IDNA Punycode encoding |
| `trim_whitespace_branchless` | Strip leading/trailing whitespace |
| `split_lines_simd` | Find line endings via SWAR |
| `csv_scan_row_simd` | Scan one CSV row for field boundaries |
| `json_find_structural_simd` | Find JSON structural characters (`{}[],:`) |
| `json_find_string_escapes_simd` | Find backslash escapes in JSON strings |
| `wildcard_match_branchless` | Pattern match with `*` and `?` wildcards |
| `soundex_encode_branchless` | Soundex phonetic encoding |
| `metaphone_encode_branchless` | Metaphone phonetic encoding |
| `lex_compare_u8_slices_branchless` | Lexicographic comparison without branches |
| `find_first_of_branchless` | Position of first byte from a character set |
| `find_last_of_branchless` | Position of last byte from a character set |
| `jaro_winkler_branchless` | Jaro-Winkler string similarity |
| `levenshtein_dist_branchless` | Edit distance (branchless DP) |

---

## Encoding / Compression Primitives (Difficulty 80-180)

| Algorithm | Description |
|-----------|-------------|
| `zigzag_encode_i64` | Map signed i64 to unsigned (ZigZag for Protobuf) |
| `zigzag_decode_i64` | Reverse ZigZag mapping |
| `leb128_encode_u64` | Variable-length Base128 encoding |
| `leb128_decode_u64` | Variable-length Base128 decoding |
| `varint_encode_simd` | SIMD batch varint encoding |
| `varint_decode_simd` | SIMD batch varint decoding |
| `bitpacking_encode_u32_k` | Bit-packing k-bit integers |
| `bitpacking_decode_u32_k` | Bit-unpacking k-bit integers |
| `delta_encode_simd_u32` | Delta encoding (sequential difference) |
| `delta_decode_simd_u32` | Delta decoding (prefix sum) |
| `bit_vector_compress_elias_fano` | Elias-Fano monotone sequence compression |
| `burrows_wheeler_transform_step` | One BWT suffix-array step |
| `rolling_hash_rabin_karp` | Rabin-Karp rolling polynomial hash |
| `rolling_hash_buzhash` | BuzHash cyclic polynomial hash |
| `rolling_hash_gear` | Gear hash for content-defined chunking |
| `content_defined_chunking_branchless` | CDC boundary detection (FastCDC-style) |
| `adler32_branchless` | Adler-32 streaming checksum |
| `clmul_u64` | Carry-less multiplication (GF(2) polynomial) |

---

## UTF-8 / Unicode (Difficulty 80-150)

| Algorithm | Description |
|-----------|-------------|
| `utf8_validate_chunk8` | Validate 8 UTF-8 bytes via SWAR |
| `utf8_to_utf16_simd` | Transcode UTF-8 to UTF-16 |
| `utf8_to_utf32_simd` | Transcode UTF-8 to UTF-32 (codepoints) |
| `utf16_to_utf8_simd` | Transcode UTF-16 to UTF-8 |
| `is_alphanumeric_simd_u8x16` | 16-byte SIMD alphanumeric classification |
| `is_digit_simd_u8x16` | 16-byte SIMD digit classification |
| `is_space_simd_u8x16` | 16-byte SIMD whitespace classification |

---

## Parsing and Pattern Matching (Difficulty 100-200)

| Algorithm | Description |
|-----------|-------------|
| `aho_corasick_simd_step` | One SIMD step in Aho-Corasick multi-pattern search |
| `regex_nfa_simd_step` | NFA simulation step (bitset-parallel) |
| `huffman_decode_table_step` | Table-driven Huffman symbol decode |
| `lcp_array_step_branchless` | Longest common prefix array construction |
| `suffix_array_step_branchless` | SA-IS suffix array construction step |

---

## Permutation Networks (Difficulty 150-250)

| Algorithm | Description |
|-----------|-------------|
| `benes_network_u64` | Full Benes network (log2(N) stages) for 64-element permutation |
| `crossbar_permute_u8x16` | 16-lane crossbar permutation via SIMD shuffle |
| `permute_u32x8` | Permute 8 u32 values according to an index array |
| `inverse_permute_u32x8` | Compute and apply inverse permutation |
| `next_lexicographic_permutation_u64` | Next permutation in lexicographic order |
| `next_combination_u64` | Next bit-combination of same popcount (Gosper's hack) |
| `bit_matrix_transpose_8x8` | Transpose an 8×8 bit matrix |
| `bit_matrix_transpose_64x64` | Transpose a 64×64 bit matrix |

---

## PRNG and Sampling (Difficulty 50-150)

| Algorithm | Period | Quality | Speed | Use Case |
|-----------|--------|---------|-------|----------|
| `splitmix64_u64` | 2^64 | Good | Very fast | Seed splitter |
| `xoroshiro128_plus` | 2^128-1 | Good | Very fast | General simulation |
| `linear_congruential_generator_u64` | 2^64 | Weak | Very fast | Embedded RNG |
| `pcg_random_u64` | 2^128 | Excellent | Fast | Games, simulation |
| `mersenne_twister_step_simd` | 2^19937-1 | Good | Moderate | Statistical work |
| `halton_sequence_u32` | — | Low discrepancy | Fast | Quasi-Monte Carlo |
| `halton_sampler_simd` | — | Low discrepancy | Fast | SIMD quasi-MC |
| `gaussian_noise_box_muller` | — | — | Moderate | Normal distribution |
| `poisson_noise_branchless` | — | — | Moderate | Poisson distribution |
| `shuffle_fisher_yates_branchless` | — | — | Fast | Random permutation |
| `random_permutation_fixed_seed` | — | — | Fast | Deterministic shuffle |
| `reservoir_sample_branchless` | — | — | Fast | Uniform reservoir |
| `weighted_reservoir_sample` | — | — | Moderate | Weighted reservoir |

---

## Geometry and Spatial (Difficulty 80-200)

| Algorithm | Description |
|-----------|-------------|
| `aabb_intersect_branchless` | Axis-aligned bounding box intersection test |
| `frustum_culling_branchless` | Camera frustum visibility test |
| `point_in_polygon_branchless` | Ray-casting polygon containment |
| `ray_sphere_intersect_branchless` | Ray-sphere intersection (branchless discriminant) |
| `ray_triangle_intersect_branchless` | Möller-Trumbore ray-triangle test |
| `convex_hull_monotone_chain_step` | One step of Andrew's monotone chain |
| `euclidean_dist_sq_u32x2` | Squared Euclidean distance in 2D |
| `manhattan_dist_u32x2` | Manhattan (L1) distance in 2D |
| `morton_encode_2d_u32` | 2D Morton (Z-order) encoding |
| `morton_decode_2d_u32` | 2D Morton decoding |
| `morton_encode_3d_u32` | 3D Morton encoding |
| `z_order_curve_2d_u32` | Z-order curve index for 2D point |
| `hilbert_curve_encode_u32` | 2D Hilbert curve encoding |
| `hilbert_curve_decode_u32` | 2D Hilbert curve decoding |
| `spatial_hash_u32` | Spatial hashing for 2D/3D grids |
| `octree_insert_branchless` | Branchless octree child selection |
| `quadtree_insert_branchless` | Branchless quadtree child selection |
| `vector_dot_product_simd_f32` | SIMD dot product for f32 slices |
| `vector_cross_product_f32` | Cross product (3D) |
| `quaternion_mul_branchless` | Quaternion multiplication |
| `matrix_mul_simd_f32` | SIMD matrix multiplication |
| `matrix_transpose_simd_f32` | SIMD in-place matrix transpose |

---

## Graph Algorithms (Difficulty 150-250)

| Algorithm | Description |
|-----------|-------------|
| `graph_bfs_simd_step` | One BFS frontier expansion step (bitset-parallel) |
| `graph_dfs_bit_parallel` | DFS with bitset-parallel adjacency |
| `clique_check_branchless` | Branchless clique membership check |
| `triangle_count_bitset` | Triangle counting via bitset intersection |
| `topological_sort_step_branchless` | One Kahn's algorithm step |
| `minimum_spanning_tree_prim_step` | One Prim's MST step (priority queue free) |
| `shortest_path_bellman_ford_branchless` | One Bellman-Ford relaxation pass |
| `max_flow_edmonds_karp_step` | One Edmonds-Karp BFS step |
| `page_rank_simd_step` | One PageRank iteration (SIMD sparse multiply) |
| `disjoint_set_union_branchless` | Union-Find with path compression |

---

## Succinct Data Structures (Difficulty 150-280)

| Algorithm | Description |
|-----------|-------------|
| `succinct_bit_vector_rank` | Rank query: count set bits in [0, i) |
| `succinct_bit_vector_select` | Select query: position of k-th set bit |
| `rank_u32x8` | Rank over a u32 array chunk |
| `rank_u128` | Rank over 128 bits |
| `rank_select_sort_u32` | Combined rank/select for sorted u32 |
| `rank_select_dictionary_rrr` | RRR compressed bitvector |
| `wavelet_tree_access_branchless` | Wavelet tree random access |
| `branchless_vtable_lookup` | Branchless virtual-dispatch table lookup |
| `branchless_priority_queue_push` | Lock-free priority queue insertion |
| `branchless_priority_queue_pop` | Lock-free priority queue removal |
| `move_to_front_branchless` | Move-to-front transform |

---

## Concurrency Primitives (Difficulty 200-300)

| Algorithm | Description |
|-----------|-------------|
| `branchless_ring_buffer_mpmc` | Multi-producer multi-consumer ring buffer step |
| `branchless_stack_spsc` | Single-producer single-consumer stack |
| `waitfree_queue_push` | Wait-free queue push (CAS-based) |
| `epoch_based_reclamation_step` | Epoch-based safe memory reclamation |
| `hazard_pointer_retire` | Hazard pointer retirement |
| `lockfree_skip_list_step` | Lock-free skip list node traversal |

---

## Set Operations (Difficulty 30-80)

| Algorithm | Description |
|-----------|-------------|
| `set_union_branchless` | Union of two sorted arrays |
| `set_intersection_branchless` | Intersection of two sorted arrays |
| `set_difference_branchless` | Difference of two sorted arrays |
| `set_symmetric_difference_branchless` | Symmetric difference of two sorted arrays |
| `is_subset_mask_u64` | Bitset subset test |
| `unique_branchless_u32` | Remove adjacent duplicates (stable) |

---

## Slice and Array Utilities (Difficulty 20-100)

| Algorithm | Description |
|-----------|-------------|
| `bool_slice_from_mask` | Convert u64 bitmask to `[bool; 64]` |
| `mask_from_bool_slice` | Convert `[bool; 64]` to u64 bitmask |
| `normalize_slice_branchless` | Scale slice to `[0, 1]` range |
| `clamp_slice_branchless` | Clamp every element to `[lo, hi]` |
| `reverse_slice_branchless` | Reverse a slice in-place |
| `rotate_slice_branchless` | Rotate a slice by k positions |
| `stable_partition_branchless` | Stable partition without branching |
| `nth_element_branchless` | Order statistics: k-th element |
| `partial_sort_branchless_k` | Sort only the smallest k elements |
| `counting_sort_branchless_u8` | Counting sort for byte arrays |
| `radix_sort_step_branchless` | One radix sort pass |
| `merge_u32_slices_branchless` | Merge two sorted slices |
| `sort_index_u32x8` | Sort with explicit index tracking |
| `sort_stable_key_value_u32x8` | Stable sort of (key, value) pairs |
| `duffs_device_simd_unroll` | Loop unrolling via Duff's device pattern |
| `prefix_sum_simd_u32x8` | SIMD parallel prefix sum (8 u32) |
| `suffix_sum_simd_u32x8` | SIMD parallel suffix sum (8 u32) |
| `top_k_u32x16` | Select top-K elements from 16 |
| `softmax_u32x4` | Softmax over 4 values (fixed-point) |

---

## Hashing Utilities (Difficulty 30-80)

| Algorithm | Description |
|-----------|-------------|
| `consistent_hash_jump_u64` | Jump consistent hash (Google, 2014) |
| `consistent_hash_maglev` | Maglev consistent hashing step |
| `perfect_hash_build_static` | Build a minimal perfect hash at compile time |
| `perfect_hash_lookup_u32` | Lookup in a static minimal perfect hash |
| `leb128_encode_u64` | LEB128 encoding |
| `leb128_decode_u64` | LEB128 decoding |

---

*Last updated: June 2026. For each algorithm's source, see*
*`crates/bcinr-logic/src/algorithms/<name>.rs`.*
