# The Chatman Equation in the Chess Factory

The Chatman Equation is the foundational architectural principle governing the `chess-factory`'s execution under extreme latency constraints (e.g., a 100-microsecond time budget per move). It dictates that the runtime system must not spend time dynamically discovering *how* to evaluate a position; instead, the search architecture is pre-compiled and selected via an $O(1)$ lookup.

The equation is defined as:

$$A = \mu(O^*)$$

Where:
* **$O^*$**: The admitted position state (comprising the board, phase, time budget, hardware class, and budget class).
* **$\mu$**: The topology selection function (an $O(1)$ table lookup).
* **$A$**: The receipted action (a move with evidence).

## Mechanics of the $O(1)$ Table Lookup

At 100µs per move, the branching factor of architecture discovery is fatal. An engine that spends 20µs deciding how to search only has 80µs left to actually search. 

To avoid this, the topology selection function **$\mu$** is implemented as a strict $O(1)$ table lookup to avoid any runtime search overhead:

1. **Pre-compilation**: Before the game is played, the "factory" pipeline benchmarks and compiles optimal architectures (topologies) for different combinations of game states based on semantic laws.
2. **Indexing**: These compiled topologies are stored in a table indexed by the deterministic parameters of $O^*$, specifically `(phase × hardware × time_budget)`.
3. **Selection over Discovery**: At runtime, when the engine receives a position, it statically determines the $O^*$ parameters (for example, classifying the position into a phase like Opening, Tactical, Quiet, Endgame, or Tablebase using zero-sized TypeStates).
4. **$O(1)$ Routing**: Using the $O^*$ parameters, the engine performs a direct $O(1)$ index lookup into the topology table. It instantly retrieves the correct POWL (Process Order With Limits) search graph for that specific state.

## The Manufacturing Inversion

The Chatman Equation represents an inversion of traditional software design. By compiling the architecture into a static table lookup, the equation moves the "intelligence" out of the runtime logic and into the offline manufacturing pipeline. The runtime engine spends 0µs figuring out *how* to search, allowing it to dedicate its entire bounded time budget to executing the pre-compiled, branchless (CC=1) evaluation topologies.
