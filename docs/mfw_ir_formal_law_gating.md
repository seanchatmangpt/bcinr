# Multifractal Workflow Intermediate Representation (MFW-IR)

The `bcinr-mfw-ir` crate serves as the strict, dependency-isolated foundation for the Multifractal Workflow (MFW) planner. Aligning with BCINR's rigorous architectural laws, it establishes the shared intermediate representation (IR) types and mathematical trait contracts while enforcing zero path-dependency on algorithmic crates like `bcinr-pddl` or `bcinr-powl`.

## 1. The Strict Dependency Boundary
`bcinr-mfw-ir` is designed as a foundational substrate. It contains **no heavy algorithmic logic**, and specific planning concepts (like PDDL or POWL specific nodes) are strictly prohibited from residing here.
- `bcinr-powl` and `bcinr-pddl` depend on `bcinr-mfw-ir`.
- `bcinr-mfw-ir` depends on neither. 

This inverted dependency model isolates the primitive types (`Digest`, `EpochBounds`, `MinimalNonFace`, `CausalPlan`) and witness algebras (`IndependenceWitness`, `ConcurrencyPreservationWitness`) from their complex implementations.

## 2. Formal Claim Ceiling and Sibling Verification
While BCINR leverages Hoare-logic proofs and proptest oracle equivalence internally, it **never claims Lean/Coq verification natively**. All formalized math and proofs (via `mathlib4`) live exclusively in the sibling repository: `/Users/sac/mfact`.

The MFW-IR bridges this gap by baking hand-curated citations into inspectable constants (`FormalLawRef`), enabling the runtime to acknowledge mathematical laws mathematically proven in `mfact`.

## 3. The `FormalStanding` Classification
Optimizations in BCINR cannot assume unverified properties. The MFW-IR defines a closed, four-way classification of mathematical standing:

1. **`Proven`**: Proven in Lean 4 (sorry-free). The citation points exactly at the proven lines. Only this standing licenses computational optimizations.
2. **`Stated`**: Defined in Lean, but lacking a proof for the specific properties a caller might assume (e.g., defined as a set-intersection but not proven downward-closed).
3. **`Conjectural`**: The open goal of the `mfact` project itself. No proof attempted.
4. **`Blocked`**: No proof and no Lean-formalized representation exists.

## 4. `SemanticOptimizationContract` and Typed Refusals
To enforce the deterministic safety of the system, any downstream code that attempts to skip a computation in favor of a cached or residualized result must prove the optimization is mathematically licensed.

The `FormalStanding::permits_optimization()` method strictly evaluates to `true` **only** for `Proven` laws. If a subsystem attempts to construct a `SemanticOptimizationContract` using a law that is merely `Stated` or `Conjectural`, the system immediately rejects the operation with a branchless typed refusal: `ContractError::LawNotProven(standing)`. 

This guarantees that the runtime behavior is never built on top of conjectural or blocked mathematics, preserving the system's "Contract with Teeth" execution guarantees.
