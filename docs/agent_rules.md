# BCINR Agent Interaction Rules and Guidelines

## 1. Project Mandate: The Deterministic Substrate
`bcinr` is a systems library designed to be a deterministic substrate for bounded, branchless, allocation-free execution. 

### Core Architectural Laws (from `GEMINI.md` and `AGENTS.md`)
- **The Radon Law ($CC=1$)**: No public primitive shall contain a single `if`, `match`, or data-dependent `loop`. Logic must be expressed as bitwise polynomials. The complete authoritative call graph must be branchless.
- **The Zero-Allocation Boundary**: Hot-path execution must be `#![no_std]` and perform zero heap allocations. Memory is managed via `BumpArena` and `LockFreeSlab`.
- **The Contract with Teeth**: Every primitive is an executable specification requiring mathematical contracts, structurally lawful implementations, independent oracles, hostile mutants, and verification at both source and object-code levels.
- **The Substrate Integrity Score (SIS)**: A file is only "PhD-Verified" if it scores 100/100 on the maturity matrix (Proof + Oracle + Hostile Tests). Any absolute failure forces SIS=0.

## 2. Agent Roster and Authority (`AGENTS.md`)
The project utilizes specialized roles with exclusive authorities:

- **`@hoare_oracle` (Oracle of Invariants)**: Owns mathematical contracts, proofs, preconditions, postconditions, and independent reference semantics.
- **`@turing_machine` (Enforcer of Determinism)**: Owns structural audits, object-code audits, panic/allocation audits, and cyclomatic-complexity enforcement.
- **`@armstrong_fault` (Master of Failure Law)**: Owns adversarial testing, counterfactual mutant design, and refusal-path verification.
- **`@von_neumann_bypass` (Architect of Arithmetic Logic)**: Owns authoritative branchless implementations and fixed-point mechanics.

*Note: Agent work must use exclusive write ownership for their respective domains. No implementation agent may self-certify.*

## 3. Mandatory Development Protocols
- **Typed Refusals**: All rejected authoritative operations must produce a bounded typed refusal code. Human-readable text belongs outside the hot path.
- **Hostile Mutation Protocol**: Every implementation file requires at least three independent, syntactically plausible mutants, verifying typed refusals or oracle mismatches.
- **Anti-Cheat Manifesto**: Strict prohibition against self-canceling operations, circular oracles, magic constants, dead-path compliance, and scanner evasion.
- **Generated Code Law**: Generated authoritative code must be reproducible, pass all cheat and structural scanners, and must not be hand-edited.
- **Object-Code Audit**: Every supported release target requires an exact production-profile disassembly audit. Source-level $CC=1$ is necessary but insufficient.
