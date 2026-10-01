# Benchmark Baseline — v26.9.15 wave, Lane 1

- **Date:** 2026-10-01 (all runs executed 2026-10-01, this session)
- **Subject:** branch `release/26.9.15`, HEAD `ee2c907b28bdb2c6f176a74fb32c500eea4213bc`
  (working tree was shared with concurrent lanes: pre-existing dirty `Cargo.lock`,
  `README.md`, `docs/sa2a.md`; sibling lanes landed `crates/bcinr-logic/benches/`,
  `crates/bcinr-cmca/benches/cascade_bench.rs`, `crates/bcinr-guarded/benches/`,
  `crates/bcinr-mfw-ir/benches/` during the session — all executed as found, not modified)
- **Toolchain:** rustc 1.99.0-nightly (daf2e5e18 2026-07-13), nightly channel per `rust-toolchain.toml`
- **Host:** Apple M3 Max, 16 cores, 48 GiB (see `METHODOLOGY.md` for full facts and caveats)
- **Harness:** divan (`harness = false`) everywhere except `encode_unicode_patch` (nightly `libtest` bench)

Standing: every table row below is an observed execution this session. Command + exit stated per suite.
Numbers are divan **mean** (and fastest where noted) unless the harness column says `libtest` (ns/iter ± error).

## 1. Suite-level index (machine-readable)

| crate | bench_target | command | harness | benches_run | wall_s | exit | date |
|---|---|---|---|---|---|---|---|
| bcinr-logic | logic_kernels_bench | `cargo bench -p bcinr-logic --features bench` | divan | 28 | not recorded (exit observed) | 0 | 2026-10-01 |
| bcinr-powl | phase3_scheduler | `cargo bench -p bcinr-powl` | divan | 8 | 43.07 (all 4 suites) | 0 | 2026-10-01 |
| bcinr-powl | powl_pipeline_bench | `cargo bench -p bcinr-powl` | divan | 9 | (same run) | 0 | 2026-10-01 |
| bcinr-powl | powl_quick_bench | `cargo bench -p bcinr-powl` | divan | 5 | (same run; suite wall 61.03 ms) | 0 | 2026-10-01 |
| bcinr-powl | receipt_bench | `cargo bench -p bcinr-powl` | divan | 10 | (same run) | 0 | 2026-10-01 |
| bcinr-pddl | pddl_80_20 | `cargo bench -p bcinr-pddl` | divan | 1 (raw per-stage prints) | 28.49 (all 3 suites) | 0 | 2026-10-01 |
| bcinr-pddl | phase1_temporal | `cargo bench -p bcinr-pddl` | divan | 9 | (same run) | 0 | 2026-10-01 |
| bcinr-pddl | scaling | `cargo bench -p bcinr-pddl` | divan | 35 | (same run) | 0 | 2026-10-01 |
| bcinr-pddl | pddl_80_20 [mfw-planner] | `cargo bench -p bcinr-pddl --features mfw-planner --bench pddl_80_20` | divan | 12 | not recorded | 0 | 2026-10-01 |
| bcinr-cmca | allocation_bench (default feats) | `cargo bench -p bcinr-cmca` | divan | 23 (escort compiled out) | 38.46 | 0 | 2026-10-01 |
| bcinr-cmca | allocation_bench (alloc feats) | `cargo bench -p bcinr-cmca --features alloc` | divan | 25 | 33.04 | 0 | 2026-10-01 |
| bcinr-cmca | cascade_bench (alloc feats) | `cargo bench -p bcinr-cmca --features alloc` | divan | 6 | (same run) | 0 | 2026-10-01 |
| bcinr-guarded | guarded_classify_bench | `cargo bench -p bcinr-guarded` | divan | 9 | 9.85 | 0 | 2026-10-01 |
| bcinr-mfw-ir | mfw_ir_bench | `cargo bench -p bcinr-mfw-ir` | divan | 13 | 2.08 | 0 | 2026-10-01 |
| encode_unicode_patch | length | `cargo bench -p encode_unicode` | libtest (nightly) | 30 measured | 41.67 (length 36.79) | 0 | 2026-10-01 |
| encode_unicode_patch | multiiterators | `cargo bench -p encode_unicode` | libtest (nightly) | 0 measured / 11 env-skipped | (same run) | 0 | 2026-10-01 |
| tools/bcinr-bench-auditor | coverage audit | `cargo run -p bcinr-bench-auditor` (cwd = workspace root) | n/a | n/a | 1.5 | **1** | 2026-10-01 |

