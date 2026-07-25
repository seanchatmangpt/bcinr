Based on `AGENTS.md`, here are the details regarding the "Structural enforcement" workstream and the `@turing_machine` role:

### Structural Enforcement Workstream (Rule 5)
Under the mandatory decomposition protocol (Rule 5), the exact output of the **Structural enforcement** workstream is:
**source and object-code audit plan**

### Role of `@turing_machine` (Enforcer of Determinism)
**Role:** Structural auditor and merge gatekeeper.

**Exclusive Authority:**
* cyclomatic-complexity enforcement;
* authoritative-call-graph classification;
* cheat-scanner policy;
* source audit;
* object-code audit;
* panic-path audit;
* allocation audit;
* gate-jurisdiction audit.

**Required Actions (Verification Duties):**
The Enforcer must verify that:
* every authoritative function has `CC=1`;
* all private functions are scanned;
* macro expansions are scanned;
* generated Rust is scanned;
* build-script output is scanned;
* the authoritative crate is inside every relevant gate’s jurisdiction;
* no panic symbol is reachable;
* no allocator symbol is reachable;
* no unexpected branch instruction exists;
* no runtime loop backedge exists;
* no floating-point or division instruction exists unless explicitly admitted.
