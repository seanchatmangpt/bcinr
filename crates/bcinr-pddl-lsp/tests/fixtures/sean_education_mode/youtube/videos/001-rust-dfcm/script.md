# Script: Rust + DfCM: Building a Receipt-Gated LSP

## Hook

"Every project has a lifecycle. Most engineers track it in their heads.
What if you could express it as a planning problem and let a planner find the path?"

## Section 1: What is DfCM?

DfCM — Design for Chatman Machines — is a methodology for building systems where
every action is bounded, admitted, and receipted.

The key principle: actuate(A) iff R ⊢ A. You don't do something unless it's admitted.

## Section 2: bcinr-pddl-lsp

Let me show you the crate. [SHOW CODE: src/education/mod.rs]

The scan() function walks your project directory and detects lifecycle stages.
Each stage maps to a PDDL8 predicate.

The emit_education_domain() function generates the full PDDL8 domain.
The planner finds the shortest path to education_week_published(sean).

## Section 3: Education Mode Demo

[DEMO: Run scan() on fixture directory]
[DEMO: Run plan() and show plan steps]
[DEMO: Run admit() and show receipt]

## Closing

Links in the description. Subscribe for weekly process mining content.
