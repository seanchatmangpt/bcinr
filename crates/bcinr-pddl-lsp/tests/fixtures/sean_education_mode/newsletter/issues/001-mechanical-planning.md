# Mechanical Planning: A Newsletter for Process Miners

## Status: PUBLISHED

REVIEWED

---

## Introduction

Welcome to the first issue of Mechanical Planning — the newsletter for engineers
who want to understand process mining through Rust and PDDL.

## What is Mechanical Planning?

Mechanical planning is the discipline of expressing your project's future as a
formal planning problem, then using a planner to find the shortest admitted path.

## The Education Mode Domain

This week I shipped the education-mode PDDL8 domain for bcinr-pddl-lsp.
Five lanes. One goal. All receipt-gated.

## Rust Corner

```rust
let workspace = scan(root, "sean");
let domain = emit_education_domain();
let problem = emit_education_problem(&workspace);
```

## What's Next

Next issue: OCEL traces and how process mining turns your receipts into discovery.

## Closing

If you found this useful, share it with a Rust engineer who wants to understand
process mining.

— Sean
