# Slow Rail Bit-Parallel Reachability Validation (BP-TCRV)

In the BCINR architecture, the execution dependency graph must be strictly acyclic to guarantee deterministic, finite execution. During ahead-of-time (AOT) compilation, the Slow Rail enforces this via a **Two-Phase Verification Protocol**:

1. **Phase 1 (Cycle Detection):** Kahn's Topological Sort algorithm is used to detect non-loop cycles. However, a malicious or malformed graph could contain disconnected, unreachable cyclic subgraphs. Because nodes in these subgraphs never reach an in-degree of 0, they evade Kahn's BFS queue entirely.
2. **Phase 2 (Reachability Validation):** To mathematically prove that every active node is accessible from the `entry_mask` and to catch any disconnected cycles, the compiler employs the **Bit-Parallel Transitive Closure Reachability Validator (BP-TCRV)**.

BP-TCRV is a constant-time ($O(1)$), branchless, and zero-allocation algorithm that satisfies the strict **BCINR Radon Law** ($CC=1$).

## The Fully Unrolled Roy-Warshall Algorithm

Because the POWL tape has a strict, compile-time bound of at most $N \le 64$ operations, the entire adjacency relation fits within register space. Instead of performing a dynamic queue-based BFS graph traversal (which would introduce variable bounds and timing side-channels), BP-TCRV computes the complete transitive closure matrix of the DAG using a bit-parallel Roy-Warshall algorithm.

### 1. Mathematical Formulation

Let the DAG reachability state be a matrix $R \in \mathbb{U}_{64}^{64}$, where each row $R[i]$ is a `u64` bitmask. The $j$-th bit of $R[i]$ is $1$ if vertex $i$ can reach vertex $j$ via a directed path of successor edges.

**Initialization:** The reachability relation is initially defined as the successor relation combined with self-reachability:
$$R[i] = \text{succ\_mask}_i \cup \{i\}$$

**Transitive Closure Propagation:** The Roy-Warshall algorithm propagates reachability transitively by evaluating all intermediate vertices $k \in [0, 64)$ sequentially. For each $k$, we update every source vertex $i \in [0, 64)$ branchlessly using full-width masks:
$$R_i^{(k+1)} = R_i^{(k)} \cup (R_k^{(k)} \text{ if } k \in R_i^{(k)})$$

The branchless register update is calculated as:
```rust
let can_reach_k = (r[i] >> k) & 1;
let mask = 0u64.wrapping_sub(can_reach_k); // !0u64 if reachable, 0u64 otherwise
r[i] |= r_k & mask;
```

Because the matrix dimensions are strictly fixed ($64 \times 64$), the algorithm is executed with precisely 64 outer and 64 inner iterations. This allows modern compilers (e.g., LLVM) to fully unroll the loops and vectorize the inner propagation, producing straight-line pipeline instruction streams devoid of branch-prediction stalls.

### 2. Validating Reachability and Catching Disconnected Cycles

Once the transitive closure $R$ is established, the validation step performs the following branchless checks:

1. **Accumulate Reachable Nodes:** The total set of nodes reachable from the `entry_mask` is computed as the union of the reachability sets for all entry nodes.
   $$\text{reachable\_from\_entry} = \bigcup_{e \in \text{entry\_mask}} R[e]$$

2. **Construct Requirement Mask:** We construct a mask of operations that *must* be reachable, which includes all active slots (index `< tape.len`) that are not loop back-edges (`OpKind::LoopRedo`).
   $$\text{must\_be\_reachable} = \bigvee_{i=0}^{63} (\{i\} \land (\text{in\_bounds}(i) \land \text{is\_not\_redo}(i)))$$

3. **Check for Containment Violations:** If any required operation is missing from the reachable set, a structural validation failure occurs.
   $$\text{violation} = \text{must\_be\_reachable} \land \neg\text{reachable\_from\_entry}$$

If $\text{violation} \neq 0$, the graph contains unreachable operations (such as disconnected malicious cycles that evaded Kahn's queue), and compilation fails with a typed `CompileError::Unreachable` refusal.

## BCINR Hot Path Compatibility

By replacing standard heap-allocated BFS queue traversals with purely register-bound operations, the BP-TCRV mechanism strictly complies with the **Zero-Allocation Boundary**. Its constant iterations and absence of data-dependent conditionals ensure identical execution latency regardless of the input graph topology, successfully closing timing side channels while proving reachability branchlessly.