## 2. bcinr-logic — `logic_kernels_bench` (divan, `--features bench`)

One representative public kernel per kernel family; 64-word bitset fixtures keyed off the
splitmix64 golden-gamma constant. Suite landed by a sibling lane 2026-10-01 08:24Z; executed as found.

| benchmark | measures (public fn) | mean | fastest | iters | note |
|---|---|---|---|---|---|
| bitset_any_bit_set_64_words | `bitset::any_bit_set_u64_slice` | 5.007 ns | 4.935 ns | 102400 | |
| bitset_intersect_64_words | `bitset::intersect` (+caller copy, part of contract) | 27.05 ns | 26.66 ns | 25600 | |
| bitset_parity_64_words | `bitset::parity_u64_slice` | 5.18 ns | 4.935 ns | 102400 | |
| bitset_rank_u64 | `bitset::rank_u64` | 0.361 ns | 0.352 ns | 819200 | sub-ns, see caveat C1 |
| bitset_select_bit_u64 | `bitset::select_bit_u64` | 33.31 ns | 32.84 ns | 12800 | |
| bitset_union_64_words | `bitset::union_u64_slices` | 27.65 ns | 27.31 ns | 25600 | |
| fix_add_sat | `fix` saturating add | 0.278 ns | 0.251 ns | 819200 | sub-ns |
| fix_clamp_u32 | `fix` clamp | 2.841 ns | 2.819 ns | 204800 | |
| fix_ilog2_u32 | `fix` ilog2 | 0.082 ns | 0.078 ns | 1638400 | sub-ns |
| fix_isqrt_u32 | `fix` isqrt | 3.055 ns | 3.022 ns | 204800 | |
| fix_q16_div | Q16.16 div | 0.408 ns | 0.393 ns | 819200 | sub-ns |
| fix_q16_log2 | Q16.16 log2 | 0.368 ns | 0.358 ns | 819200 | sub-ns |
| fix_q16_mul | Q16.16 mul | 0.193 ns | 0.185 ns | 819200 | sub-ns |
| fix_q16_recip | Q16.16 recip | 2.207 ns | 2.168 ns | 204800 | |
| mask_abs_i32 | `mask::abs_i32` | 0.079 ns | 0.075 ns | 1638400 | sub-ns |
| mask_eq_mask_u32 | `mask::eq_mask_u32` | 0.291 ns | 0.256 ns | 819200 | sub-ns |
| mask_lt_mask_u32 | `mask::lt_mask_u32` | 0.297 ns | 0.246 ns | 819200 | sub-ns |
| mask_max_u32 | `mask::max_u32` | 0.269 ns | 0.246 ns | 819200 | sub-ns |
| mask_min_u32 | `mask::min_u32` | 0.266 ns | 0.241 ns | 819200 | sub-ns |
| mask_select_u64 | `mask::select_u64` | 0.409 ns | 0.378 ns | 819200 | sub-ns |
| reduce_horizontal_sum_u8x8 | `reduce::horizontal_sum_u8x8` | 0.258 ns | 0.251 ns | 819200 | sub-ns |
| reduce_min_u32_64_lanes | `reduce::reduce_min_u32` | 94.67 ns | 93.39 ns | 6400 | |
| reduce_sum_u64_64_lanes | `reduce::reduce_sum_u64` | 4.281 ns | 4.243 ns | 102400 | |
| reduce_swar_count_eq_u8 | `reduce::swar_count_eq_u8` | 0.414 ns | 0.388 ns | 819200 | sub-ns |
| scan_find_byte_mask | `scan::find_byte_mask` | 11.42 ns | 11.28 ns | 51200 | |
| scan_gate | `scan::scan_gate` | 0.073 ns | 0.07 ns | 1638400 | sub-ns |
| swar_mask_ones | `swar::swar_mask_ones` | 0.073 ns | 0.07 ns | 1638400 | sub-ns |
| swar_phd_gate | `swar::swar_phd_gate` | 0.063 ns | 0.057 ns | 1638400 | sub-ns |

