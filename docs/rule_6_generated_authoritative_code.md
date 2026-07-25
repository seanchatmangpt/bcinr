Here are the details on the "Generated authoritative code" classification under Rule 6 from `AGENTS.md`:

### What it is
"Generated authoritative code" is defined as **generated source executed by the runtime.** 

Under **Rule 6**, this classification makes it clear that generated code is **not exempt** from the strict rules governing authoritative code. It must pass all authoritative gates after generation.

### What gates it must pass after generation
According to **Rule 21 (Generated-code law)**, generated authoritative code must be fully reproducible and is subject to the following gates and requirements:

1. **Reproducibility Process**: It must survive a strict regeneration cycle to prove it hasn't been hand-edited:
   `clean generation → digest output → regenerate → verify byte-identical output`
2. **Fixture-Free**: It must contain no fixture-specific identifiers.
3. **Branchless Execution**: It must contain no hidden branches.
4. **Cheat Scanner**: It must pass the `bcinr-cheat-scanner`.
5. **Radon Law (`CC=1`)**: It must pass the absolute cyclomatic complexity requirement of 1.
6. **Object-Code Audit**: It must pass disassembly inspection to ensure no input-dependent conditional branches exist in the final machine code.
7. **Binding**: It must bind to the source graph and certificate digests.

**Additional strict constraints:**
- Hand-editing generated output is strictly prohibited.
- Any generated files with unexplained drift immediately invalidate the project's standing.
