# Explanations: The Branchless Calculus

## Why Branchless?
Modern CPUs spend up to 20% of their logic on branch prediction. Mispredictions cause pipeline stalls that are fatal for high-throughput, latency-critical consensus algorithms. Branchless programming forces hardware to execute all paths or use predictive-free arithmetic, ensuring latency is data-independent.

## The Architecture: API vs. Logic Split
- **`src/api/`:** The facade. Purely generated, provides ergonomic, stable interfaces for applications.
- **`src/logic/`:** The substrate. Handwritten, performance-critical code optimized for specific architectural targets (SIMD, SWAR).
- **`mod.rs`:** The anchor seam. Dynamically generated to stitch generated and handwritten modules into a unified namespace.
