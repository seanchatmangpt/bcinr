# PDDL Planner / Compiler (PDDL8) Implementation Details

The PDDL planner and compiler implementation in this codebase is built as a strict, allocation-minimized, and bounded execution engine targeting the BRCE (Bounded, Branchless, Allocation-Free) stack. The primary crate is `bcinr-pddl`.

## BRCE Stack Architecture

The pipeline follows a strict isolation guarantee, preventing logic from leaking into downstream consumers unless explicitly opted-in via the `mfw-planner` feature:

1. **PDDL8 (Candidate-Future Grammar):** PDDL 3.1 syntax is parsed into canonical AST types and grounded (`parse.rs`, `ground.rs`).
2. **POWL (Process Geometry):** Grounded plans are represented as a `Pddl8Tape`.
3. **Prolog8 (Admission Gate):** Execution checks a strict $R \vdash A$ logic gate (`execute.rs`).
4. **OCEL (Execution Trace):** Emits object-centric event logs for analysis.
5. **BLAKE3 (Receipt Chaining):** Hashing secures the deterministic execution trace (`Pddl8ExecutionReceipt`).

Types are fundamentally decoupled; canonical structures like `Pddl8Domain`, `Pddl8Problem`, and `Pddl8Tape` reside in `wasm4pm-compat` to avoid pulling parsing dependencies into the core runtime.

## MFW Portfolio Search Architecture

The planner uses a "Massive Frontier Weight" (MFW) Portfolio approach (`search.rs`), splitting the search into **Exact** and **Exploit** rails driven by a **Fairness Scheduler**.

### Exact Search Rail
* **Implementation (`ExactBfsRail`)**: Wraps a strict Breadth-First Search (`GroundProblem::find_plan`). 
* **Role**: It is the *only* rail that has the authority to claim a search space is `Exhausted` (proving no plan exists) or `Bounded`. It guarantees completeness.

### Exploit Search Rail
* **Implementation (`QLensRail`)**: A greedy best-first heuristic rail with *no backtracking*.
* **Mechanism**: At each step, it scores applicable ground actions by how many new goal atoms they satisfy, normalizes these scores, and advances along the highest-weighted action.
* **Role**: Can only claim `Idle` (dead end/exhausted heuristic frontier) or `Candidate` (a potential, unverified plan). It never formally guarantees exhaustion.

### Scheduler & Portfolio
* **FairRailScheduler**: A round-robin scheduler over exploit rails that strictly enforces a fairness floor (`max_gap`). Regardless of exploit results, the Exact rail is guaranteed to be ticked periodically to avoid starvation.
* **MfwPortfolio**: The top-level bounding loop. It collects `Candidate` plans from the Exploit rails while yielding execution until the Exact rail eventually terminates the search with a definitive `Found`, `Exhausted`, or `Bounded` result, or the `max_ticks` budget is reached.

## The Q-Lens Ratio Law ($L_q$)

The `q_lens` implementation (`mfw/mod.rs`) defines the probability weighting function for the Exploit rail:
$$L_q(i) = \frac{p_i^q}{\sum_j p_j^q}$$

* **Formal Standing**: This is explicitly grounded in Lean proofs (`bcinr_mfw_ir::contracts::LAW_QLENS_RATIO`), guaranteeing that $\sum L_q(i) = 1$ and that ordering is preserved for $q > 0$.
* **Typed Refusal Constraints**: It rigidly enforces boundaries via `QLensError`, refusing non-finite values, zero/negative masses, or degenerate normalizations rather than allowing unsafe floating-point NaN/Inf contagion.

## Execution and Admission (Prolog8)

The execution of a `Pddl8Tape` (`execute.rs`) enforces the determinism and authorization required by the project laws:
* **Gatekeeping**: Every step evaluated on the tape must pass a `may_fire(label)` query against the embedded `prolog8` rule engine.
* **Receipting**: As effects apply and tick epochs, a continuous BLAKE3 hash chain aggregates the execution trace, producing a cryptographic `Pddl8ExecutionReceipt`. This makes the state transitions and rule authorizations strictly auditable.
