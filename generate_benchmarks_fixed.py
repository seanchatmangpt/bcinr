import os

SIGNATURES = {'parallel_bits_deposit_u64': ['u64', 'u64'], 'parallel_bits_extract_u64': ['u64', 'u64'], 'blsr_u64': ['u64'], 'blsi_u64': ['u64'], 'blsmsk_u64': ['u64'], 't1mskc_u64': ['u64', 'u64'], 'tzmsk_u64': ['u64', 'u64'], 'bext_u64': ['u64', 'u64'], 'bset_u64': ['u64', 'u64'], 'bclr_u64': ['u64', 'u64'], 'btst_u64': ['u64', 'u64'], 'popcount_u128': ['u64', 'u64'], 'reverse_bits_u128': ['u64', 'u64'], 'clmul_u64': ['u64', 'u64'], 'morton_encode_2d_u32': ['u64', 'u64'], 'morton_decode_2d_u32': ['u64', 'u64'], 'morton_encode_3d_u32': ['u64', 'u64'], 'gray_encode_u64': ['u64', 'u64'], 'gray_decode_u64': ['u64', 'u64'], 'parity_check_u128': ['u64', 'u64'], 'next_lexicographic_permutation_u64': ['u64', 'u64'], 'count_consecutive_set_bits_u64': ['u64', 'u64'], 'find_nth_set_bit_u128': ['u64', 'u64'], 'mask_range_u64': ['u64', 'u64'], 'rotate_left_u64': ['u64', 'u64'], 'rotate_right_u64': ['u64', 'u64'], 'funnel_shift_left_u64': ['u64', 'u64'], 'funnel_shift_right_u64': ['u64', 'u64'], 'bit_swap_u64': ['u64', 'u64'], 'gather_bits_u64': ['u64', 'u64'], 'scatter_bits_u64': ['u64', 'u64'], 'is_contiguous_mask_u64': ['u64', 'u64'], 'get_mask_boundary_low_u64': ['u64', 'u64'], 'get_mask_boundary_high_u64': ['u64', 'u64'], 'bit_matrix_transpose_8x8': ['u64', 'u64'], 'bit_matrix_transpose_64x64': ['u64', 'u64'], 'rank_u128': ['u64', 'u64'], 'select_u128': ['u64', 'u64'], 'weight_u64': ['u64', 'u64'], 'delta_swap_u64': ['u64', 'u64'], 'benes_network_u64': ['u64', 'u64'], 'bit_permute_step_u64': ['u64', 'u64'], 'compress_bits_u64': ['u64', 'u64'], 'expand_bits_u64': ['u64', 'u64'], 'crossbar_permute_u8x16': ['u64', 'u64'], 'mask_from_bool_slice': ['u64', 'u64'], 'bool_slice_from_mask': ['u64', 'u64'], 'bit_permute_identity_64': ['u64', 'u64'], 'is_subset_mask_u64': ['u64', 'u64'], 'mask_xor_reduce_u64': ['u64', 'u64'], 'mul_sat_u64': ['u64', 'u64'], 'div_sat_u64': ['u64', 'u64'], 'add_sat_i32': ['u64', 'u64'], 'sub_sat_i32': ['u64', 'u64'], 'mul_sat_i32': ['u64', 'u64'], 'abs_diff_u64': ['u64', 'u64'], 'abs_diff_i64': ['u64', 'u64'], 'avg_u64': ['u64', 'u64'], 'avg_ceil_u64': ['u64', 'u64'], 'clamp_i64': ['i64', 'i64', 'i64'], 'lerp_sat_u8': ['u64', 'u64'], 'lerp_sat_u32': ['u64', 'u64'], 'norm_u32': ['u64', 'u64'], 'fp_mul_u32_q16': ['u64', 'u64'], 'fp_div_u32_q16': ['u64', 'u64'], 'fp_sqrt_u32_q16': ['u64', 'u64'], 'fp_sin_u32_q16': ['u64', 'u64'], 'fp_cos_u32_q16': ['u64', 'u64'], 'fp_atan2_u32_q16': ['u64', 'u64'], 'log2_u64_fixed': ['u64', 'u64'], 'exp2_u64_fixed': ['u64', 'u64'], 'sigmoid_sat_u32': ['u64', 'u64'], 'relu_u32': ['u64', 'u64'], 'leaky_relu_u32': ['u64', 'u64'], 'softmax_u32x4': ['u64', 'u64'], 'fast_inverse_sqrt_u32': ['u64', 'u64'], 'gcd_u64_branchless': ['u64', 'u64'], 'lcm_u64_branchless': ['u64', 'u64'], 'modular_add_u64': ['u64', 'u64'], 'modular_sub_u64': ['u64', 'u64'], 'modular_mul_u64': ['u64', 'u64'], 'is_prime_u64_branchless': ['u64', 'u64'], 'factorial_sat_u32': ['u64', 'u64'], 'binom_sat_u32': ['u64', 'u64'], 'pow_sat_u64': ['u64', 'u64'], 'clamped_scaling_u64': ['u64', 'u64'], 'branchless_signum_i64': ['i64'], 'copy_sign_i64': ['u64', 'u64'], 'is_finite_fp32_branchless': ['u64', 'u64'], 'is_nan_fp32_branchless': ['u64', 'u64'], 'round_to_nearest_u32': ['u64', 'u64'], 'round_up_u32': ['u64', 'u64'], 'round_down_u32': ['u64', 'u64'], 'quantize_u32': ['u64', 'u64'], 'dequantize_u32': ['u64', 'u64'], 'weighted_avg_u32': ['u64', 'u64'], 'smoothstep_u32': ['u64', 'u64'], 'cubic_interpolate_u32': ['u64', 'u64'], 'manhattan_dist_u32x2': ['u64', 'u64'], 'euclidean_dist_sq_u32x2': ['u64', 'u64'], 'bitonic_sort_64u32': ['u64', 'u64'], 'odd_even_merge_sort_16u32': ['u64', 'u64'], 'halton_sequence_u32': ['u64', 'u64'], 'shuffle_fisher_yates_branchless': ['u64', 'u64'], 'bitonic_merge_u64x8': ['u64', 'u64'], 'sort_pairs_u32x4': ['u64', 'u64'], 'median3_u32': ['u64', 'u64'], 'median5_u32': ['u64', 'u64'], 'median9_u32': ['u64', 'u64'], 'top_k_u32x16': ['u64', 'u64'], 'rank_select_sort_u32': ['u64', 'u64'], 'counting_sort_branchless_u8': ['u64', 'u64'], 'radix_sort_step_branchless': ['u64', 'u64'], 'insertion_sort_branchless_fixed': ['u64', 'u64'], 'shear_sort_bitonic_2d': ['u64', 'u64'], 'green_sorting_network_16': ['u64', 'u64'], 'permute_u32x8': ['u64', 'u64'], 'inverse_permute_u32x8': ['u64', 'u64'], 'is_sorted_branchless_u32': ['u64', 'u64'], 'lex_compare_u8_slices_branchless': ['u64', 'u64'], 'stable_partition_branchless': ['u64', 'u64'], 'rotate_slice_branchless': ['u64', 'u64'], 'reverse_slice_branchless': ['u64', 'u64'], 'next_combination_u64': ['u64', 'u64'], 'random_permutation_fixed_seed': ['u64', 'u64'], 'sort_index_u32x8': ['u64', 'u64'], 'merge_u32_slices_branchless': ['u64', 'u64'], 'unique_branchless_u32': ['u64', 'u64'], 'lower_bound_branchless_u32': ['u64', 'u64'], 'upper_bound_branchless_u32': ['u64', 'u64'], 'equal_range_branchless_u32': ['u64', 'u64'], 'search_eytzinger_u32': ['u64', 'u64'], 'search_van_emde_boas': ['u64', 'u64'], 'binary_search_v_u32x4': ['u64', 'u64'], 'linear_search_simd_u8': ['u64', 'u64'], 'find_first_of_branchless': ['u64', 'u64'], 'find_last_of_branchless': ['u64', 'u64'], 'mismatch_branchless_u8': ['u64', 'u64'], 'partial_sort_branchless_k': ['u64', 'u64'], 'nth_element_branchless': ['u64', 'u64'], 'is_permutation_branchless': ['u64', 'u64'], 'set_difference_branchless': ['u64', 'u64'], 'set_symmetric_difference_branchless': ['u64', 'u64'], 'set_intersection_branchless': ['u64', 'u64'], 'set_union_branchless': ['u64', 'u64'], 'min_element_branchless_u32': ['u64', 'u64'], 'max_element_branchless_u32': ['u64', 'u64'], 'minmax_element_branchless_u32': ['u64', 'u64'], 'clamp_slice_branchless': ['u64', 'u64'], 'normalize_slice_branchless': ['u64', 'u64'], 'murmur3_x64_128': ['u64', 'u64'], 'xxhash64': ['u64', 'u64'], 'xxh3_64': ['u64', 'u64'], 'cityhash64': ['u64', 'u64'], 'farmhash64': ['u64', 'u64'], 'spookyhash_v2_128': ['u64', 'u64'], 'metrohash64': ['u64', 'u64'], 'siphash_2_4_branchless': ['u64', 'u64'], 'highwayhash_64': ['u64', 'u64'], 'clhash': ['u64', 'u64'], 'pearson_hash_u8': ['u64', 'u64'], 'knuth_hash_u64': ['u64', 'u64'], 'fibonacci_hash_u64': ['u64', 'u64'], 'zobrist_hash_64': ['u64', 'u64'], 'perfect_hash_lookup_u32': ['u64', 'u64'], 'minhash_u64_k': ['u64', 'u64'], 'hyperloglog_add_u64': ['u64', 'u64'], 'hyperloglog_merge': ['u64', 'u64'], 'count_min_sketch_add': ['u64', 'u64'], 'count_min_sketch_query': ['u64', 'u64'], 'bloom_filter_add_u64': ['u64', 'u64'], 'bloom_filter_query_u64': ['u64', 'u64'], 'cuckoo_filter_add_u64': ['u64', 'u64'], 'quotient_filter_add_u64': ['u64', 'u64'], 't_digest_add_u32': ['u64', 'u64'], 'heavy_keepers_add': ['u64', 'u64'], 'space_saving_add': ['u64', 'u64'], 'misra_gries_add': ['u64', 'u64'], 'reservoir_sample_branchless': ['u64', 'u64'], 'weighted_reservoir_sample': ['u64', 'u64'], 'consistent_hash_jump_u64': ['u64', 'u64'], 'consistent_hash_maglev': ['u64', 'u64'], 'bloom_filter_intersect': ['u64', 'u64'], 'bloom_filter_union': ['u64', 'u64'], 'hashing_trick_u64': ['u64', 'u64'], 'locality_sensitive_hash_euclidean': ['u64', 'u64'], 'locality_sensitive_hash_cosine': ['u64', 'u64'], 'k_independent_hash_gen': ['u64', 'u64'], 'rolling_hash_rabin_karp': ['u64', 'u64'], 'rolling_hash_buzhash': ['u64', 'u64'], 'rolling_hash_gear': ['u64', 'u64'], 'content_defined_chunking_branchless': ['u64', 'u64'], 'cyclic_redundancy_check_crc32c': ['u64', 'u64'], 'cyclic_redundancy_check_crc64': ['u64', 'u64'], 'adler32_branchless': ['u64', 'u64'], 'fletcher32_branchless': ['u64', 'u64'], 'bsd_checksum_u16': ['u64', 'u64'], 'internet_checksum_u16': ['u64', 'u64'], 'duffs_device_simd_unroll': ['u64', 'u64'], 'perfect_hash_build_static': ['u64', 'u64'], 'base64_encode_simd': ['u64', 'u64'], 'base64_decode_simd': ['u64', 'u64'], 'hex_encode_simd': ['u64', 'u64'], 'hex_decode_simd': ['u64', 'u64'], 'base32_encode_rfc4648': ['u64', 'u64'], 'base85_encode_ascii85': ['u64', 'u64'], 'leb128_encode_u64': ['u64', 'u64'], 'leb128_decode_u64': ['u64', 'u64'], 'varint_encode_simd': ['u64', 'u64'], 'varint_decode_simd': ['u64', 'u64'], 'bitpacking_encode_u32_k': ['u64', 'u64'], 'bitpacking_decode_u32_k': ['u64', 'u64'], 'zigzag_encode_i64': ['u64', 'u64'], 'zigzag_decode_i64': ['u64', 'u64'], 'utf8_to_utf16_simd': ['u64', 'u64'], 'utf16_to_utf8_simd': ['u64', 'u64'], 'utf8_to_utf32_simd': ['u64', 'u64'], 'ascii_to_lowercase_simd': ['u64', 'u64'], 'ascii_to_uppercase_simd': ['u64', 'u64'], 'is_alphanumeric_simd_u8x16': ['u64', 'u64'], 'is_digit_simd_u8x16': ['u64', 'u64'], 'is_space_simd_u8x16': ['u64', 'u64'], 'trim_whitespace_branchless': ['u64', 'u64'], 'split_lines_simd': ['u64', 'u64'], 'csv_scan_row_simd': ['u64', 'u64'], 'json_find_string_escapes_simd': ['u64', 'u64'], 'json_find_structural_simd': ['u64', 'u64'], 'levenshtein_dist_branchless': ['u64', 'u64'], 'hamming_dist_simd': ['u64', 'u64'], 'jaro_winkler_branchless': ['u64', 'u64'], 'soundex_encode_branchless': ['u64', 'u64'], 'metaphone_encode_branchless': ['u64', 'u64'], 'url_encode_branchless': ['u64', 'u64'], 'url_decode_branchless': ['u64', 'u64'], 'punycode_encode_branchless': ['u64', 'u64'], 'simd_strstr_branchless': ['u64', 'u64'], 'simd_memchr_u8x16': ['u64', 'u64'], 'simd_memrchr_u8x16': ['u64', 'u64'], 'wildcard_match_branchless': ['u64', 'u64'], 'regex_nfa_simd_step': ['u64', 'u64'], 'aho_corasick_simd_step': ['u64', 'u64'], 'suffix_array_step_branchless': ['u64', 'u64'], 'lcp_array_step_branchless': ['u64', 'u64'], 'burrows_wheeler_transform_step': ['u64', 'u64'], 'move_to_front_branchless': ['u64', 'u64'], 'huffman_decode_table_step': ['u64', 'u64'], 'prefix_sum_simd_u32x8': ['u64', 'u64'], 'suffix_sum_simd_u32x8': ['u64', 'u64'], 'delta_encode_simd_u32': ['u64', 'u64'], 'delta_decode_simd_u32': ['u64', 'u64'], 'branchless_stack_spsc': ['u64', 'u64'], 'branchless_ring_buffer_mpmc': ['u64', 'u64'], 'lockfree_skip_list_step': ['u64', 'u64'], 'waitfree_queue_push': ['u64', 'u64'], 'hazard_pointer_retire': ['u64', 'u64'], 'epoch_based_reclamation_step': ['u64', 'u64'], 'branchless_priority_queue_push': ['u64', 'u64'], 'branchless_priority_queue_pop': ['u64', 'u64'], 'disjoint_set_union_branchless': ['u64', 'u64'], 'graph_bfs_simd_step': ['u64', 'u64'], 'graph_dfs_bit_parallel': ['u64', 'u64'], 'shortest_path_bellman_ford_branchless': ['u64', 'u64'], 'page_rank_simd_step': ['u64', 'u64'], 'triangle_count_bitset': ['u64', 'u64'], 'clique_check_branchless': ['u64', 'u64'], 'topological_sort_step_branchless': ['u64', 'u64'], 'minimum_spanning_tree_prim_step': ['u64', 'u64'], 'max_flow_edmonds_karp_step': ['u64', 'u64'], 'bloom_filter_graph_visited': ['u64', 'u64'], 'matrix_mul_simd_f32': ['u64', 'u64'], 'matrix_transpose_simd_f32': ['u64', 'u64'], 'vector_dot_product_simd_f32': ['u64', 'u64'], 'vector_cross_product_f32': ['u64', 'u64'], 'quaternion_mul_branchless': ['u64', 'u64'], 'aabb_intersect_branchless': ['u64', 'u64'], 'ray_triangle_intersect_branchless': ['u64', 'u64'], 'ray_sphere_intersect_branchless': ['u64', 'u64'], 'frustum_culling_branchless': ['u64', 'u64'], 'point_in_polygon_branchless': ['u64', 'u64'], 'convex_hull_monotone_chain_step': ['u64', 'u64'], 'spatial_hash_u32': ['u64', 'u64'], 'quadtree_insert_branchless': ['u64', 'u64'], 'octree_insert_branchless': ['u64', 'u64'], 'hilbert_curve_encode_u32': ['u64', 'u64'], 'hilbert_curve_decode_u32': ['u64', 'u64'], 'z_order_curve_2d_u32': ['u64', 'u64'], 'bit_vector_compress_elias_fano': ['u64', 'u64'], 'rank_select_dictionary_rrr': ['u64', 'u64'], 'wavelet_tree_access_branchless': ['u64', 'u64'], 'succinct_bit_vector_rank': ['u64', 'u64'], 'succinct_bit_vector_select': ['u64', 'u64'], 'linear_congruential_generator_u64': ['u64', 'u64'], 'pcg_random_u64': ['u64', 'u64'], 'splitmix64_u64': ['u64', 'u64'], 'xoroshiro128_plus': ['u64', 'u64'], 'mersenne_twister_step_simd': ['u64', 'u64'], 'reservoir_sample_weighted_simd': ['u64', 'u64'], 'gaussian_noise_box_muller': ['u64', 'u64'], 'poisson_noise_branchless': ['u64', 'u64'], 'halton_sampler_simd': ['u64', 'u64'], 'fixed_point_log2': ['u64', 'u64'], 'branchless_vtable_lookup': ['u64', 'u64'], 'base64_decode_chunk4': ['u64', 'u64'], 'unrolled_binary_search_u32': ['u64', 'u64'], 'utf8_validate_chunk8': ['u64', 'u64'], 'hex_encode_chunk8': ['u64', 'u64'], 'bit_parallel_sort8_u32': ['u64', 'u64']}

