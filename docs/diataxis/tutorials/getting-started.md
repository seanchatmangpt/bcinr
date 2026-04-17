# Tutorials: Getting Started with Branchless Calculus

## 1. Zero to First Kernel
How to bootstrap a branchless logic primitive.

1.  **Define:** Add an entry to `ontology/bcinr.ttl`.
2.  **Generate:** Run `unrdf sync` to scaffold the API/logic split.
3.  **Implement:** Edit the handwritten `bcinr-core/src/logic/FAMILY.rs` to replace `todo!()`.
4.  **Verify:** Run `cargo test`.

## 2. SIMD Vectorization
How to lift scalar operations to SIMD lanes using `movemask` and `shuffle` patterns for $10\text{ GB/s}$ throughput.
