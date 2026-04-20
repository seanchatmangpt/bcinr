use bcinr_logic::algorithms::*;
use criterion::{criterion_group, criterion_main, Criterion, black_box};

fn algorithms_201_300(c: &mut Criterion) {
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::base64_encode_simd::base64_encode_simd;
    c.bench_function("base64_encode_simd", |b| b.iter(|| base64_encode_simd(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::base64_decode_simd::base64_decode_simd;
    c.bench_function("base64_decode_simd", |b| b.iter(|| base64_decode_simd(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::hex_encode_simd::hex_encode_simd;
    c.bench_function("hex_encode_simd", |b| b.iter(|| hex_encode_simd(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::hex_decode_simd::hex_decode_simd;
    c.bench_function("hex_decode_simd", |b| b.iter(|| hex_decode_simd(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::base32_encode_rfc4648::base32_encode_rfc4648;
    c.bench_function("base32_encode_rfc4648", |b| b.iter(|| base32_encode_rfc4648(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::base85_encode_ascii85::base85_encode_ascii85;
    c.bench_function("base85_encode_ascii85", |b| b.iter(|| base85_encode_ascii85(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::leb128_encode_u64::leb128_encode_u64;
    c.bench_function("leb128_encode_u64", |b| b.iter(|| leb128_encode_u64(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::leb128_decode_u64::leb128_decode_u64;
    c.bench_function("leb128_decode_u64", |b| b.iter(|| leb128_decode_u64(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::varint_encode_simd::varint_encode_simd;
    c.bench_function("varint_encode_simd", |b| b.iter(|| varint_encode_simd(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::varint_decode_simd::varint_decode_simd;
    c.bench_function("varint_decode_simd", |b| b.iter(|| varint_decode_simd(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::bitpacking_encode_u32_k::bitpacking_encode_u32_k;
    c.bench_function("bitpacking_encode_u32_k", |b| b.iter(|| bitpacking_encode_u32_k(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::bitpacking_decode_u32_k::bitpacking_decode_u32_k;
    c.bench_function("bitpacking_decode_u32_k", |b| b.iter(|| bitpacking_decode_u32_k(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::zigzag_encode_i64::zigzag_encode_i64;
    c.bench_function("zigzag_encode_i64", |b| b.iter(|| zigzag_encode_i64(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::zigzag_decode_i64::zigzag_decode_i64;
    c.bench_function("zigzag_decode_i64", |b| b.iter(|| zigzag_decode_i64(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::utf8_to_utf16_simd::utf8_to_utf16_simd;
    c.bench_function("utf8_to_utf16_simd", |b| b.iter(|| utf8_to_utf16_simd(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::utf16_to_utf8_simd::utf16_to_utf8_simd;
    c.bench_function("utf16_to_utf8_simd", |b| b.iter(|| utf16_to_utf8_simd(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::utf8_to_utf32_simd::utf8_to_utf32_simd;
    c.bench_function("utf8_to_utf32_simd", |b| b.iter(|| utf8_to_utf32_simd(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::ascii_to_lowercase_simd::ascii_to_lowercase_simd;
    c.bench_function("ascii_to_lowercase_simd", |b| b.iter(|| ascii_to_lowercase_simd(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::ascii_to_uppercase_simd::ascii_to_uppercase_simd;
    c.bench_function("ascii_to_uppercase_simd", |b| b.iter(|| ascii_to_uppercase_simd(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::is_alphanumeric_simd_u8x16::is_alphanumeric_simd_u8x16;
    c.bench_function("is_alphanumeric_simd_u8x16", |b| b.iter(|| is_alphanumeric_simd_u8x16(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::is_digit_simd_u8x16::is_digit_simd_u8x16;
    c.bench_function("is_digit_simd_u8x16", |b| b.iter(|| is_digit_simd_u8x16(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::is_space_simd_u8x16::is_space_simd_u8x16;
    c.bench_function("is_space_simd_u8x16", |b| b.iter(|| is_space_simd_u8x16(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::trim_whitespace_branchless::trim_whitespace_branchless;
    c.bench_function("trim_whitespace_branchless", |b| b.iter(|| trim_whitespace_branchless(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::split_lines_simd::split_lines_simd;
    c.bench_function("split_lines_simd", |b| b.iter(|| split_lines_simd(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::csv_scan_row_simd::csv_scan_row_simd;
    c.bench_function("csv_scan_row_simd", |b| b.iter(|| csv_scan_row_simd(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::json_find_string_escapes_simd::json_find_string_escapes_simd;
    c.bench_function("json_find_string_escapes_simd", |b| b.iter(|| json_find_string_escapes_simd(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::json_find_structural_simd::json_find_structural_simd;
    c.bench_function("json_find_structural_simd", |b| b.iter(|| json_find_structural_simd(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::levenshtein_dist_branchless::levenshtein_dist_branchless;
    c.bench_function("levenshtein_dist_branchless", |b| b.iter(|| levenshtein_dist_branchless(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::hamming_dist_simd::hamming_dist_simd;
    c.bench_function("hamming_dist_simd", |b| b.iter(|| hamming_dist_simd(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::jaro_winkler_branchless::jaro_winkler_branchless;
    c.bench_function("jaro_winkler_branchless", |b| b.iter(|| jaro_winkler_branchless(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::soundex_encode_branchless::soundex_encode_branchless;
    c.bench_function("soundex_encode_branchless", |b| b.iter(|| soundex_encode_branchless(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::metaphone_encode_branchless::metaphone_encode_branchless;
    c.bench_function("metaphone_encode_branchless", |b| b.iter(|| metaphone_encode_branchless(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::url_encode_branchless::url_encode_branchless;
    c.bench_function("url_encode_branchless", |b| b.iter(|| url_encode_branchless(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::url_decode_branchless::url_decode_branchless;
    c.bench_function("url_decode_branchless", |b| b.iter(|| url_decode_branchless(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::punycode_encode_branchless::punycode_encode_branchless;
    c.bench_function("punycode_encode_branchless", |b| b.iter(|| punycode_encode_branchless(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::simd_strstr_branchless::simd_strstr_branchless;
    c.bench_function("simd_strstr_branchless", |b| b.iter(|| simd_strstr_branchless(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::simd_memchr_u8x16::simd_memchr_u8x16;
    c.bench_function("simd_memchr_u8x16", |b| b.iter(|| simd_memchr_u8x16(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::simd_memrchr_u8x16::simd_memrchr_u8x16;
    c.bench_function("simd_memrchr_u8x16", |b| b.iter(|| simd_memrchr_u8x16(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::wildcard_match_branchless::wildcard_match_branchless;
    c.bench_function("wildcard_match_branchless", |b| b.iter(|| wildcard_match_branchless(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::regex_nfa_simd_step::regex_nfa_simd_step;
    c.bench_function("regex_nfa_simd_step", |b| b.iter(|| regex_nfa_simd_step(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::aho_corasick_simd_step::aho_corasick_simd_step;
    c.bench_function("aho_corasick_simd_step", |b| b.iter(|| aho_corasick_simd_step(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::suffix_array_step_branchless::suffix_array_step_branchless;
    c.bench_function("suffix_array_step_branchless", |b| b.iter(|| suffix_array_step_branchless(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::lcp_array_step_branchless::lcp_array_step_branchless;
    c.bench_function("lcp_array_step_branchless", |b| b.iter(|| lcp_array_step_branchless(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::burrows_wheeler_transform_step::burrows_wheeler_transform_step;
    c.bench_function("burrows_wheeler_transform_step", |b| b.iter(|| burrows_wheeler_transform_step(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::move_to_front_branchless::move_to_front_branchless;
    c.bench_function("move_to_front_branchless", |b| b.iter(|| move_to_front_branchless(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::huffman_decode_table_step::huffman_decode_table_step;
    c.bench_function("huffman_decode_table_step", |b| b.iter(|| huffman_decode_table_step(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::prefix_sum_simd_u32x8::prefix_sum_simd_u32x8;
    c.bench_function("prefix_sum_simd_u32x8", |b| b.iter(|| prefix_sum_simd_u32x8(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::suffix_sum_simd_u32x8::suffix_sum_simd_u32x8;
    c.bench_function("suffix_sum_simd_u32x8", |b| b.iter(|| suffix_sum_simd_u32x8(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::delta_encode_simd_u32::delta_encode_simd_u32;
    c.bench_function("delta_encode_simd_u32", |b| b.iter(|| delta_encode_simd_u32(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::delta_decode_simd_u32::delta_decode_simd_u32;
    c.bench_function("delta_decode_simd_u32", |b| b.iter(|| delta_decode_simd_u32(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::branchless_stack_spsc::branchless_stack_spsc;
    c.bench_function("branchless_stack_spsc", |b| b.iter(|| branchless_stack_spsc(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::branchless_ring_buffer_mpmc::branchless_ring_buffer_mpmc;
    c.bench_function("branchless_ring_buffer_mpmc", |b| b.iter(|| branchless_ring_buffer_mpmc(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::lockfree_skip_list_step::lockfree_skip_list_step;
    c.bench_function("lockfree_skip_list_step", |b| b.iter(|| lockfree_skip_list_step(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::waitfree_queue_push::waitfree_queue_push;
    c.bench_function("waitfree_queue_push", |b| b.iter(|| waitfree_queue_push(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::hazard_pointer_retire::hazard_pointer_retire;
    c.bench_function("hazard_pointer_retire", |b| b.iter(|| hazard_pointer_retire(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::epoch_based_reclamation_step::epoch_based_reclamation_step;
    c.bench_function("epoch_based_reclamation_step", |b| b.iter(|| epoch_based_reclamation_step(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::branchless_priority_queue_push::branchless_priority_queue_push;
    c.bench_function("branchless_priority_queue_push", |b| b.iter(|| branchless_priority_queue_push(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::branchless_priority_queue_pop::branchless_priority_queue_pop;
    c.bench_function("branchless_priority_queue_pop", |b| b.iter(|| branchless_priority_queue_pop(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::disjoint_set_union_branchless::disjoint_set_union_branchless;
    c.bench_function("disjoint_set_union_branchless", |b| b.iter(|| disjoint_set_union_branchless(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::graph_bfs_simd_step::graph_bfs_simd_step;
    c.bench_function("graph_bfs_simd_step", |b| b.iter(|| graph_bfs_simd_step(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::graph_dfs_bit_parallel::graph_dfs_bit_parallel;
    c.bench_function("graph_dfs_bit_parallel", |b| b.iter(|| graph_dfs_bit_parallel(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::shortest_path_bellman_ford_branchless::shortest_path_bellman_ford_branchless;
    c.bench_function("shortest_path_bellman_ford_branchless", |b| b.iter(|| shortest_path_bellman_ford_branchless(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::page_rank_simd_step::page_rank_simd_step;
    c.bench_function("page_rank_simd_step", |b| b.iter(|| page_rank_simd_step(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::triangle_count_bitset::triangle_count_bitset;
    c.bench_function("triangle_count_bitset", |b| b.iter(|| triangle_count_bitset(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::clique_check_branchless::clique_check_branchless;
    c.bench_function("clique_check_branchless", |b| b.iter(|| clique_check_branchless(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::topological_sort_step_branchless::topological_sort_step_branchless;
    c.bench_function("topological_sort_step_branchless", |b| b.iter(|| topological_sort_step_branchless(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::minimum_spanning_tree_prim_step::minimum_spanning_tree_prim_step;
    c.bench_function("minimum_spanning_tree_prim_step", |b| b.iter(|| minimum_spanning_tree_prim_step(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::max_flow_edmonds_karp_step::max_flow_edmonds_karp_step;
    c.bench_function("max_flow_edmonds_karp_step", |b| b.iter(|| max_flow_edmonds_karp_step(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::bloom_filter_graph_visited::bloom_filter_graph_visited;
    c.bench_function("bloom_filter_graph_visited", |b| b.iter(|| bloom_filter_graph_visited(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::matrix_mul_simd_f32::matrix_mul_simd_f32;
    c.bench_function("matrix_mul_simd_f32", |b| b.iter(|| matrix_mul_simd_f32(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::matrix_transpose_simd_f32::matrix_transpose_simd_f32;
    c.bench_function("matrix_transpose_simd_f32", |b| b.iter(|| matrix_transpose_simd_f32(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::vector_dot_product_simd_f32::vector_dot_product_simd_f32;
    c.bench_function("vector_dot_product_simd_f32", |b| b.iter(|| vector_dot_product_simd_f32(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::vector_cross_product_f32::vector_cross_product_f32;
    c.bench_function("vector_cross_product_f32", |b| b.iter(|| vector_cross_product_f32(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::quaternion_mul_branchless::quaternion_mul_branchless;
    c.bench_function("quaternion_mul_branchless", |b| b.iter(|| quaternion_mul_branchless(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::aabb_intersect_branchless::aabb_intersect_branchless;
    c.bench_function("aabb_intersect_branchless", |b| b.iter(|| aabb_intersect_branchless(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::ray_triangle_intersect_branchless::ray_triangle_intersect_branchless;
    c.bench_function("ray_triangle_intersect_branchless", |b| b.iter(|| ray_triangle_intersect_branchless(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::ray_sphere_intersect_branchless::ray_sphere_intersect_branchless;
    c.bench_function("ray_sphere_intersect_branchless", |b| b.iter(|| ray_sphere_intersect_branchless(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::frustum_culling_branchless::frustum_culling_branchless;
    c.bench_function("frustum_culling_branchless", |b| b.iter(|| frustum_culling_branchless(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::point_in_polygon_branchless::point_in_polygon_branchless;
    c.bench_function("point_in_polygon_branchless", |b| b.iter(|| point_in_polygon_branchless(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::convex_hull_monotone_chain_step::convex_hull_monotone_chain_step;
    c.bench_function("convex_hull_monotone_chain_step", |b| b.iter(|| convex_hull_monotone_chain_step(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::spatial_hash_u32::spatial_hash_u32;
    c.bench_function("spatial_hash_u32", |b| b.iter(|| spatial_hash_u32(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::quadtree_insert_branchless::quadtree_insert_branchless;
    c.bench_function("quadtree_insert_branchless", |b| b.iter(|| quadtree_insert_branchless(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::octree_insert_branchless::octree_insert_branchless;
    c.bench_function("octree_insert_branchless", |b| b.iter(|| octree_insert_branchless(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::hilbert_curve_encode_u32::hilbert_curve_encode_u32;
    c.bench_function("hilbert_curve_encode_u32", |b| b.iter(|| hilbert_curve_encode_u32(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::hilbert_curve_decode_u32::hilbert_curve_decode_u32;
    c.bench_function("hilbert_curve_decode_u32", |b| b.iter(|| hilbert_curve_decode_u32(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::z_order_curve_2d_u32::z_order_curve_2d_u32;
    c.bench_function("z_order_curve_2d_u32", |b| b.iter(|| z_order_curve_2d_u32(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::bit_vector_compress_elias_fano::bit_vector_compress_elias_fano;
    c.bench_function("bit_vector_compress_elias_fano", |b| b.iter(|| bit_vector_compress_elias_fano(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::rank_select_dictionary_rrr::rank_select_dictionary_rrr;
    c.bench_function("rank_select_dictionary_rrr", |b| b.iter(|| rank_select_dictionary_rrr(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::wavelet_tree_access_branchless::wavelet_tree_access_branchless;
    c.bench_function("wavelet_tree_access_branchless", |b| b.iter(|| wavelet_tree_access_branchless(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::succinct_bit_vector_rank::succinct_bit_vector_rank;
    c.bench_function("succinct_bit_vector_rank", |b| b.iter(|| succinct_bit_vector_rank(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::succinct_bit_vector_select::succinct_bit_vector_select;
    c.bench_function("succinct_bit_vector_select", |b| b.iter(|| succinct_bit_vector_select(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::linear_congruential_generator_u64::linear_congruential_generator_u64;
    c.bench_function("linear_congruential_generator_u64", |b| b.iter(|| linear_congruential_generator_u64(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::pcg_random_u64::pcg_random_u64;
    c.bench_function("pcg_random_u64", |b| b.iter(|| pcg_random_u64(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::splitmix64_u64::splitmix64_u64;
    c.bench_function("splitmix64_u64", |b| b.iter(|| splitmix64_u64(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::xoroshiro128_plus::xoroshiro128_plus;
    c.bench_function("xoroshiro128_plus", |b| b.iter(|| xoroshiro128_plus(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::mersenne_twister_step_simd::mersenne_twister_step_simd;
    c.bench_function("mersenne_twister_step_simd", |b| b.iter(|| mersenne_twister_step_simd(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::reservoir_sample_weighted_simd::reservoir_sample_weighted_simd;
    c.bench_function("reservoir_sample_weighted_simd", |b| b.iter(|| reservoir_sample_weighted_simd(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::gaussian_noise_box_muller::gaussian_noise_box_muller;
    c.bench_function("gaussian_noise_box_muller", |b| b.iter(|| gaussian_noise_box_muller(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::poisson_noise_branchless::poisson_noise_branchless;
    c.bench_function("poisson_noise_branchless", |b| b.iter(|| poisson_noise_branchless(black_box(42), black_box(1337))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::halton_sampler_simd::halton_sampler_simd;
    c.bench_function("halton_sampler_simd", |b| b.iter(|| halton_sampler_simd(black_box(42), black_box(1337))));
}

criterion_group!(benches, algorithms_201_300);
criterion_main!(benches);