ALGORITHMS = sorted(SIGNATURES.keys())


def write_bench(filename, subset):
    with open(filename, "w") as f:
        f.write("use bcinr_logic::algorithms::*;\n")
        f.write("use criterion::{criterion_group, criterion_main, Criterion, black_box};\n\n")
        
        bench_name = filename.split("/")[-1].replace(".rs", "")
        f.write(f"fn {bench_name}(c: &mut Criterion) {{\n")
        for algo in subset:
            types = SIGNATURES[algo]
            def get_val(t, val):
                if t == 'i64': return f"{val}i64"
                if t == 'i32': return f"{val}i32"
                if t == 'u32': return f"{val}u32"
                if t == 'u128': return f"{val}u128"
                if t == 'f64': return f"{val}.0f64"
                if t == 'f32': return f"{val}.0f32"
                return str(val)

            def get_max(t):
                if t == 'i64': return "i64::MAX"
                if t == 'i32': return "i32::MAX"
                if t == 'u32': return "u32::MAX"
                if t == 'u64': return "u64::MAX"
                if t == 'u128': return "u128::MAX"
                return "100" # fallback

            args_avg = ", ".join([f"black_box({get_val(t, 42)})" for t in types])
            args_min = ", ".join([f"black_box({get_val(t, 0)})" for t in types])
            args_max = ", ".join([f"black_box({get_max(t)})" for t in types])

            f.write(f"    use bcinr_logic::algorithms::{algo}::{algo};\n")
            f.write(f'    c.bench_function("{algo}_avg", |b| b.iter(|| {algo}({args_avg})));\n')
            f.write(f'    c.bench_function("{algo}_min", |b| b.iter(|| {algo}({args_min})));\n')
            f.write(f'    c.bench_function("{algo}_max", |b| b.iter(|| {algo}({args_max})));\n')
        f.write("}\n\n")
        
        f.write(f"criterion_group!(benches, {bench_name});\n")
        f.write("criterion_main!(benches);\n")

