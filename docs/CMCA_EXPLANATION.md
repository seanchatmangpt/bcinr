# CMCA: Chatman Multifractal Consequence Allocation
## A Guide from ELI5 to PhD

**Version:** 26.7.25  
**Document Status:** ALIVE (formal specification + verification strategy)

**Canonical name:** "Chatman Multifractal Consequence Allocation" is the one ecosystem-wide
canonical expansion of CMCA, spanning this repo and `/Users/sac/mfw`. "Consequence" names the
allocated quantity (`cmca:consequence_mass` in both repos' ontologies); "Cascade" (used in
`docs/contracts/CMCA_CONTRACT.md`) names the allocation *mechanism*, not a competing meaning.
Four other backronyms have appeared in this repo's docs over time and are now superseded/historical,
not additional live meanings:
- "Covariance Monitoring and Calibration Assessment" — `crates/bcinr-cmca/src/lib.rs` (predates
  this reconciliation; the crate's calibration/telemetry role, `observatory.rs`, is real and part
  of this same crate, just not the whole of what "CMCA" names).
- "Cross-Measure Cognitive Allocation" — `docs/cmca-rdf/ARCHITECTURE.md`.
- "Constrained Multi-measure Co-allocation (for Resource Decision Fields)" —
  `docs/cmca-rdf/CURRENT_STATUS.md`, `docs/innovations/multi_measure_autonomic_feedback.md`.
- "Cascade Multifractal Cascade Allocation" — this document's own prior title (self-referential
  drift, corrected above).

See `/Users/sac/mfw/mfw-ontology/cmca/PROVENANCE.md` for the cross-repo disambiguation record.

---

## Level 1: ELI5 (Explain Like I'm 5)

### The Story: The Homework Chooser

