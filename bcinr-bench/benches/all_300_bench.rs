use bcinr_logic::algorithms::*;
use criterion::{criterion_group, criterion_main, Criterion, black_box};

fn bench_parallel_bits_deposit_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::parallel_bits_deposit_u64::parallel_bits_deposit_u64;
    c.bench_function("parallel_bits_deposit_u64_avg", |b| b.iter(|| parallel_bits_deposit_u64(black_box(42), black_box(1337))));
    c.bench_function("parallel_bits_deposit_u64_min", |b| b.iter(|| parallel_bits_deposit_u64(black_box(0), black_box(0))));
    c.bench_function("parallel_bits_deposit_u64_max", |b| b.iter(|| parallel_bits_deposit_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_parallel_bits_extract_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::parallel_bits_extract_u64::parallel_bits_extract_u64;
    c.bench_function("parallel_bits_extract_u64_avg", |b| b.iter(|| parallel_bits_extract_u64(black_box(42), black_box(1337))));
    c.bench_function("parallel_bits_extract_u64_min", |b| b.iter(|| parallel_bits_extract_u64(black_box(0), black_box(0))));
    c.bench_function("parallel_bits_extract_u64_max", |b| b.iter(|| parallel_bits_extract_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_blsr_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::blsr_u64::blsr_u64;
    c.bench_function("blsr_u64_avg", |b| b.iter(|| blsr_u64(black_box(42), black_box(1337))));
    c.bench_function("blsr_u64_min", |b| b.iter(|| blsr_u64(black_box(0), black_box(0))));
    c.bench_function("blsr_u64_max", |b| b.iter(|| blsr_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_blsi_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::blsi_u64::blsi_u64;
    c.bench_function("blsi_u64_avg", |b| b.iter(|| blsi_u64(black_box(42), black_box(1337))));
    c.bench_function("blsi_u64_min", |b| b.iter(|| blsi_u64(black_box(0), black_box(0))));
    c.bench_function("blsi_u64_max", |b| b.iter(|| blsi_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_blsmsk_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::blsmsk_u64::blsmsk_u64;
    c.bench_function("blsmsk_u64_avg", |b| b.iter(|| blsmsk_u64(black_box(42), black_box(1337))));
    c.bench_function("blsmsk_u64_min", |b| b.iter(|| blsmsk_u64(black_box(0), black_box(0))));
    c.bench_function("blsmsk_u64_max", |b| b.iter(|| blsmsk_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_t1mskc_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::t1mskc_u64::t1mskc_u64;
    c.bench_function("t1mskc_u64_avg", |b| b.iter(|| t1mskc_u64(black_box(42), black_box(1337))));
    c.bench_function("t1mskc_u64_min", |b| b.iter(|| t1mskc_u64(black_box(0), black_box(0))));
    c.bench_function("t1mskc_u64_max", |b| b.iter(|| t1mskc_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_tzmsk_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::tzmsk_u64::tzmsk_u64;
    c.bench_function("tzmsk_u64_avg", |b| b.iter(|| tzmsk_u64(black_box(42), black_box(1337))));
    c.bench_function("tzmsk_u64_min", |b| b.iter(|| tzmsk_u64(black_box(0), black_box(0))));
    c.bench_function("tzmsk_u64_max", |b| b.iter(|| tzmsk_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_bext_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::bext_u64::bext_u64;
    c.bench_function("bext_u64_avg", |b| b.iter(|| bext_u64(black_box(42), black_box(1337))));
    c.bench_function("bext_u64_min", |b| b.iter(|| bext_u64(black_box(0), black_box(0))));
    c.bench_function("bext_u64_max", |b| b.iter(|| bext_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_bset_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::bset_u64::bset_u64;
    c.bench_function("bset_u64_avg", |b| b.iter(|| bset_u64(black_box(42), black_box(1337))));
    c.bench_function("bset_u64_min", |b| b.iter(|| bset_u64(black_box(0), black_box(0))));
    c.bench_function("bset_u64_max", |b| b.iter(|| bset_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_bclr_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::bclr_u64::bclr_u64;
    c.bench_function("bclr_u64_avg", |b| b.iter(|| bclr_u64(black_box(42), black_box(1337))));
    c.bench_function("bclr_u64_min", |b| b.iter(|| bclr_u64(black_box(0), black_box(0))));
    c.bench_function("bclr_u64_max", |b| b.iter(|| bclr_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_btst_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::btst_u64::btst_u64;
    c.bench_function("btst_u64_avg", |b| b.iter(|| btst_u64(black_box(42), black_box(1337))));
    c.bench_function("btst_u64_min", |b| b.iter(|| btst_u64(black_box(0), black_box(0))));
    c.bench_function("btst_u64_max", |b| b.iter(|| btst_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_popcount_u128(c: &mut Criterion) {
    use bcinr_logic::algorithms::popcount_u128::popcount_u128;
    c.bench_function("popcount_u128_avg", |b| b.iter(|| popcount_u128(black_box(42), black_box(1337))));
    c.bench_function("popcount_u128_min", |b| b.iter(|| popcount_u128(black_box(0), black_box(0))));
    c.bench_function("popcount_u128_max", |b| b.iter(|| popcount_u128(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_reverse_bits_u128(c: &mut Criterion) {
    use bcinr_logic::algorithms::reverse_bits_u128::reverse_bits_u128;
    c.bench_function("reverse_bits_u128_avg", |b| b.iter(|| reverse_bits_u128(black_box(42), black_box(1337))));
    c.bench_function("reverse_bits_u128_min", |b| b.iter(|| reverse_bits_u128(black_box(0), black_box(0))));
    c.bench_function("reverse_bits_u128_max", |b| b.iter(|| reverse_bits_u128(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_clmul_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::clmul_u64::clmul_u64;
    c.bench_function("clmul_u64_avg", |b| b.iter(|| clmul_u64(black_box(42), black_box(1337))));
    c.bench_function("clmul_u64_min", |b| b.iter(|| clmul_u64(black_box(0), black_box(0))));
    c.bench_function("clmul_u64_max", |b| b.iter(|| clmul_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_morton_encode_2d_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::morton_encode_2d_u32::morton_encode_2d_u32;
    c.bench_function("morton_encode_2d_u32_avg", |b| b.iter(|| morton_encode_2d_u32(black_box(42), black_box(1337))));
    c.bench_function("morton_encode_2d_u32_min", |b| b.iter(|| morton_encode_2d_u32(black_box(0), black_box(0))));
    c.bench_function("morton_encode_2d_u32_max", |b| b.iter(|| morton_encode_2d_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_morton_decode_2d_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::morton_decode_2d_u32::morton_decode_2d_u32;
    c.bench_function("morton_decode_2d_u32_avg", |b| b.iter(|| morton_decode_2d_u32(black_box(42), black_box(1337))));
    c.bench_function("morton_decode_2d_u32_min", |b| b.iter(|| morton_decode_2d_u32(black_box(0), black_box(0))));
    c.bench_function("morton_decode_2d_u32_max", |b| b.iter(|| morton_decode_2d_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_morton_encode_3d_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::morton_encode_3d_u32::morton_encode_3d_u32;
    c.bench_function("morton_encode_3d_u32_avg", |b| b.iter(|| morton_encode_3d_u32(black_box(42), black_box(1337))));
    c.bench_function("morton_encode_3d_u32_min", |b| b.iter(|| morton_encode_3d_u32(black_box(0), black_box(0))));
    c.bench_function("morton_encode_3d_u32_max", |b| b.iter(|| morton_encode_3d_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_gray_encode_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::gray_encode_u64::gray_encode_u64;
    c.bench_function("gray_encode_u64_avg", |b| b.iter(|| gray_encode_u64(black_box(42), black_box(1337))));
    c.bench_function("gray_encode_u64_min", |b| b.iter(|| gray_encode_u64(black_box(0), black_box(0))));
    c.bench_function("gray_encode_u64_max", |b| b.iter(|| gray_encode_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_gray_decode_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::gray_decode_u64::gray_decode_u64;
    c.bench_function("gray_decode_u64_avg", |b| b.iter(|| gray_decode_u64(black_box(42), black_box(1337))));
    c.bench_function("gray_decode_u64_min", |b| b.iter(|| gray_decode_u64(black_box(0), black_box(0))));
    c.bench_function("gray_decode_u64_max", |b| b.iter(|| gray_decode_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_parity_check_u128(c: &mut Criterion) {
    use bcinr_logic::algorithms::parity_check_u128::parity_check_u128;
    c.bench_function("parity_check_u128_avg", |b| b.iter(|| parity_check_u128(black_box(42), black_box(1337))));
    c.bench_function("parity_check_u128_min", |b| b.iter(|| parity_check_u128(black_box(0), black_box(0))));
    c.bench_function("parity_check_u128_max", |b| b.iter(|| parity_check_u128(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_next_lexicographic_permutation_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::next_lexicographic_permutation_u64::next_lexicographic_permutation_u64;
    c.bench_function("next_lexicographic_permutation_u64_avg", |b| b.iter(|| next_lexicographic_permutation_u64(black_box(42), black_box(1337))));
    c.bench_function("next_lexicographic_permutation_u64_min", |b| b.iter(|| next_lexicographic_permutation_u64(black_box(0), black_box(0))));
    c.bench_function("next_lexicographic_permutation_u64_max", |b| b.iter(|| next_lexicographic_permutation_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_count_consecutive_set_bits_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::count_consecutive_set_bits_u64::count_consecutive_set_bits_u64;
    c.bench_function("count_consecutive_set_bits_u64_avg", |b| b.iter(|| count_consecutive_set_bits_u64(black_box(42), black_box(1337))));
    c.bench_function("count_consecutive_set_bits_u64_min", |b| b.iter(|| count_consecutive_set_bits_u64(black_box(0), black_box(0))));
    c.bench_function("count_consecutive_set_bits_u64_max", |b| b.iter(|| count_consecutive_set_bits_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_find_nth_set_bit_u128(c: &mut Criterion) {
    use bcinr_logic::algorithms::find_nth_set_bit_u128::find_nth_set_bit_u128;
    c.bench_function("find_nth_set_bit_u128_avg", |b| b.iter(|| find_nth_set_bit_u128(black_box(42), black_box(1337))));
    c.bench_function("find_nth_set_bit_u128_min", |b| b.iter(|| find_nth_set_bit_u128(black_box(0), black_box(0))));
    c.bench_function("find_nth_set_bit_u128_max", |b| b.iter(|| find_nth_set_bit_u128(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_mask_range_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::mask_range_u64::mask_range_u64;
    c.bench_function("mask_range_u64_avg", |b| b.iter(|| mask_range_u64(black_box(42), black_box(1337))));
    c.bench_function("mask_range_u64_min", |b| b.iter(|| mask_range_u64(black_box(0), black_box(0))));
    c.bench_function("mask_range_u64_max", |b| b.iter(|| mask_range_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_rotate_left_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::rotate_left_u64::rotate_left_u64;
    c.bench_function("rotate_left_u64_avg", |b| b.iter(|| rotate_left_u64(black_box(42), black_box(1337))));
    c.bench_function("rotate_left_u64_min", |b| b.iter(|| rotate_left_u64(black_box(0), black_box(0))));
    c.bench_function("rotate_left_u64_max", |b| b.iter(|| rotate_left_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_rotate_right_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::rotate_right_u64::rotate_right_u64;
    c.bench_function("rotate_right_u64_avg", |b| b.iter(|| rotate_right_u64(black_box(42), black_box(1337))));
    c.bench_function("rotate_right_u64_min", |b| b.iter(|| rotate_right_u64(black_box(0), black_box(0))));
    c.bench_function("rotate_right_u64_max", |b| b.iter(|| rotate_right_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_funnel_shift_left_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::funnel_shift_left_u64::funnel_shift_left_u64;
    c.bench_function("funnel_shift_left_u64_avg", |b| b.iter(|| funnel_shift_left_u64(black_box(42), black_box(1337))));
    c.bench_function("funnel_shift_left_u64_min", |b| b.iter(|| funnel_shift_left_u64(black_box(0), black_box(0))));
    c.bench_function("funnel_shift_left_u64_max", |b| b.iter(|| funnel_shift_left_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_funnel_shift_right_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::funnel_shift_right_u64::funnel_shift_right_u64;
    c.bench_function("funnel_shift_right_u64_avg", |b| b.iter(|| funnel_shift_right_u64(black_box(42), black_box(1337))));
    c.bench_function("funnel_shift_right_u64_min", |b| b.iter(|| funnel_shift_right_u64(black_box(0), black_box(0))));
    c.bench_function("funnel_shift_right_u64_max", |b| b.iter(|| funnel_shift_right_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_bit_swap_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::bit_swap_u64::bit_swap_u64;
    c.bench_function("bit_swap_u64_avg", |b| b.iter(|| bit_swap_u64(black_box(42), black_box(1337))));
    c.bench_function("bit_swap_u64_min", |b| b.iter(|| bit_swap_u64(black_box(0), black_box(0))));
    c.bench_function("bit_swap_u64_max", |b| b.iter(|| bit_swap_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_gather_bits_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::gather_bits_u64::gather_bits_u64;
    c.bench_function("gather_bits_u64_avg", |b| b.iter(|| gather_bits_u64(black_box(42), black_box(1337))));
    c.bench_function("gather_bits_u64_min", |b| b.iter(|| gather_bits_u64(black_box(0), black_box(0))));
    c.bench_function("gather_bits_u64_max", |b| b.iter(|| gather_bits_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_scatter_bits_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::scatter_bits_u64::scatter_bits_u64;
    c.bench_function("scatter_bits_u64_avg", |b| b.iter(|| scatter_bits_u64(black_box(42), black_box(1337))));
    c.bench_function("scatter_bits_u64_min", |b| b.iter(|| scatter_bits_u64(black_box(0), black_box(0))));
    c.bench_function("scatter_bits_u64_max", |b| b.iter(|| scatter_bits_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_is_contiguous_mask_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::is_contiguous_mask_u64::is_contiguous_mask_u64;
    c.bench_function("is_contiguous_mask_u64_avg", |b| b.iter(|| is_contiguous_mask_u64(black_box(42), black_box(1337))));
    c.bench_function("is_contiguous_mask_u64_min", |b| b.iter(|| is_contiguous_mask_u64(black_box(0), black_box(0))));
    c.bench_function("is_contiguous_mask_u64_max", |b| b.iter(|| is_contiguous_mask_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_get_mask_boundary_low_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::get_mask_boundary_low_u64::get_mask_boundary_low_u64;
    c.bench_function("get_mask_boundary_low_u64_avg", |b| b.iter(|| get_mask_boundary_low_u64(black_box(42), black_box(1337))));
    c.bench_function("get_mask_boundary_low_u64_min", |b| b.iter(|| get_mask_boundary_low_u64(black_box(0), black_box(0))));
    c.bench_function("get_mask_boundary_low_u64_max", |b| b.iter(|| get_mask_boundary_low_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_get_mask_boundary_high_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::get_mask_boundary_high_u64::get_mask_boundary_high_u64;
    c.bench_function("get_mask_boundary_high_u64_avg", |b| b.iter(|| get_mask_boundary_high_u64(black_box(42), black_box(1337))));
    c.bench_function("get_mask_boundary_high_u64_min", |b| b.iter(|| get_mask_boundary_high_u64(black_box(0), black_box(0))));
    c.bench_function("get_mask_boundary_high_u64_max", |b| b.iter(|| get_mask_boundary_high_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_bit_matrix_transpose_8x8(c: &mut Criterion) {
    use bcinr_logic::algorithms::bit_matrix_transpose_8x8::bit_matrix_transpose_8x8;
    c.bench_function("bit_matrix_transpose_8x8_avg", |b| b.iter(|| bit_matrix_transpose_8x8(black_box(42), black_box(1337))));
    c.bench_function("bit_matrix_transpose_8x8_min", |b| b.iter(|| bit_matrix_transpose_8x8(black_box(0), black_box(0))));
    c.bench_function("bit_matrix_transpose_8x8_max", |b| b.iter(|| bit_matrix_transpose_8x8(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_bit_matrix_transpose_64x64(c: &mut Criterion) {
    use bcinr_logic::algorithms::bit_matrix_transpose_64x64::bit_matrix_transpose_64x64;
    c.bench_function("bit_matrix_transpose_64x64_avg", |b| b.iter(|| bit_matrix_transpose_64x64(black_box(42), black_box(1337))));
    c.bench_function("bit_matrix_transpose_64x64_min", |b| b.iter(|| bit_matrix_transpose_64x64(black_box(0), black_box(0))));
    c.bench_function("bit_matrix_transpose_64x64_max", |b| b.iter(|| bit_matrix_transpose_64x64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_rank_u128(c: &mut Criterion) {
    use bcinr_logic::algorithms::rank_u128::rank_u128;
    c.bench_function("rank_u128_avg", |b| b.iter(|| rank_u128(black_box(42), black_box(1337))));
    c.bench_function("rank_u128_min", |b| b.iter(|| rank_u128(black_box(0), black_box(0))));
    c.bench_function("rank_u128_max", |b| b.iter(|| rank_u128(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_select_u128(c: &mut Criterion) {
    use bcinr_logic::algorithms::select_u128::select_u128;
    c.bench_function("select_u128_avg", |b| b.iter(|| select_u128(black_box(42), black_box(1337))));
    c.bench_function("select_u128_min", |b| b.iter(|| select_u128(black_box(0), black_box(0))));
    c.bench_function("select_u128_max", |b| b.iter(|| select_u128(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_weight_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::weight_u64::weight_u64;
    c.bench_function("weight_u64_avg", |b| b.iter(|| weight_u64(black_box(42), black_box(1337))));
    c.bench_function("weight_u64_min", |b| b.iter(|| weight_u64(black_box(0), black_box(0))));
    c.bench_function("weight_u64_max", |b| b.iter(|| weight_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_delta_swap_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::delta_swap_u64::delta_swap_u64;
    c.bench_function("delta_swap_u64_avg", |b| b.iter(|| delta_swap_u64(black_box(42), black_box(1337))));
    c.bench_function("delta_swap_u64_min", |b| b.iter(|| delta_swap_u64(black_box(0), black_box(0))));
    c.bench_function("delta_swap_u64_max", |b| b.iter(|| delta_swap_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_benes_network_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::benes_network_u64::benes_network_u64;
    c.bench_function("benes_network_u64_avg", |b| b.iter(|| benes_network_u64(black_box(42), black_box(1337))));
    c.bench_function("benes_network_u64_min", |b| b.iter(|| benes_network_u64(black_box(0), black_box(0))));
    c.bench_function("benes_network_u64_max", |b| b.iter(|| benes_network_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_bit_permute_step_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::bit_permute_step_u64::bit_permute_step_u64;
    c.bench_function("bit_permute_step_u64_avg", |b| b.iter(|| bit_permute_step_u64(black_box(42), black_box(1337))));
    c.bench_function("bit_permute_step_u64_min", |b| b.iter(|| bit_permute_step_u64(black_box(0), black_box(0))));
    c.bench_function("bit_permute_step_u64_max", |b| b.iter(|| bit_permute_step_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_compress_bits_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::compress_bits_u64::compress_bits_u64;
    c.bench_function("compress_bits_u64_avg", |b| b.iter(|| compress_bits_u64(black_box(42), black_box(1337))));
    c.bench_function("compress_bits_u64_min", |b| b.iter(|| compress_bits_u64(black_box(0), black_box(0))));
    c.bench_function("compress_bits_u64_max", |b| b.iter(|| compress_bits_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_expand_bits_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::expand_bits_u64::expand_bits_u64;
    c.bench_function("expand_bits_u64_avg", |b| b.iter(|| expand_bits_u64(black_box(42), black_box(1337))));
    c.bench_function("expand_bits_u64_min", |b| b.iter(|| expand_bits_u64(black_box(0), black_box(0))));
    c.bench_function("expand_bits_u64_max", |b| b.iter(|| expand_bits_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_crossbar_permute_u8x16(c: &mut Criterion) {
    use bcinr_logic::algorithms::crossbar_permute_u8x16::crossbar_permute_u8x16;
    c.bench_function("crossbar_permute_u8x16_avg", |b| b.iter(|| crossbar_permute_u8x16(black_box(42), black_box(1337))));
    c.bench_function("crossbar_permute_u8x16_min", |b| b.iter(|| crossbar_permute_u8x16(black_box(0), black_box(0))));
    c.bench_function("crossbar_permute_u8x16_max", |b| b.iter(|| crossbar_permute_u8x16(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_mask_from_bool_slice(c: &mut Criterion) {
    use bcinr_logic::algorithms::mask_from_bool_slice::mask_from_bool_slice;
    c.bench_function("mask_from_bool_slice_avg", |b| b.iter(|| mask_from_bool_slice(black_box(42), black_box(1337))));
    c.bench_function("mask_from_bool_slice_min", |b| b.iter(|| mask_from_bool_slice(black_box(0), black_box(0))));
    c.bench_function("mask_from_bool_slice_max", |b| b.iter(|| mask_from_bool_slice(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_bool_slice_from_mask(c: &mut Criterion) {
    use bcinr_logic::algorithms::bool_slice_from_mask::bool_slice_from_mask;
    c.bench_function("bool_slice_from_mask_avg", |b| b.iter(|| bool_slice_from_mask(black_box(42), black_box(1337))));
    c.bench_function("bool_slice_from_mask_min", |b| b.iter(|| bool_slice_from_mask(black_box(0), black_box(0))));
    c.bench_function("bool_slice_from_mask_max", |b| b.iter(|| bool_slice_from_mask(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_bit_permute_identity_64(c: &mut Criterion) {
    use bcinr_logic::algorithms::bit_permute_identity_64::bit_permute_identity_64;
    c.bench_function("bit_permute_identity_64_avg", |b| b.iter(|| bit_permute_identity_64(black_box(42), black_box(1337))));
    c.bench_function("bit_permute_identity_64_min", |b| b.iter(|| bit_permute_identity_64(black_box(0), black_box(0))));
    c.bench_function("bit_permute_identity_64_max", |b| b.iter(|| bit_permute_identity_64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_is_subset_mask_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::is_subset_mask_u64::is_subset_mask_u64;
    c.bench_function("is_subset_mask_u64_avg", |b| b.iter(|| is_subset_mask_u64(black_box(42), black_box(1337))));
    c.bench_function("is_subset_mask_u64_min", |b| b.iter(|| is_subset_mask_u64(black_box(0), black_box(0))));
    c.bench_function("is_subset_mask_u64_max", |b| b.iter(|| is_subset_mask_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_mask_xor_reduce_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::mask_xor_reduce_u64::mask_xor_reduce_u64;
    c.bench_function("mask_xor_reduce_u64_avg", |b| b.iter(|| mask_xor_reduce_u64(black_box(42), black_box(1337))));
    c.bench_function("mask_xor_reduce_u64_min", |b| b.iter(|| mask_xor_reduce_u64(black_box(0), black_box(0))));
    c.bench_function("mask_xor_reduce_u64_max", |b| b.iter(|| mask_xor_reduce_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_mul_sat_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::mul_sat_u64::mul_sat_u64;
    c.bench_function("mul_sat_u64_avg", |b| b.iter(|| mul_sat_u64(black_box(42), black_box(1337))));
    c.bench_function("mul_sat_u64_min", |b| b.iter(|| mul_sat_u64(black_box(0), black_box(0))));
    c.bench_function("mul_sat_u64_max", |b| b.iter(|| mul_sat_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_div_sat_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::div_sat_u64::div_sat_u64;
    c.bench_function("div_sat_u64_avg", |b| b.iter(|| div_sat_u64(black_box(42), black_box(1337))));
    c.bench_function("div_sat_u64_min", |b| b.iter(|| div_sat_u64(black_box(0), black_box(0))));
    c.bench_function("div_sat_u64_max", |b| b.iter(|| div_sat_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_add_sat_i32(c: &mut Criterion) {
    use bcinr_logic::algorithms::add_sat_i32::add_sat_i32;
    c.bench_function("add_sat_i32_avg", |b| b.iter(|| add_sat_i32(black_box(42), black_box(1337))));
    c.bench_function("add_sat_i32_min", |b| b.iter(|| add_sat_i32(black_box(0), black_box(0))));
    c.bench_function("add_sat_i32_max", |b| b.iter(|| add_sat_i32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_sub_sat_i32(c: &mut Criterion) {
    use bcinr_logic::algorithms::sub_sat_i32::sub_sat_i32;
    c.bench_function("sub_sat_i32_avg", |b| b.iter(|| sub_sat_i32(black_box(42), black_box(1337))));
    c.bench_function("sub_sat_i32_min", |b| b.iter(|| sub_sat_i32(black_box(0), black_box(0))));
    c.bench_function("sub_sat_i32_max", |b| b.iter(|| sub_sat_i32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_mul_sat_i32(c: &mut Criterion) {
    use bcinr_logic::algorithms::mul_sat_i32::mul_sat_i32;
    c.bench_function("mul_sat_i32_avg", |b| b.iter(|| mul_sat_i32(black_box(42), black_box(1337))));
    c.bench_function("mul_sat_i32_min", |b| b.iter(|| mul_sat_i32(black_box(0), black_box(0))));
    c.bench_function("mul_sat_i32_max", |b| b.iter(|| mul_sat_i32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_abs_diff_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::abs_diff_u64::abs_diff_u64;
    c.bench_function("abs_diff_u64_avg", |b| b.iter(|| abs_diff_u64(black_box(42), black_box(1337))));
    c.bench_function("abs_diff_u64_min", |b| b.iter(|| abs_diff_u64(black_box(0), black_box(0))));
    c.bench_function("abs_diff_u64_max", |b| b.iter(|| abs_diff_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_abs_diff_i64(c: &mut Criterion) {
    use bcinr_logic::algorithms::abs_diff_i64::abs_diff_i64;
    c.bench_function("abs_diff_i64_avg", |b| b.iter(|| abs_diff_i64(black_box(42), black_box(1337))));
    c.bench_function("abs_diff_i64_min", |b| b.iter(|| abs_diff_i64(black_box(0), black_box(0))));
    c.bench_function("abs_diff_i64_max", |b| b.iter(|| abs_diff_i64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_avg_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::avg_u64::avg_u64;
    c.bench_function("avg_u64_avg", |b| b.iter(|| avg_u64(black_box(42), black_box(1337))));
    c.bench_function("avg_u64_min", |b| b.iter(|| avg_u64(black_box(0), black_box(0))));
    c.bench_function("avg_u64_max", |b| b.iter(|| avg_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_avg_ceil_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::avg_ceil_u64::avg_ceil_u64;
    c.bench_function("avg_ceil_u64_avg", |b| b.iter(|| avg_ceil_u64(black_box(42), black_box(1337))));
    c.bench_function("avg_ceil_u64_min", |b| b.iter(|| avg_ceil_u64(black_box(0), black_box(0))));
    c.bench_function("avg_ceil_u64_max", |b| b.iter(|| avg_ceil_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_clamp_i64(c: &mut Criterion) {
    use bcinr_logic::algorithms::clamp_i64::clamp_i64;
    c.bench_function("clamp_i64_avg", |b| b.iter(|| clamp_i64(black_box(42), black_box(1337))));
    c.bench_function("clamp_i64_min", |b| b.iter(|| clamp_i64(black_box(0), black_box(0))));
    c.bench_function("clamp_i64_max", |b| b.iter(|| clamp_i64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_lerp_sat_u8(c: &mut Criterion) {
    use bcinr_logic::algorithms::lerp_sat_u8::lerp_sat_u8;
    c.bench_function("lerp_sat_u8_avg", |b| b.iter(|| lerp_sat_u8(black_box(42), black_box(1337))));
    c.bench_function("lerp_sat_u8_min", |b| b.iter(|| lerp_sat_u8(black_box(0), black_box(0))));
    c.bench_function("lerp_sat_u8_max", |b| b.iter(|| lerp_sat_u8(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_lerp_sat_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::lerp_sat_u32::lerp_sat_u32;
    c.bench_function("lerp_sat_u32_avg", |b| b.iter(|| lerp_sat_u32(black_box(42), black_box(1337))));
    c.bench_function("lerp_sat_u32_min", |b| b.iter(|| lerp_sat_u32(black_box(0), black_box(0))));
    c.bench_function("lerp_sat_u32_max", |b| b.iter(|| lerp_sat_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_norm_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::norm_u32::norm_u32;
    c.bench_function("norm_u32_avg", |b| b.iter(|| norm_u32(black_box(42), black_box(1337))));
    c.bench_function("norm_u32_min", |b| b.iter(|| norm_u32(black_box(0), black_box(0))));
    c.bench_function("norm_u32_max", |b| b.iter(|| norm_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_fp_mul_u32_q16(c: &mut Criterion) {
    use bcinr_logic::algorithms::fp_mul_u32_q16::fp_mul_u32_q16;
    c.bench_function("fp_mul_u32_q16_avg", |b| b.iter(|| fp_mul_u32_q16(black_box(42), black_box(1337))));
    c.bench_function("fp_mul_u32_q16_min", |b| b.iter(|| fp_mul_u32_q16(black_box(0), black_box(0))));
    c.bench_function("fp_mul_u32_q16_max", |b| b.iter(|| fp_mul_u32_q16(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_fp_div_u32_q16(c: &mut Criterion) {
    use bcinr_logic::algorithms::fp_div_u32_q16::fp_div_u32_q16;
    c.bench_function("fp_div_u32_q16_avg", |b| b.iter(|| fp_div_u32_q16(black_box(42), black_box(1337))));
    c.bench_function("fp_div_u32_q16_min", |b| b.iter(|| fp_div_u32_q16(black_box(0), black_box(0))));
    c.bench_function("fp_div_u32_q16_max", |b| b.iter(|| fp_div_u32_q16(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_fp_sqrt_u32_q16(c: &mut Criterion) {
    use bcinr_logic::algorithms::fp_sqrt_u32_q16::fp_sqrt_u32_q16;
    c.bench_function("fp_sqrt_u32_q16_avg", |b| b.iter(|| fp_sqrt_u32_q16(black_box(42), black_box(1337))));
    c.bench_function("fp_sqrt_u32_q16_min", |b| b.iter(|| fp_sqrt_u32_q16(black_box(0), black_box(0))));
    c.bench_function("fp_sqrt_u32_q16_max", |b| b.iter(|| fp_sqrt_u32_q16(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_fp_sin_u32_q16(c: &mut Criterion) {
    use bcinr_logic::algorithms::fp_sin_u32_q16::fp_sin_u32_q16;
    c.bench_function("fp_sin_u32_q16_avg", |b| b.iter(|| fp_sin_u32_q16(black_box(42), black_box(1337))));
    c.bench_function("fp_sin_u32_q16_min", |b| b.iter(|| fp_sin_u32_q16(black_box(0), black_box(0))));
    c.bench_function("fp_sin_u32_q16_max", |b| b.iter(|| fp_sin_u32_q16(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_fp_cos_u32_q16(c: &mut Criterion) {
    use bcinr_logic::algorithms::fp_cos_u32_q16::fp_cos_u32_q16;
    c.bench_function("fp_cos_u32_q16_avg", |b| b.iter(|| fp_cos_u32_q16(black_box(42), black_box(1337))));
    c.bench_function("fp_cos_u32_q16_min", |b| b.iter(|| fp_cos_u32_q16(black_box(0), black_box(0))));
    c.bench_function("fp_cos_u32_q16_max", |b| b.iter(|| fp_cos_u32_q16(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_fp_atan2_u32_q16(c: &mut Criterion) {
    use bcinr_logic::algorithms::fp_atan2_u32_q16::fp_atan2_u32_q16;
    c.bench_function("fp_atan2_u32_q16_avg", |b| b.iter(|| fp_atan2_u32_q16(black_box(42), black_box(1337))));
    c.bench_function("fp_atan2_u32_q16_min", |b| b.iter(|| fp_atan2_u32_q16(black_box(0), black_box(0))));
    c.bench_function("fp_atan2_u32_q16_max", |b| b.iter(|| fp_atan2_u32_q16(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_log2_u64_fixed(c: &mut Criterion) {
    use bcinr_logic::algorithms::log2_u64_fixed::log2_u64_fixed;
    c.bench_function("log2_u64_fixed_avg", |b| b.iter(|| log2_u64_fixed(black_box(42), black_box(1337))));
    c.bench_function("log2_u64_fixed_min", |b| b.iter(|| log2_u64_fixed(black_box(0), black_box(0))));
    c.bench_function("log2_u64_fixed_max", |b| b.iter(|| log2_u64_fixed(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_exp2_u64_fixed(c: &mut Criterion) {
    use bcinr_logic::algorithms::exp2_u64_fixed::exp2_u64_fixed;
    c.bench_function("exp2_u64_fixed_avg", |b| b.iter(|| exp2_u64_fixed(black_box(42), black_box(1337))));
    c.bench_function("exp2_u64_fixed_min", |b| b.iter(|| exp2_u64_fixed(black_box(0), black_box(0))));
    c.bench_function("exp2_u64_fixed_max", |b| b.iter(|| exp2_u64_fixed(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_sigmoid_sat_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::sigmoid_sat_u32::sigmoid_sat_u32;
    c.bench_function("sigmoid_sat_u32_avg", |b| b.iter(|| sigmoid_sat_u32(black_box(42), black_box(1337))));
    c.bench_function("sigmoid_sat_u32_min", |b| b.iter(|| sigmoid_sat_u32(black_box(0), black_box(0))));
    c.bench_function("sigmoid_sat_u32_max", |b| b.iter(|| sigmoid_sat_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_relu_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::relu_u32::relu_u32;
    c.bench_function("relu_u32_avg", |b| b.iter(|| relu_u32(black_box(42), black_box(1337))));
    c.bench_function("relu_u32_min", |b| b.iter(|| relu_u32(black_box(0), black_box(0))));
    c.bench_function("relu_u32_max", |b| b.iter(|| relu_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_leaky_relu_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::leaky_relu_u32::leaky_relu_u32;
    c.bench_function("leaky_relu_u32_avg", |b| b.iter(|| leaky_relu_u32(black_box(42), black_box(1337))));
    c.bench_function("leaky_relu_u32_min", |b| b.iter(|| leaky_relu_u32(black_box(0), black_box(0))));
    c.bench_function("leaky_relu_u32_max", |b| b.iter(|| leaky_relu_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_softmax_u32x4(c: &mut Criterion) {
    use bcinr_logic::algorithms::softmax_u32x4::softmax_u32x4;
    c.bench_function("softmax_u32x4_avg", |b| b.iter(|| softmax_u32x4(black_box(42), black_box(1337))));
    c.bench_function("softmax_u32x4_min", |b| b.iter(|| softmax_u32x4(black_box(0), black_box(0))));
    c.bench_function("softmax_u32x4_max", |b| b.iter(|| softmax_u32x4(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_fast_inverse_sqrt_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::fast_inverse_sqrt_u32::fast_inverse_sqrt_u32;
    c.bench_function("fast_inverse_sqrt_u32_avg", |b| b.iter(|| fast_inverse_sqrt_u32(black_box(42), black_box(1337))));
    c.bench_function("fast_inverse_sqrt_u32_min", |b| b.iter(|| fast_inverse_sqrt_u32(black_box(0), black_box(0))));
    c.bench_function("fast_inverse_sqrt_u32_max", |b| b.iter(|| fast_inverse_sqrt_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_gcd_u64_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::gcd_u64_branchless::gcd_u64_branchless;
    c.bench_function("gcd_u64_branchless_avg", |b| b.iter(|| gcd_u64_branchless(black_box(42), black_box(1337))));
    c.bench_function("gcd_u64_branchless_min", |b| b.iter(|| gcd_u64_branchless(black_box(0), black_box(0))));
    c.bench_function("gcd_u64_branchless_max", |b| b.iter(|| gcd_u64_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_lcm_u64_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::lcm_u64_branchless::lcm_u64_branchless;
    c.bench_function("lcm_u64_branchless_avg", |b| b.iter(|| lcm_u64_branchless(black_box(42), black_box(1337))));
    c.bench_function("lcm_u64_branchless_min", |b| b.iter(|| lcm_u64_branchless(black_box(0), black_box(0))));
    c.bench_function("lcm_u64_branchless_max", |b| b.iter(|| lcm_u64_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_modular_add_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::modular_add_u64::modular_add_u64;
    c.bench_function("modular_add_u64_avg", |b| b.iter(|| modular_add_u64(black_box(42), black_box(1337))));
    c.bench_function("modular_add_u64_min", |b| b.iter(|| modular_add_u64(black_box(0), black_box(0))));
    c.bench_function("modular_add_u64_max", |b| b.iter(|| modular_add_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_modular_sub_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::modular_sub_u64::modular_sub_u64;
    c.bench_function("modular_sub_u64_avg", |b| b.iter(|| modular_sub_u64(black_box(42), black_box(1337))));
    c.bench_function("modular_sub_u64_min", |b| b.iter(|| modular_sub_u64(black_box(0), black_box(0))));
    c.bench_function("modular_sub_u64_max", |b| b.iter(|| modular_sub_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_modular_mul_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::modular_mul_u64::modular_mul_u64;
    c.bench_function("modular_mul_u64_avg", |b| b.iter(|| modular_mul_u64(black_box(42), black_box(1337))));
    c.bench_function("modular_mul_u64_min", |b| b.iter(|| modular_mul_u64(black_box(0), black_box(0))));
    c.bench_function("modular_mul_u64_max", |b| b.iter(|| modular_mul_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_is_prime_u64_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::is_prime_u64_branchless::is_prime_u64_branchless;
    c.bench_function("is_prime_u64_branchless_avg", |b| b.iter(|| is_prime_u64_branchless(black_box(42), black_box(1337))));
    c.bench_function("is_prime_u64_branchless_min", |b| b.iter(|| is_prime_u64_branchless(black_box(0), black_box(0))));
    c.bench_function("is_prime_u64_branchless_max", |b| b.iter(|| is_prime_u64_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_factorial_sat_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::factorial_sat_u32::factorial_sat_u32;
    c.bench_function("factorial_sat_u32_avg", |b| b.iter(|| factorial_sat_u32(black_box(42), black_box(1337))));
    c.bench_function("factorial_sat_u32_min", |b| b.iter(|| factorial_sat_u32(black_box(0), black_box(0))));
    c.bench_function("factorial_sat_u32_max", |b| b.iter(|| factorial_sat_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_binom_sat_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::binom_sat_u32::binom_sat_u32;
    c.bench_function("binom_sat_u32_avg", |b| b.iter(|| binom_sat_u32(black_box(42), black_box(1337))));
    c.bench_function("binom_sat_u32_min", |b| b.iter(|| binom_sat_u32(black_box(0), black_box(0))));
    c.bench_function("binom_sat_u32_max", |b| b.iter(|| binom_sat_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_pow_sat_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::pow_sat_u64::pow_sat_u64;
    c.bench_function("pow_sat_u64_avg", |b| b.iter(|| pow_sat_u64(black_box(42), black_box(1337))));
    c.bench_function("pow_sat_u64_min", |b| b.iter(|| pow_sat_u64(black_box(0), black_box(0))));
    c.bench_function("pow_sat_u64_max", |b| b.iter(|| pow_sat_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_clamped_scaling_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::clamped_scaling_u64::clamped_scaling_u64;
    c.bench_function("clamped_scaling_u64_avg", |b| b.iter(|| clamped_scaling_u64(black_box(42), black_box(1337))));
    c.bench_function("clamped_scaling_u64_min", |b| b.iter(|| clamped_scaling_u64(black_box(0), black_box(0))));
    c.bench_function("clamped_scaling_u64_max", |b| b.iter(|| clamped_scaling_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_branchless_signum_i64(c: &mut Criterion) {
    use bcinr_logic::algorithms::branchless_signum_i64::branchless_signum_i64;
    c.bench_function("branchless_signum_i64_avg", |b| b.iter(|| branchless_signum_i64(black_box(42), black_box(1337))));
    c.bench_function("branchless_signum_i64_min", |b| b.iter(|| branchless_signum_i64(black_box(0), black_box(0))));
    c.bench_function("branchless_signum_i64_max", |b| b.iter(|| branchless_signum_i64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_copy_sign_i64(c: &mut Criterion) {
    use bcinr_logic::algorithms::copy_sign_i64::copy_sign_i64;
    c.bench_function("copy_sign_i64_avg", |b| b.iter(|| copy_sign_i64(black_box(42), black_box(1337))));
    c.bench_function("copy_sign_i64_min", |b| b.iter(|| copy_sign_i64(black_box(0), black_box(0))));
    c.bench_function("copy_sign_i64_max", |b| b.iter(|| copy_sign_i64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_is_finite_fp32_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::is_finite_fp32_branchless::is_finite_fp32_branchless;
    c.bench_function("is_finite_fp32_branchless_avg", |b| b.iter(|| is_finite_fp32_branchless(black_box(42), black_box(1337))));
    c.bench_function("is_finite_fp32_branchless_min", |b| b.iter(|| is_finite_fp32_branchless(black_box(0), black_box(0))));
    c.bench_function("is_finite_fp32_branchless_max", |b| b.iter(|| is_finite_fp32_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_is_nan_fp32_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::is_nan_fp32_branchless::is_nan_fp32_branchless;
    c.bench_function("is_nan_fp32_branchless_avg", |b| b.iter(|| is_nan_fp32_branchless(black_box(42), black_box(1337))));
    c.bench_function("is_nan_fp32_branchless_min", |b| b.iter(|| is_nan_fp32_branchless(black_box(0), black_box(0))));
    c.bench_function("is_nan_fp32_branchless_max", |b| b.iter(|| is_nan_fp32_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_round_to_nearest_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::round_to_nearest_u32::round_to_nearest_u32;
    c.bench_function("round_to_nearest_u32_avg", |b| b.iter(|| round_to_nearest_u32(black_box(42), black_box(1337))));
    c.bench_function("round_to_nearest_u32_min", |b| b.iter(|| round_to_nearest_u32(black_box(0), black_box(0))));
    c.bench_function("round_to_nearest_u32_max", |b| b.iter(|| round_to_nearest_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_round_up_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::round_up_u32::round_up_u32;
    c.bench_function("round_up_u32_avg", |b| b.iter(|| round_up_u32(black_box(42), black_box(1337))));
    c.bench_function("round_up_u32_min", |b| b.iter(|| round_up_u32(black_box(0), black_box(0))));
    c.bench_function("round_up_u32_max", |b| b.iter(|| round_up_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_round_down_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::round_down_u32::round_down_u32;
    c.bench_function("round_down_u32_avg", |b| b.iter(|| round_down_u32(black_box(42), black_box(1337))));
    c.bench_function("round_down_u32_min", |b| b.iter(|| round_down_u32(black_box(0), black_box(0))));
    c.bench_function("round_down_u32_max", |b| b.iter(|| round_down_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_quantize_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::quantize_u32::quantize_u32;
    c.bench_function("quantize_u32_avg", |b| b.iter(|| quantize_u32(black_box(42), black_box(1337))));
    c.bench_function("quantize_u32_min", |b| b.iter(|| quantize_u32(black_box(0), black_box(0))));
    c.bench_function("quantize_u32_max", |b| b.iter(|| quantize_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_dequantize_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::dequantize_u32::dequantize_u32;
    c.bench_function("dequantize_u32_avg", |b| b.iter(|| dequantize_u32(black_box(42), black_box(1337))));
    c.bench_function("dequantize_u32_min", |b| b.iter(|| dequantize_u32(black_box(0), black_box(0))));
    c.bench_function("dequantize_u32_max", |b| b.iter(|| dequantize_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_weighted_avg_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::weighted_avg_u32::weighted_avg_u32;
    c.bench_function("weighted_avg_u32_avg", |b| b.iter(|| weighted_avg_u32(black_box(42), black_box(1337))));
    c.bench_function("weighted_avg_u32_min", |b| b.iter(|| weighted_avg_u32(black_box(0), black_box(0))));
    c.bench_function("weighted_avg_u32_max", |b| b.iter(|| weighted_avg_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_smoothstep_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::smoothstep_u32::smoothstep_u32;
    c.bench_function("smoothstep_u32_avg", |b| b.iter(|| smoothstep_u32(black_box(42), black_box(1337))));
    c.bench_function("smoothstep_u32_min", |b| b.iter(|| smoothstep_u32(black_box(0), black_box(0))));
    c.bench_function("smoothstep_u32_max", |b| b.iter(|| smoothstep_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_cubic_interpolate_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::cubic_interpolate_u32::cubic_interpolate_u32;
    c.bench_function("cubic_interpolate_u32_avg", |b| b.iter(|| cubic_interpolate_u32(black_box(42), black_box(1337))));
    c.bench_function("cubic_interpolate_u32_min", |b| b.iter(|| cubic_interpolate_u32(black_box(0), black_box(0))));
    c.bench_function("cubic_interpolate_u32_max", |b| b.iter(|| cubic_interpolate_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_manhattan_dist_u32x2(c: &mut Criterion) {
    use bcinr_logic::algorithms::manhattan_dist_u32x2::manhattan_dist_u32x2;
    c.bench_function("manhattan_dist_u32x2_avg", |b| b.iter(|| manhattan_dist_u32x2(black_box(42), black_box(1337))));
    c.bench_function("manhattan_dist_u32x2_min", |b| b.iter(|| manhattan_dist_u32x2(black_box(0), black_box(0))));
    c.bench_function("manhattan_dist_u32x2_max", |b| b.iter(|| manhattan_dist_u32x2(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_euclidean_dist_sq_u32x2(c: &mut Criterion) {
    use bcinr_logic::algorithms::euclidean_dist_sq_u32x2::euclidean_dist_sq_u32x2;
    c.bench_function("euclidean_dist_sq_u32x2_avg", |b| b.iter(|| euclidean_dist_sq_u32x2(black_box(42), black_box(1337))));
    c.bench_function("euclidean_dist_sq_u32x2_min", |b| b.iter(|| euclidean_dist_sq_u32x2(black_box(0), black_box(0))));
    c.bench_function("euclidean_dist_sq_u32x2_max", |b| b.iter(|| euclidean_dist_sq_u32x2(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_bitonic_sort_64u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::bitonic_sort_64u32::bitonic_sort_64u32;
    c.bench_function("bitonic_sort_64u32_avg", |b| b.iter(|| bitonic_sort_64u32(black_box(42), black_box(1337))));
    c.bench_function("bitonic_sort_64u32_min", |b| b.iter(|| bitonic_sort_64u32(black_box(0), black_box(0))));
    c.bench_function("bitonic_sort_64u32_max", |b| b.iter(|| bitonic_sort_64u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_odd_even_merge_sort_16u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::odd_even_merge_sort_16u32::odd_even_merge_sort_16u32;
    c.bench_function("odd_even_merge_sort_16u32_avg", |b| b.iter(|| odd_even_merge_sort_16u32(black_box(42), black_box(1337))));
    c.bench_function("odd_even_merge_sort_16u32_min", |b| b.iter(|| odd_even_merge_sort_16u32(black_box(0), black_box(0))));
    c.bench_function("odd_even_merge_sort_16u32_max", |b| b.iter(|| odd_even_merge_sort_16u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_halton_sequence_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::halton_sequence_u32::halton_sequence_u32;
    c.bench_function("halton_sequence_u32_avg", |b| b.iter(|| halton_sequence_u32(black_box(42), black_box(1337))));
    c.bench_function("halton_sequence_u32_min", |b| b.iter(|| halton_sequence_u32(black_box(0), black_box(0))));
    c.bench_function("halton_sequence_u32_max", |b| b.iter(|| halton_sequence_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_shuffle_fisher_yates_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::shuffle_fisher_yates_branchless::shuffle_fisher_yates_branchless;
    c.bench_function("shuffle_fisher_yates_branchless_avg", |b| b.iter(|| shuffle_fisher_yates_branchless(black_box(42), black_box(1337))));
    c.bench_function("shuffle_fisher_yates_branchless_min", |b| b.iter(|| shuffle_fisher_yates_branchless(black_box(0), black_box(0))));
    c.bench_function("shuffle_fisher_yates_branchless_max", |b| b.iter(|| shuffle_fisher_yates_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_bitonic_merge_u64x8(c: &mut Criterion) {
    use bcinr_logic::algorithms::bitonic_merge_u64x8::bitonic_merge_u64x8;
    c.bench_function("bitonic_merge_u64x8_avg", |b| b.iter(|| bitonic_merge_u64x8(black_box(42), black_box(1337))));
    c.bench_function("bitonic_merge_u64x8_min", |b| b.iter(|| bitonic_merge_u64x8(black_box(0), black_box(0))));
    c.bench_function("bitonic_merge_u64x8_max", |b| b.iter(|| bitonic_merge_u64x8(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_sort_pairs_u32x4(c: &mut Criterion) {
    use bcinr_logic::algorithms::sort_pairs_u32x4::sort_pairs_u32x4;
    c.bench_function("sort_pairs_u32x4_avg", |b| b.iter(|| sort_pairs_u32x4(black_box(42), black_box(1337))));
    c.bench_function("sort_pairs_u32x4_min", |b| b.iter(|| sort_pairs_u32x4(black_box(0), black_box(0))));
    c.bench_function("sort_pairs_u32x4_max", |b| b.iter(|| sort_pairs_u32x4(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_median3_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::median3_u32::median3_u32;
    c.bench_function("median3_u32_avg", |b| b.iter(|| median3_u32(black_box(42), black_box(1337))));
    c.bench_function("median3_u32_min", |b| b.iter(|| median3_u32(black_box(0), black_box(0))));
    c.bench_function("median3_u32_max", |b| b.iter(|| median3_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_median5_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::median5_u32::median5_u32;
    c.bench_function("median5_u32_avg", |b| b.iter(|| median5_u32(black_box(42), black_box(1337))));
    c.bench_function("median5_u32_min", |b| b.iter(|| median5_u32(black_box(0), black_box(0))));
    c.bench_function("median5_u32_max", |b| b.iter(|| median5_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_median9_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::median9_u32::median9_u32;
    c.bench_function("median9_u32_avg", |b| b.iter(|| median9_u32(black_box(42), black_box(1337))));
    c.bench_function("median9_u32_min", |b| b.iter(|| median9_u32(black_box(0), black_box(0))));
    c.bench_function("median9_u32_max", |b| b.iter(|| median9_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_top_k_u32x16(c: &mut Criterion) {
    use bcinr_logic::algorithms::top_k_u32x16::top_k_u32x16;
    c.bench_function("top_k_u32x16_avg", |b| b.iter(|| top_k_u32x16(black_box(42), black_box(1337))));
    c.bench_function("top_k_u32x16_min", |b| b.iter(|| top_k_u32x16(black_box(0), black_box(0))));
    c.bench_function("top_k_u32x16_max", |b| b.iter(|| top_k_u32x16(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_rank_select_sort_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::rank_select_sort_u32::rank_select_sort_u32;
    c.bench_function("rank_select_sort_u32_avg", |b| b.iter(|| rank_select_sort_u32(black_box(42), black_box(1337))));
    c.bench_function("rank_select_sort_u32_min", |b| b.iter(|| rank_select_sort_u32(black_box(0), black_box(0))));
    c.bench_function("rank_select_sort_u32_max", |b| b.iter(|| rank_select_sort_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_counting_sort_branchless_u8(c: &mut Criterion) {
    use bcinr_logic::algorithms::counting_sort_branchless_u8::counting_sort_branchless_u8;
    c.bench_function("counting_sort_branchless_u8_avg", |b| b.iter(|| counting_sort_branchless_u8(black_box(42), black_box(1337))));
    c.bench_function("counting_sort_branchless_u8_min", |b| b.iter(|| counting_sort_branchless_u8(black_box(0), black_box(0))));
    c.bench_function("counting_sort_branchless_u8_max", |b| b.iter(|| counting_sort_branchless_u8(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_radix_sort_step_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::radix_sort_step_branchless::radix_sort_step_branchless;
    c.bench_function("radix_sort_step_branchless_avg", |b| b.iter(|| radix_sort_step_branchless(black_box(42), black_box(1337))));
    c.bench_function("radix_sort_step_branchless_min", |b| b.iter(|| radix_sort_step_branchless(black_box(0), black_box(0))));
    c.bench_function("radix_sort_step_branchless_max", |b| b.iter(|| radix_sort_step_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_insertion_sort_branchless_fixed(c: &mut Criterion) {
    use bcinr_logic::algorithms::insertion_sort_branchless_fixed::insertion_sort_branchless_fixed;
    c.bench_function("insertion_sort_branchless_fixed_avg", |b| b.iter(|| insertion_sort_branchless_fixed(black_box(42), black_box(1337))));
    c.bench_function("insertion_sort_branchless_fixed_min", |b| b.iter(|| insertion_sort_branchless_fixed(black_box(0), black_box(0))));
    c.bench_function("insertion_sort_branchless_fixed_max", |b| b.iter(|| insertion_sort_branchless_fixed(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_shear_sort_bitonic_2d(c: &mut Criterion) {
    use bcinr_logic::algorithms::shear_sort_bitonic_2d::shear_sort_bitonic_2d;
    c.bench_function("shear_sort_bitonic_2d_avg", |b| b.iter(|| shear_sort_bitonic_2d(black_box(42), black_box(1337))));
    c.bench_function("shear_sort_bitonic_2d_min", |b| b.iter(|| shear_sort_bitonic_2d(black_box(0), black_box(0))));
    c.bench_function("shear_sort_bitonic_2d_max", |b| b.iter(|| shear_sort_bitonic_2d(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_green_sorting_network_16(c: &mut Criterion) {
    use bcinr_logic::algorithms::green_sorting_network_16::green_sorting_network_16;
    c.bench_function("green_sorting_network_16_avg", |b| b.iter(|| green_sorting_network_16(black_box(42), black_box(1337))));
    c.bench_function("green_sorting_network_16_min", |b| b.iter(|| green_sorting_network_16(black_box(0), black_box(0))));
    c.bench_function("green_sorting_network_16_max", |b| b.iter(|| green_sorting_network_16(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_permute_u32x8(c: &mut Criterion) {
    use bcinr_logic::algorithms::permute_u32x8::permute_u32x8;
    c.bench_function("permute_u32x8_avg", |b| b.iter(|| permute_u32x8(black_box(42), black_box(1337))));
    c.bench_function("permute_u32x8_min", |b| b.iter(|| permute_u32x8(black_box(0), black_box(0))));
    c.bench_function("permute_u32x8_max", |b| b.iter(|| permute_u32x8(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_inverse_permute_u32x8(c: &mut Criterion) {
    use bcinr_logic::algorithms::inverse_permute_u32x8::inverse_permute_u32x8;
    c.bench_function("inverse_permute_u32x8_avg", |b| b.iter(|| inverse_permute_u32x8(black_box(42), black_box(1337))));
    c.bench_function("inverse_permute_u32x8_min", |b| b.iter(|| inverse_permute_u32x8(black_box(0), black_box(0))));
    c.bench_function("inverse_permute_u32x8_max", |b| b.iter(|| inverse_permute_u32x8(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_is_sorted_branchless_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::is_sorted_branchless_u32::is_sorted_branchless_u32;
    c.bench_function("is_sorted_branchless_u32_avg", |b| b.iter(|| is_sorted_branchless_u32(black_box(42), black_box(1337))));
    c.bench_function("is_sorted_branchless_u32_min", |b| b.iter(|| is_sorted_branchless_u32(black_box(0), black_box(0))));
    c.bench_function("is_sorted_branchless_u32_max", |b| b.iter(|| is_sorted_branchless_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_lex_compare_u8_slices_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::lex_compare_u8_slices_branchless::lex_compare_u8_slices_branchless;
    c.bench_function("lex_compare_u8_slices_branchless_avg", |b| b.iter(|| lex_compare_u8_slices_branchless(black_box(42), black_box(1337))));
    c.bench_function("lex_compare_u8_slices_branchless_min", |b| b.iter(|| lex_compare_u8_slices_branchless(black_box(0), black_box(0))));
    c.bench_function("lex_compare_u8_slices_branchless_max", |b| b.iter(|| lex_compare_u8_slices_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_stable_partition_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::stable_partition_branchless::stable_partition_branchless;
    c.bench_function("stable_partition_branchless_avg", |b| b.iter(|| stable_partition_branchless(black_box(42), black_box(1337))));
    c.bench_function("stable_partition_branchless_min", |b| b.iter(|| stable_partition_branchless(black_box(0), black_box(0))));
    c.bench_function("stable_partition_branchless_max", |b| b.iter(|| stable_partition_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_rotate_slice_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::rotate_slice_branchless::rotate_slice_branchless;
    c.bench_function("rotate_slice_branchless_avg", |b| b.iter(|| rotate_slice_branchless(black_box(42), black_box(1337))));
    c.bench_function("rotate_slice_branchless_min", |b| b.iter(|| rotate_slice_branchless(black_box(0), black_box(0))));
    c.bench_function("rotate_slice_branchless_max", |b| b.iter(|| rotate_slice_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_reverse_slice_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::reverse_slice_branchless::reverse_slice_branchless;
    c.bench_function("reverse_slice_branchless_avg", |b| b.iter(|| reverse_slice_branchless(black_box(42), black_box(1337))));
    c.bench_function("reverse_slice_branchless_min", |b| b.iter(|| reverse_slice_branchless(black_box(0), black_box(0))));
    c.bench_function("reverse_slice_branchless_max", |b| b.iter(|| reverse_slice_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_next_combination_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::next_combination_u64::next_combination_u64;
    c.bench_function("next_combination_u64_avg", |b| b.iter(|| next_combination_u64(black_box(42), black_box(1337))));
    c.bench_function("next_combination_u64_min", |b| b.iter(|| next_combination_u64(black_box(0), black_box(0))));
    c.bench_function("next_combination_u64_max", |b| b.iter(|| next_combination_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_random_permutation_fixed_seed(c: &mut Criterion) {
    use bcinr_logic::algorithms::random_permutation_fixed_seed::random_permutation_fixed_seed;
    c.bench_function("random_permutation_fixed_seed_avg", |b| b.iter(|| random_permutation_fixed_seed(black_box(42), black_box(1337))));
    c.bench_function("random_permutation_fixed_seed_min", |b| b.iter(|| random_permutation_fixed_seed(black_box(0), black_box(0))));
    c.bench_function("random_permutation_fixed_seed_max", |b| b.iter(|| random_permutation_fixed_seed(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_sort_index_u32x8(c: &mut Criterion) {
    use bcinr_logic::algorithms::sort_index_u32x8::sort_index_u32x8;
    c.bench_function("sort_index_u32x8_avg", |b| b.iter(|| sort_index_u32x8(black_box(42), black_box(1337))));
    c.bench_function("sort_index_u32x8_min", |b| b.iter(|| sort_index_u32x8(black_box(0), black_box(0))));
    c.bench_function("sort_index_u32x8_max", |b| b.iter(|| sort_index_u32x8(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_merge_u32_slices_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::merge_u32_slices_branchless::merge_u32_slices_branchless;
    c.bench_function("merge_u32_slices_branchless_avg", |b| b.iter(|| merge_u32_slices_branchless(black_box(42), black_box(1337))));
    c.bench_function("merge_u32_slices_branchless_min", |b| b.iter(|| merge_u32_slices_branchless(black_box(0), black_box(0))));
    c.bench_function("merge_u32_slices_branchless_max", |b| b.iter(|| merge_u32_slices_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_unique_branchless_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::unique_branchless_u32::unique_branchless_u32;
    c.bench_function("unique_branchless_u32_avg", |b| b.iter(|| unique_branchless_u32(black_box(42), black_box(1337))));
    c.bench_function("unique_branchless_u32_min", |b| b.iter(|| unique_branchless_u32(black_box(0), black_box(0))));
    c.bench_function("unique_branchless_u32_max", |b| b.iter(|| unique_branchless_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_lower_bound_branchless_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::lower_bound_branchless_u32::lower_bound_branchless_u32;
    c.bench_function("lower_bound_branchless_u32_avg", |b| b.iter(|| lower_bound_branchless_u32(black_box(42), black_box(1337))));
    c.bench_function("lower_bound_branchless_u32_min", |b| b.iter(|| lower_bound_branchless_u32(black_box(0), black_box(0))));
    c.bench_function("lower_bound_branchless_u32_max", |b| b.iter(|| lower_bound_branchless_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_upper_bound_branchless_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::upper_bound_branchless_u32::upper_bound_branchless_u32;
    c.bench_function("upper_bound_branchless_u32_avg", |b| b.iter(|| upper_bound_branchless_u32(black_box(42), black_box(1337))));
    c.bench_function("upper_bound_branchless_u32_min", |b| b.iter(|| upper_bound_branchless_u32(black_box(0), black_box(0))));
    c.bench_function("upper_bound_branchless_u32_max", |b| b.iter(|| upper_bound_branchless_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_equal_range_branchless_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::equal_range_branchless_u32::equal_range_branchless_u32;
    c.bench_function("equal_range_branchless_u32_avg", |b| b.iter(|| equal_range_branchless_u32(black_box(42), black_box(1337))));
    c.bench_function("equal_range_branchless_u32_min", |b| b.iter(|| equal_range_branchless_u32(black_box(0), black_box(0))));
    c.bench_function("equal_range_branchless_u32_max", |b| b.iter(|| equal_range_branchless_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_search_eytzinger_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::search_eytzinger_u32::search_eytzinger_u32;
    c.bench_function("search_eytzinger_u32_avg", |b| b.iter(|| search_eytzinger_u32(black_box(42), black_box(1337))));
    c.bench_function("search_eytzinger_u32_min", |b| b.iter(|| search_eytzinger_u32(black_box(0), black_box(0))));
    c.bench_function("search_eytzinger_u32_max", |b| b.iter(|| search_eytzinger_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_search_van_emde_boas(c: &mut Criterion) {
    use bcinr_logic::algorithms::search_van_emde_boas::search_van_emde_boas;
    c.bench_function("search_van_emde_boas_avg", |b| b.iter(|| search_van_emde_boas(black_box(42), black_box(1337))));
    c.bench_function("search_van_emde_boas_min", |b| b.iter(|| search_van_emde_boas(black_box(0), black_box(0))));
    c.bench_function("search_van_emde_boas_max", |b| b.iter(|| search_van_emde_boas(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_binary_search_v_u32x4(c: &mut Criterion) {
    use bcinr_logic::algorithms::binary_search_v_u32x4::binary_search_v_u32x4;
    c.bench_function("binary_search_v_u32x4_avg", |b| b.iter(|| binary_search_v_u32x4(black_box(42), black_box(1337))));
    c.bench_function("binary_search_v_u32x4_min", |b| b.iter(|| binary_search_v_u32x4(black_box(0), black_box(0))));
    c.bench_function("binary_search_v_u32x4_max", |b| b.iter(|| binary_search_v_u32x4(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_linear_search_simd_u8(c: &mut Criterion) {
    use bcinr_logic::algorithms::linear_search_simd_u8::linear_search_simd_u8;
    c.bench_function("linear_search_simd_u8_avg", |b| b.iter(|| linear_search_simd_u8(black_box(42), black_box(1337))));
    c.bench_function("linear_search_simd_u8_min", |b| b.iter(|| linear_search_simd_u8(black_box(0), black_box(0))));
    c.bench_function("linear_search_simd_u8_max", |b| b.iter(|| linear_search_simd_u8(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_find_first_of_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::find_first_of_branchless::find_first_of_branchless;
    c.bench_function("find_first_of_branchless_avg", |b| b.iter(|| find_first_of_branchless(black_box(42), black_box(1337))));
    c.bench_function("find_first_of_branchless_min", |b| b.iter(|| find_first_of_branchless(black_box(0), black_box(0))));
    c.bench_function("find_first_of_branchless_max", |b| b.iter(|| find_first_of_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_find_last_of_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::find_last_of_branchless::find_last_of_branchless;
    c.bench_function("find_last_of_branchless_avg", |b| b.iter(|| find_last_of_branchless(black_box(42), black_box(1337))));
    c.bench_function("find_last_of_branchless_min", |b| b.iter(|| find_last_of_branchless(black_box(0), black_box(0))));
    c.bench_function("find_last_of_branchless_max", |b| b.iter(|| find_last_of_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_mismatch_branchless_u8(c: &mut Criterion) {
    use bcinr_logic::algorithms::mismatch_branchless_u8::mismatch_branchless_u8;
    c.bench_function("mismatch_branchless_u8_avg", |b| b.iter(|| mismatch_branchless_u8(black_box(42), black_box(1337))));
    c.bench_function("mismatch_branchless_u8_min", |b| b.iter(|| mismatch_branchless_u8(black_box(0), black_box(0))));
    c.bench_function("mismatch_branchless_u8_max", |b| b.iter(|| mismatch_branchless_u8(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_partial_sort_branchless_k(c: &mut Criterion) {
    use bcinr_logic::algorithms::partial_sort_branchless_k::partial_sort_branchless_k;
    c.bench_function("partial_sort_branchless_k_avg", |b| b.iter(|| partial_sort_branchless_k(black_box(42), black_box(1337))));
    c.bench_function("partial_sort_branchless_k_min", |b| b.iter(|| partial_sort_branchless_k(black_box(0), black_box(0))));
    c.bench_function("partial_sort_branchless_k_max", |b| b.iter(|| partial_sort_branchless_k(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_nth_element_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::nth_element_branchless::nth_element_branchless;
    c.bench_function("nth_element_branchless_avg", |b| b.iter(|| nth_element_branchless(black_box(42), black_box(1337))));
    c.bench_function("nth_element_branchless_min", |b| b.iter(|| nth_element_branchless(black_box(0), black_box(0))));
    c.bench_function("nth_element_branchless_max", |b| b.iter(|| nth_element_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_is_permutation_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::is_permutation_branchless::is_permutation_branchless;
    c.bench_function("is_permutation_branchless_avg", |b| b.iter(|| is_permutation_branchless(black_box(42), black_box(1337))));
    c.bench_function("is_permutation_branchless_min", |b| b.iter(|| is_permutation_branchless(black_box(0), black_box(0))));
    c.bench_function("is_permutation_branchless_max", |b| b.iter(|| is_permutation_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_set_difference_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::set_difference_branchless::set_difference_branchless;
    c.bench_function("set_difference_branchless_avg", |b| b.iter(|| set_difference_branchless(black_box(42), black_box(1337))));
    c.bench_function("set_difference_branchless_min", |b| b.iter(|| set_difference_branchless(black_box(0), black_box(0))));
    c.bench_function("set_difference_branchless_max", |b| b.iter(|| set_difference_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_set_symmetric_difference_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::set_symmetric_difference_branchless::set_symmetric_difference_branchless;
    c.bench_function("set_symmetric_difference_branchless_avg", |b| b.iter(|| set_symmetric_difference_branchless(black_box(42), black_box(1337))));
    c.bench_function("set_symmetric_difference_branchless_min", |b| b.iter(|| set_symmetric_difference_branchless(black_box(0), black_box(0))));
    c.bench_function("set_symmetric_difference_branchless_max", |b| b.iter(|| set_symmetric_difference_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_set_intersection_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::set_intersection_branchless::set_intersection_branchless;
    c.bench_function("set_intersection_branchless_avg", |b| b.iter(|| set_intersection_branchless(black_box(42), black_box(1337))));
    c.bench_function("set_intersection_branchless_min", |b| b.iter(|| set_intersection_branchless(black_box(0), black_box(0))));
    c.bench_function("set_intersection_branchless_max", |b| b.iter(|| set_intersection_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_set_union_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::set_union_branchless::set_union_branchless;
    c.bench_function("set_union_branchless_avg", |b| b.iter(|| set_union_branchless(black_box(42), black_box(1337))));
    c.bench_function("set_union_branchless_min", |b| b.iter(|| set_union_branchless(black_box(0), black_box(0))));
    c.bench_function("set_union_branchless_max", |b| b.iter(|| set_union_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_min_element_branchless_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::min_element_branchless_u32::min_element_branchless_u32;
    c.bench_function("min_element_branchless_u32_avg", |b| b.iter(|| min_element_branchless_u32(black_box(42), black_box(1337))));
    c.bench_function("min_element_branchless_u32_min", |b| b.iter(|| min_element_branchless_u32(black_box(0), black_box(0))));
    c.bench_function("min_element_branchless_u32_max", |b| b.iter(|| min_element_branchless_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_max_element_branchless_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::max_element_branchless_u32::max_element_branchless_u32;
    c.bench_function("max_element_branchless_u32_avg", |b| b.iter(|| max_element_branchless_u32(black_box(42), black_box(1337))));
    c.bench_function("max_element_branchless_u32_min", |b| b.iter(|| max_element_branchless_u32(black_box(0), black_box(0))));
    c.bench_function("max_element_branchless_u32_max", |b| b.iter(|| max_element_branchless_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_minmax_element_branchless_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::minmax_element_branchless_u32::minmax_element_branchless_u32;
    c.bench_function("minmax_element_branchless_u32_avg", |b| b.iter(|| minmax_element_branchless_u32(black_box(42), black_box(1337))));
    c.bench_function("minmax_element_branchless_u32_min", |b| b.iter(|| minmax_element_branchless_u32(black_box(0), black_box(0))));
    c.bench_function("minmax_element_branchless_u32_max", |b| b.iter(|| minmax_element_branchless_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_clamp_slice_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::clamp_slice_branchless::clamp_slice_branchless;
    c.bench_function("clamp_slice_branchless_avg", |b| b.iter(|| clamp_slice_branchless(black_box(42), black_box(1337))));
    c.bench_function("clamp_slice_branchless_min", |b| b.iter(|| clamp_slice_branchless(black_box(0), black_box(0))));
    c.bench_function("clamp_slice_branchless_max", |b| b.iter(|| clamp_slice_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_normalize_slice_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::normalize_slice_branchless::normalize_slice_branchless;
    c.bench_function("normalize_slice_branchless_avg", |b| b.iter(|| normalize_slice_branchless(black_box(42), black_box(1337))));
    c.bench_function("normalize_slice_branchless_min", |b| b.iter(|| normalize_slice_branchless(black_box(0), black_box(0))));
    c.bench_function("normalize_slice_branchless_max", |b| b.iter(|| normalize_slice_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_murmur3_x64_128(c: &mut Criterion) {
    use bcinr_logic::algorithms::murmur3_x64_128::murmur3_x64_128;
    c.bench_function("murmur3_x64_128_avg", |b| b.iter(|| murmur3_x64_128(black_box(42), black_box(1337))));
    c.bench_function("murmur3_x64_128_min", |b| b.iter(|| murmur3_x64_128(black_box(0), black_box(0))));
    c.bench_function("murmur3_x64_128_max", |b| b.iter(|| murmur3_x64_128(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_xxhash64(c: &mut Criterion) {
    use bcinr_logic::algorithms::xxhash64::xxhash64;
    c.bench_function("xxhash64_avg", |b| b.iter(|| xxhash64(black_box(42), black_box(1337))));
    c.bench_function("xxhash64_min", |b| b.iter(|| xxhash64(black_box(0), black_box(0))));
    c.bench_function("xxhash64_max", |b| b.iter(|| xxhash64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_xxh3_64(c: &mut Criterion) {
    use bcinr_logic::algorithms::xxh3_64::xxh3_64;
    c.bench_function("xxh3_64_avg", |b| b.iter(|| xxh3_64(black_box(42), black_box(1337))));
    c.bench_function("xxh3_64_min", |b| b.iter(|| xxh3_64(black_box(0), black_box(0))));
    c.bench_function("xxh3_64_max", |b| b.iter(|| xxh3_64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_cityhash64(c: &mut Criterion) {
    use bcinr_logic::algorithms::cityhash64::cityhash64;
    c.bench_function("cityhash64_avg", |b| b.iter(|| cityhash64(black_box(42), black_box(1337))));
    c.bench_function("cityhash64_min", |b| b.iter(|| cityhash64(black_box(0), black_box(0))));
    c.bench_function("cityhash64_max", |b| b.iter(|| cityhash64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_farmhash64(c: &mut Criterion) {
    use bcinr_logic::algorithms::farmhash64::farmhash64;
    c.bench_function("farmhash64_avg", |b| b.iter(|| farmhash64(black_box(42), black_box(1337))));
    c.bench_function("farmhash64_min", |b| b.iter(|| farmhash64(black_box(0), black_box(0))));
    c.bench_function("farmhash64_max", |b| b.iter(|| farmhash64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_spookyhash_v2_128(c: &mut Criterion) {
    use bcinr_logic::algorithms::spookyhash_v2_128::spookyhash_v2_128;
    c.bench_function("spookyhash_v2_128_avg", |b| b.iter(|| spookyhash_v2_128(black_box(42), black_box(1337))));
    c.bench_function("spookyhash_v2_128_min", |b| b.iter(|| spookyhash_v2_128(black_box(0), black_box(0))));
    c.bench_function("spookyhash_v2_128_max", |b| b.iter(|| spookyhash_v2_128(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_metrohash64(c: &mut Criterion) {
    use bcinr_logic::algorithms::metrohash64::metrohash64;
    c.bench_function("metrohash64_avg", |b| b.iter(|| metrohash64(black_box(42), black_box(1337))));
    c.bench_function("metrohash64_min", |b| b.iter(|| metrohash64(black_box(0), black_box(0))));
    c.bench_function("metrohash64_max", |b| b.iter(|| metrohash64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_siphash_2_4_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::siphash_2_4_branchless::siphash_2_4_branchless;
    c.bench_function("siphash_2_4_branchless_avg", |b| b.iter(|| siphash_2_4_branchless(black_box(42), black_box(1337))));
    c.bench_function("siphash_2_4_branchless_min", |b| b.iter(|| siphash_2_4_branchless(black_box(0), black_box(0))));
    c.bench_function("siphash_2_4_branchless_max", |b| b.iter(|| siphash_2_4_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_highwayhash_64(c: &mut Criterion) {
    use bcinr_logic::algorithms::highwayhash_64::highwayhash_64;
    c.bench_function("highwayhash_64_avg", |b| b.iter(|| highwayhash_64(black_box(42), black_box(1337))));
    c.bench_function("highwayhash_64_min", |b| b.iter(|| highwayhash_64(black_box(0), black_box(0))));
    c.bench_function("highwayhash_64_max", |b| b.iter(|| highwayhash_64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_clhash(c: &mut Criterion) {
    use bcinr_logic::algorithms::clhash::clhash;
    c.bench_function("clhash_avg", |b| b.iter(|| clhash(black_box(42), black_box(1337))));
    c.bench_function("clhash_min", |b| b.iter(|| clhash(black_box(0), black_box(0))));
    c.bench_function("clhash_max", |b| b.iter(|| clhash(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_pearson_hash_u8(c: &mut Criterion) {
    use bcinr_logic::algorithms::pearson_hash_u8::pearson_hash_u8;
    c.bench_function("pearson_hash_u8_avg", |b| b.iter(|| pearson_hash_u8(black_box(42), black_box(1337))));
    c.bench_function("pearson_hash_u8_min", |b| b.iter(|| pearson_hash_u8(black_box(0), black_box(0))));
    c.bench_function("pearson_hash_u8_max", |b| b.iter(|| pearson_hash_u8(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_knuth_hash_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::knuth_hash_u64::knuth_hash_u64;
    c.bench_function("knuth_hash_u64_avg", |b| b.iter(|| knuth_hash_u64(black_box(42), black_box(1337))));
    c.bench_function("knuth_hash_u64_min", |b| b.iter(|| knuth_hash_u64(black_box(0), black_box(0))));
    c.bench_function("knuth_hash_u64_max", |b| b.iter(|| knuth_hash_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_fibonacci_hash_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::fibonacci_hash_u64::fibonacci_hash_u64;
    c.bench_function("fibonacci_hash_u64_avg", |b| b.iter(|| fibonacci_hash_u64(black_box(42), black_box(1337))));
    c.bench_function("fibonacci_hash_u64_min", |b| b.iter(|| fibonacci_hash_u64(black_box(0), black_box(0))));
    c.bench_function("fibonacci_hash_u64_max", |b| b.iter(|| fibonacci_hash_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_zobrist_hash_64(c: &mut Criterion) {
    use bcinr_logic::algorithms::zobrist_hash_64::zobrist_hash_64;
    c.bench_function("zobrist_hash_64_avg", |b| b.iter(|| zobrist_hash_64(black_box(42), black_box(1337))));
    c.bench_function("zobrist_hash_64_min", |b| b.iter(|| zobrist_hash_64(black_box(0), black_box(0))));
    c.bench_function("zobrist_hash_64_max", |b| b.iter(|| zobrist_hash_64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_perfect_hash_lookup_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::perfect_hash_lookup_u32::perfect_hash_lookup_u32;
    c.bench_function("perfect_hash_lookup_u32_avg", |b| b.iter(|| perfect_hash_lookup_u32(black_box(42), black_box(1337))));
    c.bench_function("perfect_hash_lookup_u32_min", |b| b.iter(|| perfect_hash_lookup_u32(black_box(0), black_box(0))));
    c.bench_function("perfect_hash_lookup_u32_max", |b| b.iter(|| perfect_hash_lookup_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_minhash_u64_k(c: &mut Criterion) {
    use bcinr_logic::algorithms::minhash_u64_k::minhash_u64_k;
    c.bench_function("minhash_u64_k_avg", |b| b.iter(|| minhash_u64_k(black_box(42), black_box(1337))));
    c.bench_function("minhash_u64_k_min", |b| b.iter(|| minhash_u64_k(black_box(0), black_box(0))));
    c.bench_function("minhash_u64_k_max", |b| b.iter(|| minhash_u64_k(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_hyperloglog_add_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::hyperloglog_add_u64::hyperloglog_add_u64;
    c.bench_function("hyperloglog_add_u64_avg", |b| b.iter(|| hyperloglog_add_u64(black_box(42), black_box(1337))));
    c.bench_function("hyperloglog_add_u64_min", |b| b.iter(|| hyperloglog_add_u64(black_box(0), black_box(0))));
    c.bench_function("hyperloglog_add_u64_max", |b| b.iter(|| hyperloglog_add_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_hyperloglog_merge(c: &mut Criterion) {
    use bcinr_logic::algorithms::hyperloglog_merge::hyperloglog_merge;
    c.bench_function("hyperloglog_merge_avg", |b| b.iter(|| hyperloglog_merge(black_box(42), black_box(1337))));
    c.bench_function("hyperloglog_merge_min", |b| b.iter(|| hyperloglog_merge(black_box(0), black_box(0))));
    c.bench_function("hyperloglog_merge_max", |b| b.iter(|| hyperloglog_merge(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_count_min_sketch_add(c: &mut Criterion) {
    use bcinr_logic::algorithms::count_min_sketch_add::count_min_sketch_add;
    c.bench_function("count_min_sketch_add_avg", |b| b.iter(|| count_min_sketch_add(black_box(42), black_box(1337))));
    c.bench_function("count_min_sketch_add_min", |b| b.iter(|| count_min_sketch_add(black_box(0), black_box(0))));
    c.bench_function("count_min_sketch_add_max", |b| b.iter(|| count_min_sketch_add(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_count_min_sketch_query(c: &mut Criterion) {
    use bcinr_logic::algorithms::count_min_sketch_query::count_min_sketch_query;
    c.bench_function("count_min_sketch_query_avg", |b| b.iter(|| count_min_sketch_query(black_box(42), black_box(1337))));
    c.bench_function("count_min_sketch_query_min", |b| b.iter(|| count_min_sketch_query(black_box(0), black_box(0))));
    c.bench_function("count_min_sketch_query_max", |b| b.iter(|| count_min_sketch_query(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_bloom_filter_add_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::bloom_filter_add_u64::bloom_filter_add_u64;
    c.bench_function("bloom_filter_add_u64_avg", |b| b.iter(|| bloom_filter_add_u64(black_box(42), black_box(1337))));
    c.bench_function("bloom_filter_add_u64_min", |b| b.iter(|| bloom_filter_add_u64(black_box(0), black_box(0))));
    c.bench_function("bloom_filter_add_u64_max", |b| b.iter(|| bloom_filter_add_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_bloom_filter_query_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::bloom_filter_query_u64::bloom_filter_query_u64;
    c.bench_function("bloom_filter_query_u64_avg", |b| b.iter(|| bloom_filter_query_u64(black_box(42), black_box(1337))));
    c.bench_function("bloom_filter_query_u64_min", |b| b.iter(|| bloom_filter_query_u64(black_box(0), black_box(0))));
    c.bench_function("bloom_filter_query_u64_max", |b| b.iter(|| bloom_filter_query_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_cuckoo_filter_add_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::cuckoo_filter_add_u64::cuckoo_filter_add_u64;
    c.bench_function("cuckoo_filter_add_u64_avg", |b| b.iter(|| cuckoo_filter_add_u64(black_box(42), black_box(1337))));
    c.bench_function("cuckoo_filter_add_u64_min", |b| b.iter(|| cuckoo_filter_add_u64(black_box(0), black_box(0))));
    c.bench_function("cuckoo_filter_add_u64_max", |b| b.iter(|| cuckoo_filter_add_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_quotient_filter_add_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::quotient_filter_add_u64::quotient_filter_add_u64;
    c.bench_function("quotient_filter_add_u64_avg", |b| b.iter(|| quotient_filter_add_u64(black_box(42), black_box(1337))));
    c.bench_function("quotient_filter_add_u64_min", |b| b.iter(|| quotient_filter_add_u64(black_box(0), black_box(0))));
    c.bench_function("quotient_filter_add_u64_max", |b| b.iter(|| quotient_filter_add_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_t_digest_add_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::t_digest_add_u32::t_digest_add_u32;
    c.bench_function("t_digest_add_u32_avg", |b| b.iter(|| t_digest_add_u32(black_box(42), black_box(1337))));
    c.bench_function("t_digest_add_u32_min", |b| b.iter(|| t_digest_add_u32(black_box(0), black_box(0))));
    c.bench_function("t_digest_add_u32_max", |b| b.iter(|| t_digest_add_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_heavy_keepers_add(c: &mut Criterion) {
    use bcinr_logic::algorithms::heavy_keepers_add::heavy_keepers_add;
    c.bench_function("heavy_keepers_add_avg", |b| b.iter(|| heavy_keepers_add(black_box(42), black_box(1337))));
    c.bench_function("heavy_keepers_add_min", |b| b.iter(|| heavy_keepers_add(black_box(0), black_box(0))));
    c.bench_function("heavy_keepers_add_max", |b| b.iter(|| heavy_keepers_add(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_space_saving_add(c: &mut Criterion) {
    use bcinr_logic::algorithms::space_saving_add::space_saving_add;
    c.bench_function("space_saving_add_avg", |b| b.iter(|| space_saving_add(black_box(42), black_box(1337))));
    c.bench_function("space_saving_add_min", |b| b.iter(|| space_saving_add(black_box(0), black_box(0))));
    c.bench_function("space_saving_add_max", |b| b.iter(|| space_saving_add(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_misra_gries_add(c: &mut Criterion) {
    use bcinr_logic::algorithms::misra_gries_add::misra_gries_add;
    c.bench_function("misra_gries_add_avg", |b| b.iter(|| misra_gries_add(black_box(42), black_box(1337))));
    c.bench_function("misra_gries_add_min", |b| b.iter(|| misra_gries_add(black_box(0), black_box(0))));
    c.bench_function("misra_gries_add_max", |b| b.iter(|| misra_gries_add(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_reservoir_sample_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::reservoir_sample_branchless::reservoir_sample_branchless;
    c.bench_function("reservoir_sample_branchless_avg", |b| b.iter(|| reservoir_sample_branchless(black_box(42), black_box(1337))));
    c.bench_function("reservoir_sample_branchless_min", |b| b.iter(|| reservoir_sample_branchless(black_box(0), black_box(0))));
    c.bench_function("reservoir_sample_branchless_max", |b| b.iter(|| reservoir_sample_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_weighted_reservoir_sample(c: &mut Criterion) {
    use bcinr_logic::algorithms::weighted_reservoir_sample::weighted_reservoir_sample;
    c.bench_function("weighted_reservoir_sample_avg", |b| b.iter(|| weighted_reservoir_sample(black_box(42), black_box(1337))));
    c.bench_function("weighted_reservoir_sample_min", |b| b.iter(|| weighted_reservoir_sample(black_box(0), black_box(0))));
    c.bench_function("weighted_reservoir_sample_max", |b| b.iter(|| weighted_reservoir_sample(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_consistent_hash_jump_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::consistent_hash_jump_u64::consistent_hash_jump_u64;
    c.bench_function("consistent_hash_jump_u64_avg", |b| b.iter(|| consistent_hash_jump_u64(black_box(42), black_box(1337))));
    c.bench_function("consistent_hash_jump_u64_min", |b| b.iter(|| consistent_hash_jump_u64(black_box(0), black_box(0))));
    c.bench_function("consistent_hash_jump_u64_max", |b| b.iter(|| consistent_hash_jump_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_consistent_hash_maglev(c: &mut Criterion) {
    use bcinr_logic::algorithms::consistent_hash_maglev::consistent_hash_maglev;
    c.bench_function("consistent_hash_maglev_avg", |b| b.iter(|| consistent_hash_maglev(black_box(42), black_box(1337))));
    c.bench_function("consistent_hash_maglev_min", |b| b.iter(|| consistent_hash_maglev(black_box(0), black_box(0))));
    c.bench_function("consistent_hash_maglev_max", |b| b.iter(|| consistent_hash_maglev(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_bloom_filter_intersect(c: &mut Criterion) {
    use bcinr_logic::algorithms::bloom_filter_intersect::bloom_filter_intersect;
    c.bench_function("bloom_filter_intersect_avg", |b| b.iter(|| bloom_filter_intersect(black_box(42), black_box(1337))));
    c.bench_function("bloom_filter_intersect_min", |b| b.iter(|| bloom_filter_intersect(black_box(0), black_box(0))));
    c.bench_function("bloom_filter_intersect_max", |b| b.iter(|| bloom_filter_intersect(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_bloom_filter_union(c: &mut Criterion) {
    use bcinr_logic::algorithms::bloom_filter_union::bloom_filter_union;
    c.bench_function("bloom_filter_union_avg", |b| b.iter(|| bloom_filter_union(black_box(42), black_box(1337))));
    c.bench_function("bloom_filter_union_min", |b| b.iter(|| bloom_filter_union(black_box(0), black_box(0))));
    c.bench_function("bloom_filter_union_max", |b| b.iter(|| bloom_filter_union(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_hashing_trick_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::hashing_trick_u64::hashing_trick_u64;
    c.bench_function("hashing_trick_u64_avg", |b| b.iter(|| hashing_trick_u64(black_box(42), black_box(1337))));
    c.bench_function("hashing_trick_u64_min", |b| b.iter(|| hashing_trick_u64(black_box(0), black_box(0))));
    c.bench_function("hashing_trick_u64_max", |b| b.iter(|| hashing_trick_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_locality_sensitive_hash_euclidean(c: &mut Criterion) {
    use bcinr_logic::algorithms::locality_sensitive_hash_euclidean::locality_sensitive_hash_euclidean;
    c.bench_function("locality_sensitive_hash_euclidean_avg", |b| b.iter(|| locality_sensitive_hash_euclidean(black_box(42), black_box(1337))));
    c.bench_function("locality_sensitive_hash_euclidean_min", |b| b.iter(|| locality_sensitive_hash_euclidean(black_box(0), black_box(0))));
    c.bench_function("locality_sensitive_hash_euclidean_max", |b| b.iter(|| locality_sensitive_hash_euclidean(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_locality_sensitive_hash_cosine(c: &mut Criterion) {
    use bcinr_logic::algorithms::locality_sensitive_hash_cosine::locality_sensitive_hash_cosine;
    c.bench_function("locality_sensitive_hash_cosine_avg", |b| b.iter(|| locality_sensitive_hash_cosine(black_box(42), black_box(1337))));
    c.bench_function("locality_sensitive_hash_cosine_min", |b| b.iter(|| locality_sensitive_hash_cosine(black_box(0), black_box(0))));
    c.bench_function("locality_sensitive_hash_cosine_max", |b| b.iter(|| locality_sensitive_hash_cosine(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_k_independent_hash_gen(c: &mut Criterion) {
    use bcinr_logic::algorithms::k_independent_hash_gen::k_independent_hash_gen;
    c.bench_function("k_independent_hash_gen_avg", |b| b.iter(|| k_independent_hash_gen(black_box(42), black_box(1337))));
    c.bench_function("k_independent_hash_gen_min", |b| b.iter(|| k_independent_hash_gen(black_box(0), black_box(0))));
    c.bench_function("k_independent_hash_gen_max", |b| b.iter(|| k_independent_hash_gen(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_rolling_hash_rabin_karp(c: &mut Criterion) {
    use bcinr_logic::algorithms::rolling_hash_rabin_karp::rolling_hash_rabin_karp;
    c.bench_function("rolling_hash_rabin_karp_avg", |b| b.iter(|| rolling_hash_rabin_karp(black_box(42), black_box(1337))));
    c.bench_function("rolling_hash_rabin_karp_min", |b| b.iter(|| rolling_hash_rabin_karp(black_box(0), black_box(0))));
    c.bench_function("rolling_hash_rabin_karp_max", |b| b.iter(|| rolling_hash_rabin_karp(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_rolling_hash_buzhash(c: &mut Criterion) {
    use bcinr_logic::algorithms::rolling_hash_buzhash::rolling_hash_buzhash;
    c.bench_function("rolling_hash_buzhash_avg", |b| b.iter(|| rolling_hash_buzhash(black_box(42), black_box(1337))));
    c.bench_function("rolling_hash_buzhash_min", |b| b.iter(|| rolling_hash_buzhash(black_box(0), black_box(0))));
    c.bench_function("rolling_hash_buzhash_max", |b| b.iter(|| rolling_hash_buzhash(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_rolling_hash_gear(c: &mut Criterion) {
    use bcinr_logic::algorithms::rolling_hash_gear::rolling_hash_gear;
    c.bench_function("rolling_hash_gear_avg", |b| b.iter(|| rolling_hash_gear(black_box(42), black_box(1337))));
    c.bench_function("rolling_hash_gear_min", |b| b.iter(|| rolling_hash_gear(black_box(0), black_box(0))));
    c.bench_function("rolling_hash_gear_max", |b| b.iter(|| rolling_hash_gear(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_content_defined_chunking_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::content_defined_chunking_branchless::content_defined_chunking_branchless;
    c.bench_function("content_defined_chunking_branchless_avg", |b| b.iter(|| content_defined_chunking_branchless(black_box(42), black_box(1337))));
    c.bench_function("content_defined_chunking_branchless_min", |b| b.iter(|| content_defined_chunking_branchless(black_box(0), black_box(0))));
    c.bench_function("content_defined_chunking_branchless_max", |b| b.iter(|| content_defined_chunking_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_cyclic_redundancy_check_crc32c(c: &mut Criterion) {
    use bcinr_logic::algorithms::cyclic_redundancy_check_crc32c::cyclic_redundancy_check_crc32c;
    c.bench_function("cyclic_redundancy_check_crc32c_avg", |b| b.iter(|| cyclic_redundancy_check_crc32c(black_box(42), black_box(1337))));
    c.bench_function("cyclic_redundancy_check_crc32c_min", |b| b.iter(|| cyclic_redundancy_check_crc32c(black_box(0), black_box(0))));
    c.bench_function("cyclic_redundancy_check_crc32c_max", |b| b.iter(|| cyclic_redundancy_check_crc32c(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_cyclic_redundancy_check_crc64(c: &mut Criterion) {
    use bcinr_logic::algorithms::cyclic_redundancy_check_crc64::cyclic_redundancy_check_crc64;
    c.bench_function("cyclic_redundancy_check_crc64_avg", |b| b.iter(|| cyclic_redundancy_check_crc64(black_box(42), black_box(1337))));
    c.bench_function("cyclic_redundancy_check_crc64_min", |b| b.iter(|| cyclic_redundancy_check_crc64(black_box(0), black_box(0))));
    c.bench_function("cyclic_redundancy_check_crc64_max", |b| b.iter(|| cyclic_redundancy_check_crc64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_adler32_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::adler32_branchless::adler32_branchless;
    c.bench_function("adler32_branchless_avg", |b| b.iter(|| adler32_branchless(black_box(42), black_box(1337))));
    c.bench_function("adler32_branchless_min", |b| b.iter(|| adler32_branchless(black_box(0), black_box(0))));
    c.bench_function("adler32_branchless_max", |b| b.iter(|| adler32_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_fletcher32_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::fletcher32_branchless::fletcher32_branchless;
    c.bench_function("fletcher32_branchless_avg", |b| b.iter(|| fletcher32_branchless(black_box(42), black_box(1337))));
    c.bench_function("fletcher32_branchless_min", |b| b.iter(|| fletcher32_branchless(black_box(0), black_box(0))));
    c.bench_function("fletcher32_branchless_max", |b| b.iter(|| fletcher32_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_bsd_checksum_u16(c: &mut Criterion) {
    use bcinr_logic::algorithms::bsd_checksum_u16::bsd_checksum_u16;
    c.bench_function("bsd_checksum_u16_avg", |b| b.iter(|| bsd_checksum_u16(black_box(42), black_box(1337))));
    c.bench_function("bsd_checksum_u16_min", |b| b.iter(|| bsd_checksum_u16(black_box(0), black_box(0))));
    c.bench_function("bsd_checksum_u16_max", |b| b.iter(|| bsd_checksum_u16(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_internet_checksum_u16(c: &mut Criterion) {
    use bcinr_logic::algorithms::internet_checksum_u16::internet_checksum_u16;
    c.bench_function("internet_checksum_u16_avg", |b| b.iter(|| internet_checksum_u16(black_box(42), black_box(1337))));
    c.bench_function("internet_checksum_u16_min", |b| b.iter(|| internet_checksum_u16(black_box(0), black_box(0))));
    c.bench_function("internet_checksum_u16_max", |b| b.iter(|| internet_checksum_u16(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_duffs_device_simd_unroll(c: &mut Criterion) {
    use bcinr_logic::algorithms::duffs_device_simd_unroll::duffs_device_simd_unroll;
    c.bench_function("duffs_device_simd_unroll_avg", |b| b.iter(|| duffs_device_simd_unroll(black_box(42), black_box(1337))));
    c.bench_function("duffs_device_simd_unroll_min", |b| b.iter(|| duffs_device_simd_unroll(black_box(0), black_box(0))));
    c.bench_function("duffs_device_simd_unroll_max", |b| b.iter(|| duffs_device_simd_unroll(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_perfect_hash_build_static(c: &mut Criterion) {
    use bcinr_logic::algorithms::perfect_hash_build_static::perfect_hash_build_static;
    c.bench_function("perfect_hash_build_static_avg", |b| b.iter(|| perfect_hash_build_static(black_box(42), black_box(1337))));
    c.bench_function("perfect_hash_build_static_min", |b| b.iter(|| perfect_hash_build_static(black_box(0), black_box(0))));
    c.bench_function("perfect_hash_build_static_max", |b| b.iter(|| perfect_hash_build_static(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_base64_encode_simd(c: &mut Criterion) {
    use bcinr_logic::algorithms::base64_encode_simd::base64_encode_simd;
    c.bench_function("base64_encode_simd_avg", |b| b.iter(|| base64_encode_simd(black_box(42), black_box(1337))));
    c.bench_function("base64_encode_simd_min", |b| b.iter(|| base64_encode_simd(black_box(0), black_box(0))));
    c.bench_function("base64_encode_simd_max", |b| b.iter(|| base64_encode_simd(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_base64_decode_simd(c: &mut Criterion) {
    use bcinr_logic::algorithms::base64_decode_simd::base64_decode_simd;
    c.bench_function("base64_decode_simd_avg", |b| b.iter(|| base64_decode_simd(black_box(42), black_box(1337))));
    c.bench_function("base64_decode_simd_min", |b| b.iter(|| base64_decode_simd(black_box(0), black_box(0))));
    c.bench_function("base64_decode_simd_max", |b| b.iter(|| base64_decode_simd(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_hex_encode_simd(c: &mut Criterion) {
    use bcinr_logic::algorithms::hex_encode_simd::hex_encode_simd;
    c.bench_function("hex_encode_simd_avg", |b| b.iter(|| hex_encode_simd(black_box(42), black_box(1337))));
    c.bench_function("hex_encode_simd_min", |b| b.iter(|| hex_encode_simd(black_box(0), black_box(0))));
    c.bench_function("hex_encode_simd_max", |b| b.iter(|| hex_encode_simd(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_hex_decode_simd(c: &mut Criterion) {
    use bcinr_logic::algorithms::hex_decode_simd::hex_decode_simd;
    c.bench_function("hex_decode_simd_avg", |b| b.iter(|| hex_decode_simd(black_box(42), black_box(1337))));
    c.bench_function("hex_decode_simd_min", |b| b.iter(|| hex_decode_simd(black_box(0), black_box(0))));
    c.bench_function("hex_decode_simd_max", |b| b.iter(|| hex_decode_simd(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_base32_encode_rfc4648(c: &mut Criterion) {
    use bcinr_logic::algorithms::base32_encode_rfc4648::base32_encode_rfc4648;
    c.bench_function("base32_encode_rfc4648_avg", |b| b.iter(|| base32_encode_rfc4648(black_box(42), black_box(1337))));
    c.bench_function("base32_encode_rfc4648_min", |b| b.iter(|| base32_encode_rfc4648(black_box(0), black_box(0))));
    c.bench_function("base32_encode_rfc4648_max", |b| b.iter(|| base32_encode_rfc4648(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_base85_encode_ascii85(c: &mut Criterion) {
    use bcinr_logic::algorithms::base85_encode_ascii85::base85_encode_ascii85;
    c.bench_function("base85_encode_ascii85_avg", |b| b.iter(|| base85_encode_ascii85(black_box(42), black_box(1337))));
    c.bench_function("base85_encode_ascii85_min", |b| b.iter(|| base85_encode_ascii85(black_box(0), black_box(0))));
    c.bench_function("base85_encode_ascii85_max", |b| b.iter(|| base85_encode_ascii85(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_leb128_encode_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::leb128_encode_u64::leb128_encode_u64;
    c.bench_function("leb128_encode_u64_avg", |b| b.iter(|| leb128_encode_u64(black_box(42), black_box(1337))));
    c.bench_function("leb128_encode_u64_min", |b| b.iter(|| leb128_encode_u64(black_box(0), black_box(0))));
    c.bench_function("leb128_encode_u64_max", |b| b.iter(|| leb128_encode_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_leb128_decode_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::leb128_decode_u64::leb128_decode_u64;
    c.bench_function("leb128_decode_u64_avg", |b| b.iter(|| leb128_decode_u64(black_box(42), black_box(1337))));
    c.bench_function("leb128_decode_u64_min", |b| b.iter(|| leb128_decode_u64(black_box(0), black_box(0))));
    c.bench_function("leb128_decode_u64_max", |b| b.iter(|| leb128_decode_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_varint_encode_simd(c: &mut Criterion) {
    use bcinr_logic::algorithms::varint_encode_simd::varint_encode_simd;
    c.bench_function("varint_encode_simd_avg", |b| b.iter(|| varint_encode_simd(black_box(42), black_box(1337))));
    c.bench_function("varint_encode_simd_min", |b| b.iter(|| varint_encode_simd(black_box(0), black_box(0))));
    c.bench_function("varint_encode_simd_max", |b| b.iter(|| varint_encode_simd(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_varint_decode_simd(c: &mut Criterion) {
    use bcinr_logic::algorithms::varint_decode_simd::varint_decode_simd;
    c.bench_function("varint_decode_simd_avg", |b| b.iter(|| varint_decode_simd(black_box(42), black_box(1337))));
    c.bench_function("varint_decode_simd_min", |b| b.iter(|| varint_decode_simd(black_box(0), black_box(0))));
    c.bench_function("varint_decode_simd_max", |b| b.iter(|| varint_decode_simd(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_bitpacking_encode_u32_k(c: &mut Criterion) {
    use bcinr_logic::algorithms::bitpacking_encode_u32_k::bitpacking_encode_u32_k;
    c.bench_function("bitpacking_encode_u32_k_avg", |b| b.iter(|| bitpacking_encode_u32_k(black_box(42), black_box(1337))));
    c.bench_function("bitpacking_encode_u32_k_min", |b| b.iter(|| bitpacking_encode_u32_k(black_box(0), black_box(0))));
    c.bench_function("bitpacking_encode_u32_k_max", |b| b.iter(|| bitpacking_encode_u32_k(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_bitpacking_decode_u32_k(c: &mut Criterion) {
    use bcinr_logic::algorithms::bitpacking_decode_u32_k::bitpacking_decode_u32_k;
    c.bench_function("bitpacking_decode_u32_k_avg", |b| b.iter(|| bitpacking_decode_u32_k(black_box(42), black_box(1337))));
    c.bench_function("bitpacking_decode_u32_k_min", |b| b.iter(|| bitpacking_decode_u32_k(black_box(0), black_box(0))));
    c.bench_function("bitpacking_decode_u32_k_max", |b| b.iter(|| bitpacking_decode_u32_k(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_zigzag_encode_i64(c: &mut Criterion) {
    use bcinr_logic::algorithms::zigzag_encode_i64::zigzag_encode_i64;
    c.bench_function("zigzag_encode_i64_avg", |b| b.iter(|| zigzag_encode_i64(black_box(42), black_box(1337))));
    c.bench_function("zigzag_encode_i64_min", |b| b.iter(|| zigzag_encode_i64(black_box(0), black_box(0))));
    c.bench_function("zigzag_encode_i64_max", |b| b.iter(|| zigzag_encode_i64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_zigzag_decode_i64(c: &mut Criterion) {
    use bcinr_logic::algorithms::zigzag_decode_i64::zigzag_decode_i64;
    c.bench_function("zigzag_decode_i64_avg", |b| b.iter(|| zigzag_decode_i64(black_box(42), black_box(1337))));
    c.bench_function("zigzag_decode_i64_min", |b| b.iter(|| zigzag_decode_i64(black_box(0), black_box(0))));
    c.bench_function("zigzag_decode_i64_max", |b| b.iter(|| zigzag_decode_i64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_utf8_to_utf16_simd(c: &mut Criterion) {
    use bcinr_logic::algorithms::utf8_to_utf16_simd::utf8_to_utf16_simd;
    c.bench_function("utf8_to_utf16_simd_avg", |b| b.iter(|| utf8_to_utf16_simd(black_box(42), black_box(1337))));
    c.bench_function("utf8_to_utf16_simd_min", |b| b.iter(|| utf8_to_utf16_simd(black_box(0), black_box(0))));
    c.bench_function("utf8_to_utf16_simd_max", |b| b.iter(|| utf8_to_utf16_simd(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_utf16_to_utf8_simd(c: &mut Criterion) {
    use bcinr_logic::algorithms::utf16_to_utf8_simd::utf16_to_utf8_simd;
    c.bench_function("utf16_to_utf8_simd_avg", |b| b.iter(|| utf16_to_utf8_simd(black_box(42), black_box(1337))));
    c.bench_function("utf16_to_utf8_simd_min", |b| b.iter(|| utf16_to_utf8_simd(black_box(0), black_box(0))));
    c.bench_function("utf16_to_utf8_simd_max", |b| b.iter(|| utf16_to_utf8_simd(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_utf8_to_utf32_simd(c: &mut Criterion) {
    use bcinr_logic::algorithms::utf8_to_utf32_simd::utf8_to_utf32_simd;
    c.bench_function("utf8_to_utf32_simd_avg", |b| b.iter(|| utf8_to_utf32_simd(black_box(42), black_box(1337))));
    c.bench_function("utf8_to_utf32_simd_min", |b| b.iter(|| utf8_to_utf32_simd(black_box(0), black_box(0))));
    c.bench_function("utf8_to_utf32_simd_max", |b| b.iter(|| utf8_to_utf32_simd(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_ascii_to_lowercase_simd(c: &mut Criterion) {
    use bcinr_logic::algorithms::ascii_to_lowercase_simd::ascii_to_lowercase_simd;
    c.bench_function("ascii_to_lowercase_simd_avg", |b| b.iter(|| ascii_to_lowercase_simd(black_box(42), black_box(1337))));
    c.bench_function("ascii_to_lowercase_simd_min", |b| b.iter(|| ascii_to_lowercase_simd(black_box(0), black_box(0))));
    c.bench_function("ascii_to_lowercase_simd_max", |b| b.iter(|| ascii_to_lowercase_simd(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_ascii_to_uppercase_simd(c: &mut Criterion) {
    use bcinr_logic::algorithms::ascii_to_uppercase_simd::ascii_to_uppercase_simd;
    c.bench_function("ascii_to_uppercase_simd_avg", |b| b.iter(|| ascii_to_uppercase_simd(black_box(42), black_box(1337))));
    c.bench_function("ascii_to_uppercase_simd_min", |b| b.iter(|| ascii_to_uppercase_simd(black_box(0), black_box(0))));
    c.bench_function("ascii_to_uppercase_simd_max", |b| b.iter(|| ascii_to_uppercase_simd(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_is_alphanumeric_simd_u8x16(c: &mut Criterion) {
    use bcinr_logic::algorithms::is_alphanumeric_simd_u8x16::is_alphanumeric_simd_u8x16;
    c.bench_function("is_alphanumeric_simd_u8x16_avg", |b| b.iter(|| is_alphanumeric_simd_u8x16(black_box(42), black_box(1337))));
    c.bench_function("is_alphanumeric_simd_u8x16_min", |b| b.iter(|| is_alphanumeric_simd_u8x16(black_box(0), black_box(0))));
    c.bench_function("is_alphanumeric_simd_u8x16_max", |b| b.iter(|| is_alphanumeric_simd_u8x16(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_is_digit_simd_u8x16(c: &mut Criterion) {
    use bcinr_logic::algorithms::is_digit_simd_u8x16::is_digit_simd_u8x16;
    c.bench_function("is_digit_simd_u8x16_avg", |b| b.iter(|| is_digit_simd_u8x16(black_box(42), black_box(1337))));
    c.bench_function("is_digit_simd_u8x16_min", |b| b.iter(|| is_digit_simd_u8x16(black_box(0), black_box(0))));
    c.bench_function("is_digit_simd_u8x16_max", |b| b.iter(|| is_digit_simd_u8x16(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_is_space_simd_u8x16(c: &mut Criterion) {
    use bcinr_logic::algorithms::is_space_simd_u8x16::is_space_simd_u8x16;
    c.bench_function("is_space_simd_u8x16_avg", |b| b.iter(|| is_space_simd_u8x16(black_box(42), black_box(1337))));
    c.bench_function("is_space_simd_u8x16_min", |b| b.iter(|| is_space_simd_u8x16(black_box(0), black_box(0))));
    c.bench_function("is_space_simd_u8x16_max", |b| b.iter(|| is_space_simd_u8x16(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_trim_whitespace_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::trim_whitespace_branchless::trim_whitespace_branchless;
    c.bench_function("trim_whitespace_branchless_avg", |b| b.iter(|| trim_whitespace_branchless(black_box(42), black_box(1337))));
    c.bench_function("trim_whitespace_branchless_min", |b| b.iter(|| trim_whitespace_branchless(black_box(0), black_box(0))));
    c.bench_function("trim_whitespace_branchless_max", |b| b.iter(|| trim_whitespace_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_split_lines_simd(c: &mut Criterion) {
    use bcinr_logic::algorithms::split_lines_simd::split_lines_simd;
    c.bench_function("split_lines_simd_avg", |b| b.iter(|| split_lines_simd(black_box(42), black_box(1337))));
    c.bench_function("split_lines_simd_min", |b| b.iter(|| split_lines_simd(black_box(0), black_box(0))));
    c.bench_function("split_lines_simd_max", |b| b.iter(|| split_lines_simd(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_csv_scan_row_simd(c: &mut Criterion) {
    use bcinr_logic::algorithms::csv_scan_row_simd::csv_scan_row_simd;
    c.bench_function("csv_scan_row_simd_avg", |b| b.iter(|| csv_scan_row_simd(black_box(42), black_box(1337))));
    c.bench_function("csv_scan_row_simd_min", |b| b.iter(|| csv_scan_row_simd(black_box(0), black_box(0))));
    c.bench_function("csv_scan_row_simd_max", |b| b.iter(|| csv_scan_row_simd(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_json_find_string_escapes_simd(c: &mut Criterion) {
    use bcinr_logic::algorithms::json_find_string_escapes_simd::json_find_string_escapes_simd;
    c.bench_function("json_find_string_escapes_simd_avg", |b| b.iter(|| json_find_string_escapes_simd(black_box(42), black_box(1337))));
    c.bench_function("json_find_string_escapes_simd_min", |b| b.iter(|| json_find_string_escapes_simd(black_box(0), black_box(0))));
    c.bench_function("json_find_string_escapes_simd_max", |b| b.iter(|| json_find_string_escapes_simd(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_json_find_structural_simd(c: &mut Criterion) {
    use bcinr_logic::algorithms::json_find_structural_simd::json_find_structural_simd;
    c.bench_function("json_find_structural_simd_avg", |b| b.iter(|| json_find_structural_simd(black_box(42), black_box(1337))));
    c.bench_function("json_find_structural_simd_min", |b| b.iter(|| json_find_structural_simd(black_box(0), black_box(0))));
    c.bench_function("json_find_structural_simd_max", |b| b.iter(|| json_find_structural_simd(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_levenshtein_dist_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::levenshtein_dist_branchless::levenshtein_dist_branchless;
    c.bench_function("levenshtein_dist_branchless_avg", |b| b.iter(|| levenshtein_dist_branchless(black_box(42), black_box(1337))));
    c.bench_function("levenshtein_dist_branchless_min", |b| b.iter(|| levenshtein_dist_branchless(black_box(0), black_box(0))));
    c.bench_function("levenshtein_dist_branchless_max", |b| b.iter(|| levenshtein_dist_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_hamming_dist_simd(c: &mut Criterion) {
    use bcinr_logic::algorithms::hamming_dist_simd::hamming_dist_simd;
    c.bench_function("hamming_dist_simd_avg", |b| b.iter(|| hamming_dist_simd(black_box(42), black_box(1337))));
    c.bench_function("hamming_dist_simd_min", |b| b.iter(|| hamming_dist_simd(black_box(0), black_box(0))));
    c.bench_function("hamming_dist_simd_max", |b| b.iter(|| hamming_dist_simd(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_jaro_winkler_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::jaro_winkler_branchless::jaro_winkler_branchless;
    c.bench_function("jaro_winkler_branchless_avg", |b| b.iter(|| jaro_winkler_branchless(black_box(42), black_box(1337))));
    c.bench_function("jaro_winkler_branchless_min", |b| b.iter(|| jaro_winkler_branchless(black_box(0), black_box(0))));
    c.bench_function("jaro_winkler_branchless_max", |b| b.iter(|| jaro_winkler_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_soundex_encode_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::soundex_encode_branchless::soundex_encode_branchless;
    c.bench_function("soundex_encode_branchless_avg", |b| b.iter(|| soundex_encode_branchless(black_box(42), black_box(1337))));
    c.bench_function("soundex_encode_branchless_min", |b| b.iter(|| soundex_encode_branchless(black_box(0), black_box(0))));
    c.bench_function("soundex_encode_branchless_max", |b| b.iter(|| soundex_encode_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_metaphone_encode_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::metaphone_encode_branchless::metaphone_encode_branchless;
    c.bench_function("metaphone_encode_branchless_avg", |b| b.iter(|| metaphone_encode_branchless(black_box(42), black_box(1337))));
    c.bench_function("metaphone_encode_branchless_min", |b| b.iter(|| metaphone_encode_branchless(black_box(0), black_box(0))));
    c.bench_function("metaphone_encode_branchless_max", |b| b.iter(|| metaphone_encode_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_url_encode_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::url_encode_branchless::url_encode_branchless;
    c.bench_function("url_encode_branchless_avg", |b| b.iter(|| url_encode_branchless(black_box(42), black_box(1337))));
    c.bench_function("url_encode_branchless_min", |b| b.iter(|| url_encode_branchless(black_box(0), black_box(0))));
    c.bench_function("url_encode_branchless_max", |b| b.iter(|| url_encode_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_url_decode_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::url_decode_branchless::url_decode_branchless;
    c.bench_function("url_decode_branchless_avg", |b| b.iter(|| url_decode_branchless(black_box(42), black_box(1337))));
    c.bench_function("url_decode_branchless_min", |b| b.iter(|| url_decode_branchless(black_box(0), black_box(0))));
    c.bench_function("url_decode_branchless_max", |b| b.iter(|| url_decode_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_punycode_encode_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::punycode_encode_branchless::punycode_encode_branchless;
    c.bench_function("punycode_encode_branchless_avg", |b| b.iter(|| punycode_encode_branchless(black_box(42), black_box(1337))));
    c.bench_function("punycode_encode_branchless_min", |b| b.iter(|| punycode_encode_branchless(black_box(0), black_box(0))));
    c.bench_function("punycode_encode_branchless_max", |b| b.iter(|| punycode_encode_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_simd_strstr_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::simd_strstr_branchless::simd_strstr_branchless;
    c.bench_function("simd_strstr_branchless_avg", |b| b.iter(|| simd_strstr_branchless(black_box(42), black_box(1337))));
    c.bench_function("simd_strstr_branchless_min", |b| b.iter(|| simd_strstr_branchless(black_box(0), black_box(0))));
    c.bench_function("simd_strstr_branchless_max", |b| b.iter(|| simd_strstr_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_simd_memchr_u8x16(c: &mut Criterion) {
    use bcinr_logic::algorithms::simd_memchr_u8x16::simd_memchr_u8x16;
    c.bench_function("simd_memchr_u8x16_avg", |b| b.iter(|| simd_memchr_u8x16(black_box(42), black_box(1337))));
    c.bench_function("simd_memchr_u8x16_min", |b| b.iter(|| simd_memchr_u8x16(black_box(0), black_box(0))));
    c.bench_function("simd_memchr_u8x16_max", |b| b.iter(|| simd_memchr_u8x16(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_simd_memrchr_u8x16(c: &mut Criterion) {
    use bcinr_logic::algorithms::simd_memrchr_u8x16::simd_memrchr_u8x16;
    c.bench_function("simd_memrchr_u8x16_avg", |b| b.iter(|| simd_memrchr_u8x16(black_box(42), black_box(1337))));
    c.bench_function("simd_memrchr_u8x16_min", |b| b.iter(|| simd_memrchr_u8x16(black_box(0), black_box(0))));
    c.bench_function("simd_memrchr_u8x16_max", |b| b.iter(|| simd_memrchr_u8x16(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_wildcard_match_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::wildcard_match_branchless::wildcard_match_branchless;
    c.bench_function("wildcard_match_branchless_avg", |b| b.iter(|| wildcard_match_branchless(black_box(42), black_box(1337))));
    c.bench_function("wildcard_match_branchless_min", |b| b.iter(|| wildcard_match_branchless(black_box(0), black_box(0))));
    c.bench_function("wildcard_match_branchless_max", |b| b.iter(|| wildcard_match_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_regex_nfa_simd_step(c: &mut Criterion) {
    use bcinr_logic::algorithms::regex_nfa_simd_step::regex_nfa_simd_step;
    c.bench_function("regex_nfa_simd_step_avg", |b| b.iter(|| regex_nfa_simd_step(black_box(42), black_box(1337))));
    c.bench_function("regex_nfa_simd_step_min", |b| b.iter(|| regex_nfa_simd_step(black_box(0), black_box(0))));
    c.bench_function("regex_nfa_simd_step_max", |b| b.iter(|| regex_nfa_simd_step(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_aho_corasick_simd_step(c: &mut Criterion) {
    use bcinr_logic::algorithms::aho_corasick_simd_step::aho_corasick_simd_step;
    c.bench_function("aho_corasick_simd_step_avg", |b| b.iter(|| aho_corasick_simd_step(black_box(42), black_box(1337))));
    c.bench_function("aho_corasick_simd_step_min", |b| b.iter(|| aho_corasick_simd_step(black_box(0), black_box(0))));
    c.bench_function("aho_corasick_simd_step_max", |b| b.iter(|| aho_corasick_simd_step(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_suffix_array_step_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::suffix_array_step_branchless::suffix_array_step_branchless;
    c.bench_function("suffix_array_step_branchless_avg", |b| b.iter(|| suffix_array_step_branchless(black_box(42), black_box(1337))));
    c.bench_function("suffix_array_step_branchless_min", |b| b.iter(|| suffix_array_step_branchless(black_box(0), black_box(0))));
    c.bench_function("suffix_array_step_branchless_max", |b| b.iter(|| suffix_array_step_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_lcp_array_step_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::lcp_array_step_branchless::lcp_array_step_branchless;
    c.bench_function("lcp_array_step_branchless_avg", |b| b.iter(|| lcp_array_step_branchless(black_box(42), black_box(1337))));
    c.bench_function("lcp_array_step_branchless_min", |b| b.iter(|| lcp_array_step_branchless(black_box(0), black_box(0))));
    c.bench_function("lcp_array_step_branchless_max", |b| b.iter(|| lcp_array_step_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_burrows_wheeler_transform_step(c: &mut Criterion) {
    use bcinr_logic::algorithms::burrows_wheeler_transform_step::burrows_wheeler_transform_step;
    c.bench_function("burrows_wheeler_transform_step_avg", |b| b.iter(|| burrows_wheeler_transform_step(black_box(42), black_box(1337))));
    c.bench_function("burrows_wheeler_transform_step_min", |b| b.iter(|| burrows_wheeler_transform_step(black_box(0), black_box(0))));
    c.bench_function("burrows_wheeler_transform_step_max", |b| b.iter(|| burrows_wheeler_transform_step(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_move_to_front_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::move_to_front_branchless::move_to_front_branchless;
    c.bench_function("move_to_front_branchless_avg", |b| b.iter(|| move_to_front_branchless(black_box(42), black_box(1337))));
    c.bench_function("move_to_front_branchless_min", |b| b.iter(|| move_to_front_branchless(black_box(0), black_box(0))));
    c.bench_function("move_to_front_branchless_max", |b| b.iter(|| move_to_front_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_huffman_decode_table_step(c: &mut Criterion) {
    use bcinr_logic::algorithms::huffman_decode_table_step::huffman_decode_table_step;
    c.bench_function("huffman_decode_table_step_avg", |b| b.iter(|| huffman_decode_table_step(black_box(42), black_box(1337))));
    c.bench_function("huffman_decode_table_step_min", |b| b.iter(|| huffman_decode_table_step(black_box(0), black_box(0))));
    c.bench_function("huffman_decode_table_step_max", |b| b.iter(|| huffman_decode_table_step(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_prefix_sum_simd_u32x8(c: &mut Criterion) {
    use bcinr_logic::algorithms::prefix_sum_simd_u32x8::prefix_sum_simd_u32x8;
    c.bench_function("prefix_sum_simd_u32x8_avg", |b| b.iter(|| prefix_sum_simd_u32x8(black_box(42), black_box(1337))));
    c.bench_function("prefix_sum_simd_u32x8_min", |b| b.iter(|| prefix_sum_simd_u32x8(black_box(0), black_box(0))));
    c.bench_function("prefix_sum_simd_u32x8_max", |b| b.iter(|| prefix_sum_simd_u32x8(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_suffix_sum_simd_u32x8(c: &mut Criterion) {
    use bcinr_logic::algorithms::suffix_sum_simd_u32x8::suffix_sum_simd_u32x8;
    c.bench_function("suffix_sum_simd_u32x8_avg", |b| b.iter(|| suffix_sum_simd_u32x8(black_box(42), black_box(1337))));
    c.bench_function("suffix_sum_simd_u32x8_min", |b| b.iter(|| suffix_sum_simd_u32x8(black_box(0), black_box(0))));
    c.bench_function("suffix_sum_simd_u32x8_max", |b| b.iter(|| suffix_sum_simd_u32x8(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_delta_encode_simd_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::delta_encode_simd_u32::delta_encode_simd_u32;
    c.bench_function("delta_encode_simd_u32_avg", |b| b.iter(|| delta_encode_simd_u32(black_box(42), black_box(1337))));
    c.bench_function("delta_encode_simd_u32_min", |b| b.iter(|| delta_encode_simd_u32(black_box(0), black_box(0))));
    c.bench_function("delta_encode_simd_u32_max", |b| b.iter(|| delta_encode_simd_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_delta_decode_simd_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::delta_decode_simd_u32::delta_decode_simd_u32;
    c.bench_function("delta_decode_simd_u32_avg", |b| b.iter(|| delta_decode_simd_u32(black_box(42), black_box(1337))));
    c.bench_function("delta_decode_simd_u32_min", |b| b.iter(|| delta_decode_simd_u32(black_box(0), black_box(0))));
    c.bench_function("delta_decode_simd_u32_max", |b| b.iter(|| delta_decode_simd_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_branchless_stack_spsc(c: &mut Criterion) {
    use bcinr_logic::algorithms::branchless_stack_spsc::branchless_stack_spsc;
    c.bench_function("branchless_stack_spsc_avg", |b| b.iter(|| branchless_stack_spsc(black_box(42), black_box(1337))));
    c.bench_function("branchless_stack_spsc_min", |b| b.iter(|| branchless_stack_spsc(black_box(0), black_box(0))));
    c.bench_function("branchless_stack_spsc_max", |b| b.iter(|| branchless_stack_spsc(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_branchless_ring_buffer_mpmc(c: &mut Criterion) {
    use bcinr_logic::algorithms::branchless_ring_buffer_mpmc::branchless_ring_buffer_mpmc;
    c.bench_function("branchless_ring_buffer_mpmc_avg", |b| b.iter(|| branchless_ring_buffer_mpmc(black_box(42), black_box(1337))));
    c.bench_function("branchless_ring_buffer_mpmc_min", |b| b.iter(|| branchless_ring_buffer_mpmc(black_box(0), black_box(0))));
    c.bench_function("branchless_ring_buffer_mpmc_max", |b| b.iter(|| branchless_ring_buffer_mpmc(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_lockfree_skip_list_step(c: &mut Criterion) {
    use bcinr_logic::algorithms::lockfree_skip_list_step::lockfree_skip_list_step;
    c.bench_function("lockfree_skip_list_step_avg", |b| b.iter(|| lockfree_skip_list_step(black_box(42), black_box(1337))));
    c.bench_function("lockfree_skip_list_step_min", |b| b.iter(|| lockfree_skip_list_step(black_box(0), black_box(0))));
    c.bench_function("lockfree_skip_list_step_max", |b| b.iter(|| lockfree_skip_list_step(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_waitfree_queue_push(c: &mut Criterion) {
    use bcinr_logic::algorithms::waitfree_queue_push::waitfree_queue_push;
    c.bench_function("waitfree_queue_push_avg", |b| b.iter(|| waitfree_queue_push(black_box(42), black_box(1337))));
    c.bench_function("waitfree_queue_push_min", |b| b.iter(|| waitfree_queue_push(black_box(0), black_box(0))));
    c.bench_function("waitfree_queue_push_max", |b| b.iter(|| waitfree_queue_push(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_hazard_pointer_retire(c: &mut Criterion) {
    use bcinr_logic::algorithms::hazard_pointer_retire::hazard_pointer_retire;
    c.bench_function("hazard_pointer_retire_avg", |b| b.iter(|| hazard_pointer_retire(black_box(42), black_box(1337))));
    c.bench_function("hazard_pointer_retire_min", |b| b.iter(|| hazard_pointer_retire(black_box(0), black_box(0))));
    c.bench_function("hazard_pointer_retire_max", |b| b.iter(|| hazard_pointer_retire(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_epoch_based_reclamation_step(c: &mut Criterion) {
    use bcinr_logic::algorithms::epoch_based_reclamation_step::epoch_based_reclamation_step;
    c.bench_function("epoch_based_reclamation_step_avg", |b| b.iter(|| epoch_based_reclamation_step(black_box(42), black_box(1337))));
    c.bench_function("epoch_based_reclamation_step_min", |b| b.iter(|| epoch_based_reclamation_step(black_box(0), black_box(0))));
    c.bench_function("epoch_based_reclamation_step_max", |b| b.iter(|| epoch_based_reclamation_step(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_branchless_priority_queue_push(c: &mut Criterion) {
    use bcinr_logic::algorithms::branchless_priority_queue_push::branchless_priority_queue_push;
    c.bench_function("branchless_priority_queue_push_avg", |b| b.iter(|| branchless_priority_queue_push(black_box(42), black_box(1337))));
    c.bench_function("branchless_priority_queue_push_min", |b| b.iter(|| branchless_priority_queue_push(black_box(0), black_box(0))));
    c.bench_function("branchless_priority_queue_push_max", |b| b.iter(|| branchless_priority_queue_push(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_branchless_priority_queue_pop(c: &mut Criterion) {
    use bcinr_logic::algorithms::branchless_priority_queue_pop::branchless_priority_queue_pop;
    c.bench_function("branchless_priority_queue_pop_avg", |b| b.iter(|| branchless_priority_queue_pop(black_box(42), black_box(1337))));
    c.bench_function("branchless_priority_queue_pop_min", |b| b.iter(|| branchless_priority_queue_pop(black_box(0), black_box(0))));
    c.bench_function("branchless_priority_queue_pop_max", |b| b.iter(|| branchless_priority_queue_pop(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_disjoint_set_union_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::disjoint_set_union_branchless::disjoint_set_union_branchless;
    c.bench_function("disjoint_set_union_branchless_avg", |b| b.iter(|| disjoint_set_union_branchless(black_box(42), black_box(1337))));
    c.bench_function("disjoint_set_union_branchless_min", |b| b.iter(|| disjoint_set_union_branchless(black_box(0), black_box(0))));
    c.bench_function("disjoint_set_union_branchless_max", |b| b.iter(|| disjoint_set_union_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_graph_bfs_simd_step(c: &mut Criterion) {
    use bcinr_logic::algorithms::graph_bfs_simd_step::graph_bfs_simd_step;
    c.bench_function("graph_bfs_simd_step_avg", |b| b.iter(|| graph_bfs_simd_step(black_box(42), black_box(1337))));
    c.bench_function("graph_bfs_simd_step_min", |b| b.iter(|| graph_bfs_simd_step(black_box(0), black_box(0))));
    c.bench_function("graph_bfs_simd_step_max", |b| b.iter(|| graph_bfs_simd_step(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_graph_dfs_bit_parallel(c: &mut Criterion) {
    use bcinr_logic::algorithms::graph_dfs_bit_parallel::graph_dfs_bit_parallel;
    c.bench_function("graph_dfs_bit_parallel_avg", |b| b.iter(|| graph_dfs_bit_parallel(black_box(42), black_box(1337))));
    c.bench_function("graph_dfs_bit_parallel_min", |b| b.iter(|| graph_dfs_bit_parallel(black_box(0), black_box(0))));
    c.bench_function("graph_dfs_bit_parallel_max", |b| b.iter(|| graph_dfs_bit_parallel(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_shortest_path_bellman_ford_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::shortest_path_bellman_ford_branchless::shortest_path_bellman_ford_branchless;
    c.bench_function("shortest_path_bellman_ford_branchless_avg", |b| b.iter(|| shortest_path_bellman_ford_branchless(black_box(42), black_box(1337))));
    c.bench_function("shortest_path_bellman_ford_branchless_min", |b| b.iter(|| shortest_path_bellman_ford_branchless(black_box(0), black_box(0))));
    c.bench_function("shortest_path_bellman_ford_branchless_max", |b| b.iter(|| shortest_path_bellman_ford_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_page_rank_simd_step(c: &mut Criterion) {
    use bcinr_logic::algorithms::page_rank_simd_step::page_rank_simd_step;
    c.bench_function("page_rank_simd_step_avg", |b| b.iter(|| page_rank_simd_step(black_box(42), black_box(1337))));
    c.bench_function("page_rank_simd_step_min", |b| b.iter(|| page_rank_simd_step(black_box(0), black_box(0))));
    c.bench_function("page_rank_simd_step_max", |b| b.iter(|| page_rank_simd_step(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_triangle_count_bitset(c: &mut Criterion) {
    use bcinr_logic::algorithms::triangle_count_bitset::triangle_count_bitset;
    c.bench_function("triangle_count_bitset_avg", |b| b.iter(|| triangle_count_bitset(black_box(42), black_box(1337))));
    c.bench_function("triangle_count_bitset_min", |b| b.iter(|| triangle_count_bitset(black_box(0), black_box(0))));
    c.bench_function("triangle_count_bitset_max", |b| b.iter(|| triangle_count_bitset(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_clique_check_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::clique_check_branchless::clique_check_branchless;
    c.bench_function("clique_check_branchless_avg", |b| b.iter(|| clique_check_branchless(black_box(42), black_box(1337))));
    c.bench_function("clique_check_branchless_min", |b| b.iter(|| clique_check_branchless(black_box(0), black_box(0))));
    c.bench_function("clique_check_branchless_max", |b| b.iter(|| clique_check_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_topological_sort_step_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::topological_sort_step_branchless::topological_sort_step_branchless;
    c.bench_function("topological_sort_step_branchless_avg", |b| b.iter(|| topological_sort_step_branchless(black_box(42), black_box(1337))));
    c.bench_function("topological_sort_step_branchless_min", |b| b.iter(|| topological_sort_step_branchless(black_box(0), black_box(0))));
    c.bench_function("topological_sort_step_branchless_max", |b| b.iter(|| topological_sort_step_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_minimum_spanning_tree_prim_step(c: &mut Criterion) {
    use bcinr_logic::algorithms::minimum_spanning_tree_prim_step::minimum_spanning_tree_prim_step;
    c.bench_function("minimum_spanning_tree_prim_step_avg", |b| b.iter(|| minimum_spanning_tree_prim_step(black_box(42), black_box(1337))));
    c.bench_function("minimum_spanning_tree_prim_step_min", |b| b.iter(|| minimum_spanning_tree_prim_step(black_box(0), black_box(0))));
    c.bench_function("minimum_spanning_tree_prim_step_max", |b| b.iter(|| minimum_spanning_tree_prim_step(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_max_flow_edmonds_karp_step(c: &mut Criterion) {
    use bcinr_logic::algorithms::max_flow_edmonds_karp_step::max_flow_edmonds_karp_step;
    c.bench_function("max_flow_edmonds_karp_step_avg", |b| b.iter(|| max_flow_edmonds_karp_step(black_box(42), black_box(1337))));
    c.bench_function("max_flow_edmonds_karp_step_min", |b| b.iter(|| max_flow_edmonds_karp_step(black_box(0), black_box(0))));
    c.bench_function("max_flow_edmonds_karp_step_max", |b| b.iter(|| max_flow_edmonds_karp_step(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_bloom_filter_graph_visited(c: &mut Criterion) {
    use bcinr_logic::algorithms::bloom_filter_graph_visited::bloom_filter_graph_visited;
    c.bench_function("bloom_filter_graph_visited_avg", |b| b.iter(|| bloom_filter_graph_visited(black_box(42), black_box(1337))));
    c.bench_function("bloom_filter_graph_visited_min", |b| b.iter(|| bloom_filter_graph_visited(black_box(0), black_box(0))));
    c.bench_function("bloom_filter_graph_visited_max", |b| b.iter(|| bloom_filter_graph_visited(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_matrix_mul_simd_f32(c: &mut Criterion) {
    use bcinr_logic::algorithms::matrix_mul_simd_f32::matrix_mul_simd_f32;
    c.bench_function("matrix_mul_simd_f32_avg", |b| b.iter(|| matrix_mul_simd_f32(black_box(42), black_box(1337))));
    c.bench_function("matrix_mul_simd_f32_min", |b| b.iter(|| matrix_mul_simd_f32(black_box(0), black_box(0))));
    c.bench_function("matrix_mul_simd_f32_max", |b| b.iter(|| matrix_mul_simd_f32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_matrix_transpose_simd_f32(c: &mut Criterion) {
    use bcinr_logic::algorithms::matrix_transpose_simd_f32::matrix_transpose_simd_f32;
    c.bench_function("matrix_transpose_simd_f32_avg", |b| b.iter(|| matrix_transpose_simd_f32(black_box(42), black_box(1337))));
    c.bench_function("matrix_transpose_simd_f32_min", |b| b.iter(|| matrix_transpose_simd_f32(black_box(0), black_box(0))));
    c.bench_function("matrix_transpose_simd_f32_max", |b| b.iter(|| matrix_transpose_simd_f32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_vector_dot_product_simd_f32(c: &mut Criterion) {
    use bcinr_logic::algorithms::vector_dot_product_simd_f32::vector_dot_product_simd_f32;
    c.bench_function("vector_dot_product_simd_f32_avg", |b| b.iter(|| vector_dot_product_simd_f32(black_box(42), black_box(1337))));
    c.bench_function("vector_dot_product_simd_f32_min", |b| b.iter(|| vector_dot_product_simd_f32(black_box(0), black_box(0))));
    c.bench_function("vector_dot_product_simd_f32_max", |b| b.iter(|| vector_dot_product_simd_f32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_vector_cross_product_f32(c: &mut Criterion) {
    use bcinr_logic::algorithms::vector_cross_product_f32::vector_cross_product_f32;
    c.bench_function("vector_cross_product_f32_avg", |b| b.iter(|| vector_cross_product_f32(black_box(42), black_box(1337))));
    c.bench_function("vector_cross_product_f32_min", |b| b.iter(|| vector_cross_product_f32(black_box(0), black_box(0))));
    c.bench_function("vector_cross_product_f32_max", |b| b.iter(|| vector_cross_product_f32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_quaternion_mul_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::quaternion_mul_branchless::quaternion_mul_branchless;
    c.bench_function("quaternion_mul_branchless_avg", |b| b.iter(|| quaternion_mul_branchless(black_box(42), black_box(1337))));
    c.bench_function("quaternion_mul_branchless_min", |b| b.iter(|| quaternion_mul_branchless(black_box(0), black_box(0))));
    c.bench_function("quaternion_mul_branchless_max", |b| b.iter(|| quaternion_mul_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_aabb_intersect_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::aabb_intersect_branchless::aabb_intersect_branchless;
    c.bench_function("aabb_intersect_branchless_avg", |b| b.iter(|| aabb_intersect_branchless(black_box(42), black_box(1337))));
    c.bench_function("aabb_intersect_branchless_min", |b| b.iter(|| aabb_intersect_branchless(black_box(0), black_box(0))));
    c.bench_function("aabb_intersect_branchless_max", |b| b.iter(|| aabb_intersect_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_ray_triangle_intersect_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::ray_triangle_intersect_branchless::ray_triangle_intersect_branchless;
    c.bench_function("ray_triangle_intersect_branchless_avg", |b| b.iter(|| ray_triangle_intersect_branchless(black_box(42), black_box(1337))));
    c.bench_function("ray_triangle_intersect_branchless_min", |b| b.iter(|| ray_triangle_intersect_branchless(black_box(0), black_box(0))));
    c.bench_function("ray_triangle_intersect_branchless_max", |b| b.iter(|| ray_triangle_intersect_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_ray_sphere_intersect_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::ray_sphere_intersect_branchless::ray_sphere_intersect_branchless;
    c.bench_function("ray_sphere_intersect_branchless_avg", |b| b.iter(|| ray_sphere_intersect_branchless(black_box(42), black_box(1337))));
    c.bench_function("ray_sphere_intersect_branchless_min", |b| b.iter(|| ray_sphere_intersect_branchless(black_box(0), black_box(0))));
    c.bench_function("ray_sphere_intersect_branchless_max", |b| b.iter(|| ray_sphere_intersect_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_frustum_culling_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::frustum_culling_branchless::frustum_culling_branchless;
    c.bench_function("frustum_culling_branchless_avg", |b| b.iter(|| frustum_culling_branchless(black_box(42), black_box(1337))));
    c.bench_function("frustum_culling_branchless_min", |b| b.iter(|| frustum_culling_branchless(black_box(0), black_box(0))));
    c.bench_function("frustum_culling_branchless_max", |b| b.iter(|| frustum_culling_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_point_in_polygon_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::point_in_polygon_branchless::point_in_polygon_branchless;
    c.bench_function("point_in_polygon_branchless_avg", |b| b.iter(|| point_in_polygon_branchless(black_box(42), black_box(1337))));
    c.bench_function("point_in_polygon_branchless_min", |b| b.iter(|| point_in_polygon_branchless(black_box(0), black_box(0))));
    c.bench_function("point_in_polygon_branchless_max", |b| b.iter(|| point_in_polygon_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_convex_hull_monotone_chain_step(c: &mut Criterion) {
    use bcinr_logic::algorithms::convex_hull_monotone_chain_step::convex_hull_monotone_chain_step;
    c.bench_function("convex_hull_monotone_chain_step_avg", |b| b.iter(|| convex_hull_monotone_chain_step(black_box(42), black_box(1337))));
    c.bench_function("convex_hull_monotone_chain_step_min", |b| b.iter(|| convex_hull_monotone_chain_step(black_box(0), black_box(0))));
    c.bench_function("convex_hull_monotone_chain_step_max", |b| b.iter(|| convex_hull_monotone_chain_step(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_spatial_hash_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::spatial_hash_u32::spatial_hash_u32;
    c.bench_function("spatial_hash_u32_avg", |b| b.iter(|| spatial_hash_u32(black_box(42), black_box(1337))));
    c.bench_function("spatial_hash_u32_min", |b| b.iter(|| spatial_hash_u32(black_box(0), black_box(0))));
    c.bench_function("spatial_hash_u32_max", |b| b.iter(|| spatial_hash_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_quadtree_insert_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::quadtree_insert_branchless::quadtree_insert_branchless;
    c.bench_function("quadtree_insert_branchless_avg", |b| b.iter(|| quadtree_insert_branchless(black_box(42), black_box(1337))));
    c.bench_function("quadtree_insert_branchless_min", |b| b.iter(|| quadtree_insert_branchless(black_box(0), black_box(0))));
    c.bench_function("quadtree_insert_branchless_max", |b| b.iter(|| quadtree_insert_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_octree_insert_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::octree_insert_branchless::octree_insert_branchless;
    c.bench_function("octree_insert_branchless_avg", |b| b.iter(|| octree_insert_branchless(black_box(42), black_box(1337))));
    c.bench_function("octree_insert_branchless_min", |b| b.iter(|| octree_insert_branchless(black_box(0), black_box(0))));
    c.bench_function("octree_insert_branchless_max", |b| b.iter(|| octree_insert_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_hilbert_curve_encode_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::hilbert_curve_encode_u32::hilbert_curve_encode_u32;
    c.bench_function("hilbert_curve_encode_u32_avg", |b| b.iter(|| hilbert_curve_encode_u32(black_box(42), black_box(1337))));
    c.bench_function("hilbert_curve_encode_u32_min", |b| b.iter(|| hilbert_curve_encode_u32(black_box(0), black_box(0))));
    c.bench_function("hilbert_curve_encode_u32_max", |b| b.iter(|| hilbert_curve_encode_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_hilbert_curve_decode_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::hilbert_curve_decode_u32::hilbert_curve_decode_u32;
    c.bench_function("hilbert_curve_decode_u32_avg", |b| b.iter(|| hilbert_curve_decode_u32(black_box(42), black_box(1337))));
    c.bench_function("hilbert_curve_decode_u32_min", |b| b.iter(|| hilbert_curve_decode_u32(black_box(0), black_box(0))));
    c.bench_function("hilbert_curve_decode_u32_max", |b| b.iter(|| hilbert_curve_decode_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_z_order_curve_2d_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::z_order_curve_2d_u32::z_order_curve_2d_u32;
    c.bench_function("z_order_curve_2d_u32_avg", |b| b.iter(|| z_order_curve_2d_u32(black_box(42), black_box(1337))));
    c.bench_function("z_order_curve_2d_u32_min", |b| b.iter(|| z_order_curve_2d_u32(black_box(0), black_box(0))));
    c.bench_function("z_order_curve_2d_u32_max", |b| b.iter(|| z_order_curve_2d_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_bit_vector_compress_elias_fano(c: &mut Criterion) {
    use bcinr_logic::algorithms::bit_vector_compress_elias_fano::bit_vector_compress_elias_fano;
    c.bench_function("bit_vector_compress_elias_fano_avg", |b| b.iter(|| bit_vector_compress_elias_fano(black_box(42), black_box(1337))));
    c.bench_function("bit_vector_compress_elias_fano_min", |b| b.iter(|| bit_vector_compress_elias_fano(black_box(0), black_box(0))));
    c.bench_function("bit_vector_compress_elias_fano_max", |b| b.iter(|| bit_vector_compress_elias_fano(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_rank_select_dictionary_rrr(c: &mut Criterion) {
    use bcinr_logic::algorithms::rank_select_dictionary_rrr::rank_select_dictionary_rrr;
    c.bench_function("rank_select_dictionary_rrr_avg", |b| b.iter(|| rank_select_dictionary_rrr(black_box(42), black_box(1337))));
    c.bench_function("rank_select_dictionary_rrr_min", |b| b.iter(|| rank_select_dictionary_rrr(black_box(0), black_box(0))));
    c.bench_function("rank_select_dictionary_rrr_max", |b| b.iter(|| rank_select_dictionary_rrr(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_wavelet_tree_access_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::wavelet_tree_access_branchless::wavelet_tree_access_branchless;
    c.bench_function("wavelet_tree_access_branchless_avg", |b| b.iter(|| wavelet_tree_access_branchless(black_box(42), black_box(1337))));
    c.bench_function("wavelet_tree_access_branchless_min", |b| b.iter(|| wavelet_tree_access_branchless(black_box(0), black_box(0))));
    c.bench_function("wavelet_tree_access_branchless_max", |b| b.iter(|| wavelet_tree_access_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_succinct_bit_vector_rank(c: &mut Criterion) {
    use bcinr_logic::algorithms::succinct_bit_vector_rank::succinct_bit_vector_rank;
    c.bench_function("succinct_bit_vector_rank_avg", |b| b.iter(|| succinct_bit_vector_rank(black_box(42), black_box(1337))));
    c.bench_function("succinct_bit_vector_rank_min", |b| b.iter(|| succinct_bit_vector_rank(black_box(0), black_box(0))));
    c.bench_function("succinct_bit_vector_rank_max", |b| b.iter(|| succinct_bit_vector_rank(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_succinct_bit_vector_select(c: &mut Criterion) {
    use bcinr_logic::algorithms::succinct_bit_vector_select::succinct_bit_vector_select;
    c.bench_function("succinct_bit_vector_select_avg", |b| b.iter(|| succinct_bit_vector_select(black_box(42), black_box(1337))));
    c.bench_function("succinct_bit_vector_select_min", |b| b.iter(|| succinct_bit_vector_select(black_box(0), black_box(0))));
    c.bench_function("succinct_bit_vector_select_max", |b| b.iter(|| succinct_bit_vector_select(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_linear_congruential_generator_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::linear_congruential_generator_u64::linear_congruential_generator_u64;
    c.bench_function("linear_congruential_generator_u64_avg", |b| b.iter(|| linear_congruential_generator_u64(black_box(42), black_box(1337))));
    c.bench_function("linear_congruential_generator_u64_min", |b| b.iter(|| linear_congruential_generator_u64(black_box(0), black_box(0))));
    c.bench_function("linear_congruential_generator_u64_max", |b| b.iter(|| linear_congruential_generator_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_pcg_random_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::pcg_random_u64::pcg_random_u64;
    c.bench_function("pcg_random_u64_avg", |b| b.iter(|| pcg_random_u64(black_box(42), black_box(1337))));
    c.bench_function("pcg_random_u64_min", |b| b.iter(|| pcg_random_u64(black_box(0), black_box(0))));
    c.bench_function("pcg_random_u64_max", |b| b.iter(|| pcg_random_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_splitmix64_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::splitmix64_u64::splitmix64_u64;
    c.bench_function("splitmix64_u64_avg", |b| b.iter(|| splitmix64_u64(black_box(42), black_box(1337))));
    c.bench_function("splitmix64_u64_min", |b| b.iter(|| splitmix64_u64(black_box(0), black_box(0))));
    c.bench_function("splitmix64_u64_max", |b| b.iter(|| splitmix64_u64(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_xoroshiro128_plus(c: &mut Criterion) {
    use bcinr_logic::algorithms::xoroshiro128_plus::xoroshiro128_plus;
    c.bench_function("xoroshiro128_plus_avg", |b| b.iter(|| xoroshiro128_plus(black_box(42), black_box(1337))));
    c.bench_function("xoroshiro128_plus_min", |b| b.iter(|| xoroshiro128_plus(black_box(0), black_box(0))));
    c.bench_function("xoroshiro128_plus_max", |b| b.iter(|| xoroshiro128_plus(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_mersenne_twister_step_simd(c: &mut Criterion) {
    use bcinr_logic::algorithms::mersenne_twister_step_simd::mersenne_twister_step_simd;
    c.bench_function("mersenne_twister_step_simd_avg", |b| b.iter(|| mersenne_twister_step_simd(black_box(42), black_box(1337))));
    c.bench_function("mersenne_twister_step_simd_min", |b| b.iter(|| mersenne_twister_step_simd(black_box(0), black_box(0))));
    c.bench_function("mersenne_twister_step_simd_max", |b| b.iter(|| mersenne_twister_step_simd(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_reservoir_sample_weighted_simd(c: &mut Criterion) {
    use bcinr_logic::algorithms::reservoir_sample_weighted_simd::reservoir_sample_weighted_simd;
    c.bench_function("reservoir_sample_weighted_simd_avg", |b| b.iter(|| reservoir_sample_weighted_simd(black_box(42), black_box(1337))));
    c.bench_function("reservoir_sample_weighted_simd_min", |b| b.iter(|| reservoir_sample_weighted_simd(black_box(0), black_box(0))));
    c.bench_function("reservoir_sample_weighted_simd_max", |b| b.iter(|| reservoir_sample_weighted_simd(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_gaussian_noise_box_muller(c: &mut Criterion) {
    use bcinr_logic::algorithms::gaussian_noise_box_muller::gaussian_noise_box_muller;
    c.bench_function("gaussian_noise_box_muller_avg", |b| b.iter(|| gaussian_noise_box_muller(black_box(42), black_box(1337))));
    c.bench_function("gaussian_noise_box_muller_min", |b| b.iter(|| gaussian_noise_box_muller(black_box(0), black_box(0))));
    c.bench_function("gaussian_noise_box_muller_max", |b| b.iter(|| gaussian_noise_box_muller(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_poisson_noise_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::poisson_noise_branchless::poisson_noise_branchless;
    c.bench_function("poisson_noise_branchless_avg", |b| b.iter(|| poisson_noise_branchless(black_box(42), black_box(1337))));
    c.bench_function("poisson_noise_branchless_min", |b| b.iter(|| poisson_noise_branchless(black_box(0), black_box(0))));
    c.bench_function("poisson_noise_branchless_max", |b| b.iter(|| poisson_noise_branchless(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_halton_sampler_simd(c: &mut Criterion) {
    use bcinr_logic::algorithms::halton_sampler_simd::halton_sampler_simd;
    c.bench_function("halton_sampler_simd_avg", |b| b.iter(|| halton_sampler_simd(black_box(42), black_box(1337))));
    c.bench_function("halton_sampler_simd_min", |b| b.iter(|| halton_sampler_simd(black_box(0), black_box(0))));
    c.bench_function("halton_sampler_simd_max", |b| b.iter(|| halton_sampler_simd(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_fixed_point_log2(c: &mut Criterion) {
    use bcinr_logic::algorithms::fixed_point_log2::fixed_point_log2;
    c.bench_function("fixed_point_log2_avg", |b| b.iter(|| fixed_point_log2(black_box(42), black_box(1337))));
    c.bench_function("fixed_point_log2_min", |b| b.iter(|| fixed_point_log2(black_box(0), black_box(0))));
    c.bench_function("fixed_point_log2_max", |b| b.iter(|| fixed_point_log2(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_branchless_vtable_lookup(c: &mut Criterion) {
    use bcinr_logic::algorithms::branchless_vtable_lookup::branchless_vtable_lookup;
    c.bench_function("branchless_vtable_lookup_avg", |b| b.iter(|| branchless_vtable_lookup(black_box(42), black_box(1337))));
    c.bench_function("branchless_vtable_lookup_min", |b| b.iter(|| branchless_vtable_lookup(black_box(0), black_box(0))));
    c.bench_function("branchless_vtable_lookup_max", |b| b.iter(|| branchless_vtable_lookup(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_base64_decode_chunk4(c: &mut Criterion) {
    use bcinr_logic::algorithms::base64_decode_chunk4::base64_decode_chunk4;
    c.bench_function("base64_decode_chunk4_avg", |b| b.iter(|| base64_decode_chunk4(black_box(42), black_box(1337))));
    c.bench_function("base64_decode_chunk4_min", |b| b.iter(|| base64_decode_chunk4(black_box(0), black_box(0))));
    c.bench_function("base64_decode_chunk4_max", |b| b.iter(|| base64_decode_chunk4(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_unrolled_binary_search_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::unrolled_binary_search_u32::unrolled_binary_search_u32;
    c.bench_function("unrolled_binary_search_u32_avg", |b| b.iter(|| unrolled_binary_search_u32(black_box(42), black_box(1337))));
    c.bench_function("unrolled_binary_search_u32_min", |b| b.iter(|| unrolled_binary_search_u32(black_box(0), black_box(0))));
    c.bench_function("unrolled_binary_search_u32_max", |b| b.iter(|| unrolled_binary_search_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_utf8_validate_chunk8(c: &mut Criterion) {
    use bcinr_logic::algorithms::utf8_validate_chunk8::utf8_validate_chunk8;
    c.bench_function("utf8_validate_chunk8_avg", |b| b.iter(|| utf8_validate_chunk8(black_box(42), black_box(1337))));
    c.bench_function("utf8_validate_chunk8_min", |b| b.iter(|| utf8_validate_chunk8(black_box(0), black_box(0))));
    c.bench_function("utf8_validate_chunk8_max", |b| b.iter(|| utf8_validate_chunk8(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_hex_encode_chunk8(c: &mut Criterion) {
    use bcinr_logic::algorithms::hex_encode_chunk8::hex_encode_chunk8;
    c.bench_function("hex_encode_chunk8_avg", |b| b.iter(|| hex_encode_chunk8(black_box(42), black_box(1337))));
    c.bench_function("hex_encode_chunk8_min", |b| b.iter(|| hex_encode_chunk8(black_box(0), black_box(0))));
    c.bench_function("hex_encode_chunk8_max", |b| b.iter(|| hex_encode_chunk8(black_box(u64::MAX), black_box(u64::MAX))));
}

fn bench_bit_parallel_sort8_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::bit_parallel_sort8_u32::bit_parallel_sort8_u32;
    c.bench_function("bit_parallel_sort8_u32_avg", |b| b.iter(|| bit_parallel_sort8_u32(black_box(42), black_box(1337))));
    c.bench_function("bit_parallel_sort8_u32_min", |b| b.iter(|| bit_parallel_sort8_u32(black_box(0), black_box(0))));
    c.bench_function("bit_parallel_sort8_u32_max", |b| b.iter(|| bit_parallel_sort8_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

criterion_group!(benches,
    bench_parallel_bits_deposit_u64,
    bench_parallel_bits_extract_u64,
    bench_blsr_u64,
    bench_blsi_u64,
    bench_blsmsk_u64,
    bench_t1mskc_u64,
    bench_tzmsk_u64,
    bench_bext_u64,
    bench_bset_u64,
    bench_bclr_u64,
    bench_btst_u64,
    bench_popcount_u128,
    bench_reverse_bits_u128,
    bench_clmul_u64,
    bench_morton_encode_2d_u32,
    bench_morton_decode_2d_u32,
    bench_morton_encode_3d_u32,
    bench_gray_encode_u64,
    bench_gray_decode_u64,
    bench_parity_check_u128,
    bench_next_lexicographic_permutation_u64,
    bench_count_consecutive_set_bits_u64,
    bench_find_nth_set_bit_u128,
    bench_mask_range_u64,
    bench_rotate_left_u64,
    bench_rotate_right_u64,
    bench_funnel_shift_left_u64,
    bench_funnel_shift_right_u64,
    bench_bit_swap_u64,
    bench_gather_bits_u64,
    bench_scatter_bits_u64,
    bench_is_contiguous_mask_u64,
    bench_get_mask_boundary_low_u64,
    bench_get_mask_boundary_high_u64,
    bench_bit_matrix_transpose_8x8,
    bench_bit_matrix_transpose_64x64,
    bench_rank_u128,
    bench_select_u128,
    bench_weight_u64,
    bench_delta_swap_u64,
    bench_benes_network_u64,
    bench_bit_permute_step_u64,
    bench_compress_bits_u64,
    bench_expand_bits_u64,
    bench_crossbar_permute_u8x16,
    bench_mask_from_bool_slice,
    bench_bool_slice_from_mask,
    bench_bit_permute_identity_64,
    bench_is_subset_mask_u64,
    bench_mask_xor_reduce_u64,
    bench_mul_sat_u64,
    bench_div_sat_u64,
    bench_add_sat_i32,
    bench_sub_sat_i32,
    bench_mul_sat_i32,
    bench_abs_diff_u64,
    bench_abs_diff_i64,
    bench_avg_u64,
    bench_avg_ceil_u64,
    bench_clamp_i64,
    bench_lerp_sat_u8,
    bench_lerp_sat_u32,
    bench_norm_u32,
    bench_fp_mul_u32_q16,
    bench_fp_div_u32_q16,
    bench_fp_sqrt_u32_q16,
    bench_fp_sin_u32_q16,
    bench_fp_cos_u32_q16,
    bench_fp_atan2_u32_q16,
    bench_log2_u64_fixed,
    bench_exp2_u64_fixed,
    bench_sigmoid_sat_u32,
    bench_relu_u32,
    bench_leaky_relu_u32,
    bench_softmax_u32x4,
    bench_fast_inverse_sqrt_u32,
    bench_gcd_u64_branchless,
    bench_lcm_u64_branchless,
    bench_modular_add_u64,
    bench_modular_sub_u64,
    bench_modular_mul_u64,
    bench_is_prime_u64_branchless,
    bench_factorial_sat_u32,
    bench_binom_sat_u32,
    bench_pow_sat_u64,
    bench_clamped_scaling_u64,
    bench_branchless_signum_i64,
    bench_copy_sign_i64,
    bench_is_finite_fp32_branchless,
    bench_is_nan_fp32_branchless,
    bench_round_to_nearest_u32,
    bench_round_up_u32,
    bench_round_down_u32,
    bench_quantize_u32,
    bench_dequantize_u32,
    bench_weighted_avg_u32,
    bench_smoothstep_u32,
    bench_cubic_interpolate_u32,
    bench_manhattan_dist_u32x2,
    bench_euclidean_dist_sq_u32x2,
    bench_bitonic_sort_64u32,
    bench_odd_even_merge_sort_16u32,
    bench_halton_sequence_u32,
    bench_shuffle_fisher_yates_branchless,
    bench_bitonic_merge_u64x8,
    bench_sort_pairs_u32x4,
    bench_median3_u32,
    bench_median5_u32,
    bench_median9_u32,
    bench_top_k_u32x16,
    bench_rank_select_sort_u32,
    bench_counting_sort_branchless_u8,
    bench_radix_sort_step_branchless,
    bench_insertion_sort_branchless_fixed,
    bench_shear_sort_bitonic_2d,
    bench_green_sorting_network_16,
    bench_permute_u32x8,
    bench_inverse_permute_u32x8,
    bench_is_sorted_branchless_u32,
    bench_lex_compare_u8_slices_branchless,
    bench_stable_partition_branchless,
    bench_rotate_slice_branchless,
    bench_reverse_slice_branchless,
    bench_next_combination_u64,
    bench_random_permutation_fixed_seed,
    bench_sort_index_u32x8,
    bench_merge_u32_slices_branchless,
    bench_unique_branchless_u32,
    bench_lower_bound_branchless_u32,
    bench_upper_bound_branchless_u32,
    bench_equal_range_branchless_u32,
    bench_search_eytzinger_u32,
    bench_search_van_emde_boas,
    bench_binary_search_v_u32x4,
    bench_linear_search_simd_u8,
    bench_find_first_of_branchless,
    bench_find_last_of_branchless,
    bench_mismatch_branchless_u8,
    bench_partial_sort_branchless_k,
    bench_nth_element_branchless,
    bench_is_permutation_branchless,
    bench_set_difference_branchless,
    bench_set_symmetric_difference_branchless,
    bench_set_intersection_branchless,
    bench_set_union_branchless,
    bench_min_element_branchless_u32,
    bench_max_element_branchless_u32,
    bench_minmax_element_branchless_u32,
    bench_clamp_slice_branchless,
    bench_normalize_slice_branchless,
    bench_murmur3_x64_128,
    bench_xxhash64,
    bench_xxh3_64,
    bench_cityhash64,
    bench_farmhash64,
    bench_spookyhash_v2_128,
    bench_metrohash64,
    bench_siphash_2_4_branchless,
    bench_highwayhash_64,
    bench_clhash,
    bench_pearson_hash_u8,
    bench_knuth_hash_u64,
    bench_fibonacci_hash_u64,
    bench_zobrist_hash_64,
    bench_perfect_hash_lookup_u32,
    bench_minhash_u64_k,
    bench_hyperloglog_add_u64,
    bench_hyperloglog_merge,
    bench_count_min_sketch_add,
    bench_count_min_sketch_query,
    bench_bloom_filter_add_u64,
    bench_bloom_filter_query_u64,
    bench_cuckoo_filter_add_u64,
    bench_quotient_filter_add_u64,
    bench_t_digest_add_u32,
    bench_heavy_keepers_add,
    bench_space_saving_add,
    bench_misra_gries_add,
    bench_reservoir_sample_branchless,
    bench_weighted_reservoir_sample,
    bench_consistent_hash_jump_u64,
    bench_consistent_hash_maglev,
    bench_bloom_filter_intersect,
    bench_bloom_filter_union,
    bench_hashing_trick_u64,
    bench_locality_sensitive_hash_euclidean,
    bench_locality_sensitive_hash_cosine,
    bench_k_independent_hash_gen,
    bench_rolling_hash_rabin_karp,
    bench_rolling_hash_buzhash,
    bench_rolling_hash_gear,
    bench_content_defined_chunking_branchless,
    bench_cyclic_redundancy_check_crc32c,
    bench_cyclic_redundancy_check_crc64,
    bench_adler32_branchless,
    bench_fletcher32_branchless,
    bench_bsd_checksum_u16,
    bench_internet_checksum_u16,
    bench_duffs_device_simd_unroll,
    bench_perfect_hash_build_static,
    bench_base64_encode_simd,
    bench_base64_decode_simd,
    bench_hex_encode_simd,
    bench_hex_decode_simd,
    bench_base32_encode_rfc4648,
    bench_base85_encode_ascii85,
    bench_leb128_encode_u64,
    bench_leb128_decode_u64,
    bench_varint_encode_simd,
    bench_varint_decode_simd,
    bench_bitpacking_encode_u32_k,
    bench_bitpacking_decode_u32_k,
    bench_zigzag_encode_i64,
    bench_zigzag_decode_i64,
    bench_utf8_to_utf16_simd,
    bench_utf16_to_utf8_simd,
    bench_utf8_to_utf32_simd,
    bench_ascii_to_lowercase_simd,
    bench_ascii_to_uppercase_simd,
    bench_is_alphanumeric_simd_u8x16,
    bench_is_digit_simd_u8x16,
    bench_is_space_simd_u8x16,
    bench_trim_whitespace_branchless,
    bench_split_lines_simd,
    bench_csv_scan_row_simd,
    bench_json_find_string_escapes_simd,
    bench_json_find_structural_simd,
    bench_levenshtein_dist_branchless,
    bench_hamming_dist_simd,
    bench_jaro_winkler_branchless,
    bench_soundex_encode_branchless,
    bench_metaphone_encode_branchless,
    bench_url_encode_branchless,
    bench_url_decode_branchless,
    bench_punycode_encode_branchless,
    bench_simd_strstr_branchless,
    bench_simd_memchr_u8x16,
    bench_simd_memrchr_u8x16,
    bench_wildcard_match_branchless,
    bench_regex_nfa_simd_step,
    bench_aho_corasick_simd_step,
    bench_suffix_array_step_branchless,
    bench_lcp_array_step_branchless,
    bench_burrows_wheeler_transform_step,
    bench_move_to_front_branchless,
    bench_huffman_decode_table_step,
    bench_prefix_sum_simd_u32x8,
    bench_suffix_sum_simd_u32x8,
    bench_delta_encode_simd_u32,
    bench_delta_decode_simd_u32,
    bench_branchless_stack_spsc,
    bench_branchless_ring_buffer_mpmc,
    bench_lockfree_skip_list_step,
    bench_waitfree_queue_push,
    bench_hazard_pointer_retire,
    bench_epoch_based_reclamation_step,
    bench_branchless_priority_queue_push,
    bench_branchless_priority_queue_pop,
    bench_disjoint_set_union_branchless,
    bench_graph_bfs_simd_step,
    bench_graph_dfs_bit_parallel,
    bench_shortest_path_bellman_ford_branchless,
    bench_page_rank_simd_step,
    bench_triangle_count_bitset,
    bench_clique_check_branchless,
    bench_topological_sort_step_branchless,
    bench_minimum_spanning_tree_prim_step,
    bench_max_flow_edmonds_karp_step,
    bench_bloom_filter_graph_visited,
    bench_matrix_mul_simd_f32,
    bench_matrix_transpose_simd_f32,
    bench_vector_dot_product_simd_f32,
    bench_vector_cross_product_f32,
    bench_quaternion_mul_branchless,
    bench_aabb_intersect_branchless,
    bench_ray_triangle_intersect_branchless,
    bench_ray_sphere_intersect_branchless,
    bench_frustum_culling_branchless,
    bench_point_in_polygon_branchless,
    bench_convex_hull_monotone_chain_step,
    bench_spatial_hash_u32,
    bench_quadtree_insert_branchless,
    bench_octree_insert_branchless,
    bench_hilbert_curve_encode_u32,
    bench_hilbert_curve_decode_u32,
    bench_z_order_curve_2d_u32,
    bench_bit_vector_compress_elias_fano,
    bench_rank_select_dictionary_rrr,
    bench_wavelet_tree_access_branchless,
    bench_succinct_bit_vector_rank,
    bench_succinct_bit_vector_select,
    bench_linear_congruential_generator_u64,
    bench_pcg_random_u64,
    bench_splitmix64_u64,
    bench_xoroshiro128_plus,
    bench_mersenne_twister_step_simd,
    bench_reservoir_sample_weighted_simd,
    bench_gaussian_noise_box_muller,
    bench_poisson_noise_branchless,
    bench_halton_sampler_simd,
    bench_fixed_point_log2,
    bench_branchless_vtable_lookup,
    bench_base64_decode_chunk4,
    bench_unrolled_binary_search_u32,
    bench_utf8_validate_chunk8,
    bench_hex_encode_chunk8,
    bench_bit_parallel_sort8_u32,
);
criterion_main!(benches);
