# CMCA Subsystem Contract

**Version:** 26.7.17-c2  
**Scope:** Chatman Multifractal Consequence Allocation (CMCA) — deterministic branchless semantic
resource allocation. The term "cascade" below names the allocation mechanism (flow propagation
down a semantic-object forest); "Consequence" is this document's canonical acronym expansion —
see `../CMCA_EXPLANATION.md`.  
**Responsibility:** Fixed-point MWU routing, hierarchy-respecting cascade, saturation-safe arithmetic, stability verification

## Preconditions

### Input: CMCA Specification (RDF/Turtle, Pre-Generation)
```
∀ spec ∈ CmcaSpecification:

1. Semantic Objects (F ∈ [0, F_MAX]):
   ∀ obj ∈ objects:
     - cmca:consequence_mass: mass ∈ ℤ (semantic weight units)
     - cmca:dependsOn: [obj'] ⊆ objects (dependency DAG)
     - Dependencies acyclic (topological sort exists)

2. Business Values (N ∈ [0, N_MAX]):
   ∀ val ∈ business_values:
     - cmca:factor_index: i ∈ ℕ (0-indexed)
     - cmca:value: v ∈ ℚ (rational, encoded as 64-bit fixed-point)
     - |v| ≤ 2^15 (fits in Q16.16 range)

3. Measure Heads (K ∈ [0, K_MAX]):
   ∀ measure ∈ measures:
     - cmca:measureIndex: k ∈ [0, K)
     - cmca:label: name ∈ String (e.g., "Cache", "Search")
     - Measures distinct by index; no duplicates

4. Lenses (Q ∈ [0, Q_MAX]):
   ∀ lens ∈ lenses:
     - cmca:lensIndex: q ∈ [0, Q)
     - cmca:lensExponent: e ∈ ℚ, |e| ≤ 2^6
     - Lenses distinct by index; no duplicates

5. Lambda Matrix (K × Q coefficient matrix):
   ∀ λ[k,q] ∈ LAMBDA:
     - λ[k,q] ∈ Q16.16 fixed-point
     - |λ[k,q]| ≤ 2^15 (saturation boundary)
     - ∑_k λ[k,q] ≤ 2^15 per column (prevents overflow in MWU)

6. Constants:
   - η (exploration_floor): ∈ (0, 1), typical 0.001..0.1
   - ζ (exponential_scale): ∈ (0, 1), typical 0.5..2.0
   - M_max (max absolute divergence clip): ∈ ℚ, typical ±10
   - δ (contraction_margin): ∈ (0, 1), typical 0.01..0.1
```

### Input: Subtree State (Pre-Allocate)
```
∀ state ∈ RuntimeState:
   1. Masses: m[i] ∈ ℤ (accumulated resource request per object i)
      - m[i] ≥ 0 (non-negative resources)
      - ∑_i m[i] = total_semantic_mass (conserved quantity)

   2. Weights: w[i] ∈ [Q16.16] (unnormalized routing probabilities)
      - w[i] > 0 (strictly positive; prevents zero-division)
      - Fresh allocation: w[i] := 1.0 for all i

   3. Routing Probabilities: π[i] ∈ [0, 1]
      - π[i] := w[i] / ∑_j w[j] (normalized)
      - ∑_i π[i] = 1.0

   4. Nodes: hierarchy of allocation targets
      - Root node; internal nodes (aggregators); leaves (endpoints)
      - Each node has children[] DAG (no cycles)
```

### Input: Request Distribution (Pre-Update)
```
∀ distribution ∈ RequestMetrics:
   - F_v: feedback for route v (profit/loss signal)
   - S_v: supply available at v (resource capacity)
   - E_v: expected demand at v (forecast)
   - All values in Q16.16 fixed-point
   - Bounded: |F_v|, S_v, E_v ≤ 2^14 (margin for MWU products)
```

