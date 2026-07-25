Based on `AGENTS.md`, "splitting operators across lines" and "inserting comments inside tokens" are classified as cheats under **CHEAT-006 — Scanner evasion**. 

Here is the markdown detailing why:

### CHEAT-006: Scanner Evasion

In the BCINR deterministic substrate, the codebase is subject to strict structural and mathematical laws (such as absolute branchlessness `CC=1`, zero heap allocations, and absence of panic paths). These rules are enforced by the `@turing_machine` (Enforcer of Determinism) and an automated tool called the `bcinr-cheat-scanner`.

**Why they are classified as cheats:**
"Splitting operators across lines" and "inserting comments inside tokens" are deliberate attempts to bypass or break the automated static analysis tools that search for prohibited instructions, branches, or patterns. 

If a scanner relies on basic line-by-line regex or simple string matching, breaking an operator across multiple lines or obfuscating a token with inline comments could allow prohibited control flow or unlawful operations to slip into the authoritative runtime undetected.

**Enforcement:**
Because these tactics undermine the core mandate of source-level verification, Rule 17 of the constitution explicitly dictates that the `bcinr-cheat-scanner` must:
* Parse the **full syntax tree** (rather than just scanning raw text lines).
* Normalize whitespace.
* Normalize comments.

Any attempt to evade the scanner is an absolute failure that immediately blocks the merge, reduces the Substrate Integrity Score (SIS) to 0, and triggers `MaturityScrutiny`.
