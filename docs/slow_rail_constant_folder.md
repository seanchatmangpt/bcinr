# Slow Rail Constant Folder in BCINR

In the `bcinr` (BranchlessCInRust) architecture, the Authoritative Runtime (**Hot Path**) operates under severe deterministic laws. According to the constitution, it is strictly forbidden from parsing semantic data, discovering theorems dynamically, or executing variable-length iterations (Rule 12: *No Runtime Theorem Discovery*). 

To achieve advanced autonomic capabilities while adhering to the $CC=1$ Radon Law, the architecture delegates all complex analysis to the **Slow Rail**. In this capacity, the Slow Rail functions as a massive, ahead-of-time (AOT) **Constant Folder**—executing unbounded, branch-heavy, and memory-allocating algorithms offline, and collapsing their conclusions into statically resolvable primitives.

## 1. Folding Semantic Logic into Static IR

Semantic web payloads (RDF, SHACL) represent unbound cyclic graphs with variable-length URIs. The Hot Path cannot parse these without violating zero-allocation and bounded-execution laws. The Slow Rail "folds" these dynamic structures into rigid constants:

* **Index-Sorting and Sequence Bounds:** Ontological entities (e.g., `cmca:MeasureHead`, `cmca:Lens`) are deterministically assigned fixed array offsets. The total counts of these entities are folded into absolute numeric constants like `K` and `Q` injected directly into the runtime.
* **Topological Mask Flattening:** Dynamic dependency traversals (like resolving `cmca:dependsOn` chains) are calculated AOT on the Slow Rail using Kahn's Topological Sort. The resulting dependencies are folded down into fixed hardware bitmasks (e.g., `pred_mask`, `succ_mask`). Instead of chasing pointers or executing `while` loops, the Hot Path merely performs constant-time bitwise operations (like `AND` or `XOR`) over these fixed `u64` integers.
* **Interning and Padding:** Dynamic string labels and complex rules are stripped of variable layouts and folded into fixed-width, cache-aligned `#[repr(C, align(64))]` structs.

## 2. Pre-Calculating Theorems and Thresholds

Mathematical workloads required for closed-loop stability control are similarly folded. 

* **Theorem Resolution:** Algorithms requiring continuous domain searches, such as Lyapunov search, adaptive threshold discovery, or spectral-radius estimation (eigenvalue derivations), are fully computed on the Slow Rail. 
* **Fixed-Point Conversion:** Floating-point approximations natively produced by these complex searches are explicitly converted into exact **Q16.16 fixed-point representations** on the Slow Rail. 
* **The Witness:** Rather than giving the Hot Path an equation to solve, the Slow Rail collapses its findings into a mathematical "Witness" (e.g., a static comparison matrix $G$, a scaling vector $d$, and a contraction margin $\delta$). The Hot Path simply verifies the folded constants against algebraic laws (e.g., $Gd \leq (1-\delta)d$) in $O(1)$ constant time.

## 3. Embedding into `cmca_generated.rs` (The `Gamma_CMCA` Contract)

The final product of this massive constant-folding operation is explicitly written out to **`cmca_generated.rs`**. This file acts as the topological boundary—the `Gamma_CMCA` contract—between the semantic world and the determinism of the substrate.

* **Static Rust Intermediates:** `cmca_generated.rs` is filled with `pub const` declarations, zero-cost typestates, fixed-point matrices (e.g., the `LAMBDA` Matrix), and pure bounded tables representing `PackedSemanticState`.
* **Cryptographic Source Binding:** To guarantee the folded constants are legitimate, the Slow Rail embeds BLAKE3 content-identity digests into the file. The `RDF_INPUT_DIGEST` proves the constants originate from an admitted semantic graph, and the `GENERATOR_SOURCE_DIGEST` proves the exact folding script used.
* **Scanner Compliance:** Despite being generated, `cmca_generated.rs` is treated as Authoritative Code. It must pass the `bcinr-cheat-scanner`, containing zero instances of `if`, `match`, or magic constants, ensuring the Slow Rail correctly folded the logic down to a Cyclomatic Complexity of $CC=1$.

By acting as a comprehensive constant folder, the Slow Rail allows the Hot Path to "know" the results of complex graph traversals and unbounded math without ever executing them.
