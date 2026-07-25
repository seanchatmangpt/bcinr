# Autonomic Module Architecture (`bcinr-logic/src/autonomic/`)

The `autonomic` module provides generic building blocks for self-managing systems following the MAPE-K (Monitor-Analyze-Plan-Execute-Knowledge) autonomic loop. It is designed to be branchless and deterministic, adhering to the project's core architectural laws (Radon Law, `CC=1`, and zero-allocation boundaries).

## Directory Structure
The module consists of the following components:
- `mod.rs`: The module's entry point exposing sub-modules and defining the baseline `AutonomicHealth` struct.
- `kernel.rs`: Defines the formal `AutonomicKernel` interface.
- `autonomic_substrate.rs`: Provides the `AutonomicSubstrate` implementation, serving as a generic MAPE-K container.
- `rl_state.rs`: Defines the Reinforcement Learning state.
- `policy_guard.rs`: Defines boundary guards and acceptance policies.
- `packed_key_table.rs`: An allocation-free, fixed-capacity key-value table.
- `metric_accumulator.rs`: Accumulator for metrics branchlessly.

## Key Architectural Components

### 1. `AutonomicKernel` (`kernel.rs`)
The kernel defines the axiomatic state transition interface for self-management:
- Represents the MAPE-K lifecycle with branchless methods: `observe()`, `infer()`, `propose()`, `accept()`, `execute()`, `manifest()`, and `adapt()`.
- Uses domain types like `AutonomicState`, `AutonomicAction` (with `ActionKind` & `ActionRisk`), and `AutonomicFeedback`.
- Public primitives ensure `CC=1` (no hidden branches). Features requiring allocations (`alloc`) are strictly feature-gated to separate hot-path operations.

### 2. `AutonomicSubstrate` (`autonomic_substrate.rs`)
A concrete modular container that pairs **knowledge** with **state**:
- **Knowledge Base**: Modeled using `PackedKeyTable<K, V, N>`, keeping the dataset allocation-free and bounded to a capacity of `N`.
- **System State**: Modeled using `RlState`.
- Designed for zero-allocation execution inside a constant-time hot path.

### 3. Formal Constraints
- **Radon Law (`CC=1`)**: The code rigorously follows deterministic execution rules with no conditional jumps or data-dependent loops. 
- **Axiomatic Design**: Hoare-logic proofs and counterfactual mutant tests (e.g., `mutant_kernel_1`, `mutant_substrate_1`) are baked in to guarantee equivalence to the mathematical specification.