### Input: Bounds (Pre-Execution)
```
∀ bounds ∈ AllocationBounds:
   - max_cascade_depth: depth ∈ [1, 32]
   - max_children_per_node: width ∈ [1, 64]
   - max_mwu_iterations: iterations ∈ [1, 1000]
   - max_runtime_cycles: cycles ∈ [1, 2^31]
```

## Postconditions

### Successful Initialization → Allocator
```
allocator ∈ CmcaAllocator ⟹

1. Constant Table Loaded:
   ✓ FACTOR_SET: F constants stored and validated
   ✓ MEASURE_HEADS: K measure indices all distinct, k ∈ [0, K)
   ✓ LENSES: Q lens indices all distinct, q ∈ [0, Q)
   ✓ LAMBDA[k,q]: coefficient matrix loaded; no NaN, no Inf
   ✓ EXPLORATION_FLOOR (η): ∈ (0, 1)
   ✓ EXPONENTIAL_SCALE (ζ): well-defined for 2^(ζ·κ) approximation
   ✓ MAX_DIVERGENCE_CLIP (M_max): symmetric clip bounds [−M_max, M_max]
   ✓ CONTRACTION_MARGIN (δ): ∈ (0, 1)

2. Stability Check Passed:
   ✓ Gain matrix G: eigenvalues λ_max < 1.0 (stable)
   ✓ Weight vector d: d_min, d_max bounded
   ✓ Contraction condition: G·d ≤ (1−δ)·d (convergence proof)
   ✓ If check fails: Err(StabilityRefusal::ContractionFailure) returned

3. Hierarchy Constructed:
   ✓ Node DAG acyclic (topological sort succeeds)
   ✓ Each node initialized: mass=0, weights assigned 1.0
   ✓ Routing probabilities computed: π[i] = w[i] / ∑_j w[j]
   ✓ Total semantic mass = sum of all requests received

4. Allocation State Ready:
   ✓ allocator.allocate() callable with request distribution
   ✓ Deterministic state: same spec + same requests ⟹ identical outcome
```

### Successful Allocation → RoutingDecision
```
decision ∈ RoutingDecision ⟹

1. Routing Probabilities Computed:
   ✓ ∀ i: π[i] ∈ [η/K, (1−η) + η/K]
     (bounded by exploration floor, no path fully starved)
   ✓ ∑_i π[i] = 1.0 (normalized)
   ✓ π[i] deterministic: same state + same request ⟹ identical π[i]

2. Divergence Calculated:
   ∀ lens q, route v:
     κ[v,q] := clip(
       (F_v · S_v) / E_v,  [feedback × supply / expected demand]
       −M_max, M_max
     ) ∈ Q16.16
   ✓ All divisions branchless (reciprocal via Newton-Raphson)
   ✓ Clipping applied unconditionally (no if/else)

3. Exponential Update Computed:
   ∀ measure k, route v, lens q:
     ψ[v,k,q] := 2^(ζ · κ[v,q])  [exponential scaling in Q16.16]
   ✓ Evaluated via branchless minimax polynomial
   ✓ Output saturated to [0, 2^16) range

4. Weight Aggregation via Lambda Matrix:
   w_v^(t+1) := w_v^(t) · ∑_k λ[k,q] · ψ[v,k,q]  [MWU product]
   ✓ All intermediate products checked for overflow (u128 accumulator)
   ✓ Final weight normalized back to Q16.16 range
   ✓ w_v^(t+1) > 0 (never zero; prevents future divide-by-zero)

5. Saturation Applied Unconditionally:
   ✓ All intermediate values clipped to [Q16.16 bounds]
   ✓ No unbounded growth possible (per-operation bound)
   ✓ No negative values stored (masses, weights)

6. Allocation Generated:
   ✓ decision.allocations: [RouteAllocation] {
       route_id: u32,
       probability: Q16.16 ∈ [0, 1],
       expected_mass: ∫ π · m dt
     }
   ✓ ∑ allocations.probability = 1.0

7. Receipt Generated:
   ✓ decision.digest: BLAKE3(weights + masses + lambda products)
   ✓ Deterministic: same state/request ⟹ identical digest
   ✓ Collision-resistant: (probability of hash collision) < 2^−128
```

