Here is the information you requested regarding the "Object-code audit" under Rule 20 in `AGENTS.md`.

### Symbols and Instructions to Inspect
The disassembly audit must inspect the following:
* all authoritative root symbols;
* all transitive helper symbols;
* panic and bounds-check symbols;
* allocator symbols;
* conditional jumps;
* loop backedges;
* indirect calls;
* floating-point instructions;
* division instructions;
* unexpected runtime library calls.

### Permitted Evidence Format
The audit result must list each symbol individually using the exact table format below:

| Symbol            | CC | Conditional jumps | Loop backedges | Panic path | Allocator | Standing |
| ----------------- | -: | ----------------: | -------------: | ---------: | --------: | -------- |
| `cmca_allocate`   |  1 |                 0 |              0 |         No |        No | ALIVE    |
| `verify_envelope` |  1 |                 0 |              0 |         No |        No | ALIVE    |