## 3. bcinr-powl (divan, default features, one `cargo bench -p bcinr-powl` run, exit 0, wall 43.07 s)

### phase3_scheduler

| benchmark | mean | fastest | slowest |
|---|---|---|---|
| scheduler_tick_complex_dependencies | 2.341 µs | 1.665 µs | 40.91 µs |
| scheduler_tick_linear_sequence | 1.613 µs | 1.53 µs | 2.009 µs |
| scheduler_tick_single_op | 1.51 µs | 1.437 µs | 1.572 µs |
| scheduler_tick_wide_parallelism | 1.883 µs | 1.77 µs | 4.687 µs |
| full_pipeline_linear_sequence | 5.22 µs | 4.916 µs | 14.12 µs |
| seal_receipt_linear | 5.013 µs | 4.832 µs | 7.249 µs |
| validate_complex_dependencies | 6.307 µs | 5.832 µs | 8.832 µs |
| validate_linear_sequence | 5.069 µs | 4.79 µs | 6.541 µs |

### powl_pipeline_bench

| benchmark | mean | fastest | slowest |
|---|---|---|---|
| compile/mixed | 2.318 µs | 1.624 µs | 41.12 µs |
| compile/partial_order | 1.692 µs | 1.614 µs | 6.395 µs |
| compile/sequence | 1.565 µs | 1.52 µs | 1.999 µs |
| end_to_end/compile_schedule_log_seal_validate | 5.498 µs | 4.874 µs | 37.99 µs |
| ocel/record_and_seal_sequence | 4.347 µs | 4.207 µs | 5.374 µs |
| ocel/seal_receipt | 5.243 µs | 5.082 µs | 7.624 µs |
| ocel/validate_against_tape | 3.537 µs | 3.437 µs | 3.666 µs |
| schedule/partial_order_to_completion | 1.67 µs | 1.603 µs | 1.749 µs |
| schedule/sequence_to_completion | 1.551 µs | 1.509 µs | 1.926 µs |

### powl_quick_bench (sample_count=10; suite wall clock 61.03 ms)

| benchmark | mean | fastest | slowest |
|---|---|---|---|
| compile_powl_bench/4 | 3.1 µs | 1.774 µs | 10.28 µs |
| compile_powl_bench/16 | 2.641 µs | 2.491 µs | 3.383 µs |
| compile_powl_bench/64 | 5.077 µs | 4.874 µs | 5.624 µs |
| scheduler_legacy_linear_16ops | 2.856 µs | 2.728 µs | 3.324 µs |
| scheduler_wired_linear_16ops | 5.182 µs | 5.008 µs | 5.441 µs |

### receipt_bench

| benchmark | mean | fastest | throughput (mean) |
|---|---|---|---|
| chain_1_frame_blake3 | 328.2 ns | 319.9 ns | 3.046 Mitem/s |
| chain_100_frames_rolling | 24.48 µs | 24.16 µs | 4.084 Mitem/s |
| conformance_check_fail | 1.031 ns | 1.005 ns | — |
| conformance_check_pass | 1.03 ns | 1.016 ns | — |
| denial_to_fired_mask | 7.292 ns | 7.201 ns | — |
| emit_8_objects | 8.136 ns | 6.101 ns | 122.9 Mitem/s |
| emit_no_objects | 8.678 ns | 7.566 ns | 115.2 Mitem/s |
| emit_sla_breach | 8.619 ns | 7.566 ns | 116 Mitem/s |
| replay_10_frames | 11.46 ns | 11.31 ns | 872.5 Mitem/s |
| replay_64_frames_max | 61.1 ns | 60.45 ns | 1.047 Gitem/s |

## 4. bcinr-pddl (divan)

### phase1_temporal (default features)

| benchmark | mean | fastest | slowest |
|---|---|---|---|
| find_plan_no_deadline | 10.53 µs | 7.374 µs | 223.2 µs |
| find_plan_with_loose_deadline | 7.479 µs | 7.249 µs | 12.04 µs |
| find_plan_with_maintenance_window | 9.055 µs | 8.666 µs | 15.99 µs |
| find_plan_with_tight_deadline | 7.505 µs | 7.29 µs | 10.49 µs |
| request_lease_nonoverlapping_exclusive | 281.2 ns | 270.3 ns | 629.7 ns |
| request_lease_overlapping_exclusive | 271.3 ns | 265.2 ns | 286 ns |
| request_lease_shared_exceeds_capacity | 392.7 ns | 385 ns | 489.2 ns |
| request_lease_shared_within_capacity | 286.2 ns | 283.4 ns | 301.7 ns |
| sequential_nonoverlapping_leases | 2.446 µs | 2.208 µs | 17.49 µs |

