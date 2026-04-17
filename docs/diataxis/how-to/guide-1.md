# How-To: Optimizing Primitive 1$ for Throughput

Optimizing for civilizational scale requires shifting from scalar logic to register-width parallel execution.

## Strategy
1. **Unroll:** Ensure your loops are unrolled to maximize instruction-level parallelism.
2. **Vectorize:** If applicable, map the logic to the SIMD family primitives (e.g., `shuffle_u8x16`).
3. **Analyze:** Check `target/criterion/report/index.html` to ensure the variance ($\sigma^2$) remains near-zero.

By following this guide, you ensure primitive 1$ contributes to a stable, synchronous backbone for your distributed swarm.
