# bcinr Benchmark Charter & Performance Baselines

This document outlines the performance, memory, and complexity targets for each algorithm family in the `bcinr` library.

## Benchmark Table by Algorithm Family

| Algorithm Family | Primary Metrics | Memory Profile | Complexity | Key Primitive |
| :--- | :--- | :--- | :--- | :--- |
| **Mask** | Cycles / Op | O(1) | O(1) | `select_u32` |
| **Int** | Throughput (GB/s) | O(1) | O(1) | `popcount_u64` |
| **Fix** | Cycles / Op | O(1) | O(1) | `add_sat_u8` |
| **Network** | Latency (ns) | O(N) | O(N log N) | `bitonic_sort_32` |
| **Bitset** | Throughput (GB/s) | O(N) | O(N) | `jaccard_u64` |
| **Scan** | Throughput (GB/s) | O(1) (Streaming) | O(N) | `find_byte_mask` |
| **UTF-8** | Throughput (GB/s) | O(1) (Streaming) | O(N) | `validate_utf8` |
| **Parse** | Cycles / Byte | O(1) | O(N) | `parse_decimal_u64` |
| **DFA** | Latency (ns) | O(1) | O(N) | `dfa_advance` |
| **Reduce** | Throughput (GB/s) | O(1) | O(N) | `horizontal_xor` |
| **Sketch** | Cycles / Byte | O(1) | O(N) | `murmur3_32` |
| **SIMD** | Throughput (GB/s) | O(1) | O(N/lane) | `splat_u8x16` |

## Performance Discipline
- **Branchless Mandate:** All kernels must be branchless in the hot path. Misprediction counts must be negligible (`< 0.1%` of total instructions) under `perf stat` or equivalent.
- **Cache-Conscious:** Bulk operations (Scan, Bitset, Reduce) must maintain sequential memory access patterns to saturate L1 bandwidth.
- **Determinism:** Algorithms must be reproducible across architectures, ensuring that the same input consistently produces the same result (crucial for receipt-based systems).
- **Zero-Allocation Steady State:** The `Mem` substrate (Arenas/Rings) must be used to eliminate heap allocation during PICTL runtime loops.

## Benchmark Strategy per Family
1.  **Micro-Benchmarks (`bcinr-bench`):** Use Criterion to measure nanosecond latencies for `select`, `popcount`, and `add_sat`.
2.  **Throughput Benchmarks:** Measure GB/s for `Scan`, `UTF-8`, `Bitset`, and `Reduce` using large buffers (1MB to 1GB).
3.  **Adversarial Fuzzing:** Verify kernel correctness against random and boundary-case inputs using `testkit`.
