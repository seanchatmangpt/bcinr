# MFW Auto-Select Oracle and Proof Obligations: Causal Order Preserving Buffers

**Owner:** `@hoare_oracle`
**Jurisdiction:** `mfw-auto-select` authoritative hot path causal buffers (`/Users/sac/bcinr/mfw-auto-select/src/causal_buffer.rs`)

This document defines the strict mathematical laws, Hoare contracts, valid domains, and proof obligations for integrating zero-allocation deterministic causal order preserving buffers into the `mfw-auto-select` pipeline, in accordance with the BCINR Deterministic Substrate Constitution (`AGENTS.md`).

---

## 1. Mathematical Law and Execution Domain

The objective of the zero-allocation causal buffer in the `mfw-auto-select` pipeline is to sequentially capture and emit selection actions or cognition receipts while strictly preserving causal temporal order without heap allocation, dynamic resizing, or data-dependent branching.

Let $N$ be the statically fixed buffer capacity (e.g., $N=256$).
Let $M$ be the universe of valid messages (events, receipts, or selection frames).
Let $C: M \to \mathbb{N}$ be a strictly monotonic causal clock function that extracts the temporal index of a message.
Let $S_t = \{m_0, m_1, \dots, m_{k-1}\}$ be the buffer state at operational step $t$, where $0 \le k \le N$.

The causal buffer must enforce:
1. **Capacity Bound:** $|S_t| \le N$.
2. **Causal Monotonicity:** $\forall i, j \in [0, k-1], i < j \implies C(m_i) < C(m_j)$.

State transitions must be formulated as fully branchless bitwise polynomial assignments.

---

## 2. Hoare Contracts

For every primitive in the causal buffer hot path, a strict Hoare contract $\{P(x)\} \quad f(x) \quad \{Q(x,f(x))\}$ is enforced.

### 2.1 Causal Push (`push_causal`)

**Mathematical Law:**
Computes the admission mask for a candidate message $m_{cand}$ based on capacity and causal clock constraints, and unconditionally writes to the masked array slot, updating length and clock tracking.

**Hoare Contract:**
* **Valid Input Domain:** $m_{cand} \in M$, with defined $C(m_{cand})$. Active buffer state $S_{active}$ with length $L \in [0, N]$.
* **Output Range:** `Result<CausalBufferState, AutoSelectRefusal>`.
* **Conservation Law:** If admitted, $S_{next} = S_{active} \cup \{m_{cand}\}$ at index $L$. If rejected, $S_{next}$ is bit-for-bit identical to $S_{active}$.
* **Monotonicity Law:** Buffer length $L_{next} = L_{active} + \text{admitted\_mask}$. Last causal clock $C_{max, next} = \text{select}(\text{admitted\_mask}, C(m_{cand}), C_{max, active})$.
* **Overflow Behavior:** Statically bounded by $N$. Write indices strictly clamped to $N-1$ via modular or saturating arithmetic prior to mask application.
* **Invalid-Input Refusal:**
  - `CapacityExhausted` if $L_{active} \ge N$.
  - `CausalOrderViolation` if $C(m_{cand}) \le C_{max, active}$.
* **Determinism:** Branchless state projection (CC=1) via bitwise masking. 
* **State-Mutation Boundary:** 0 heap allocations. Pre-allocated static array or `LockFreeSlab`.
* **Numeric Error Envelope:** Exact integer mathematics. $E_{abs} = 0$.

### 2.2 Causal Pop (`pop_causal`)

**Mathematical Law:**
Retrieves the causally oldest message via a bounded read and structurally advances the read head, mapping out-of-bounds to a strictly zeroed/empty mask.

**Hoare Contract:**
* **Valid Input Domain:** Valid buffer state $S_{active}$ with read head $R \in [0, N]$ and write head $W \in [0, N]$.
* **Output Range:** `(NextState, BoundedOptional<Message>)`.
* **Conservation Law:** State remains topologically unchanged except for $R_{next} = R_{active} + \text{available\_mask}$. The exact message at index $R_{active}$ is returned if $R_{active} < W$.
* **Monotonicity Law:** Read head $R$ strictly monotonically increases until reaching $W$.
* **Overflow Behavior:** $R$ saturates at $W$. Safe from buffer underrun.
* **Invalid-Input Refusal:** No refusal; yields empty payload representation (e.g. `BoundedOptional::None`) when $R = W$.
* **Determinism:** Arithmetic mask-based selection. Contains no branching constructs (CC=1).
* **State-Mutation Boundary:** Fixed inputs, pure value return. 0 heap allocations.
* **Numeric Error Envelope:** $E_{abs} = 0$.

---

## 3. Proof Obligations

To ensure rigorous integrity of the causal buffer, the following independent proof obligations must be certified:

1. **Topological Loop Freedom (@turing_machine):**
   Must formally verify that the generated assembly for `push_causal` and `pop_causal` contains exactly zero loop backedges, branch instructions based on message data or length thresholds, and no heap allocator calls. State selection must use conditional moves (`cmov`) or bitwise masking (`and`/`or`/`not`).

2. **Exhaustive Causal Matrix (@hoare_oracle):**
   Prove that for any sequence of messages mapped onto the branchless `push_causal` and `pop_causal` logic, the output identically matches a purely abstract mathematical queue oracle. 

3. **Refusal Adherence (@armstrong_fault):**
   Must assert `CausalOrderViolation` properly triggers when $C(m_{cand}) \le C_{max, active}$, explicitly verifying the negative edge case $C(m_{cand}) = C_{max, active}$, and that persistent state remains bit-for-bit unchanged upon this typed refusal.

4. **Mutant Survivability (@armstrong_fault):**
   All structural mutants (e.g., dropping the capacity mask, changing strict inequality to non-strict inequality for clock comparisons, or mutating state before full admission) must unequivocally trigger a refusal mismatch or oracle failure. Surviving mutants reduce the SIS to 0.
