# v26.7.28 Ownership Boundary

## Capability agents

The capability workstream owns source semantics and capability-local tests for:

- PDDL grounding, durations, and temporal planning;
- POWL partial-order execution;
- scheduler and resource law;
- typed capability refusals;
- OCEL evidence, BLAKE3 execution roots, and replay semantics;
- deterministic swarm scenario behavior;
- hostile capability fixtures and semantic mutants.

## Production-admission workstream

`tools/bcinr-release`, `release/v26.7.28`, and the production-admission workflow own:

- exact repository and dependency object admission;
- clean-tree and exact-head enforcement;
- bounded execution of verifier rails;
- retained command logs and log digests;
- generated-artifact byte identity;
- workspace, cross-target, FFI, and benchmark execution;
- release standing calculation;
- production promotion, evidence retention, and rollback law.

The production-admission workstream may invoke capability verifiers but may not weaken, duplicate, or redefine capability semantics.

## Integration order

1. Capability agents return repository-grounded gap matrices.
2. Capability changes converge onto one authoritative temporal surface.
3. The production-admission branch incorporates the exact accepted capability commit.
4. The release profile executes every required verifier against that exact integrated tree.
5. The draft remains blocked until the resulting receipt is `ALIVE`.

Shared manifests, `Cargo.lock`, generated-artifact integration, release workflows, and final mergeability are owned by the production-admission integration pass to prevent concurrent edits from creating divergent authority surfaces.