### scaling (default features; per-stage, N objects)

| bench | N=8 | N=16 | N=32 | N=64 | N=128 | N=256 | N=512 (mean) |
|---|---|---|---|---|---|---|---|
| bench_ir | 18.05 µs | 24.47 µs | 37.91 µs | 63.31 µs | 116.4 µs | 228.6 µs | 439 µs |
| bench_ground | 7.481 µs | 15.17 µs | 29.98 µs | 58.02 µs | 117.7 µs | 237.4 µs | 548.2 µs |
| bench_solve | 6.415 µs | 13.85 µs | 27.29 µs | 57.19 µs | 114.6 µs | 220.1 µs | 420.7 µs |
| bench_powl | 1.514 µs | 3.363 µs | 7.975 µs | 17.42 µs | 5.299 ns | 5.27 ns | 5.297 ns |
| bench_e2e | 29.83 µs | 45.73 µs | 82.44 µs | 160.9 µs | 294.9 µs | 595.9 µs | 1.341 ms |

`bench_powl` N>=128 rows measure the documented O(1) typed-refusal early exit
(`MAX_POWL_TAPE_STEPS`) of `temporal_plan_to_powl_tape`, not tape lowering — see DEF-3.

### pddl_80_20 (default features; divan value cell is the bench's own per-stage print — see DEF-2)

| benchmark | mean (divan) | per-stage over n=800 prints (min/avg/max) |
|---|---|---|
| classical/todo_dependencies | 21.33 µs | IR 4/8.995/179 µs; Ground 0/0.984/125 µs; Solve 1/1.66/11 µs; POWL 0 µs (classical path has no POWL projection) |

### pddl_80_20 (variant run: `--features mfw-planner --bench pddl_80_20`, exit 0)

| benchmark | mean (divan) |
|---|---|
| classical/todo_dependencies | 21.33 µs |
| composed/composed_crown_5_feature | 26.09 µs |
| composed/composed_enterprise_rollout | 18.61 µs |
| constraints/compliant_rollout | 11.87 µs |
| derived/derived_readiness | 15.27 µs |
| ingress/rdf_direct_to_ir | 6.237 µs |
| ingress/rdf_via_text_to_ir | 9.66 µs |
| numeric/budgeted_migration | 15.99 µs |
| production_pipeline/classical_end_to_end | 50.85 µs |
| production_pipeline/temporal_unsupported_refusal | 8.94 µs |
| temporal/deploy_independent_services | 15.54 µs |
| til/maintenance_windows | 11.57 µs |

POWL stage printed 0 µs on 799/800 iterations in both variants (max 1 µs): the
production POWL runtime is not on these fixtures' measured path; do not read the
`POWL:` field as the cost of `compile_powl2`.

## 5. bcinr-cmca (divan)

### allocation_bench — default features (exit 0, wall 38.46 s; escort module compiled out)

| benchmark | mean | fastest |
|---|---|---|
| allocate_kernel | 121.8 µs | 118.4 µs |
| allocate_single_lens_query | 10.89 µs | 10.49 µs |
| power_fractional | 9.372 ns | 9.087 ns |
| decomposition/nn_add_half_half | 1.171 ns | 1.131 ns |
| decomposition/nn_div_one_over_eighth | 10.49 ns | 10.3 ns |
| decomposition/nn_div_one_over_one | 10.46 ns | 10.22 ns |
| decomposition/nn_div_one_over_two | 10.51 ns | 10.38 ns |
| decomposition/nn_div_zero_over_one | 10.58 ns | 10.3 ns |
| decomposition/nn_log2_eighth | 1.537 ns | 1.507 ns |
| decomposition/nn_log2_one | 1.543 ns | 1.517 ns |
| decomposition/nn_log2_three | 1.539 ns | 1.507 ns |
| decomposition/nn_mul_half_half | 1.295 ns | 1.263 ns |
| decomposition/nn_mul_three_half | 1.314 ns | 1.283 ns |
| decomposition/sf_exp2_minus_1_5 | 2.198 ns | 2.148 ns |
| decomposition/sf_exp2_plus_0_5 | 2.186 ns | 2.148 ns |
| decomposition/sf_exp2_zero | 2.152 ns | 2.107 ns |
| decomposition/sf_exp_minus_0_5 | 2.52 ns | 2.473 ns |
| decomposition/sf_exp_zero | 2.479 ns | 2.412 ns |
| decomposition/sf_mul_half_minus_1 | 1.447 ns | 1.416 ns |
| decomposition/topology_acyclic_flat | 1.812 µs | 1.77 µs |
| decomposition/throughput/nn_div_indep8 | 62.96 ns | 62.14 ns |
| decomposition/throughput/nn_log2_indep8 | 4.429 ns | 4.365 ns |
| decomposition/throughput/nn_mul_indep8 | 3.639 ns | 3.511 ns |
| decomposition/throughput/sf_exp2_indep8 | 5.369 ns | 5.261 ns |

