# bcinr v26.9.16 — RFC Closure Contract

Status: DRAFT IMPLEMENTATION PR.

## Canonical Jira tickets

- A2A-2605 — CMCA bounded resource-allocation control plane
- A2A-2606 — recursive MFW / DME closure
- A2A-2612 — machine-experience compile-back

## RFC ownership

This repo owns the bounded formal planning/allocation substrate used by the DME architecture:

- admitted planning tasks and capability profiles;
- PDDL/POWL/Prolog admission and execution receipts;
- MFW frontier, q-lens, rails, consequence horizons and bounded epochs;
- cost/capability vectors needed by CMCA;
- LLM-first-mile candidate admission without authority;
- reusable deterministic planning machinery produced from admitted discoveries.

## Required closure

1. Expose a stable allocator-facing interface over existing `FrontierMeasure`, `MassVector`, q-lens, `CostVector`, fair-rail and consequence-horizon machinery.
2. Represent the DME work classes `KNOWN`, `UNKNOWN_LOCAL`, `UNKNOWN_DEFERRED`, `UNKNOWN_FRONTIER`, `REFUSED`, `BLOCKED`, `UNSUPPORTED` without granting execution authority.
3. Guarantee hard epoch/budget/fan-out/depth ceilings; exhaustion returns a typed witness instead of silently expanding search.
4. Provide a machine-experience compilation contract that can turn a qualified admitted solution into a reusable capability/planning artifact with exact content identity.
5. Preserve the existing law: LLM proposes; substrate admits.

## Chicago falsifiers

- requesting additional search mass automatically increases the budget;
- equivalent KNOWN work enters an exploratory LLM/search rail when an admitted deterministic route exists;
- bound exhaustion is represented as success;
- candidate PDDL/domain text bypasses admission;
- compiled experience changes semantic identity without changing its digest;
- a planning artifact acquires DO authority.

## Definition of done

Repository-native exact-head verification proves bounded allocation, bounded exhaustion, deterministic reuse, admitted compile-back, and receipts for each transition. The resulting interfaces are consumed by the ggen and ash_a2a v26.9.16 branches without duplicate planner/allocation implementations.
