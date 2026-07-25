I have reviewed `AGENTS.md` regarding the `PARTIAL_ALIVE` standing found under Rule 28 (Standing vocabulary).

### What it means
The `PARTIAL_ALIVE` standing means that **"Some required gates remain incomplete."** 

### When it is applied
A piece of code receives this standing when it has not yet passed the full, mandatory suite of repository gates required for authoritative implementation verification. 

For context, under this deterministic substrate constitution, fully verified code is subject to a strict matrix of automated verification gates (as outlined in Rule 23, "Required repository gates"), such as:
- `scan-cheats`
- `contract-gate`
- `ci`
- `test-mutants`
- `audit-object-code`
- `verify-generated`

If an implementation is still undergoing these checks or has not successfully run through all required validations (missing disassembly evidence, missing mutant coverage, missing mathematical contract, etc.), its standing cannot be elevated to fully `ALIVE` or `BRANCHLESS_ALIVE`, and it must be labeled `PARTIAL_ALIVE`.
