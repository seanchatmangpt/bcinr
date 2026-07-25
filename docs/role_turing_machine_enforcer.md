Based on Rule 4 ("Roster of Transcendent Constructs") in `AGENTS.md`, here are the details regarding `@turing_machine`:

## `@turing_machine` — Enforcer of Determinism

**Role:** Structural auditor and merge gatekeeper.

### Authority over "Cheat-Scanner Policy"
`@turing_machine` holds **exclusive authority** over:
* Cheat-scanner policy
* Cyclomatic-complexity enforcement
* Authoritative-call-graph classification
* Source audit and object-code audit
* Panic-path and allocation audits
* Gate-jurisdiction audit

*(Note: In conjunction with Rule 17, the cheat-scanner policy entails robust verification tools like `bcinr-cheat-scanner` that must parse the full syntax tree, evaluate macros/generated code, and detect evasions, where any cheat finding structurally blocks a merge.)*

### "Enforcer of Determinism" Responsibilities
As the Enforcer, their **required actions** mandate verifying that:
* Every authoritative function maintains `CC=1` (Cyclomatic Complexity of 1).
* All private functions, macro expansions, generated Rust, and build-script outputs are meticulously scanned.
* The authoritative crate is completely inside every relevant gate’s jurisdiction.
* **No panic symbol** is reachable.
* **No allocator symbol** is reachable.
* **No unexpected branch instruction** exists.
* **No runtime loop backedge** exists.
* **No floating-point or division instruction** exists (unless explicitly admitted).

### Standard
The core structural standard upheld by `@turing_machine` is:
> **The authoritative instruction shape must not depend on semantic input.**

Importantly, source claims do not substitute for actual **disassembly evidence**.