### allocation_bench — `--features alloc` (exit 0, wall 33.04 s; adds escort module)

Same 23 rows within run-to-run noise (allocate_kernel mean 129.7 µs vs 121.8 µs; ~7%,
see METHODOLOGY caveats), plus:

| benchmark | mean | fastest |
|---|---|---|
| escort/escort_exact_integer_q | 92.68 ns | 88.8 ns |
| escort/escort_fractional_q | 113.5 ns | 106.4 ns |

### cascade_bench — `--features alloc` (exit 0)

| benchmark | mean | fastest |
|---|---|---|
| admit_fixed_clean | 1.018 ns | 0.998 ns |
| consequence_mass_lens_0 | 1.547 µs | 1.04 µs |
| consequence_mass_lens_1 | 1.086 µs | 1.02 µs |
| consequence_mass_traced_full_tree | 1.175 µs | 1.114 µs |
| escort_weight_negative_lens | 4.292 ns | 4.203 ns |
| escort_weight_positive_lens | 3.596 ns | 3.552 ns |

## 6. bcinr-guarded — `guarded_classify_bench` (divan, exit 0, wall 9.85 s)

| benchmark | mean | fastest | throughput (mean) |
|---|---|---|---|
| classify_activity | 1.074 ns | 1.059 ns | — |
| classify_all_cascade_refusals | 4.79 ns | 4.69 ns | 2.087 Gitem/s |
| classify_all_refusal_reasons | 6.949 ns | 6.237 ns | 1.439 Gitem/s |
| classify_all_six_constructors | 4.362 ns | 4.284 ns | 1.375 Gitem/s |
| classify_choice_graph | 1.075 ns | 1.059 ns | — |
| classify_do_redo | 1.108 ns | 1.059 ns | — |
| classify_partial_order | 1.075 ns | 1.06 ns | — |
| classify_sequence | 1.074 ns | 1.059 ns | — |
| classify_silent | 1.074 ns | 1.059 ns | — |

## 7. bcinr-mfw-ir — `mfw_ir_bench` (divan, exit 0, wall 2.08 s)

| benchmark | mean | fastest | slowest |
|---|---|---|---|
| advance_epoch_closed | 848 ns | 166.6 ns | 30.95 µs |
| advance_epoch_successor | 643.8 ns | 374.6 ns | 14.79 µs |
| digest_hash_64b | 77.15 ns | 75.81 ns | 79.72 ns |
| digest_mix | 234.4 ns | 230.1 ns | 240.5 ns |
| dme_epoch_root_4_residuals | 918.8 ns | 895.3 ns | 1.536 µs |
| event_set_contains_hit | 1.429 ns | 1.416 ns | 1.456 ns |
| event_set_contains_miss | 1.467 ns | 1.416 ns | 3.359 ns |
| event_set_insert_single | 2.855 ns | 2.819 ns | 2.901 ns |
| event_set_intersects | 1.789 ns | 1.761 ns | 1.843 ns |
| event_set_is_subset_of | 4.689 ns | 4.446 ns | 6.318 ns |
| event_set_iter_stable_full | 322.1 ns | 314.7 ns | 332.9 ns |
| event_set_len | 1.787 ns | 1.761 ns | 1.822 ns |
| event_set_union | 2.014 ns | 1.985 ns | 2.291 ns |

