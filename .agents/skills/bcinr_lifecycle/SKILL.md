---
name: bcinr-lifecycle
description: "Instructions for synchronizing with the bcinr-pddl-lsp lifecycle (CANDIDATE vs ADMITTED) and reading virtual documents."
---

# `bcinr-lifecycle` Skill

When working in the `bcinr` workspace, you are governed by the `bcinr-pddl-lsp` language server which maintains the project lifecycle. 

## Lifecycle Status
The workspace is always in one of two states:
1. **CANDIDATE**: The plan is proposed but not yet executed/admitted.
2. **ADMITTED**: The plan has been executed, receipts generated, and OCEL traces emitted.

## Finding the Current State
Whenever you need to know the current status, next steps, or what evidence is missing:
- Do not guess or try to manually parse the PRD/ARD files.
- Instead, read the virtual documents maintained by the LSP. You can read them using the provided MCP bridge or the helper scripts in `scripts/`.
- Important URIs:
  - `bcinr-pddl://status`
  - `bcinr-pddl://next_step`
  - `bcinr-pddl://evidence`

## Using the Broker
The LSP enforces a strict Build Broker. Heavy commands (`cargo build`, `test`, etc.) will fail with `DIRECT_HEAVY_COMMAND_BLOCKED` unless you request a slot first.
See `AGENTS.md` for the strict protocol.

## Executing the Tape
When you believe a candidate plan is ready to be admitted, you must invoke `bcinr_execute_tape`. This will advance the state from CANDIDATE to ADMITTED and generate the cryptographic receipts.
