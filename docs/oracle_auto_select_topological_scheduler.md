# MFW Auto-Select Oracle and Proof Obligations: CMCA-Guided Topological Scheduler

**Owner:** `@hoare_oracle`
**Jurisdiction:** `mfw-auto-select` authoritative hot path topological scheduler (`/Users/sac/bcinr/mfw-auto-select/src/scheduler.rs`)
**Iteration:** 9

This document defines the strict mathematical laws, Hoare contracts, valid domains, and proof obligations for integrating a zero-allocation deterministic CMCA-guided topological scheduler into the `mfw-auto-select` pipeline, in accordance with the BCINR Deterministic Substrate Constitution (`AGENTS.md`).

---

## 1. Mathematical Law and Execution Domain

The objective of the CMCA-guided topological scheduler is to dynamically resolve execution readiness of workflow nodes (Kahn's partial-order algorithm) and apply the CMCA deterministic choice over the set of ready nodes without branching or heap allocation.

Let $V$ be the bounded set of maximum $N$ workflow nodes (e.g., $N=32$).
Let $E \subset V \times V$ be the dependency edges, represented as a fixed-size adjacency matrix or bitmasks.
Let $D$ be the array of in-degrees for each node $v \in V$.
Let $S_{ready} = \{ v \in V \mid D[v] = 0 \text{ and } \text{eligible\_mask}(v) = 1 \}$ be the dynamically maintained ready set.
Let $M$ be the array of CMCA masses derived from the semantic-to-measure projection.

The topological scheduler must enforce:
1. **Bounded Node Domain:** $|V| \le N$.
2. **Deterministic Selection:** Given $S_{ready}$ and mass vector $M$, select exactly one node $v^* \in S_{ready}$ maximizing the CMCA lens equation $\arg\max (M[v]^q)$, resolved deterministically in $O(N)$ branchless steps.
3. **Monotonic Progress:** Upon selecting $v^*$, the in-degrees of all dependents $v \in dependents(v^*)$ are monotonically decremented using bitwise mask operations.

---

## 2. Hoare Contracts

For every primitive in the scheduler hot path, a strict Hoare contract $\{P(x)\} \quad f(x) \quad \{Q(x,f(x))\}$ is enforced.

### 2.1 Branchless Ready Set Resolution (`resolve_ready_mask`)

**Mathematical Law:**
Computes the binary mask of nodes whose in-degree is zero and have not yet been executed.

**Hoare Contract:**
* **Valid Input Domain:** In-degree array $D \in [0, N]^N$, execution mask $E_{mask} \in [0, 2^N-1]$.
* **Output Range:** `ReadyMask` $\in [0, 2^N-1]$.
* **Conservation Law:** Output bit $i$ is 1 if and only if $D[i] == 0$ and bit $i$ of $E_{mask}$ is 0.
* **Determinism:** Branchless projection via SWAR or SIMD equality and bitwise NOT. (CC=1).

### 2.2 CMCA Selection (`select_cmca_node`)

**Mathematical Law:**
Selects the index $v^*$ from the `ReadyMask` that maximizes the mass $M[v]$ without conditional branches.

**Hoare Contract:**
* **Valid Input Domain:** `ReadyMask` $\in [0, 2^N-1]$, Mass array $M \in [0, 255]^N$.
* **Output Range:** `Result<NodeIndex, AutoSelectRefusal>`.
* **Conservation Law:** Selected node strictly has bit set in `ReadyMask` and mass $\ge$ any other node in `ReadyMask`. Ties broken by static index order.
* **Invalid-Input Refusal:** `NoReadyNodes` if `ReadyMask` == 0.
* **Determinism:** Parallel bitwise maximum reduction mask. (CC=1).

### 2.3 Edge Elimination (`decrement_in_degrees`)

**Mathematical Law:**
Decrements the in-degrees of all nodes dependent on the selected node $v^*$.

**Hoare Contract:**
* **Valid Input Domain:** $v^* \in [0, N-1]$, current in-degrees $D$, Dependency matrix $\mathbf{A} \in \{0, 1\}^{N \times N}$.
* **Output Range:** Updated in-degrees $D_{next}$.
* **Monotonicity Law:** $D_{next}[j] = D[j] - \mathbf{A}[v^*][j]$.
* **Overflow Behavior:** Safe from underflow; initial in-degrees strictly match matrix column sums.
* **Determinism:** Bitwise arithmetic subtraction masked by dependency row. (CC=1).

---

## 3. Proof Obligations

1. **Topological Loop Freedom (@turing_machine):**
   Formally verify the assembly for `select_cmca_node` contains zero loop backedges or data-dependent jumps. Max-finding must use parallel reduction (`pmaxub` equivalents or bitwise SWAR).
2. **Determinism over Deadlocks (@hoare_oracle):**
   Prove that cyclic subgraphs inherently do not enter the `ReadyMask`, safely halting the scheduler which then emits a typed `WorkflowDeadlock` refusal when `ReadyMask == 0` but unexecuted nodes remain.
3. **Mutant Survivability (@armstrong_fault):**
   Mutating the tie-breaking static index order or in-degree decrement arithmetic must instantly trigger oracle failures.