## Invariants

### Maintained Throughout Execution

1. **Mass Conservation (Semantic Continuity)**
   ```
   ∀ allocation step:
     ∑_v allocation[v].expected_mass ≤ total_semantic_mass_received
   ```
   **Proof:** Normalization step ensures ∑π[i] = 1.0; multiply by total ⟹ conserved

2. **Positive Weights (Non-Zero Division)**
   ```
   ∀ iteration t, ∀ route v:
     w_v^(t) > 0
   ```
   **Proof:** Initialization w^(0) = 1.0; MWU product 2^(ζ·κ) ∈ (0, ∞); never reaches zero

3. **Probability Normalization**
   ```
   ∀ lens q, ∀ probability vector π:
     ∑_i π[i] = 1.0
     ∀ i: π[i] ∈ [0, 1]
   ```
   **Proof:** π[i] := w[i] / ∑_j w[j]; normalization is identity operation

4. **Exploration Floor (No Starvation)**
   ```
   ∀ route v, ∀ lens q:
     π[v] ≥ η / |children(parent)|
   ```
   **Proof:** Explicit unconditional mixture: π_v := (1−η)·(w_v/∑w) + η/K; lower bound guaranteed

5. **Divergence Clipping (Bounded MWU)**
   ```
   ∀ divergence κ[v,q]:
     κ[v,q] ∈ [−M_max, M_max]
   ```
   **Proof:** Explicit clip operation applied before exponentiation

6. **Saturation Safety (No Overflow)**
   ```
   ∀ value v ∈ allocation computations:
     |v| ≤ 2^15  [Q16.16 signed 32-bit bounds]
   ```
   **Proof:** Saturation applied at each step; overflow detected via u128 accumulator

7. **Deterministic Routing**
   ```
   allocate(state, request) is deterministic:
     Same inputs ⟹ identical probability vector + receipt digest
   ```
   **Proof:** All operations branchless; no floating-point (only fixed-point); no RNG

8. **Stability (Convergence)**
   ```
   ∃ equilibrium weights w* such that:
     ∀ t ≥ t_0: |w_v^(t) − w_v^*| ≤ ε  [exponential decay to equilibrium]
   ```
   **Proof:** Eigenvalue λ_max < 1.0 from contraction check ⟹ stable convergence

9. **Hierarchy Acyclicity (DAG Property)**
   ```
   ∀ node v:
     ¬∃ path v → ... → v (no cycles in node dependency graph)
   ```
   **Proof:** Topological sort during initialization; failure ⟹ Err(HierarchyRefusal::Cyclic)

10. **Branchless Execution (Timing Safety)**
    ```
    ∀ operation in allocate() loop:
      Cyclomatic complexity CC = 1
      (no data-dependent if/else; all control flow determined at compile time)
    ```
    **Proof:** Hand-verified CC in `crates/bcinr-cmca/src/allocator.rs`; tested by static analysis

## Refusal Conditions

> **Naming note (2026-07-27):** the `SpecError`/`AllocationError` variant names below, and the
> specific `StabilityRefusal` variant names below, do not match current source. The real
> `StabilityRefusal` enum (`allocator.rs:362`) uses a different, non-overlapping variant set
> (`CertificateMissing`, `BlockGainBoundExceeded`, `ContractionMarginInsufficient`,
> `ModeDwellTimeViolated`, `RuntimeEnvelopeViolated`, `LearningFrozen`, `ContractViolation`, etc.),
> and no `AllocationError` or `SpecError` type exists anywhere in the crate (only
> `StabilityRefusal`, `HierarchyRefusal` in `allocator.rs`, and `CertificationRefusal` in
> `certification.rs`). Treat this section as the original design target/intent, not an
> as-built API reference — check `allocator.rs`/`certification.rs` directly for the real refusal
> taxonomy.

Typed refusal enumeration (design target, see note above): `enum StabilityRefusal { ... }`, `enum AllocationError { ... }`

