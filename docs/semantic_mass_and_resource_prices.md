# Semantic Mass, Resource Prices, and Standing Projections in BCINR

The concepts of **semantic mass**, **resource prices**, and **standing projections** constitute the core components of the Chatman Multifractal Cascade Allocation (CMCA) mechanism within `bcinr`. They are fundamentally related to the overarching goal of building a deterministic, branchless, and allocation-free substrate for Artificial General Intelligence (AGI) governed by autonomic control theory.

Here is an analysis of each concept and how they interlock to achieve this civilizational-scale mandate:

## 1. Semantic Mass
**Definition**: Semantic mass represents the raw allocated capacity or intrinsic multi-measure "weight" (e.g., token quotas, compute quanta) of a node within the system's hierarchical directed acyclic forest. 
**Role**: Rather than allocating resources using discrete, conditional logic (which introduces indeterminism), CMCA distributes semantic mass downwards through a continuous flow propagation matrix. The mass routed to specific AI components or internal representations adjusts organically over time using Multiplicative Weights Updates (MWU) driven by relative entropy constraints.
**Control Theory Context**: Semantic mass behaves like a continuous fluid or energy in a thermodynamic system. By tracking its conservation and flow rather than discrete allocation states, the system maintains $O(1)$ cycle predictability.

## 2. Resource Prices
**Definition**: Resource prices (often denoted mathematically as $\mu_x$) act as dual variables for system capacity constraints.
**Role**: During the "Stable Projections" phase, semantic mass allocations are scaled by these prices alongside operational costs ($c_x$). This applies an exponential damping factor ($\exp(-\mu_x \cdot c_x)$) to prevent any single branch from exceeding structural limits. To remain perfectly branchless ($CC=1$) and avoid variable latency, `bcinr` evaluates these scaling factors without floating-point math or hardware division, relying entirely on Q16.16 fixed-point polynomial approximations and saturating reciprocals via Newton-Raphson refinement.
**Control Theory Context**: Resource prices serve as the system's dynamic damping mechanism (the "valves"). They slowly adapt to environmental noise and feedback, restricting allocations computationally without ever executing an `if` or `else` branch.

## 3. Standing Projections
**Definition**: Standing projections refer to the deterministic, branchless mapping of current telemetry and allocations to a projected stability state (e.g., tracking `support_standing` and `dependence_standing`).
**Role**: `bcinr` collects constant-time telemetry across the autonomic loop (MAPE-K) to observe variables like Gram degeneracy and drift inertia. Standing projections compute the mathematical bounding envelope for these variables. If a standing projection exceeds bounded limits (a policy violation), the system generates an unconditional `StabilityRefusal`. Because branching is illegal in the authoritative runtime, this refusal is generated structurally using bitwise masks (`const_select`) to zeroes-out invalid states rather than terminating via early returns.
**Control Theory Context**: Standing projections form the closed-loop safety verification. They guarantee that the system remains globally asymptotically stable by projecting future states and blocking invalid mutations identically across 100% of execution paths.

## Synthesis: The Deterministic AGI Substrate
Placing these three concepts firmly within the **Authoritative Runtime** classification (Rule 6 in `AGENTS.md`) is what guarantees the deterministic nature of the `bcinr` substrate. 

In traditional resource allocation, reacting to prices and validating constraints requires conditional decision trees, which introduce combinatorial path explosions and timing side channels. By transforming AGI resource management into a continuous control-loop problem:
1. **Observe**: Standing projections measure constraints.
2. **Infer & Propose**: Resource prices weigh the allocations.
3. **Execute**: Semantic mass flows through fixed-point mathematical transitions.

Ultimately, this enforces the **Radon Law ($CC=1$)**. Every resource decision in the AGI's runtime is rendered as a bit-parallel Boolean polynomial, eliminating side channels and providing a mathematically provable, ironclad execution environment.