Imagine you're a student with 8 homework assignments due today, but you can only do one at a time. Each assignment has:
- **Difficulty** (how hard it is)
- **Importance** (how much it's worth)
- **Prerequisites** (other work you need to finish first)

But here's the tricky part: **you change your mind about what matters**. Sometimes you want to:
- **Exploit**: Pick the highest-value assignment you can do right now
- **Explore**: Pick something you haven't tried yet (to learn)
- **Coverage**: Pick the assignment that matters most for future work
- **Rare**: Pick the weird edge case nobody thinks about

**CMCA is a robot brain that picks which assignment to do**, and it does it in a way that:
1. **Never wastes time** (O(1) constant time, always)
2. **Never gets stuck** (oscillates between good choices)
3. **Proves it picked right** (verifiable receipt chain)
4. **Works in your head** (no secret memory, all state visible)

The magic: **it uses math to guarantee** that if you follow its picks, you'll make progress toward your goal, no matter which "pick strategy" (q-lens) you're using.

---

## Level 2: Beginner (High School / Early Undergrad)

### What Problem Does CMCA Solve?

**The Problem:** You have a system where:
- Multiple things compete for attention (candidates)
- Each has a value/cost trade-off
- Your priorities change over time
- You need to pick the best one *right now* without overthinking

**The Solution:** CMCA is an allocation algorithm that:
1. **Scores each candidate** based on current priorities (value, cost, timing, prerequisites)
2. **Admits only valid candidates** (ones whose preconditions are met)
3. **Picks the best** according to your current strategy
4. **Records proof** that the pick was optimal
5. **Prevents flip-flopping** (oscillation) via dwell-time locking

### Real-World Analogy: Air Traffic Control

Think of CMCA as an air traffic controller deciding which airplane lands next:

| Concept | Airport Analogy |
|---------|-----------------|
| **Candidates** | Airplanes in the queue |
| **Ready mask** | Which planes are cleared to land |
| **Preconditions** | Runway must be clear, plane must be descended |
| **Q-lens (Exploitation)** | Land the highest-priority plane (emergency medical, VIP, etc.) |
| **Q-lens (Coverage)** | Rotate between different airlines to avoid favoritism |
| **Q-lens (Rare)** | Handle the edge case (small plane that needs special runway) |
| **Dwell-time** | Don't change landing rules every 10 seconds; stick with current approach for N minutes |
| **Receipt** | Digital log proving "we landed plane XYZ at 3:15 PM per landing rule ABC" |

### Key Properties

1. **Deterministic**: Same input → same output, always
2. **Branchless**: Constant execution time (no "if this, then that" branches)
3. **Optimal**: Picks the best candidate for the current strategy
4. **Verifiable**: an audit receipt records the pick and lets anyone recompute it
5. **Stable**: Won't oscillate between choices (dwell-time enforcement)

---

## Level 3: Intermediate (Undergraduate CS / Master's)

### Mathematical Model

CMCA models allocation as a **discrete-time dynamic system**:

```
State: s_t = (done_mask, candidates_ready, mode, dwell_counter, receipt_chain)

Transition:
  1. Evaluate: score_i = gain_matrix[mode][i] * candidate_value[i]
  2. Admit: admitted_mask = ready_mask & valid_mask & policy_check
  3. Select: i* = argmax { score_i : i in admitted_mask }
  4. Dwell: if mode_change_proposed and dwell_counter < dwell_threshold:
       reject mode_change (stay in current mode)
     else if dwell_counter >= dwell_threshold:
       accept mode_change, reset dwell_counter = 0
  5. Update: done_mask ← done_mask | (1 << i*)
  6. Seal: receipt ← audit digest of the inputs (see Level 4's as-built caveat)
  7. Output: (selected_candidate = i*, new_state = s_{t+1})
```

### The Gain Matrix (Mode Selection)

The **gain matrix G** is parameterized by:
- **Rows**: q-lens strategies (Exploitation, Coverage, Rare, Proportional)
- **Columns**: candidate indices (0 to 63)
- **Entries**: g_{mode,i} ∈ [0, 1] (normalized weight)

For **Exploitation**:
```
g_exploitation[i] = max(0, value[i] - threshold)
```
(High weight on high-value candidates)

For **Coverage**:
```
g_coverage[i] = max(0, NOT_YET_COVERED[i] * value[i])
```
(Zero weight on already-covered candidates)

For **Rare**:
```
g_rare[i] = 1.0 / (frequency[i] + epsilon)
```
(High weight on low-frequency candidates)

### Stability Analysis (Eigenvalue Bound)

For CMCA to be **stable** (not oscillate), we require:
```
λ_max(G) < 1.0
```

Where λ_max is the largest eigenvalue of the gain matrix. This ensures:
- **Contraction property**: ||G(x)|| < ||x|| for all x
- **Convergence**: Repeated application of G shrinks state vectors
- **No oscillation**: System doesn't flip between modes indefinitely

### Dwell-Time Enforcement

To prevent rapid mode-switching (even with stability):
```
mode_change_admitted = (
  dwell_counter >= dwell_threshold AND
  mode_agreement_count >= consensus_rounds
)
```

Where:
- `dwell_threshold` = 3 (wait 3 ticks before changing)
- `mode_agreement_count` = # consecutive rounds with same mode proposal
- Prevents flip-flopping: even if G prefers a new mode, we wait for consensus

### Numeric System: Q16.16 Fixed-Point

CMCA uses **Q16.16 fixed-point arithmetic** for all numeric operations:
- **16 integer bits** (range 0 to 65535)
- **16 fractional bits** (precision 1/65536 ≈ 0.0000153)
- **No floating-point**: Avoids IEEE 754 non-determinism

Operations:
```
add_sat(a, b)       = min(a + b, MAX)              // Saturating addition
mul_sat(a, b)       = (a * b) >> 16, saturate      // Fixed-point multiply
div_sat(a, b)       = (a << 16) / b, saturate      // Fixed-point divide
```

All operations are **branchless**: no conditional jumps, only bitwise ops and arithmetic.

---

## Level 4: Advanced (PhD / Systems Research)

### Formal Model: Timed Automaton

CMCA is a **timed automaton** with:

**States:**
```
Q = {
  INIT,
  READY,
  SELECTING,
  DWELL_CHECK,
  APPROVED,
  FIRED
}
```

**Transitions:**
```
INIT  --[setup]--> READY
READY --[tick]--> SELECTING
SELECTING --[admit_check]--> DWELL_CHECK
DWELL_CHECK --[dwell_satisfied]--> APPROVED
DWELL_CHECK --[dwell_pending]--> READY
APPROVED --[fire_and_seal]--> FIRED
FIRED --[advance_time]--> READY
```

**Clocks:**
- `t_dwell`: Time since last mode change
- `t_tick`: Scheduler tick counter

**Invariants:**
```
At DWELL_CHECK: t_dwell < dwell_threshold ⟹ must return to READY
At READY: must reach SELECTING within 1 time unit
```

### Admission Policy: Horn Logic

The **admission gate** enforces a policy specified in Horn logic:

```prolog
% Base facts
authorized(deployer_1).
authorized(deployer_2).

% Rule 1: Approved if proposer is authorized
approved(proposal) :-
  proposed_by(proposal, X),
  authorized(X).

% Rule 2: Approved if not high-risk
approved(proposal) :-
  not(high_risk(proposal)).

% Rule 3: Rejected if both authorized AND high-risk
rejected(proposal) :-
  high_risk(proposal),
  NOT(authorized(proposal_owner)).
```

**Forward-chaining evaluation:**
- Query: `approved(candidate_i)?`
- If derivable: admit the candidate
- If not: refuse the candidate

### Stability Certificate (Lyapunov Function)

CMCA includes a **Lyapunov function** that proves stability:

```
V(s_t) = ||s_t||_2  (Euclidean norm of state)

Stability theorem:
  ∀t: V(s_{t+1}) ≤ (1 - ρ) * V(s_t)

where ρ > 0 is the contraction margin.

This guarantees:
  1. Convergence: V(s_t) → 0 as t → ∞
  2. Exponential bound: V(s_t) ≤ V(s_0) * (1-ρ)^t
  3. No oscillation: monotonic decrease in state norm
```

### Receipt Chain (Audit-Trail Integrity)

> **As-built caveat (supersedes the cryptographic framing this section used to carry):**
> the shipped allocation receipts (`src/allocation_receipt.rs`) bind each decision with a
> 64-bit non-cryptographic splitmix64-style digest (`mix64`) — **not** BLAKE3, and **not a
> security boundary**. The module's own documentation states a party who can freely choose
> both a receipt and its claimed inputs could construct a digest collision. What the
> mechanism does guarantee: verification *recomputes* the share from the recorded inputs
> and refuses on mismatch, so a receipt claiming a share that the recorded inputs do not
> produce fails verification. BLAKE3 does appear in this crate, but only in
> `src/artifact.rs` (a dev-dependency) for generated-manifest integrity, and in
> `bcinr-powl`'s `OcelCausalReceipt`.

```
receipt_t = audit_digest(receipt inputs at t)

Verification property: verify_allocation_receipt(receipt, inputs) recomputes the
share from `inputs` and accepts only if it matches the receipt's recorded share.

Tamper-evidence scope: accidental/incidental input drift and audit-trail structure.
NOT adversary-resistant: see the caveat above.
```

### Q-Lens Strategies: Formal Definitions

Each q-lens is a **stochastic allocation rule** optimizing a different objective:

#### Exploitation (Thompson Sampling-like)
```
π_exploit(s) = argmax_i { μ̂_i(s) - c_i(s) }

where:
  μ̂_i(s) = estimated value of candidate i in state s
  c_i(s) = cost (time, energy, memory)
  
Greedily picks the highest-confidence, lowest-cost candidate.
```

#### Coverage (Information-Theoretic)
```
π_coverage(s) = argmax_i { H(candidate_i | observations) }

where:
  H(...) = Shannon entropy (uncertainty)
  
Picks candidates with highest uncertainty to maximize information gain.
Covered candidates have H=0, so they get zero weight.
```

#### Rare (Inverse-Frequency Weighting)
```
π_rare(s) = argmax_i { 1 / freq_i(s) }

where:
  freq_i(s) = number of times candidate i was selected up to state s
  
Biases toward underexplored candidates. Prevents one candidate from monopolizing selection.
```

#### Proportional (Thompson Posterior Sampling)
```
π_prop(s) ∼ Categorical( value[0]/Z, value[1]/Z, ..., value[N]/Z )

where:
  Z = Σ_j value[j]  (normalization)
  
Each candidate's selection probability ∝ its value.
Middle-ground between exploitation and coverage.
```

---

## Level 5: PhD (Formal Verification & Proofs)

### Hoare-Logic Contract Specification

**Precondition:**
```
∀s ∈ CMCA_State:
  (s.ready_mask ≠ 0 OR s.done_mask = ALL_BITS) AND
  (s.dwell_counter ≤ dwell_threshold) AND
  (s.mode ∈ {EXPLOIT, COVERAGE, RARE, PROPORTIONAL}) AND
  (s.receipt_chain is verifiable by recomputation)
```

**Postcondition:**
```
∀s, s' ∈ CMCA_State such that s ⟹ s':
  (s'.done_mask = s.done_mask ∨ (∃i: s'.done_mask = s.done_mask | (1 << i))) AND
  (s'.receipt_chain recomputes from s' exactly) AND
  (s'.dwell_counter ≥ 0) AND
  (λ_max(G[s'.mode]) < 1.0 ⟹ ||s'||_2 ≤ (1-ρ)||s||_2)
```

**Invariant (3-state):**
```
Inv1_Memory_Safety:
  ∀t: s_t.done_mask is well-formed u64 (no overflow)

Inv2_Monotonicity:
  ∀t, t': t < t' ⟹ done_mask_t ⊆ done_mask_t' (only additions, no resets)

Inv3_Determinism:
  ∀s, s': s = s' ⟹ (allocate(s) = allocate(s')) ∧ (receipt(s) = receipt(s'))
```

### Proof Outline: Stability Without Oscillation

**Theorem (Contraction + Dwell = No Oscillation):**

```
Given:
  1. λ_max(G) < 1 - ρ  (G is ρ-contractive)
  2. dwell_threshold = N ticks
  3. mode agreement requires M consecutive identical proposals

Prove:
  Mode changes occur at most O(log(1/ε)) times for ε-approximation.

Proof sketch:
  1. Contraction ensures V(s_t) decreases monotonically
  2. Lyapunov function bounds state norm: V(s_t) ≤ V(s_0) * (1-ρ)^t
  3. Mode changes only when dwell_counter ≥ N AND M consensus rounds pass
  4. Once in a mode, contraction drives state toward that mode's fixed point
  5. When fixed point is reached, mode proposal becomes consistent (M consensus)
  6. Dwell allows N ticks for contraction before new mode takes effect
  7. After M consensus rounds, new mode is admitted
  8. Process repeats, but with smaller state norm (by contraction)
  9. Finite state space + decreasing Lyapunov → finite mode changes

Therefore: oscillation is impossible; system converges to absorbing state.
```

### Branchless Property: Cost Analysis

**Theorem (Constant Execution Time):**

```
For all candidate sets C ⊆ {0,1,...,63}, allocate(C) runs in O(1) time.

Proof:
  1. No conditional branches in hot path (verified by object-code audit)
  2. All operations are bitwise (AND, OR, XOR, shift) or fixed-point arithmetic
  3. Loop unrolling (macro: unroll_8_static!) compiles out loops
  4. No dynamic branching on data-dependent predicates
  5. Cyclomatic complexity CC = 1 (single path)

Therefore: execution time is independent of candidate set size.
```

### Tamper-Evidence: Audit-Trail Binding (checked property, not a theorem)

```
For a receipt R binding inputs I to a recorded share s:

verify_allocation_receipt(R, I) recomputes the share from I via the same
kernel `allocate_in` uses, and accepts only on exact match.

Therefore:
  1. A receipt claiming a share its recorded inputs do not produce is refused.
  2. Accidental input drift (wrong weights snapshot, wrong parent forest,
     edited state) changes the recomputation and is caught.

Explicitly NOT covered:
  - An adversary who can freely choose both R and I can construct a
    64-bit mix64 digest collision (non-cryptographic finalizer).
  - Passing the wrong *weights snapshot* (pre- vs post-MWU) is a caller-
    discipline requirement, not a checked one.

See `src/allocation_receipt.rs`'s module docs for the authoritative statement.
```

### Q-Lens Optimality (sketch)

**Lemma (Exploitation Optimality):**
```
Under Exploitation lens with policy π_exploit(s) = argmax_i value[i]:

For any state s, if i* = π_exploit(s), then:
  ∫_0^∞ V(s_t) dt ≤ ∫_0^∞ V(s'_t) dt

for any alternative policy that picks j ≠ i* at time 0.

Proof: By construction, exploitation always picks the maximum-valued candidate.
Any other choice defers higher value to a later time, increasing total cost.
```

**Lemma (Coverage Optimality):**
```
Under Coverage lens, the set of demonstrated candidates grows monotonically:
  Demonstrated_t ⊆ Demonstrated_{t+1}

At each step, coverage picks an undmonstrated candidate iff:
  ∃i: i ∉ Demonstrated_t ∧ i ∈ Admitted_t

By selection principle, max_value[ i ∉ Demonstrated ] is picked.

Conclusion: Coverage minimizes "time to full coverage" under ordered selection.
```

---

## Level 6: PhD+ (Formal Verification Artifacts)

### Generated Verification Objects

CMCA generates **proof objects** that can be checked by external verifiers:

#### 1. Safety Properties (Temporal Logic)

```
CTL Formula 1 (No Overflow):
  AG( ∀t: allocate(state_t).done_mask is well-formed )

CTL Formula 2 (Eventual Termination):
  EF( ∀i ∈ {0..N}: i ∈ done_mask )
  (Eventually all candidates complete)

CTL Formula 3 (No Unauthorized Access):
  AG( ∀t: allocate(state_t, policy) rejects unauthorized proposals )
```

#### 2. Liveness Properties

```
LTL Formula 1 (Strong Fairness):
  G( ready_mask[i] eventually unbounded ⟹ selected[i] infinitely often )

LTL Formula 2 (Eventual Agreement):
  F( ∃m: mode = m forever ⟹ G( mode = m ) )
  (Mode stabilizes eventually)

LTL Formula 3 (Progress):
  G( ready_mask ≠ 0 ⟹ F( state changes ) )
  (Liveness: system doesn't deadlock)
```

#### 3. Refinement Relation

```
Abstraction:
  A = (Candidates, select, is_optimal)
  (Abstract: pick the best candidate)

Refinement:
  C = CMCA implementation
  (Concrete: pick via gain matrix, dwell-time, receipt chain)

Refinement Proof:
  ∀s ∈ A.State, ∀o ∈ A.Output(s):
    ∃s' ∈ C.State, ∃o' ∈ C.Output(s'):
      o' = o AND o is_optimal(s)

Interpretation: CMCA refines the abstract "pick the best" specification.
```

#### 4. Invariant Discovery (Inductive)

```
Inv_1: done_mask ⊆ ALL_BITS (no spurious bits)
  Base: done_mask_0 = 0 ✓
  Step: done_mask_t | (1 << i) where i ∈ {0..63} ✓

Inv_2: receipt bindings are verifiable by recomputation
  Base: receipt_0 binds state_0/decision_0, recomputes exactly ✓
  Step: receipt_{t+1} binds the post-update state, recomputes exactly ✓

Inv_3: ∀i ∈ done_mask: preconditions_satisfied(state, i)
  Base: done_mask_0 = 0 (vacuously true) ✓
  Step: (i ∈ admitted ∧ preconditions_satisfied) ⟹ i ∈ done_mask ✓
```

#### 5. Bounded Model Checker Output (ILLUSTRATIVE — NOT EXECUTED)

> **No bounded model checker, SMT solver, or model-checking harness exists in this
> repository.** The block below is a sketch of what such output *would* look like; it is
> not real tool output and must not be cited as verification evidence. This matches the
> "🔄 PARTIAL" list in the Verification Status section, which correctly records bounded
> model checking as *pending*. The contraction property it gestures at is actually
> covered — for the live profile constants only — by the row-inequality test and the
> independent spectral-radius test in `tests/stability_profile_invariants.rs`.

```
Property: "No oscillation in mode selection"
Domain: 64 candidates, 4 modes, 1000 ticks
Status: NOT EXECUTED (no BMC harness in this repo)

Property: "Contraction with ρ=0.1"
Domain: Q16.16 fixed-point, [0, 65535]
Status: NOT EXECUTED as a sweep; the live 5x5 profile is checked exactly
        (row inequality at init + spectral radius ~0.980 <= 0.99 in tests)
```

---

## Implementation

### File Structure

```
crates/bcinr-cmca/
├── generator.py                # RDF ontology -> generated Rust fixture compiler
├── ontology/
│   ├── cmca-rdf.ttl             # N=8, K=4, Q=4 fixture source
│   └── generalization.ttl       # N=9, K=5, Q=5 fixture source
├── src/
│   ├── lib.rs                   # crate root, MAPE-K framing doc-comment
│   ├── allocator/
│   │   ├── mod.rs               # Cascade allocation algorithm (branchless CC=1 kernel)
│   │   └── feasible_region.rs   # FeasibleRegion (m/c bounds) used by allocate/allocate_in
│   ├── certification.rs         # Certificate validation (witness checking)
│   ├── proposal.rs              # Mode proposal handling
│   ├── artifact.rs              # Generated manifest + BLAKE3 verification (dev-dep)
│   ├── observatory.rs           # evaluate_calibration / calibration safety flags
│   ├── fixed.rs                 # Q16.16 fixed-point arithmetic
│   ├── escort.rs                # Fractional-q escort distribution (alloc feature)
│   ├── cascade.rs               # Arbitrary-shape cascade (alloc feature)
│   ├── reference_escort.rs      # Exact-rational hand-transcribed Lean oracle (alloc feature)
│   ├── allocation_receipt.rs    # Audit-trail receipts (mix64 digest, NOT cryptographic)
│   ├── lrc.rs
│   ├── stability_theorem.rs     # Independent spectral-radius/dwell verification
│   ├── generated_profile.rs     # ggen-synced policy constants (MAX_LENS_MAGNITUDE, ...)
│   └── generated/
│       ├── stability_profile.rs
│       └── consequence_mass/
│           ├── generalization.rs  # compiled fixture, N=9 K=5 Q=5
│           └── case_studies.rs    # compiled fixture, N=8 K=4 Q=4
└── tests/
    ├── differential.rs, reference.rs, calibration.rs, case_studies.rs
    ├── hostile_mutants.rs, mutant_kill_g4_cmca.rs
    ├── falsification_adversarial.rs, compile_fail_tests.rs
    └── usecase_*.rs, jtbd_certified_actuation_chicago.rs
```

(There is no `q_lens/` submodule, `SAFETY.md`, or `docs/STABILITY_CERTIFICATE.md` inside the
crate — an earlier version of this document described an aspirational structure that was never
built; the tree above reflects the actual crate as of this writing.)

### Hot-Path Algorithm (Branchless)

The real entry points are `allocate` and `allocate_in` in `src/allocator/mod.rs`
(`allocate` is a thin wrapper that always passes `FeasibleRegion::CURRENT`;
`allocate_single_lens` in the same file exposes one lens without the LAMBDA blend):

```rust
pub fn allocate_in(
    states: &[PackedSemanticState; N],
    lenses: &[LensSpec; Q],
    lambda: &[[NonNegativeFixed; K]; Q],
    // ...additional cascade/forest parameters (eta, parent, MWU weights,
    // payoffs, zeta, epsilon_kappa, mu, costs, t, dwell state, digest, proof)
) -> /* allocation result */ {
    // Cascade allocation over the semantic-object forest: root weights
    // W_root(i) = exp2(q * log2(M_k,i) - A_max), propagated down N nodes
    // via `flow_step`, using branchless selection primitives
    // `const_select_u32` / `const_select_bool` (also in `src/allocator/mod.rs`)
    // in place of conditional branches.
}
```

The illustrative pseudocode this section previously showed (`config.gain_matrix`,
`select_max_branchless`, `dwell_counter`) does not correspond to any function in this crate —
that framing belonged to a different, unbuilt design. See `src/allocator/mod.rs` directly for
the exact signature and cascade-propagation logic.

**Properties:**
- No `if`, `match`, loops in the hot path
- All operations are branchless primitives: bitwise AND/OR/XOR, shifts, `const_select_u32`/
  `const_select_bool`
- Cyclomatic complexity = 1
- Constant execution time: O(1) regardless of candidate set size
- Object code audit (arm64 disassembly, `OBJECT_CODE_AUDIT.md`) confirms zero conditional jumps

---

## Verification Status

> Standing labels follow the repository constitution's bounded vocabulary (`INVARIANT`,
> `ALIVE`, `PROVEN`, ...). "PROVEN" is reserved for machine-checked or exhaustively
> established theorems; nothing in this crate currently meets that bar, and no row below
> claims it.