## 8. encode_unicode_patch (nightly libtest bench, `cargo bench -p encode_unicode`; exit 0, wall 41.67 s)

`length` (30 measured, ns/iter ± error). Iteration cost includes scanning a multi-KB text fixture.

| benchmark | ns/iter | ± |
|---|---|---|
| utf16_is_leading_surrogate_ascii | 311,645.70 | 117,829.34 |
| utf16_is_leading_surrogate_en | 252,003.39 | 47,482.10 |
| utf16_is_leading_surrogate_es | 305,514.06 | 184,180.45 |
| utf16_is_leading_surrogate_ru | 412,619.27 | 935,811.02 |
| utf16_is_leading_surrogate_zh | 441,504.94 | 873,267.66 |
| utf16_needs_extra_unit_ascii | 320,627.05 | 32,753.70 |
| utf16_needs_extra_unit_en | 320,260.40 | 221,284.53 |
| utf16_needs_extra_unit_es | 248,639.58 | 4,520.83 |
| utf16_needs_extra_unit_ru | 248,404.20 | 2,821.29 |
| utf16_needs_extra_unit_zh | 287,510.40 | 31,977.31 |
| utf16char_len_ascii | 108,802.09 | 1,139.85 |
| utf16char_len_en | 108,779.63 | 1,351.39 |
| utf16char_len_es | 108,629.17 | 5,272.14 |
| utf16char_len_ru | 108,852.09 | 61,007.35 |
| utf16char_len_zh | 109,068.75 | 2,412.30 |
| utf8_extra_bytes_ascii | 405,312.50 | 4,769.64 |
| utf8_extra_bytes_en | 432,731.25 | 3,115.16 |
| utf8_extra_bytes_es | 464,520.80 | 4,327.08 |
| utf8_extra_bytes_ru | 610,691.60 | 27,098.96 |
| utf8_extra_bytes_unchecked_ascii | 2,063,641.60 | 27,259.45 |
| utf8_extra_bytes_unchecked_en | 2,070,004.30 | 28,399.68 |
| utf8_extra_bytes_unchecked_es | 2,066,383.40 | 84,272.42 |
| utf8_extra_bytes_unchecked_ru | 2,223,949.90 | 600,037.12 |
| utf8_extra_bytes_unchecked_zh | 2,064,775.10 | 326,685.78 |
| utf8_extra_bytes_zh | 564,891.70 | 11,521.86 |
| utf8char_len_ascii | 96,137.92 | 42,446.45 |
| utf8char_len_en | 117,217.86 | 603,400.23 |
| utf8char_len_es | 124,615.74 | 117,816.84 |
| utf8char_len_ru | 101,475.47 | 351,437.49 |
| utf8char_len_zh | 76,934.17 | 35,945.29 |

`multiiterators`: 11 benches, 0 measured — harness prints `running 11 tests`, then
`/usr/share/dict/american-english not found, skipping benchmarks.`, exit 0. See DEF-4.

## 9. Coverage audit (tools/bcinr-bench-auditor)

```
command : cargo run -p bcinr-bench-auditor     (cwd = workspace root)
exit    : 1  (both runs, before and after the sibling lane's logic bench landed)
output  : FAILED: Found 337 public functions NOT benchmarked out of 337
```

The auditor's universe is `crates/bcinr-logic/src/algorithms/**` (661 `pub fn` occurrences;
337 after the auditor's own filters: ignores `new`/`len`/`*_gate`/`bench_*`/cfg(test) etc.).
The new `logic_kernels_bench` covers module kernels in `src/{bitset,fix,mask,reduce,scan,swar}.rs`
— outside that universe — so the 337 count is unchanged. The full missing list is preserved in
the run log; sample head: `aabb_intersect_branchless`, `abs_diff_i64`, `abs_diff_u64`,
`add_sat_i32`, `adler32_branchless`, ...

## 10. Defects found (feeds lanes 2/9/10 — none fixed by Lane 1)

