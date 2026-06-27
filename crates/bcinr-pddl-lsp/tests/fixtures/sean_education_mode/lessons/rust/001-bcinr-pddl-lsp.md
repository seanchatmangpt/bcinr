# Lesson: bcinr-pddl-lsp Internals

## Status: PUBLISHED

## Learning Goals

1. Understand PDDL8 domain/problem structure
2. Use scan() to detect lifecycle stages
3. Use plan() and admit() for receipt-gated planning

## Background

bcinr-pddl-lsp is a language server for PDDL8 lifecycle planning. It turns your
project docs (PRD, ARD, ADR) into a planning problem and gates publication behind
BLAKE3 receipts.

## Example

```rust
use bcinr_pddl_lsp::education::{scan, emit_education_domain, emit_education_problem};
use std::path::Path;

fn main() {
    let root = Path::new(".");
    let workspace = scan(root, "sean");
    println!("True stages: {}", workspace.true_stages.len());
    println!("Missing: {}", workspace.missing.len());
}
```

## Exercises

1. Add a new lane to the education-mode domain
2. Write a scan() extension for a new content type
3. Run the planner on an incomplete fixture and observe the plan

## References

- bcinr-pddl-lsp crate documentation
- PDDL8 specification
- DfCM methodology guide