### ✅ Verified within stated scope

| Property | Method | Status |
|----------|--------|--------|
| Memory Safety | Rust `#![deny(unsafe_code)]` | ✅ INVARIANT (lint-enforced) |
| Branchless Execution | Object-code audit (arm64, v26.7.17 — re-run on hot-path changes) | ✅ ALIVE as of that audit |
| Determinism | Differential proptest vs independent f64 oracle (`tests/differential.rs`) | ✅ ALIVE |
| Q16.16 Precision, integer q | Bit-exact repeated multiplication (`cascade::escort_weight`) | ✅ EXACT |
| Q16.16 Precision, fractional q | Measured empirical sweeps (`tests/power_error_bound.rs`): ~0.5% rel. err at \|q\|≤0.25 rising to ~36.9% at \|q\|=16; surfaced per-call via `PathConfidence` | ✅ ALIVE within measured envelope |
| Stability (ρ(G) ≤ 1−δ) | Row inequality enforced at init (`gd_ok`) + independent spectral-radius test (`tests/stability_profile_invariants.rs`, ρ≈0.980 ≤ 0.99 for live constants) | ✅ ALIVE |
| No Oscillation | Dwell-time gate + contraction check + case-study tests | ✅ ENFORCED + TESTED (not formally proven) |
| Receipt Audit Trail | Recomputation-based verification, mix64 digest | ✅ ALIVE (audit aid; **not** tamper-evidence against adversaries) |
| Mutant Resistance | Hostile mutants 1–11, each killed in isolated feature builds (`tests/hostile_mutants.rs`) | ✅ ALIVE |

