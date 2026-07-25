# Prolog8 Admission Gate ($R \vdash A$)

The `Prolog8` admission gate is the fundamental zero-trust authorization boundary within the BRCE (Branchless, Receipt-producing, Cryptographic, Execution) stack. It mathematically enforces authorization logic by evaluating whether a defined policy (Rules and Facts, $R$) logically entails the permission to execute a specific operation (Assertion, $A$).

## The BRCE Execution Loop
The integration of Prolog8 is part of the strict candidate-future execution loop:
`PDDL8 → POWL tape → Prolog8 admission → OCEL → BLAKE3 receipt`

Before any operation from a PDDL plan (represented as a `Pddl8Tape` or `TemporalPlan`) is allowed to mutate state, it must pass the Prolog8 gate.

## $R \vdash A$ Logic 

The logic evaluates $R \vdash A$:
* **$R$ (Policy Rules & Facts):** A collection of facts and Horn clauses dictating system authorization constraints. These are loaded into the `prolog8::Kernel` before execution begins.
* **$A$ (Assertion):** The `may_fire(label)` query. For every operational step, the engine asks the kernel if the specific action label is permitted to execute.

If $R$ logically entails $A$ (`QueryResult::Answered`), the gate opens and the step runs. If not, the gate triggers a typed refusal (`Pddl8Error::StepDenied`).

## Implementation Details

The implementation lies primarily in `crates/bcinr-pddl/src/execute.rs` via the `Ctx` struct wrapping `prolog8::Kernel`.

### 1. Context and Policy Loading
When initiating an execution loop (e.g., via `execute_tape` or `execute_temporal_plan`), the system initializes a Prolog8 `Kernel` and a `Catalog`.
* **Empty Policy:** If no explicit policy rules are provided, every scheduled operation in the tape is automatically pre-admitted via a `FactBlock8` fact (`ctx.load_may_fire(&op.label)`).
* **Explicit Policy:** If policy rules are provided, they are loaded as Horn clauses via `load_may_fire_rule(head, body)`. These rules take the shape of a `Rule8` bounded to a maximum of 8 body atoms, ensuring deterministic execution complexity.

### 2. The `may_fire` Query ($A$)
For every operation `op` in the execution tape:
1. A `QueryAtom8` is constructed for `may_fire(op.label)` with `ProofMode::PositiveOnly`.
2. The `prolog8::Kernel` processes the query deterministically.
3. If the kernel returns `QueryResult::Answered(_)`, the operation's effects are applied, state is mutated, and the BLAKE3 receipt chain hashes the step.
4. If the kernel denies the query, the execution halts instantly with a typed refusal (`Pddl8Error::StepDenied`) and reason `"Prolog8 denied may_fire(<label>)"`.

### 3. BRCE Constraints and Cryptographic Receipting
The `Prolog8` gate adheres to the branchless constraints of the BCINR mandate:
* **Zero-Trust**: Without explicit fact loading or rule deduction, all operations are denied.
* **Cryptographic Attestation**: The admission gate runs atomically with the BLAKE3 hash chain. Only actions that successfully pass the $R \vdash A$ check have their effects, labels, and timestamps merged into the rolling state chain (`Pddl8ExecutionReceipt`), thereby providing cryptographically verifiable proof of authorized state mutation.
* **Bounded Deduction**: Rule complexity is physically bounded (e.g., maximum 8 body atoms per rule), preventing unbounded loops or exponential search paths in the policy engine and guaranteeing $O(1)$ verification execution.
