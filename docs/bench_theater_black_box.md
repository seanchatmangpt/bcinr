Here is the analysis on how BCINR performs benchmarking while actively avoiding "benchmark theater" (as mandated by CHEAT-008), based on the contents of `/Users/sac/bcinr/bcinr-bench/benches/throughput_bench.rs`.

### Avoiding "Benchmark Theater" (CHEAT-008)

Rule `CHEAT-008` in `AGENTS.md` explicitly prohibits "Benchmarking a stub, constant-folded path, dead result, or reduced problem not equivalent to production." To enforce this, BCINR leverages Criterion alongside precise optimizer barriers (`core::hint::black_box`) and intentionally crafted workload data.

#### 1. Strategic Usage of `core::hint::black_box`

To prevent the LLVM compiler from bypassing the workload via constant folding or dead-code elimination, `black_box` is used at multiple critical interaction points:

*   **Input Masking:** All variables, masks, or slice references passed into the test primitive are obscured from the optimizer (e.g., `popcount_u64(black_box(x))`). This forces the CPU to evaluate the function dynamically at runtime, even if the inputs were statically generated beforehand.
*   **Result Consumption:** The accumulated outcome of every benchmark loop is sunk into a `black_box(acc)` or `black_box(sum)` return statement. This ensures the compiler believes the output is used externally, preventing it from optimizing out the loop entirely.
*   **Preventing Branch Speculation:** In baseline comparisons between branching and branchless execution, predicates are individually black-boxed (e.g., `if black_box(i) & 1 == 0`). This ensures the branch predictor actually has to work and the compiler cannot unroll the loop into a fixed, branch-free pattern.

*Example from `bench_mask_ops` where both the mask input and sum output are boxed:*
```rust
b.iter(|| {
    let mut sum = 0u64;
    for i in 0u64..N {
        let mask = 0u64.wrapping_sub(i & 1); // 0x000…0 or 0xFFF…F
        sum = sum.wrapping_add(select_u64(black_box(mask), i, i * 2));
    }
    black_box(sum)
});
```

#### 2. Realistic Workload Construction

Benchmark theater often stems from testing tiny or highly predictable inputs that don't reflect production. BCINR mitigates this through rigorous workload design:

*   **Pseudo-Random Data Distributions:** Rather than testing arrays of zeros or simple incrementing loops, input vectors are generated with high-entropy distributions using LCGs or prime-modulo arithmetic (e.g., `(i as u64).wrapping_mul(0x9e3779b9_7f4a7c15)`). This guarantees the logic is tested against diverse bit patterns.
*   **Data Dependency Chaining:** For iterative operations (like `xxhash64` or delta encoding), the output state of the previous iteration is explicitly passed as an argument into the next (`h = xxhash64(black_box(word), h);`). This enforces a strict execution dependency chain, accurately modeling real-world latency by preventing the CPU from illegally parallelizing the loop using superscalar execution.
*   **Magnitude Matrices:** Benchmarks don't just run one arbitrary size. Throughput scaling is proven across multiple magnitudes (e.g., `64`, `256`, `1024`, `4096`, `65536` bytes) using Criterion's `Throughput` API. This exposes how algorithms handle L1/L2 cache boundaries, exposing performance cliff-edges rather than just best-case warm-cache scenarios.