# Split into chunks of 100
for i in range(0, len(ALGORITHMS), 100):
    subset = ALGORITHMS[i:i+100]
    filename = f"bcinr-bench/benches/algorithms_{i+1}_{min(i+100, len(ALGORITHMS))}.rs"
    write_bench(filename, subset)

# Also update all_300_bench.rs
with open("bcinr-bench/benches/all_300_bench.rs", "w") as f:
    f.write("use bcinr_logic::algorithms::*;\n")
    f.write("use criterion::{criterion_group, criterion_main, Criterion, black_box};\n\n")
    
    for algo in ALGORITHMS:
        f.write(f"fn bench_{algo}(c: &mut Criterion) {{\n")
        f.write(f"    use bcinr_logic::algorithms::{algo}::{algo};\n")
        types = SIGNATURES[algo]
        def get_val(t, val):
            if t == 'i64': return f"{val}i64"
            if t == 'i32': return f"{val}i32"
            if t == 'u32': return f"{val}u32"
            if t == 'u128': return f"{val}u128"
            if t == 'f64': return f"{val}.0f64"
            if t == 'f32': return f"{val}.0f32"
            return str(val)

        def get_max(t):
            if t == 'i64': return "i64::MAX"
            if t == 'i32': return "i32::MAX"
            if t == 'u32': return "u32::MAX"
            if t == 'u64': return "u64::MAX"
            if t == 'u128': return "u128::MAX"
            return "100" # fallback

        args_avg = ", ".join([f"black_box({get_val(t, 42)})" for t in types])
        args_min = ", ".join([f"black_box({get_val(t, 0)})" for t in types])
        args_max = ", ".join([f"black_box({get_max(t)})" for t in types])

        f.write(f'    c.bench_function("{algo}_avg", |b| b.iter(|| {algo}({args_avg})));\n')
        f.write(f'    c.bench_function("{algo}_min", |b| b.iter(|| {algo}({args_min})));\n')
        f.write(f'    c.bench_function("{algo}_max", |b| b.iter(|| {algo}({args_max})));\n')
        f.write("}\n\n")
    
    f.write("criterion_group!(benches,\n")
    for algo in ALGORITHMS:
        f.write(f"    bench_{algo},\n")
    f.write(");\n")
    f.write("criterion_main!(benches);\n")