### Specification Errors (SpecError::*)
```
| SpecError::ParseError { line, expected }
    Reason: Invalid Turtle/RDF syntax
    Recovery: Validate spec file; check UTF-8 encoding
    
| SpecError::UndefinedMeasure { k }
    Reason: Lambda matrix references measure k ∉ [0, K)
    Recovery: Add measure definition; reindex matrix
    
| SpecError::UndefinedLens { q }
    Reason: Lambda matrix references lens q ∉ [0, Q)
    Recovery: Add lens definition; reindex matrix
    
| SpecError::DependencyCycle { cycle }
    Reason: Object dependency graph contains cycle
    Recovery: Break cycle; reorder objects
    
| SpecError::InvalidFactorValue { factor, value }
    Reason: Factor value |value| > 2^15 (exceeds Q16.16 range)
    Recovery: Normalize factor; scale down large values
```

### Initialization Errors (StabilityRefusal::*)
```
| StabilityRefusal::ContractionFailure
    Reason: Gain matrix eigenvalue λ_max ≥ 1.0
            (system unstable; weights diverge to ±∞)
    Recovery:
      1. Reduce exponential_scale (ζ) to dampen MWU updates
      2. Reduce contraction_margin (δ) acceptance threshold
      3. Recompute gain matrix from specification
      4. Verify factor values in [−M_max, M_max]
    
| StabilityRefusal::WeightVectorInvalid
    Reason: Weight vector d has d_min ≤ 0 or d_max overflow
    Recovery: Ensure d_i > 0; scale down large weights
    
| StabilityRefusal::LambdaMatrixBad
    Reason: Coefficient overflow in λ[k,q] product or column sum
    Recovery: Normalize lambda coefficients; ensure ∑_k λ[k,q] ≤ 2^15
    
| StabilityRefusal::ExplorationFloorViolation
    Reason: η ∉ (0, 1) or η too small (starves routes)
    Recovery: Set η ∈ [0.001, 0.1]; increase if starvation observed
    
| StabilityRefusal::DivergenceClipTooSmall
    Reason: M_max so small that all divergences clipped (no signal)
    Recovery: Increase M_max to typical ±10 or observe actual κ values
```

### Runtime Allocation Errors (AllocationError::*)
```
| AllocationError::DivisionByZero { supply, expected }
    Reason: Expected demand E_v = 0 (κ = F·S / 0)
    Recovery:
      1. Check request distribution; ensure E_v > 0
      2. Use safe reciprocal: clip E_v to [ε, ∞) where ε = 2^−10
      3. Fallback: uniform distribution π[i] := 1/K
    
| AllocationError::MassOverflow
    Reason: Total semantic mass > 2^31 (exceeds counter range)
    Recovery: Normalize mass values; use relative allocations (proportions)
    
| AllocationError::WeightNaN
    Reason: 2^(ζ·κ) produces NaN (overflow in exponent)
    Recovery: Check exponential_scale (ζ); reduce if κ values large
    
| AllocationError::WeightUnderflow
    Reason: 2^(ζ·κ) underflows to zero (κ → −∞)
    Recovery: Check feedback signal; ensure F_v not too negative
    
| AllocationError::ProbabilityNotNormalized
    Reason: ∑π[i] ≠ 1.0 (normalization failed)
    Recovery: Re-run normalization loop; check for accumulation errors
    
| AllocationError::ExplorationFloorNotMaintained
    Reason: π[v] < η/K (exploration floor violated)
    Recovery: Check mixture formula; verify η value applied
    
| AllocationError::ReceiptDigestMismatch
    Reason: Computed digest ≠ expected digest
    Recovery: Re-run allocation; verify determinism of inputs
```