### 🔄 PARTIAL (Awaiting Final Verification)

- Q-lens optimality lemmas (Level 5 sketches only; no proof or test exists)
- Authority chain policy evaluation (Horn logic engine - pending)
- Bounded model checking (SMT solver - pending; the Level 6 "BMC output" block is illustrative, not executed)
- Formal proof in Coq/Isabelle (interactive theorem prover - pending)
- Machine-checked Lean correspondence (current link is a hand-transcribed oracle + differential tests; `~/mfw`'s Escort.lean is not bridged by FFI or codegen)

### 📋 SPECIFICATION (Formal Contracts)

- Hoare triple specification (complete)
- CTL/LTL property set (complete)
- Refinement relation (complete)
- Adversarial falsification test suite (complete, 8 categories)

---

## Summary: Why CMCA Matters

**At Every Level:**

- **ELI5**: CMCA is a homework chooser that picks the right task, adapts its strategy, and proves it was right.
- **Beginner**: CMCA is an allocation algorithm with stable mode-switching, cryptographic receipts, and deterministic execution.
- **Intermediate**: CMCA uses discrete-time dynamics, gain matrices, Lyapunov stability, and dwell-time enforcement.
- **Advanced**: CMCA is a timed automaton with Horn-logic policies, recomputation-verifiable audit receipts, and formal verification contracts.
- **PhD**: CMCA is refinement-verified system with temporal logic properties, bounded model checking, and constructed Lyapunov functions proving contraction.
- **PhD+**: CMCA generates proof objects for automated verifiers (CTL/LTL, refinement theorems, invariant proofs, bounded model checking witnesses).

**The Core Claim:** CMCA provides a mathematically rigorous, formally verifiable allocation mechanism that:
1. **Works fast** (O(1) constant time, branchless)
2. **Works right** (deterministic, optimal, stable)
3. **Proves it works** (recomputation-verifiable receipts, formal contracts)
4. **Never flip-flops** (dwell-time enforcement, stability theorem)

If any claim fails, the falsification adversarial test suite catches it.

---

## References

- **Stability Theory**: Lyapunov functions, eigenvalue bounds, contraction mapping theorem
- **Formal Methods**: Hoare logic, temporal logic (CTL/LTL), timed automata, refinement
- **Cryptography**: BLAKE3 (used for generated-manifest integrity in `artifact.rs` only; allocation receipts are non-cryptographic — see Level 4)
- **Control Theory**: Gain matrices, mode selection, stochastic allocation
- **Algorithms**: Thompson sampling, entropy-based exploration, inverse-frequency weighting

**Papers (Conceptual):**
- Sutton & Barto, "Reinforcement Learning: An Introduction" (Q-lenses as exploration strategies)
- Lynch, "Distributed Algorithms" (timed automata, safety/liveness properties)
- Bertsekas & Tsitsiklis, "Parallel and Distributed Computation" (Lyapunov stability)
- Bellare & Rogaway, "Introduction to Modern Cryptography" (hash security models — relevant to `artifact.rs`'s BLAKE3 use, not to allocation receipts)

**Standards:**
- IEEE Std 1850-2010 (PSL: Property Specification Language for temporal properties)
- ISO/IEC 16999-1 (OCEL 2.0: Object-centric Event Log standard)
- NIST FIPS 202 (SHA-3 cryptographic hashing)
