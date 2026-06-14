import re

batch_7 = ["mismatch_branchless_u8.rs", "misra_gries_add.rs", "modular_add_u64.rs", "modular_mul_u64.rs", "modular_sub_u64.rs", "morton_decode_2d_u32.rs", "morton_encode_2d_u32.rs", "morton_encode_3d_u32.rs", "move_to_front_branchless.rs", "mul_sat_i32.rs", "mul_sat_u64.rs", "murmur3_x64_128.rs", "next_combination_u64.rs", "next_lexicographic_permutation_u64.rs", "norm_u32.rs", "normalize_slice_branchless.rs", "nth_element_branchless.rs", "octree_insert_branchless.rs", "odd_even_merge_sort_16u32.rs", "page_rank_simd_step.rs", "parallel_bits_extract_u64.rs", "parity_check_u128.rs", "partial_sort_branchless_k.rs", "pcg_random_u64.rs"]

batch_8 = ["pow_sat_u64.rs", "prefix_sum_simd_u32x8.rs", "punycode_encode_branchless.rs", "quadtree_insert_branchless.rs", "quantize_u32.rs", "quaternion_mul_branchless.rs", "quotient_filter_add_u64.rs", "radix_sort_step_branchless.rs", "random_permutation_fixed_seed.rs", "rank_select_dictionary_rrr.rs", "rank_select_sort_u32.rs", "rank_u128.rs", "ray_sphere_intersect_branchless.rs", "ray_triangle_intersect_branchless.rs", "regex_nfa_simd_step.rs", "relu_u32.rs", "reservoir_sample_branchless.rs", "reservoir_sample_weighted_simd.rs", "reverse_bits_u128.rs", "reverse_slice_branchless.rs", "rolling_hash_buzhash.rs", "rolling_hash_gear.rs", "rolling_hash_rabin_karp.rs", "rotate_left_u64.rs", "rotate_right_u64.rs", "rotate_slice_branchless.rs", "round_down_u32.rs"]

batch_10 = ["base64_encode_simd.rs", "base64_decode_simd.rs", "hex_encode_simd.rs", "hex_decode_simd.rs", "base32_encode_rfc4648.rs", "base85_encode_ascii85.rs", "leb128_encode_u64.rs", "leb128_decode_u64.rs", "varint_encode_simd.rs", "varint_decode_simd.rs", "bitpacking_encode_u32_k.rs", "bitpacking_decode_u32_k.rs", "zigzag_encode_i64.rs", "zigzag_decode_i64.rs", "utf8_to_utf16_simd.rs", "utf16_to_utf8_simd.rs", "utf8_to_utf32_simd.rs", "ascii_to_lowercase_simd.rs", "ascii_to_uppercase_simd.rs", "is_alphanumeric_simd_u8x16.rs", "is_digit_simd_u8x16.rs", "is_space_simd_u8x16.rs", "trim_whitespace_branchless.rs", "split_lines_simd.rs", "csv_scan_row_simd.rs", "json_find_string_escapes_simd.rs", "json_find_structural_simd.rs", "levenshtein_dist_branchless.rs", "hamming_dist_simd.rs", "jaro_winkler_branchless.rs", "soundex_encode_branchless.rs", "metaphone_encode_branchless.rs", "url_encode_branchless.rs", "url_decode_branchless.rs", "punycode_encode_branchless.rs", "simd_strstr_branchless.rs", "simd_memchr_u8x16.rs", "simd_memrchr_u8x16.rs", "wildcard_match_branchless.rs", "regex_nfa_simd_step.rs", "aho_corasick_simd_step.rs", "suffix_array_step_branchless.rs", "lcp_array_step_branchless.rs", "burrows_wheeler_transform_step.rs", "move_to_front_branchless.rs", "huffman_decode_table_step.rs", "prefix_sum_simd_u32x8.rs", "suffix_sum_simd_u32x8.rs", "delta_encode_simd_u32.rs", "delta_decode_simd_u32.rs", "branchless_stack_spsc.rs", "branchless_ring_buffer_mpmc.rs", "lockfree_skip_list_step.rs", "waitfree_queue_push.rs", "hazard_pointer_retire.rs", "epoch_based_reclamation_step.rs", "branchless_priority_queue_push.rs", "branchless_priority_queue_pop.rs", "disjoint_set_union_branchless.rs", "graph_bfs_simd_step.rs", "graph_dfs_bit_parallel.rs", "shortest_path_bellman_ford_branchless.rs", "page_rank_simd_step.rs", "triangle_count_bitset.rs", "clique_check_branchless.rs", "topological_sort_step_branchless.rs", "minimum_spanning_tree_prim_step.rs", "max_flow_edmonds_karp_step.rs", "bloom_filter_graph_visited.rs", "matrix_mul_simd_f32.rs", "matrix_transpose_simd_f32.rs", "vector_dot_product_simd_f32.rs", "vector_cross_product_f32.rs", "quaternion_mul_branchless.rs", "aabb_intersect_branchless.rs", "ray_triangle_intersect_branchless.rs", "ray_sphere_intersect_branchless.rs", "frustum_culling_branchless.rs", "point_in_polygon_branchless.rs", "convex_hull_monotone_chain_step.rs", "spatial_hash_u32.rs", "quadtree_insert_branchless.rs", "octree_insert_branchless.rs", "hilbert_curve_encode_u32.rs", "hilbert_curve_decode_u32.rs", "z_order_curve_2d_u32.rs", "bit_vector_compress_elias_fano.rs", "rank_select_dictionary_rrr.rs", "wavelet_tree_access_branchless.rs", "succinct_bit_vector_rank.rs", "succinct_bit_vector_select.rs", "linear_congruential_generator_u64.rs", "pcg_random_u64.rs", "splitmix64_u64.rs", "xoroshiro128_plus.rs", "mersenne_twister_step_simd.rs", "reservoir_sample_weighted_simd.rs", "gaussian_noise_box_muller.rs", "poisson_noise_branchless.rs", "halton_sampler_simd.rs"]

all_files = list(set(batch_7 + batch_8 + batch_10))

for f in all_files:
    path = f"crates/bcinr-logic/src/algorithms/{f}"
    try:
        with open(path, "r") as file:
            content = file.read()
            # check if it contains a simple val ^ aux or just simple wrapping add
            # by looking at the body of the function.
            match = re.search(r'pub fn .*?\(.*?\) -> .*? \{(.*?)\}', content, re.DOTALL)
            if match:
                body = match.group(1).strip()
                if body == "val ^ aux" or body == "val.wrapping_add(aux)" or "val ^ aux" in body:
                    print(f"FAKE: {f} - body: {body}")
                elif "if " in body or "match " in body or "while " in body:
                    print(f"BRANCHES: {f}")
    except Exception as e:
        pass

