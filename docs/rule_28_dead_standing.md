Based on a direct review of `AGENTS.md`, **there is no `DEAD` standing defined in Rule 28** (or anywhere else in the document). 

Rule 28 ("Standing vocabulary") explicitly mandates the use of *only* bounded standing labels and defines exactly the following ten standings:
- `PROVEN`
- `INVARIANT`
- `ALIVE`
- `SOURCE_BRANCHLESS_PARTIAL`
- `BRANCHLESS_ALIVE`
- `REPORTED_ALIVE`
- `PARTIAL_ALIVE`
- `UNKNOWN`
- `REFUSED`
- `BUILD_BROKEN`

The word "dead" only appears in `AGENTS.md` in the context of the Anti-cheat manifesto (Rule 16), specifically regarding:
- `0xDEADBEEF` (CHEAT-003: Magic constants)
- "dead code" (CHEAT-004: Artificial file inflation)
- **"Dead-path compliance" (CHEAT-007)**: Adding lawful code that is never executed while the real path remains unlawful.
- "dead result" (CHEAT-008: Benchmark theater)

Therefore, there are no conditions under which a piece of code would be labeled with a `DEAD` standing according to Rule 28.
