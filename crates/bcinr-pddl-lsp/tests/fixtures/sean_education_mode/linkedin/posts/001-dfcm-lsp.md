# Design for Chatman Machines: A New LSP

## Status: PUBLISHED

REVIEWED

---

I've been building a language server for PDDL8 lifecycle planning.

The key insight: every software project has a lifecycle that can be expressed as a planning problem.
bcinr-pddl-lsp turns your docs (PRD, ARD, ADR) into a PDDL8 domain, runs BFS to find the shortest
path to your goal, and gates publication behind BLAKE3 receipts.

No hand-flip path. No "mark as done." Just receipts.

This is Design for Chatman Machines (DfCM): bounded action spaces, admission gates, replay-able
audit trails.

The education-mode domain covers my weekly output cycle: interview, LinkedIn post, newsletter,
YouTube video, Rust lesson. Five parallel lanes. One education_week_published goal.

Try it: cargo add bcinr-pddl-lsp

#Rust #ProcessMining #DfCM #PDDL #LSP