| id | severity | where | defect |
|---|---|---|---|
| DEF-1 | HIGH (pre-existing) | `tools/bcinr-bench-auditor` vs `crates/bcinr-logic/src/algorithms/**` | 337/337 public algorithm functions have zero benchmark call sites; auditor exits 1. The charter tool for benchmark coverage is permanently red. |
| DEF-2 | MED | `crates/bcinr-pddl/benches/pddl_80_20.rs` | The bench's custom `IR:/Ground:/Solve:/POWL:` prints interleave with divan's table rows (one logical row spans two physical lines). Machine parsing of divan output misreads the table; humans can too. |
| DEF-3 | MED (documented in source) | `crates/bcinr-pddl/benches/scaling.rs` `bench_powl` | N in {128,256,512} rows (~5.3 ns) measure `temporal_plan_to_powl_tape`'s O(1) `MAX_POWL_TAPE_STEPS` typed refusal, not tape lowering — a 3300x discontinuity vs N=64 (17.42 µs). Honest per the in-file comment, but any naive trend comparison across N is invalid. |
| DEF-4 | LOW | `crates/encode_unicode_patch/benches/multiiterators.rs` | Exits 0 with zero measurements when `/usr/share/dict/american-english` is absent; an env-dependent suite that silently degrades to "ran fine". |
| DEF-5 | LOW | `crates/bcinr-cmca/benches/allocation_bench.rs` header | References `docs/BENCH_BASELINE_v26.9.15.md` — that file does not exist in the tree (this document supersedes the reference's intent). |
| DEF-6 | MEASUREMENT (not a code defect) | `logic_kernels_bench`, `allocation_bench` decomposition, `guarded_classify_bench`, `mfw_ir_bench` | Many rows report sub-ns means (0.057–1.1 ns) — at M3 Max clock that is ≤ ~4 cycles. These are divan-iterated means (amortized over up to 1.6M iters), inputs and outputs black-boxed, so they are not proof of const-folding, but they are instruction-throughput figures, not callable latencies. Do not cite them as end-to-end latency without an object-code audit (AGENTS.md §20). |

## 11. Coverage gaps — authoritative hot paths with NO benchmark (ranked)

1. **`bcinr-logic/src/algorithms/**` — 337 public fns, 0 benched** (DEF-1, mechanically
   witnessed). Highest importance: the workspace charter tool exists to police exactly this
   family and has never been green.
2. **`bcinr-cmca` receipt / certification / admission paths** (all `pub`, zero bench call sites):
   `seal_allocation_receipt`, `verify_allocation_receipt` (allocation_receipt.rs — blake3-bound,
   on the ReceiptSound path, AGENTS.md §11), `admit_proposal` (proposal.rs),
   `seal_certificate`, `observe_dwell` (certification.rs), `verify_generated_profile`
   (artifact.rs), `feasible_region::admit_inputs`, `escort_weight_support`,
   `uniform_sibling_weight` (cascade.rs). `allocate`/`escort`/`cascade` kernels are covered;
   their receipt-seal/verify wrappers are not.
3. **`bcinr-powl` scheduler + compiler variants**: `scheduler_tick_guarded`,
   `scheduler_tick_with_resources` (scheduler.rs), `scheduler_tick_v2` (scheduler_v2.rs),
   `const_tick`/`static_tick` (const_scheduler.rs — the v26.6.24 charter's 436 ps headline
   primitive has no current bench), `wide_tick`, and compiler validators
   `check_full_graph_acyclic`, `bp_tcrv_validate_reachability`, `check_all_ops_reachable`,
   `compile_powl_v2`, `compile_powl2` (powl2.rs) — only the v1 `compile` module of
   `powl_pipeline_bench` and the quick-bench legacy/wired pair are measured.
4. **`encode_unicode_patch::multiiterators`** — nominal only (DEF-4); requires a system dict
   file to produce numbers.

## 12. Notes on concurrent-lane interference

- Every wall-clock figure above was measured while sibling lanes were issuing builds against
  the same `target/` (one observed `Blocking waiting for file lock on build directory`).
  Divan per-bench means are internal to the bench process and are less affected than the
  suite wall times; treat cross-suite wall-time comparisons and >5% deltas as suspect until a
  quiet-machine rerun (see METHODOLOGY.md).
- `crates/bcinr-logic/benches/logic_kernels_bench.rs` (08:24Z), `cascade_bench.rs` manifest
  wiring, `bcinr-guarded/benches/`, `bcinr-mfw-ir/benches/` landed during this session from
  sibling lanes and were executed as found. The 337-count in DEF-1 predates and postdates
  those landings identically.
