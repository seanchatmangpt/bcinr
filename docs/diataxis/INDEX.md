# Documentation Index: bcinr Branchless Calculus

Welcome to the comprehensive documentation suite for the bcinr library. This index maps our full Diátaxis documentation, categorized by intent.

> **Removed 2026-10-01 (gate lane, release/26.9.15):** the former "wasm4games: The
> Game-Pattern Foundry" section and its seven links (`tutorials/wasm4games-add-a-pattern.md`,
> `how-to/wasm4games-run-wasm4pm-admission.md`, `explanation/wasm4games-overview.md`,
> `explanation/ggen-only-user-surface.md`, `explanation/wasm4games-the-honest-kernel.md`,
> `explanation/wasm4games-server-architecture.md`, `reference/wasm4games-patterns.md`).
> None of these pages exist in the tree, and `crates/wasm4games` — the crate the section
> described — is not a workspace member or on disk. Restore the section only together
> with the crate and the pages (history retains this file).

## Tutorials: Learning by Doing
- [Getting Started](tutorials/getting-started.md) — Your first branchless kernel
- `tutorial-1.md` through `tutorial-10.md` — walk-through lessons on the real kernels (`bitset`, `int`, `mask`, ...)

## How-To Guides: Problem-Solving
- [Guarantee WCET](how-to/guarantee-wcet.md) — Worst-case execution time bounding strategies
- [Add a new algorithm](how-to/add_new_algorithm.md) — kernel + oracle + mutants + bench, end to end
- `guide-1.md` through `guide-10.md` — task guides

## Explanations: Deep-Dive Theory
- [Why Branchless?](explanation/why-branchless.md) — Philosophy and performance rationale
- [Anti-Patterns](explanation/anti-patterns.md) — Pitfalls and structural hazards to avoid (covers all 9 scanner-detected cheat patterns)
- [POWL 2.0 chess typestate](explanation/powl-v2-chess-typestate.md) — the receipt-bearing scheduler state machine
- `theory-1.md` through `theory-10.md` — theory notes

## Reference: API Specifications
- [Algorithm Families](reference/algorithm-families.md) — Categorized primitives and their complexity profiles
- [API Catalog](reference/api-catalog.md) — Complete function and type registry
- [PhD Gates](reference/phd_gates.md) — Hoare-logic annotations and formal verification anchors
- `ref-1.md` through `ref-10.md` — reference notes
