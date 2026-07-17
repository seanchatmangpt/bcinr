# Branchless CMCA over RDF-Connected State

Based on our architectural deep-dive, here is the technical specification for implementing the **Cross-Measure Cognitive Allocation (CMCA)** model over RDF-aligned semantic state within the strict $CC=1$ constraints of `bcinr`.

## 1. Unified Semantic Identity (The RDF Bridge)
Instead of fragmented arrays with isolated states, we construct a unified semantic index. Every object $v$ (e.g., `ProofObligation42`) is assigned a stable 32-bit `SemanticId`.

The heterogenous datasets (Lean graph, POWL workflow, cache metrics) project their states into parallel dense arrays keyed by this ID:
```rust
struct CmcaState {
    /// Fast: Cache and runtime metrics (updated via atomic fetch_add)
    cache_metrics: [AtomicU64; N],
    /// Medium: POWL workflow and lock states
    workflow_state: [u32; N],
    /// Slow: Formal proof standing (Lean) and Market value
    standing_matrix: [u32; N],
}
```
This preserves the independence of schemas and update frequencies ($\beta_k$) while physically aligning them for branchless composition.

## 2. Separate Valuation Functions (Measure Heads)
We define separate, independent measure laws $M_k(z_v)$. These are implemented as pure polynomial functions evaluated over the packed state.

```rust
// Example: High volatility increases search priority but penalizes cache priority.
// All operations are branchless.
fn measure_search(state: PackedState) -> f32 {
    let base = state.proof_promise() * state.market_value();
    base + (state.volatility() * SEARCH_VOLATILITY_WEIGHT)
}

fn measure_cache(state: PackedState) -> f32 {
    let cost = state.recompute_cost() + state.verify_cost();
    (cost * state.reuse_prob()) - (state.volatility() * CACHE_VOLATILITY_PENALTY)
}
```

## 3. Branchless Normalization (Log-Space)
To avoid division-by-zero hazards when computing $L_{k,q}(i) = M_k(z_i)^q / \sum_j M_k(z_j)^q$, we compute the allocation in log-space.
The division becomes a branchless subtraction from a pre-computed `log-sum-exp` denominator:

```rust
// Compute log(M_k) * q for all candidates
let log_m_q = q_factor * log_measure;
// Allocate
let log_allocation = log_m_q - log_sum_exp_denom;
```

## 4. Deterministic Multi-Measure Summation
The allocator must combine multiple lenses $q$ and measure heads $k$. To guarantee bounded execution and strict branchlessness, we will use **const generics and macro unrolling** (the idiomatic Rust systems approach).

```rust
// The compiler fully unrolls this summation, generating linear, branchless SIMD instructions.
let mut pi = eta * u_i;
for_unrolled!(k in 0..NUM_MEASURES {
    for_unrolled!(q in 0..NUM_LENSES {
        pi += (1.0 - eta) * LAMBDA[k][q] * exp(log_alloc[k][q][i]);
    })
});
```

## 5. Downstream Consequence Mass
Because traversing the dependency graph $v \leadsto u$ dynamically requires branching and unbounded loops, the consequence mass $m(v) = \sum_{u} w(v,u)\text{Value}(u)$ will be computed **off-path**.
A background Markov process resolves the topological transitive closures and writes the stationary values into a flat `[f32; N]` array. During the $CC=1$ allocation cycle, downstream consequence is just an $O(1)$ array lookup.

## Next Steps
I am ready to implement this directly into the repository by writing `crates/bcinr-logic/src/autonomic/cmca.rs` and hooking it into the Autonomic Substrate.

Shall I proceed with writing the code?
