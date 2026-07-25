# Cheat Scanner AST Requirements

Rule 17 of `AGENTS.md` explicitly details the rigorous requirements for the `bcinr-cheat-scanner`. The overarching design principle is that the scanner must verify code structurally and exhaustively, ensuring that no `CHEAT` violations or hidden branches bypass the strict deterministic constitution.

## Requirements of `bcinr-cheat-scanner`

According to Rule 17, the scanner is required to perform the following operations:
- **Parse the full syntax tree** rather than performing naive line-by-line or string-based inspections.
- **Scan all functions**, both public and private.
- **Inspect macro definitions and their expanded output**.
- **Scan generated Rust** code that will be executed by the runtime.
- **Normalize whitespace** and **normalize comments** where required to eliminate superficial variations.
- **Strip numeric separators** (e.g., matching `0xDEAD_BEEF` to `0xDEADBEEF`).
- **Detect equivalent hex spellings**.
- **Inspect test references** and **benchmark targets** to prevent Benchmark Theater (CHEAT-008).
- **Report findings precisely**, including the exact file, span, and the specific rule identifier.

Every violation must be reported in the exact format `CHEAT[rule-id]` (e.g., `CHEAT[CHEAT-006]: prohibited operator hidden in macro expansion`), and any single finding categorically blocks merging.

## Why a Full Syntax Tree Parser is Mandated

Using simple regular expressions to catch hidden branches and `CHEAT` violations is vastly insufficient and fails to uphold the substrate constitution. A full AST parser is mandated for several critical reasons, reinforced by Rule 8 (Absolute `CC=1` law) and Rule 16 (Anti-cheat manifesto):

1. **Scanner Evasion Resilience (CHEAT-006):** Regex operates on raw strings and is easily defeated by trivial formatting tricks. Developers might split operators across multiple lines, insert comments inside tokens, or use string construction that evaluates to prohibited source. An AST normalizes these syntactic distractions, making the true semantic structure visible regardless of text-level obfuscation.
2. **Evaluating Macro Indirection:** Rule 8 states that macro-generated branches count against the `CC=1` limit. Regular expressions cannot reliably evaluate or expand macros. A full AST parser is necessary to inspect the post-expansion code for hidden control flow.
3. **Catching Trait and Transitive Obfuscation:** Code branches might be hidden inside generic monomorphizations, private wrappers, or trait implementations. Regex cannot reliably traverse these boundaries, whereas AST parsing allows for the structural context needed to audit operations accurately.
4. **Semantic Equivalence Checking:** The scanner must recognize semantically equivalent elements, like detecting identical magic constants regardless of casing or underscores (`0xCAFE_BABE` vs `0xcafebabe`). An AST and proper lexing treat these as identical numeric values rather than disparate text strings.