### Verification Errors (VerificationRefusal::*)
```
| VerificationRefusal::ContractionNotProven
    Reason: Gain matrix G·d > (1−δ)·d (not provably stable)
    Recovery: Audit gain matrix derivation; lower eigenvalues
    
| VerificationRefusal::HierarchyNotAcyclic
    Reason: Topological sort fails (cycle detected)
    Recovery: Break dependency cycle; verify DAG property
    
| VerificationRefusal::BranchlessViolation
    Reason: Static analysis detected data-dependent branch
    Recovery: Refactor to bitwise multiplexing; eliminate if/else
    
| VerificationRefusal::BoundsExceeded
    Reason: Cascade depth > max_cascade_depth or other limit exceeded
    Recovery: Increase bounds; or partition allocation problem
```

## Semantics

### Multiplicative Weights Update (MWU)

At each allocation step, for each route v and lens q:

1. **Compute divergence:**
   ```
   κ[v,q] := clip( (F_v · S_v) / E_v, −M_max, M_max )
   ```
   - F_v: feedback signal (profit if positive, loss if negative)
   - S_v: available supply at route
   - E_v: expected demand (forecast)
   - Result: bounded signal in [−M_max, M_max]

2. **Exponential weight update:**
   ```
   w_v^(t+1) := w_v^(t) · 2^(ζ·κ[v,q])
   ```
   - If κ > 0: weights increase (exploit good route)
   - If κ < 0: weights decrease (avoid bad route)
   - Exponential scale ζ ∈ (0, 1) controls damping

3. **Aggregation via lens-specific measures:**
   ```
   w_v^(t+1) := w_v^(t) · ∑_k λ[k,q] · 2^(ζ·κ[v,q])
   ```
   - Lambda matrix mixes K measure heads for each lens q
   - Weighted by lens-specific coefficients λ[k,q]

4. **Normalization:**
   ```
   π[v] := (1−η) · (w[v] / ∑_j w[j]) + η/|children|
   ```
   - Normalization ensures ∑π = 1.0
   - Exploration floor η prevents starvation

### Q16.16 Fixed-Point Arithmetic

All values stored and computed in **Q16.16 format**:
- Sign bit: 1 bit (two's complement)
- Integer part: 15 bits (range [−2^15, 2^15))
- Fractional part: 16 bits (precision 2^−16 ≈ 1.5 × 10^−5)
- Storage: i32 (32-bit signed integer) with implicit scaling by 2^16

**Operations:**
```
Addition:     a + b (no scaling adjustment)
Subtraction:  a − b (no scaling adjustment)
Multiplication: (a × b) >> 16  (remove extra scaling)
Division:      (a << 16) / b  (add scaling before divide)
Exponentiation: 2^(ζ·κ) via minimax polynomial (iterative refinement)
```

### Branchless Guarantees

- No `if/else` on divergence magnitude (always apply update)
- No `if/else` on weight polarity (weights always positive)
- No `if/else` on probability normalization (always divide)
- All multiplexing via bitwise operations or masked arithmetic
- Cyclomatic complexity CC = 1 throughout

### Stability Analysis

The CMCA system is stable if:

1. **Eigenvalue condition:** All eigenvalues of gain matrix G satisfy λ < 1.0
   ```
   Proof: Monotonicity of w updates; bounded growth via exponential damping
   ```

2. **Contraction condition:** ∃ weight vector d such that
   ```
   G·d ≤ (1−δ)·d  [strict contraction mapping]
   ```
   Verified at initialization via Newton's method.

3. **Convergence:** As t → ∞, weights w^(t) → equilibrium w*
   ```
   |w^(t) − w*| ≤ λ_max^t · |w^(0) − w*|  [exponential decay]
   ```

## Standing

- **Scope:** Deterministic semantic resource allocation; fixed-point arithmetic; multifractal cascading
- **Cyclomatic Complexity:** CC = 1 (guaranteed branchless)
- **Allocation:** Zero heap allocations; stack-only state
- **Determinism:** Fully deterministic; same inputs ⟹ identical routing probabilities + digest
- **Safety:** All arithmetic saturated to Q16.16 range; no overflow, underflow, or NaN possible
- **Proofs:** Hoare-logic contracts in `HOARE_TRIPLES.md`; stability proofs in `docs/thesis/bcinr-cmca.md`; oracles in `crates/bcinr-cmca/tests/`
