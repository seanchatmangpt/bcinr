# TPS Cognitive Assembly: The Deterministic Semantic-to-Measure Projection (v26.7.17)

## 1. The Core Inversion: Quality at the Source

From the perspective of the Toyota Production System (TPS), if data is already in the correct shape, an algorithm should not be required to manufacture quality at runtime.

The fundamental inversion is:
**Do not make the algorithm repair its incoming work.**

In conventional software engineering, algorithms spend the majority of their cycles performing nonessential preparation: parsing input, checking fields, normalizing names, allocating structures, removing duplicates, and selecting strategies. TPS classifies this as waste (muda), rework, and overprocessing.

Auto Select instead enforces:
`Receive shape-certified cognitive part → perform bounded transformation → deliver shape-certified part`

Formally:
If \( x \in [\![ S_i ]\!] \), then \( B_i(x) = y \).
The cognitive breed (kernel) should not need to ask whether \( x \) is structurally usable. The upstream process must deliver a component that is correctly typed, normalized, bounded, cache-ready, and arranged in the exact mathematical representation the downstream breed consumes.

### The Upstream Production Gauge
The upstream producer is responsible for making the work suitable for its immediate downstream customer:
\( \text{Output}(P_{i-1}) \models \text{InputShape}(P_i) \)

If the supplier cannot produce the required shape, the line stops (Andon). Over time, via kaizen, the upstream producer is constrained so that malformed output becomes impossible to represent. SHACL defines the law, but upstream generated types make violations unrepresentable.

## 2. Just-In-Time Semantic Projection

The system avoids cognitive inventory—unused representations, plans, and models. The downstream requirement pulls the exact shape it needs:
`Goal → selected breed → required input shape → upstream projection`

This Just-in-Time semantic projection means that if the world graph contains rich information, we do not produce Bayesian, SAT, and Temporal projections blindly. Auto Select pulls only the projection required by the specific downstream breed. 

When data is already in shape:
- **Muda decreases:** No repeated parsing or defensive validation.
- **Mura decreases:** Every input has bounded width and cardinality.
- **Muri decreases:** The algorithm never receives work beyond its capacity.

## 3. Mathematical Sequentialization: CMCA-Guided Topological Scheduling

When a partial order workflow must be executed on a sequential line, the core mathematical operation is a topological ordering of a partial order (a linear extension).

For Auto Select and TPS, Kahn’s algorithm exposes the set of work currently ready to run:
\( R_t = \{v \in V_t : \text{indegree}_{V_t}(v) = 0\} \)

However, TPS asks not just what is legal, but what best supports flow. This transforms topological sorting into precedence-constrained scheduling. A selection rule over the ready set based on downstream pull, due time, and starvation looks like:
\( v^\star = \arg\min_{v \in R_t} \text{FlowCost}(v) \)

In Auto Select, this translates to CMCA-guided online topological scheduling:
\( B^\star_t = \text{CMCA}(R_t, \mu_q, K, \Gamma) \)

### Serialization Entropy
Fixing a single line from a partial order consumes production optionality. The serialization entropy is:
\( H_{ser}(P) = \log |\text{Lin}(P)| \)

Auto Select delays unnecessary sequencing (least-commitment planning) to preserve this flow flexibility until downstream pull demands a choice.

## 4. The Canonical Auto Select Pipeline

The bridge between semantics and numbers is a deterministic semantic-to-measure projection:
`RDF semantics → SHACL eligibility → partial-order workflow → numeric measure vector → CMCA → selected tool`

### 4.1 SHACL Eligibility
SHACL defines legality, ensuring tools like VectorSearch are filtered out before reaching CMCA if they violate hard constraints (e.g., requires determinism).

### 4.2 Semantic-to-Numeric Projection
The RDF compiler produces a coordinate vector for each eligible tool:
\( x_i = (s_i, e_i, a_i, t_i, d_i, r_i, c_i) \), where each \( x_i^k \in [0, 255] \).

These values are generated deterministically based on ontology rules (e.g., Exact output type = 255, Lossy adapter = 128).

### 4.3 Canonical Mass and CMCA
A canonical mass is computed using an unweighted geometric mean, preventing one high score from obscuring a fatal weakness:
\( m_i = 255 \prod_{k=1}^{7} \left(\frac{x_i^k}{255}\right)^{1/7} \)

The CMCA lens then evaluates the priority over the currently enabled ready set:
\( P_q(i) = \frac{m_i^q}{\sum_j m_j^q} \)
\( i^\star = \arg\max_{i \in T_t} m_i^q \)

## 5. Conclusion: The Full Cognitive Production Flow
The ideal steady-state timing path is:
`correct part arrives → correct transformation happens → correct part leaves`

By utilizing Just-in-Time cognition (\text{Right cognitive part + right shape + right breed + right time}), Auto Select ensures the entire information supply chain delivers defect-free, bounded, immediately consumable intelligence at nanosecond latency.
