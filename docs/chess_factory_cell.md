# The `chess-factory` Manufacturing Cell

The `chess-factory` in the BCINR codebase is a demonstration of **manufacturing intelligence**. It inverts the traditional approach to building a chess engine: instead of hand-coding domain expertise (heuristics, evaluation parameters) into a monolithic program, the system functions as a *factory* that generates the engine from semantic laws. 

The core philosophy is: **The program is the output of the factory. The factory is the intelligence.**

## The Factory Pipeline (`ggen sync`)

The manufacturing process is driven by semantic law and code generation:

1. **Law Encoding (`ontology/chess.ttl`)**: Chess evaluation laws (like piece values, PSTs, passed pawn bonuses) are encoded as RDF triples in a TTL ontology.
2. **Manufacturing (`ggen sync`)**: The `ggen sync` CLI tool reads the TTL ontology and manufactures Rust source code. For every evaluation station or motif defined in the law, `ggen` generates:
   - A branchless kernel (CC=1) representing the production implementation.
   - A branchful reference oracle.
   - A proptest that verifies the kernel against the oracle.
3. **Verification**: If the branchless kernel deviates from the oracle by even a single bit, the proptest fails, preventing admission of the generated code.
4. **Receipt Generation**: The engine produces OCEL 2.0 execution receipts, ensuring every move can be traced back to the exact evidence and topology that manufactured it.

## The GGEN Heuristic: Factory vs. Craftsman

The architecture strictly divides responsibilities between the automated factory and the human programmer (craftsman).

**The Factory Manufactures:**
* **Parametric Identity**: Logic that varies only in numbers (e.g., `cf:passed_pawn` rank bonuses). Changing behavior means editing a TTL literal and re-running `ggen sync`, not touching Rust code.
* **Law-Oracle Duality**: Components requiring mathematical proof via a branchless kernel and an independent branchful reference oracle.
* **Structural Multiplicity**: N instances of the same structural shape (e.g., generating 10 evaluation stations and 4 tactical motifs).

**The Craftsman Writes:**
* **Algorithmic Identity**: Complex control-flow logic (like alpha-beta search or null-move pruning) where the branching structure *is* the algorithm.
* **Runtime State**: Mutable state spanning recursive calls, such as transposition tables and history heuristics.

*Litmus Test*: If you can change the component's behavior by editing a TTL literal and running `ggen sync`, it belongs in the factory.

## The Chatman Equation

To meet a strict 100-microsecond time budget per move, the system cannot spend time dynamically searching for the correct evaluation architecture at runtime. This is governed by the **Chatman Equation**:

```text
A = μ(O*)
```
Where:
* **`O*`**: The admitted position state (board, phase, time budget, hardware class).
* **`μ`**: The topology selection function (an O(1) table lookup).
* **`A`**: The receipted action (a move with evidence).

Instead of discovering the right architecture during the move, the factory *compiles* the winning architecture before the game. 

## POWL v2 Integration

The factory's control flow uses the **POWL v2 (Process Order With Limits)** architecture to create a schedulable representation of search operations (`Powl64Op`). 

* **Topology-derived concurrency**: Operations fire automatically via branchless SWAR evaluation when their `pred_mask` dependencies are satisfied.
* The search graph topology serves as the concurrency specification, eliminating explicit thread spawning and enabling parallel evaluations based purely on data readiness.
* **Phase TypeStates**: Game phases (Opening, Tactical, Quiet, Endgame, Tablebase) are represented as zero-sized TypeStates (`SearchState<Phase>`) that guarantee safe compile-time ordering rather than relying on runtime depth metrics.

## `cargo-make` Orchestration

The factory's lifecycle is orchestrated via `Makefile.toml`, executing the manufacturing and verification steps sequentially:

1. **`chess-factory-sync`**: Runs `ggen sync` in `crates/chess-factory` against `chess.ttl`.
2. **`factory-build`**: Builds the `chess-factory` crate.
3. **`factory-verify`**: Applies the `bcinr-contract-gate` tool over generated stations and motifs to strictly enforce the CC=1 branchless contract.
4. **`factory-bench`**: Runs a smoke benchmark (e.g., testing against a `sanity_random` cell) and emits an Elo curve to measure performance.
5. **`factory-kaizen`**: Analyzes the benchmark curve and emits architectural feedback (e.g., a `DemoteStationWeight` recommendation) back into the system.
6. **`factory`**: A meta-task that executes this entire lifecycle chain.
