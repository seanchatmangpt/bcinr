use bcinr_logic::algorithms::*;
use criterion::{criterion_group, criterion_main, Criterion, black_box};

fn algorithms_1_100(c: &mut Criterion) {
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::parallel_bits_deposit_u64::parallel_bits_deposit_u64;
    c.bench_function("parallel_bits_deposit_u64_avg", |b| b.iter(|| parallel_bits_deposit_u64(black_box(42), black_box(1337))));
    c.bench_function("parallel_bits_deposit_u64_min", |b| b.iter(|| parallel_bits_deposit_u64(black_box(0), black_box(0))));
    c.bench_function("parallel_bits_deposit_u64_max", |b| b.iter(|| parallel_bits_deposit_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::parallel_bits_extract_u64::parallel_bits_extract_u64;
    c.bench_function("parallel_bits_extract_u64_avg", |b| b.iter(|| parallel_bits_extract_u64(black_box(42), black_box(1337))));
    c.bench_function("parallel_bits_extract_u64_min", |b| b.iter(|| parallel_bits_extract_u64(black_box(0), black_box(0))));
    c.bench_function("parallel_bits_extract_u64_max", |b| b.iter(|| parallel_bits_extract_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::blsr_u64::blsr_u64;
    c.bench_function("blsr_u64_avg", |b| b.iter(|| blsr_u64(black_box(42), black_box(1337))));
    c.bench_function("blsr_u64_min", |b| b.iter(|| blsr_u64(black_box(0), black_box(0))));
    c.bench_function("blsr_u64_max", |b| b.iter(|| blsr_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::blsi_u64::blsi_u64;
    c.bench_function("blsi_u64_avg", |b| b.iter(|| blsi_u64(black_box(42), black_box(1337))));
    c.bench_function("blsi_u64_min", |b| b.iter(|| blsi_u64(black_box(0), black_box(0))));
    c.bench_function("blsi_u64_max", |b| b.iter(|| blsi_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::blsmsk_u64::blsmsk_u64;
    c.bench_function("blsmsk_u64_avg", |b| b.iter(|| blsmsk_u64(black_box(42), black_box(1337))));
    c.bench_function("blsmsk_u64_min", |b| b.iter(|| blsmsk_u64(black_box(0), black_box(0))));
    c.bench_function("blsmsk_u64_max", |b| b.iter(|| blsmsk_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::t1mskc_u64::t1mskc_u64;
    c.bench_function("t1mskc_u64_avg", |b| b.iter(|| t1mskc_u64(black_box(42), black_box(1337))));
    c.bench_function("t1mskc_u64_min", |b| b.iter(|| t1mskc_u64(black_box(0), black_box(0))));
    c.bench_function("t1mskc_u64_max", |b| b.iter(|| t1mskc_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::tzmsk_u64::tzmsk_u64;
    c.bench_function("tzmsk_u64_avg", |b| b.iter(|| tzmsk_u64(black_box(42), black_box(1337))));
    c.bench_function("tzmsk_u64_min", |b| b.iter(|| tzmsk_u64(black_box(0), black_box(0))));
    c.bench_function("tzmsk_u64_max", |b| b.iter(|| tzmsk_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::bext_u64::bext_u64;
    c.bench_function("bext_u64_avg", |b| b.iter(|| bext_u64(black_box(42), black_box(1337))));
    c.bench_function("bext_u64_min", |b| b.iter(|| bext_u64(black_box(0), black_box(0))));
    c.bench_function("bext_u64_max", |b| b.iter(|| bext_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::bset_u64::bset_u64;
    c.bench_function("bset_u64_avg", |b| b.iter(|| bset_u64(black_box(42), black_box(1337))));
    c.bench_function("bset_u64_min", |b| b.iter(|| bset_u64(black_box(0), black_box(0))));
    c.bench_function("bset_u64_max", |b| b.iter(|| bset_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::bclr_u64::bclr_u64;
    c.bench_function("bclr_u64_avg", |b| b.iter(|| bclr_u64(black_box(42), black_box(1337))));
    c.bench_function("bclr_u64_min", |b| b.iter(|| bclr_u64(black_box(0), black_box(0))));
    c.bench_function("bclr_u64_max", |b| b.iter(|| bclr_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::btst_u64::btst_u64;
    c.bench_function("btst_u64_avg", |b| b.iter(|| btst_u64(black_box(42), black_box(1337))));
    c.bench_function("btst_u64_min", |b| b.iter(|| btst_u64(black_box(0), black_box(0))));
    c.bench_function("btst_u64_max", |b| b.iter(|| btst_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::popcount_u128::popcount_u128;
    c.bench_function("popcount_u128_avg", |b| b.iter(|| popcount_u128(black_box(42), black_box(1337))));
    c.bench_function("popcount_u128_min", |b| b.iter(|| popcount_u128(black_box(0), black_box(0))));
    c.bench_function("popcount_u128_max", |b| b.iter(|| popcount_u128(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::reverse_bits_u128::reverse_bits_u128;
    c.bench_function("reverse_bits_u128_avg", |b| b.iter(|| reverse_bits_u128(black_box(42), black_box(1337))));
    c.bench_function("reverse_bits_u128_min", |b| b.iter(|| reverse_bits_u128(black_box(0), black_box(0))));
    c.bench_function("reverse_bits_u128_max", |b| b.iter(|| reverse_bits_u128(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::clmul_u64::clmul_u64;
    c.bench_function("clmul_u64_avg", |b| b.iter(|| clmul_u64(black_box(42), black_box(1337))));
    c.bench_function("clmul_u64_min", |b| b.iter(|| clmul_u64(black_box(0), black_box(0))));
    c.bench_function("clmul_u64_max", |b| b.iter(|| clmul_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::morton_encode_2d_u32::morton_encode_2d_u32;
    c.bench_function("morton_encode_2d_u32_avg", |b| b.iter(|| morton_encode_2d_u32(black_box(42), black_box(1337))));
    c.bench_function("morton_encode_2d_u32_min", |b| b.iter(|| morton_encode_2d_u32(black_box(0), black_box(0))));
    c.bench_function("morton_encode_2d_u32_max", |b| b.iter(|| morton_encode_2d_u32(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::morton_decode_2d_u32::morton_decode_2d_u32;
    c.bench_function("morton_decode_2d_u32_avg", |b| b.iter(|| morton_decode_2d_u32(black_box(42), black_box(1337))));
    c.bench_function("morton_decode_2d_u32_min", |b| b.iter(|| morton_decode_2d_u32(black_box(0), black_box(0))));
    c.bench_function("morton_decode_2d_u32_max", |b| b.iter(|| morton_decode_2d_u32(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::morton_encode_3d_u32::morton_encode_3d_u32;
    c.bench_function("morton_encode_3d_u32_avg", |b| b.iter(|| morton_encode_3d_u32(black_box(42), black_box(1337))));
    c.bench_function("morton_encode_3d_u32_min", |b| b.iter(|| morton_encode_3d_u32(black_box(0), black_box(0))));
    c.bench_function("morton_encode_3d_u32_max", |b| b.iter(|| morton_encode_3d_u32(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::gray_encode_u64::gray_encode_u64;
    c.bench_function("gray_encode_u64_avg", |b| b.iter(|| gray_encode_u64(black_box(42), black_box(1337))));
    c.bench_function("gray_encode_u64_min", |b| b.iter(|| gray_encode_u64(black_box(0), black_box(0))));
    c.bench_function("gray_encode_u64_max", |b| b.iter(|| gray_encode_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::gray_decode_u64::gray_decode_u64;
    c.bench_function("gray_decode_u64_avg", |b| b.iter(|| gray_decode_u64(black_box(42), black_box(1337))));
    c.bench_function("gray_decode_u64_min", |b| b.iter(|| gray_decode_u64(black_box(0), black_box(0))));
    c.bench_function("gray_decode_u64_max", |b| b.iter(|| gray_decode_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::parity_check_u128::parity_check_u128;
    c.bench_function("parity_check_u128_avg", |b| b.iter(|| parity_check_u128(black_box(42), black_box(1337))));
    c.bench_function("parity_check_u128_min", |b| b.iter(|| parity_check_u128(black_box(0), black_box(0))));
    c.bench_function("parity_check_u128_max", |b| b.iter(|| parity_check_u128(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::next_lexicographic_permutation_u64::next_lexicographic_permutation_u64;
    c.bench_function("next_lexicographic_permutation_u64_avg", |b| b.iter(|| next_lexicographic_permutation_u64(black_box(42), black_box(1337))));
    c.bench_function("next_lexicographic_permutation_u64_min", |b| b.iter(|| next_lexicographic_permutation_u64(black_box(0), black_box(0))));
    c.bench_function("next_lexicographic_permutation_u64_max", |b| b.iter(|| next_lexicographic_permutation_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::count_consecutive_set_bits_u64::count_consecutive_set_bits_u64;
    c.bench_function("count_consecutive_set_bits_u64_avg", |b| b.iter(|| count_consecutive_set_bits_u64(black_box(42), black_box(1337))));
    c.bench_function("count_consecutive_set_bits_u64_min", |b| b.iter(|| count_consecutive_set_bits_u64(black_box(0), black_box(0))));
    c.bench_function("count_consecutive_set_bits_u64_max", |b| b.iter(|| count_consecutive_set_bits_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::find_nth_set_bit_u128::find_nth_set_bit_u128;
    c.bench_function("find_nth_set_bit_u128_avg", |b| b.iter(|| find_nth_set_bit_u128(black_box(42), black_box(1337))));
    c.bench_function("find_nth_set_bit_u128_min", |b| b.iter(|| find_nth_set_bit_u128(black_box(0), black_box(0))));
    c.bench_function("find_nth_set_bit_u128_max", |b| b.iter(|| find_nth_set_bit_u128(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::mask_range_u64::mask_range_u64;
    c.bench_function("mask_range_u64_avg", |b| b.iter(|| mask_range_u64(black_box(42), black_box(1337))));
    c.bench_function("mask_range_u64_min", |b| b.iter(|| mask_range_u64(black_box(0), black_box(0))));
    c.bench_function("mask_range_u64_max", |b| b.iter(|| mask_range_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::rotate_left_u64::rotate_left_u64;
    c.bench_function("rotate_left_u64_avg", |b| b.iter(|| rotate_left_u64(black_box(42), black_box(1337))));
    c.bench_function("rotate_left_u64_min", |b| b.iter(|| rotate_left_u64(black_box(0), black_box(0))));
    c.bench_function("rotate_left_u64_max", |b| b.iter(|| rotate_left_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::rotate_right_u64::rotate_right_u64;
    c.bench_function("rotate_right_u64_avg", |b| b.iter(|| rotate_right_u64(black_box(42), black_box(1337))));
    c.bench_function("rotate_right_u64_min", |b| b.iter(|| rotate_right_u64(black_box(0), black_box(0))));
    c.bench_function("rotate_right_u64_max", |b| b.iter(|| rotate_right_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::funnel_shift_left_u64::funnel_shift_left_u64;
    c.bench_function("funnel_shift_left_u64_avg", |b| b.iter(|| funnel_shift_left_u64(black_box(42), black_box(1337))));
    c.bench_function("funnel_shift_left_u64_min", |b| b.iter(|| funnel_shift_left_u64(black_box(0), black_box(0))));
    c.bench_function("funnel_shift_left_u64_max", |b| b.iter(|| funnel_shift_left_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::funnel_shift_right_u64::funnel_shift_right_u64;
    c.bench_function("funnel_shift_right_u64_avg", |b| b.iter(|| funnel_shift_right_u64(black_box(42), black_box(1337))));
    c.bench_function("funnel_shift_right_u64_min", |b| b.iter(|| funnel_shift_right_u64(black_box(0), black_box(0))));
    c.bench_function("funnel_shift_right_u64_max", |b| b.iter(|| funnel_shift_right_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::bit_swap_u64::bit_swap_u64;
    c.bench_function("bit_swap_u64_avg", |b| b.iter(|| bit_swap_u64(black_box(42), black_box(1337))));
    c.bench_function("bit_swap_u64_min", |b| b.iter(|| bit_swap_u64(black_box(0), black_box(0))));
    c.bench_function("bit_swap_u64_max", |b| b.iter(|| bit_swap_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::gather_bits_u64::gather_bits_u64;
    c.bench_function("gather_bits_u64_avg", |b| b.iter(|| gather_bits_u64(black_box(42), black_box(1337))));
    c.bench_function("gather_bits_u64_min", |b| b.iter(|| gather_bits_u64(black_box(0), black_box(0))));
    c.bench_function("gather_bits_u64_max", |b| b.iter(|| gather_bits_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::scatter_bits_u64::scatter_bits_u64;
    c.bench_function("scatter_bits_u64_avg", |b| b.iter(|| scatter_bits_u64(black_box(42), black_box(1337))));
    c.bench_function("scatter_bits_u64_min", |b| b.iter(|| scatter_bits_u64(black_box(0), black_box(0))));
    c.bench_function("scatter_bits_u64_max", |b| b.iter(|| scatter_bits_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::is_contiguous_mask_u64::is_contiguous_mask_u64;
    c.bench_function("is_contiguous_mask_u64_avg", |b| b.iter(|| is_contiguous_mask_u64(black_box(42), black_box(1337))));
    c.bench_function("is_contiguous_mask_u64_min", |b| b.iter(|| is_contiguous_mask_u64(black_box(0), black_box(0))));
    c.bench_function("is_contiguous_mask_u64_max", |b| b.iter(|| is_contiguous_mask_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::get_mask_boundary_low_u64::get_mask_boundary_low_u64;
    c.bench_function("get_mask_boundary_low_u64_avg", |b| b.iter(|| get_mask_boundary_low_u64(black_box(42), black_box(1337))));
    c.bench_function("get_mask_boundary_low_u64_min", |b| b.iter(|| get_mask_boundary_low_u64(black_box(0), black_box(0))));
    c.bench_function("get_mask_boundary_low_u64_max", |b| b.iter(|| get_mask_boundary_low_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::get_mask_boundary_high_u64::get_mask_boundary_high_u64;
    c.bench_function("get_mask_boundary_high_u64_avg", |b| b.iter(|| get_mask_boundary_high_u64(black_box(42), black_box(1337))));
    c.bench_function("get_mask_boundary_high_u64_min", |b| b.iter(|| get_mask_boundary_high_u64(black_box(0), black_box(0))));
    c.bench_function("get_mask_boundary_high_u64_max", |b| b.iter(|| get_mask_boundary_high_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::bit_matrix_transpose_8x8::bit_matrix_transpose_8x8;
    c.bench_function("bit_matrix_transpose_8x8_avg", |b| b.iter(|| bit_matrix_transpose_8x8(black_box(42), black_box(1337))));
    c.bench_function("bit_matrix_transpose_8x8_min", |b| b.iter(|| bit_matrix_transpose_8x8(black_box(0), black_box(0))));
    c.bench_function("bit_matrix_transpose_8x8_max", |b| b.iter(|| bit_matrix_transpose_8x8(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::bit_matrix_transpose_64x64::bit_matrix_transpose_64x64;
    c.bench_function("bit_matrix_transpose_64x64_avg", |b| b.iter(|| bit_matrix_transpose_64x64(black_box(42), black_box(1337))));
    c.bench_function("bit_matrix_transpose_64x64_min", |b| b.iter(|| bit_matrix_transpose_64x64(black_box(0), black_box(0))));
    c.bench_function("bit_matrix_transpose_64x64_max", |b| b.iter(|| bit_matrix_transpose_64x64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::rank_u128::rank_u128;
    c.bench_function("rank_u128_avg", |b| b.iter(|| rank_u128(black_box(42), black_box(1337))));
    c.bench_function("rank_u128_min", |b| b.iter(|| rank_u128(black_box(0), black_box(0))));
    c.bench_function("rank_u128_max", |b| b.iter(|| rank_u128(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::select_u128::select_u128;
    c.bench_function("select_u128_avg", |b| b.iter(|| select_u128(black_box(42), black_box(1337))));
    c.bench_function("select_u128_min", |b| b.iter(|| select_u128(black_box(0), black_box(0))));
    c.bench_function("select_u128_max", |b| b.iter(|| select_u128(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::weight_u64::weight_u64;
    c.bench_function("weight_u64_avg", |b| b.iter(|| weight_u64(black_box(42), black_box(1337))));
    c.bench_function("weight_u64_min", |b| b.iter(|| weight_u64(black_box(0), black_box(0))));
    c.bench_function("weight_u64_max", |b| b.iter(|| weight_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::delta_swap_u64::delta_swap_u64;
    c.bench_function("delta_swap_u64_avg", |b| b.iter(|| delta_swap_u64(black_box(42), black_box(1337))));
    c.bench_function("delta_swap_u64_min", |b| b.iter(|| delta_swap_u64(black_box(0), black_box(0))));
    c.bench_function("delta_swap_u64_max", |b| b.iter(|| delta_swap_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::benes_network_u64::benes_network_u64;
    c.bench_function("benes_network_u64_avg", |b| b.iter(|| benes_network_u64(black_box(42), black_box(1337))));
    c.bench_function("benes_network_u64_min", |b| b.iter(|| benes_network_u64(black_box(0), black_box(0))));
    c.bench_function("benes_network_u64_max", |b| b.iter(|| benes_network_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::bit_permute_step_u64::bit_permute_step_u64;
    c.bench_function("bit_permute_step_u64_avg", |b| b.iter(|| bit_permute_step_u64(black_box(42), black_box(1337))));
    c.bench_function("bit_permute_step_u64_min", |b| b.iter(|| bit_permute_step_u64(black_box(0), black_box(0))));
    c.bench_function("bit_permute_step_u64_max", |b| b.iter(|| bit_permute_step_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::compress_bits_u64::compress_bits_u64;
    c.bench_function("compress_bits_u64_avg", |b| b.iter(|| compress_bits_u64(black_box(42), black_box(1337))));
    c.bench_function("compress_bits_u64_min", |b| b.iter(|| compress_bits_u64(black_box(0), black_box(0))));
    c.bench_function("compress_bits_u64_max", |b| b.iter(|| compress_bits_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::expand_bits_u64::expand_bits_u64;
    c.bench_function("expand_bits_u64_avg", |b| b.iter(|| expand_bits_u64(black_box(42), black_box(1337))));
    c.bench_function("expand_bits_u64_min", |b| b.iter(|| expand_bits_u64(black_box(0), black_box(0))));
    c.bench_function("expand_bits_u64_max", |b| b.iter(|| expand_bits_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::crossbar_permute_u8x16::crossbar_permute_u8x16;
    c.bench_function("crossbar_permute_u8x16_avg", |b| b.iter(|| crossbar_permute_u8x16(black_box(42), black_box(1337))));
    c.bench_function("crossbar_permute_u8x16_min", |b| b.iter(|| crossbar_permute_u8x16(black_box(0), black_box(0))));
    c.bench_function("crossbar_permute_u8x16_max", |b| b.iter(|| crossbar_permute_u8x16(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::mask_from_bool_slice::mask_from_bool_slice;
    c.bench_function("mask_from_bool_slice_avg", |b| b.iter(|| mask_from_bool_slice(black_box(42), black_box(1337))));
    c.bench_function("mask_from_bool_slice_min", |b| b.iter(|| mask_from_bool_slice(black_box(0), black_box(0))));
    c.bench_function("mask_from_bool_slice_max", |b| b.iter(|| mask_from_bool_slice(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::bool_slice_from_mask::bool_slice_from_mask;
    c.bench_function("bool_slice_from_mask_avg", |b| b.iter(|| bool_slice_from_mask(black_box(42), black_box(1337))));
    c.bench_function("bool_slice_from_mask_min", |b| b.iter(|| bool_slice_from_mask(black_box(0), black_box(0))));
    c.bench_function("bool_slice_from_mask_max", |b| b.iter(|| bool_slice_from_mask(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::bit_permute_identity_64::bit_permute_identity_64;
    c.bench_function("bit_permute_identity_64_avg", |b| b.iter(|| bit_permute_identity_64(black_box(42), black_box(1337))));
    c.bench_function("bit_permute_identity_64_min", |b| b.iter(|| bit_permute_identity_64(black_box(0), black_box(0))));
    c.bench_function("bit_permute_identity_64_max", |b| b.iter(|| bit_permute_identity_64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::is_subset_mask_u64::is_subset_mask_u64;
    c.bench_function("is_subset_mask_u64_avg", |b| b.iter(|| is_subset_mask_u64(black_box(42), black_box(1337))));
    c.bench_function("is_subset_mask_u64_min", |b| b.iter(|| is_subset_mask_u64(black_box(0), black_box(0))));
    c.bench_function("is_subset_mask_u64_max", |b| b.iter(|| is_subset_mask_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::mask_xor_reduce_u64::mask_xor_reduce_u64;
    c.bench_function("mask_xor_reduce_u64_avg", |b| b.iter(|| mask_xor_reduce_u64(black_box(42), black_box(1337))));
    c.bench_function("mask_xor_reduce_u64_min", |b| b.iter(|| mask_xor_reduce_u64(black_box(0), black_box(0))));
    c.bench_function("mask_xor_reduce_u64_max", |b| b.iter(|| mask_xor_reduce_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::mul_sat_u64::mul_sat_u64;
    c.bench_function("mul_sat_u64_avg", |b| b.iter(|| mul_sat_u64(black_box(42), black_box(1337))));
    c.bench_function("mul_sat_u64_min", |b| b.iter(|| mul_sat_u64(black_box(0), black_box(0))));
    c.bench_function("mul_sat_u64_max", |b| b.iter(|| mul_sat_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::div_sat_u64::div_sat_u64;
    c.bench_function("div_sat_u64_avg", |b| b.iter(|| div_sat_u64(black_box(42), black_box(1337))));
    c.bench_function("div_sat_u64_min", |b| b.iter(|| div_sat_u64(black_box(0), black_box(0))));
    c.bench_function("div_sat_u64_max", |b| b.iter(|| div_sat_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::add_sat_i32::add_sat_i32;
    c.bench_function("add_sat_i32_avg", |b| b.iter(|| add_sat_i32(black_box(42), black_box(1337))));
    c.bench_function("add_sat_i32_min", |b| b.iter(|| add_sat_i32(black_box(0), black_box(0))));
    c.bench_function("add_sat_i32_max", |b| b.iter(|| add_sat_i32(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::sub_sat_i32::sub_sat_i32;
    c.bench_function("sub_sat_i32_avg", |b| b.iter(|| sub_sat_i32(black_box(42), black_box(1337))));
    c.bench_function("sub_sat_i32_min", |b| b.iter(|| sub_sat_i32(black_box(0), black_box(0))));
    c.bench_function("sub_sat_i32_max", |b| b.iter(|| sub_sat_i32(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::mul_sat_i32::mul_sat_i32;
    c.bench_function("mul_sat_i32_avg", |b| b.iter(|| mul_sat_i32(black_box(42), black_box(1337))));
    c.bench_function("mul_sat_i32_min", |b| b.iter(|| mul_sat_i32(black_box(0), black_box(0))));
    c.bench_function("mul_sat_i32_max", |b| b.iter(|| mul_sat_i32(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::abs_diff_u64::abs_diff_u64;
    c.bench_function("abs_diff_u64_avg", |b| b.iter(|| abs_diff_u64(black_box(42), black_box(1337))));
    c.bench_function("abs_diff_u64_min", |b| b.iter(|| abs_diff_u64(black_box(0), black_box(0))));
    c.bench_function("abs_diff_u64_max", |b| b.iter(|| abs_diff_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::abs_diff_i64::abs_diff_i64;
    c.bench_function("abs_diff_i64_avg", |b| b.iter(|| abs_diff_i64(black_box(42), black_box(1337))));
    c.bench_function("abs_diff_i64_min", |b| b.iter(|| abs_diff_i64(black_box(0), black_box(0))));
    c.bench_function("abs_diff_i64_max", |b| b.iter(|| abs_diff_i64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::avg_u64::avg_u64;
    c.bench_function("avg_u64_avg", |b| b.iter(|| avg_u64(black_box(42), black_box(1337))));
    c.bench_function("avg_u64_min", |b| b.iter(|| avg_u64(black_box(0), black_box(0))));
    c.bench_function("avg_u64_max", |b| b.iter(|| avg_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::avg_ceil_u64::avg_ceil_u64;
    c.bench_function("avg_ceil_u64_avg", |b| b.iter(|| avg_ceil_u64(black_box(42), black_box(1337))));
    c.bench_function("avg_ceil_u64_min", |b| b.iter(|| avg_ceil_u64(black_box(0), black_box(0))));
    c.bench_function("avg_ceil_u64_max", |b| b.iter(|| avg_ceil_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::clamp_i64::clamp_i64;
    c.bench_function("clamp_i64_avg", |b| b.iter(|| clamp_i64(black_box(42), black_box(1337))));
    c.bench_function("clamp_i64_min", |b| b.iter(|| clamp_i64(black_box(0), black_box(0))));
    c.bench_function("clamp_i64_max", |b| b.iter(|| clamp_i64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::lerp_sat_u8::lerp_sat_u8;
    c.bench_function("lerp_sat_u8_avg", |b| b.iter(|| lerp_sat_u8(black_box(42), black_box(1337))));
    c.bench_function("lerp_sat_u8_min", |b| b.iter(|| lerp_sat_u8(black_box(0), black_box(0))));
    c.bench_function("lerp_sat_u8_max", |b| b.iter(|| lerp_sat_u8(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::lerp_sat_u32::lerp_sat_u32;
    c.bench_function("lerp_sat_u32_avg", |b| b.iter(|| lerp_sat_u32(black_box(42), black_box(1337))));
    c.bench_function("lerp_sat_u32_min", |b| b.iter(|| lerp_sat_u32(black_box(0), black_box(0))));
    c.bench_function("lerp_sat_u32_max", |b| b.iter(|| lerp_sat_u32(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::norm_u32::norm_u32;
    c.bench_function("norm_u32_avg", |b| b.iter(|| norm_u32(black_box(42), black_box(1337))));
    c.bench_function("norm_u32_min", |b| b.iter(|| norm_u32(black_box(0), black_box(0))));
    c.bench_function("norm_u32_max", |b| b.iter(|| norm_u32(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::fp_mul_u32_q16::fp_mul_u32_q16;
    c.bench_function("fp_mul_u32_q16_avg", |b| b.iter(|| fp_mul_u32_q16(black_box(42), black_box(1337))));
    c.bench_function("fp_mul_u32_q16_min", |b| b.iter(|| fp_mul_u32_q16(black_box(0), black_box(0))));
    c.bench_function("fp_mul_u32_q16_max", |b| b.iter(|| fp_mul_u32_q16(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::fp_div_u32_q16::fp_div_u32_q16;
    c.bench_function("fp_div_u32_q16_avg", |b| b.iter(|| fp_div_u32_q16(black_box(42), black_box(1337))));
    c.bench_function("fp_div_u32_q16_min", |b| b.iter(|| fp_div_u32_q16(black_box(0), black_box(0))));
    c.bench_function("fp_div_u32_q16_max", |b| b.iter(|| fp_div_u32_q16(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::fp_sqrt_u32_q16::fp_sqrt_u32_q16;
    c.bench_function("fp_sqrt_u32_q16_avg", |b| b.iter(|| fp_sqrt_u32_q16(black_box(42), black_box(1337))));
    c.bench_function("fp_sqrt_u32_q16_min", |b| b.iter(|| fp_sqrt_u32_q16(black_box(0), black_box(0))));
    c.bench_function("fp_sqrt_u32_q16_max", |b| b.iter(|| fp_sqrt_u32_q16(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::fp_sin_u32_q16::fp_sin_u32_q16;
    c.bench_function("fp_sin_u32_q16_avg", |b| b.iter(|| fp_sin_u32_q16(black_box(42), black_box(1337))));
    c.bench_function("fp_sin_u32_q16_min", |b| b.iter(|| fp_sin_u32_q16(black_box(0), black_box(0))));
    c.bench_function("fp_sin_u32_q16_max", |b| b.iter(|| fp_sin_u32_q16(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::fp_cos_u32_q16::fp_cos_u32_q16;
    c.bench_function("fp_cos_u32_q16_avg", |b| b.iter(|| fp_cos_u32_q16(black_box(42), black_box(1337))));
    c.bench_function("fp_cos_u32_q16_min", |b| b.iter(|| fp_cos_u32_q16(black_box(0), black_box(0))));
    c.bench_function("fp_cos_u32_q16_max", |b| b.iter(|| fp_cos_u32_q16(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::fp_atan2_u32_q16::fp_atan2_u32_q16;
    c.bench_function("fp_atan2_u32_q16_avg", |b| b.iter(|| fp_atan2_u32_q16(black_box(42), black_box(1337))));
    c.bench_function("fp_atan2_u32_q16_min", |b| b.iter(|| fp_atan2_u32_q16(black_box(0), black_box(0))));
    c.bench_function("fp_atan2_u32_q16_max", |b| b.iter(|| fp_atan2_u32_q16(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::log2_u64_fixed::log2_u64_fixed;
    c.bench_function("log2_u64_fixed_avg", |b| b.iter(|| log2_u64_fixed(black_box(42), black_box(1337))));
    c.bench_function("log2_u64_fixed_min", |b| b.iter(|| log2_u64_fixed(black_box(0), black_box(0))));
    c.bench_function("log2_u64_fixed_max", |b| b.iter(|| log2_u64_fixed(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::exp2_u64_fixed::exp2_u64_fixed;
    c.bench_function("exp2_u64_fixed_avg", |b| b.iter(|| exp2_u64_fixed(black_box(42), black_box(1337))));
    c.bench_function("exp2_u64_fixed_min", |b| b.iter(|| exp2_u64_fixed(black_box(0), black_box(0))));
    c.bench_function("exp2_u64_fixed_max", |b| b.iter(|| exp2_u64_fixed(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::sigmoid_sat_u32::sigmoid_sat_u32;
    c.bench_function("sigmoid_sat_u32_avg", |b| b.iter(|| sigmoid_sat_u32(black_box(42), black_box(1337))));
    c.bench_function("sigmoid_sat_u32_min", |b| b.iter(|| sigmoid_sat_u32(black_box(0), black_box(0))));
    c.bench_function("sigmoid_sat_u32_max", |b| b.iter(|| sigmoid_sat_u32(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::relu_u32::relu_u32;
    c.bench_function("relu_u32_avg", |b| b.iter(|| relu_u32(black_box(42), black_box(1337))));
    c.bench_function("relu_u32_min", |b| b.iter(|| relu_u32(black_box(0), black_box(0))));
    c.bench_function("relu_u32_max", |b| b.iter(|| relu_u32(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::leaky_relu_u32::leaky_relu_u32;
    c.bench_function("leaky_relu_u32_avg", |b| b.iter(|| leaky_relu_u32(black_box(42), black_box(1337))));
    c.bench_function("leaky_relu_u32_min", |b| b.iter(|| leaky_relu_u32(black_box(0), black_box(0))));
    c.bench_function("leaky_relu_u32_max", |b| b.iter(|| leaky_relu_u32(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::softmax_u32x4::softmax_u32x4;
    c.bench_function("softmax_u32x4_avg", |b| b.iter(|| softmax_u32x4(black_box(42), black_box(1337))));
    c.bench_function("softmax_u32x4_min", |b| b.iter(|| softmax_u32x4(black_box(0), black_box(0))));
    c.bench_function("softmax_u32x4_max", |b| b.iter(|| softmax_u32x4(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::fast_inverse_sqrt_u32::fast_inverse_sqrt_u32;
    c.bench_function("fast_inverse_sqrt_u32_avg", |b| b.iter(|| fast_inverse_sqrt_u32(black_box(42), black_box(1337))));
    c.bench_function("fast_inverse_sqrt_u32_min", |b| b.iter(|| fast_inverse_sqrt_u32(black_box(0), black_box(0))));
    c.bench_function("fast_inverse_sqrt_u32_max", |b| b.iter(|| fast_inverse_sqrt_u32(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::gcd_u64_branchless::gcd_u64_branchless;
    c.bench_function("gcd_u64_branchless_avg", |b| b.iter(|| gcd_u64_branchless(black_box(42), black_box(1337))));
    c.bench_function("gcd_u64_branchless_min", |b| b.iter(|| gcd_u64_branchless(black_box(0), black_box(0))));
    c.bench_function("gcd_u64_branchless_max", |b| b.iter(|| gcd_u64_branchless(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::lcm_u64_branchless::lcm_u64_branchless;
    c.bench_function("lcm_u64_branchless_avg", |b| b.iter(|| lcm_u64_branchless(black_box(42), black_box(1337))));
    c.bench_function("lcm_u64_branchless_min", |b| b.iter(|| lcm_u64_branchless(black_box(0), black_box(0))));
    c.bench_function("lcm_u64_branchless_max", |b| b.iter(|| lcm_u64_branchless(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::modular_add_u64::modular_add_u64;
    c.bench_function("modular_add_u64_avg", |b| b.iter(|| modular_add_u64(black_box(42), black_box(1337))));
    c.bench_function("modular_add_u64_min", |b| b.iter(|| modular_add_u64(black_box(0), black_box(0))));
    c.bench_function("modular_add_u64_max", |b| b.iter(|| modular_add_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::modular_sub_u64::modular_sub_u64;
    c.bench_function("modular_sub_u64_avg", |b| b.iter(|| modular_sub_u64(black_box(42), black_box(1337))));
    c.bench_function("modular_sub_u64_min", |b| b.iter(|| modular_sub_u64(black_box(0), black_box(0))));
    c.bench_function("modular_sub_u64_max", |b| b.iter(|| modular_sub_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::modular_mul_u64::modular_mul_u64;
    c.bench_function("modular_mul_u64_avg", |b| b.iter(|| modular_mul_u64(black_box(42), black_box(1337))));
    c.bench_function("modular_mul_u64_min", |b| b.iter(|| modular_mul_u64(black_box(0), black_box(0))));
    c.bench_function("modular_mul_u64_max", |b| b.iter(|| modular_mul_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::is_prime_u64_branchless::is_prime_u64_branchless;
    c.bench_function("is_prime_u64_branchless_avg", |b| b.iter(|| is_prime_u64_branchless(black_box(42), black_box(1337))));
    c.bench_function("is_prime_u64_branchless_min", |b| b.iter(|| is_prime_u64_branchless(black_box(0), black_box(0))));
    c.bench_function("is_prime_u64_branchless_max", |b| b.iter(|| is_prime_u64_branchless(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::factorial_sat_u32::factorial_sat_u32;
    c.bench_function("factorial_sat_u32_avg", |b| b.iter(|| factorial_sat_u32(black_box(42), black_box(1337))));
    c.bench_function("factorial_sat_u32_min", |b| b.iter(|| factorial_sat_u32(black_box(0), black_box(0))));
    c.bench_function("factorial_sat_u32_max", |b| b.iter(|| factorial_sat_u32(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::binom_sat_u32::binom_sat_u32;
    c.bench_function("binom_sat_u32_avg", |b| b.iter(|| binom_sat_u32(black_box(42), black_box(1337))));
    c.bench_function("binom_sat_u32_min", |b| b.iter(|| binom_sat_u32(black_box(0), black_box(0))));
    c.bench_function("binom_sat_u32_max", |b| b.iter(|| binom_sat_u32(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::pow_sat_u64::pow_sat_u64;
    c.bench_function("pow_sat_u64_avg", |b| b.iter(|| pow_sat_u64(black_box(42), black_box(1337))));
    c.bench_function("pow_sat_u64_min", |b| b.iter(|| pow_sat_u64(black_box(0), black_box(0))));
    c.bench_function("pow_sat_u64_max", |b| b.iter(|| pow_sat_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::clamped_scaling_u64::clamped_scaling_u64;
    c.bench_function("clamped_scaling_u64_avg", |b| b.iter(|| clamped_scaling_u64(black_box(42), black_box(1337))));
    c.bench_function("clamped_scaling_u64_min", |b| b.iter(|| clamped_scaling_u64(black_box(0), black_box(0))));
    c.bench_function("clamped_scaling_u64_max", |b| b.iter(|| clamped_scaling_u64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::branchless_signum_i64::branchless_signum_i64;
    c.bench_function("branchless_signum_i64_avg", |b| b.iter(|| branchless_signum_i64(black_box(42), black_box(1337))));
    c.bench_function("branchless_signum_i64_min", |b| b.iter(|| branchless_signum_i64(black_box(0), black_box(0))));
    c.bench_function("branchless_signum_i64_max", |b| b.iter(|| branchless_signum_i64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::copy_sign_i64::copy_sign_i64;
    c.bench_function("copy_sign_i64_avg", |b| b.iter(|| copy_sign_i64(black_box(42), black_box(1337))));
    c.bench_function("copy_sign_i64_min", |b| b.iter(|| copy_sign_i64(black_box(0), black_box(0))));
    c.bench_function("copy_sign_i64_max", |b| b.iter(|| copy_sign_i64(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::is_finite_fp32_branchless::is_finite_fp32_branchless;
    c.bench_function("is_finite_fp32_branchless_avg", |b| b.iter(|| is_finite_fp32_branchless(black_box(42), black_box(1337))));
    c.bench_function("is_finite_fp32_branchless_min", |b| b.iter(|| is_finite_fp32_branchless(black_box(0), black_box(0))));
    c.bench_function("is_finite_fp32_branchless_max", |b| b.iter(|| is_finite_fp32_branchless(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::is_nan_fp32_branchless::is_nan_fp32_branchless;
    c.bench_function("is_nan_fp32_branchless_avg", |b| b.iter(|| is_nan_fp32_branchless(black_box(42), black_box(1337))));
    c.bench_function("is_nan_fp32_branchless_min", |b| b.iter(|| is_nan_fp32_branchless(black_box(0), black_box(0))));
    c.bench_function("is_nan_fp32_branchless_max", |b| b.iter(|| is_nan_fp32_branchless(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::round_to_nearest_u32::round_to_nearest_u32;
    c.bench_function("round_to_nearest_u32_avg", |b| b.iter(|| round_to_nearest_u32(black_box(42), black_box(1337))));
    c.bench_function("round_to_nearest_u32_min", |b| b.iter(|| round_to_nearest_u32(black_box(0), black_box(0))));
    c.bench_function("round_to_nearest_u32_max", |b| b.iter(|| round_to_nearest_u32(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::round_up_u32::round_up_u32;
    c.bench_function("round_up_u32_avg", |b| b.iter(|| round_up_u32(black_box(42), black_box(1337))));
    c.bench_function("round_up_u32_min", |b| b.iter(|| round_up_u32(black_box(0), black_box(0))));
    c.bench_function("round_up_u32_max", |b| b.iter(|| round_up_u32(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::round_down_u32::round_down_u32;
    c.bench_function("round_down_u32_avg", |b| b.iter(|| round_down_u32(black_box(42), black_box(1337))));
    c.bench_function("round_down_u32_min", |b| b.iter(|| round_down_u32(black_box(0), black_box(0))));
    c.bench_function("round_down_u32_max", |b| b.iter(|| round_down_u32(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::quantize_u32::quantize_u32;
    c.bench_function("quantize_u32_avg", |b| b.iter(|| quantize_u32(black_box(42), black_box(1337))));
    c.bench_function("quantize_u32_min", |b| b.iter(|| quantize_u32(black_box(0), black_box(0))));
    c.bench_function("quantize_u32_max", |b| b.iter(|| quantize_u32(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::dequantize_u32::dequantize_u32;
    c.bench_function("dequantize_u32_avg", |b| b.iter(|| dequantize_u32(black_box(42), black_box(1337))));
    c.bench_function("dequantize_u32_min", |b| b.iter(|| dequantize_u32(black_box(0), black_box(0))));
    c.bench_function("dequantize_u32_max", |b| b.iter(|| dequantize_u32(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::weighted_avg_u32::weighted_avg_u32;
    c.bench_function("weighted_avg_u32_avg", |b| b.iter(|| weighted_avg_u32(black_box(42), black_box(1337))));
    c.bench_function("weighted_avg_u32_min", |b| b.iter(|| weighted_avg_u32(black_box(0), black_box(0))));
    c.bench_function("weighted_avg_u32_max", |b| b.iter(|| weighted_avg_u32(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::smoothstep_u32::smoothstep_u32;
    c.bench_function("smoothstep_u32_avg", |b| b.iter(|| smoothstep_u32(black_box(42), black_box(1337))));
    c.bench_function("smoothstep_u32_min", |b| b.iter(|| smoothstep_u32(black_box(0), black_box(0))));
    c.bench_function("smoothstep_u32_max", |b| b.iter(|| smoothstep_u32(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::cubic_interpolate_u32::cubic_interpolate_u32;
    c.bench_function("cubic_interpolate_u32_avg", |b| b.iter(|| cubic_interpolate_u32(black_box(42), black_box(1337))));
    c.bench_function("cubic_interpolate_u32_min", |b| b.iter(|| cubic_interpolate_u32(black_box(0), black_box(0))));
    c.bench_function("cubic_interpolate_u32_max", |b| b.iter(|| cubic_interpolate_u32(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::manhattan_dist_u32x2::manhattan_dist_u32x2;
    c.bench_function("manhattan_dist_u32x2_avg", |b| b.iter(|| manhattan_dist_u32x2(black_box(42), black_box(1337))));
    c.bench_function("manhattan_dist_u32x2_min", |b| b.iter(|| manhattan_dist_u32x2(black_box(0), black_box(0))));
    c.bench_function("manhattan_dist_u32x2_max", |b| b.iter(|| manhattan_dist_u32x2(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::euclidean_dist_sq_u32x2::euclidean_dist_sq_u32x2;
    c.bench_function("euclidean_dist_sq_u32x2_avg", |b| b.iter(|| euclidean_dist_sq_u32x2(black_box(42), black_box(1337))));
    c.bench_function("euclidean_dist_sq_u32x2_min", |b| b.iter(|| euclidean_dist_sq_u32x2(black_box(0), black_box(0))));
    c.bench_function("euclidean_dist_sq_u32x2_max", |b| b.iter(|| euclidean_dist_sq_u32x2(black_box(u64::MAX), black_box(u64::MAX))));
}

criterion_group!(benches, algorithms_1_100);
criterion_main!(benches);
