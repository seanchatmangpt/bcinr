# CHEAT-003: Magic Constants

## Definition
According to the BCINR Anti-Cheat Manifesto (Rule 16), a **Magic Constant** (`CHEAT-003`) is defined as "any unexplained literal controlling production behavior." Classic examples of these arbitrary patterns include `0xDEADBEEF`, `0xDEAD_BEEF`, `0xCAFEBABE`, and `0xCAFE_BABE`.

## Why Unexplained Literals are Banned
BCINR is a civilizational-scale deterministic computational substrate (Rule 1). Its foundational law dictates that all logic is expressed as mathematically proven, bitwise polynomials.

Unexplained literals violate core constitutional requirements for several reasons:
1. **Lack of Mathematical Provenance (Rule 14):** Every constant—such as those used for smoothing or clamping—must be explicitly **named**, mathematically **derived**, formally **admitted**, and **included in the influence digest**.
2. **Breach of Contract:** Every primitive in BCINR requires an independent oracle, a Hoare contract, and proof obligations. Unexplained literals bypass this requirement, obscuring the structural logic and breaking the "contract with teeth" managed by `@hoare_oracle`.
3. **Incompatible with the Hot Path:** An unexplained literal cannot be formally verified against a structural mathematical law, which is mandatory for the authoritative, allocation-free, branchless execution model.

## Why Formatting Hex Does Not Satisfy the Scanner
Merely changing the visual formatting of a constant (e.g., swapping `0xDEADBEEF` to `0xDEAD_BEEF`) does not make a constant lawful. 

This is structurally enforced by the `bcinr-cheat-scanner` (Rule 17), which is designed to prevent **CHEAT-006 (Scanner evasion)**. The scanner is not a simple text parser; it:
- Parses the full Abstract Syntax Tree (AST), rather than relying on raw source strings.
- Automatically normalizes whitespace and explicitly **strips numeric separators**.
- Detects equivalent hex spellings natively before applying its rules engine.

Because the system evaluates the intrinsic value and derivation of the token rather than its textual representation, superficial formatting changes are automatically flattened and caught by the rule matrix. The only way to satisfy the system is through proper mathematical derivation and admission.
