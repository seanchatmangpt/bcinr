# The Lifecycle of a Primitive in BCINR

In accordance with the BCINR Deterministic Substrate Constitution, every nontrivial implementation task is subject to a **Mandatory Decomposition Protocol** (Rule 5). A feature is not complete until it satisfies rigorous mathematical, structural, and hostile verification requirements. 

Crucially, **independence is mandatory**. No agent may self-certify, and no agent may cross jurisdictional bounds to weaken higher-order rules for implementation convenience. The chronological workflow of a primitive follows four strictly separated phases governed by dedicated transcendent constructs.

---

## Phase 1: Mathematical Law & Specification (`@hoare_oracle`)
**Role:** Axiomatic proof lead and specification owner.

Before any production implementation is written, `@hoare_oracle` defines the unyielding mathematical reality of the primitive.
- **Contract Definition:** Constructs a Hoare contract $\{P(x)\} \quad f(x) \quad \{Q(x,f(x))\}$. This defines the valid input domain, output range, conservation and monotonicity laws, overflow behavior, numeric error envelope, and determinism.
- **Independent Oracle Creation:** Builds an independent reference semantics (oracle) that is structurally and logically distinct from any eventual production implementation (Rule 15).
- **Full-Domain Proof:** Establishes full-domain standing over the valid domain (e.g., via formal proof, bit-vector solver certificate, or exhaustive proof over a finite partition).
- **Refusal Specification:** Defines the exact bounded typed refusals for invalid input domains, ensuring state-mutation boundaries are strict.

## Phase 2: Authoritative Implementation (`@von_neumann_bypass`)
**Role:** Architect of Arithmetic Logic and authoritative implementation owner.

Once the mathematical law is written, `@von_neumann_bypass` builds the branchless bounded code that respects all transcendent rules.
- **Branchless Architecture:** Transforms sequential semantic decisions into masks, arithmetic selection, fixed lookup tables, and generated straight-line code.
- **Execution Mandate:** Implements the primitive utilizing bit-parallel mechanics over byte-sequential control flow (e.g., SWAR, SIMD shuffles, PDEP/PEXT). 
- **State Selection:** Implements state transitions entirely through mask-based selection formulas (`(m & a) | (~m & b)`), completely devoid of branching constructs.
- **Compliance:** Refrains from self-certifying mathematical correctness, leaving equivalence verification to the independent oracle and subsequent auditors.

## Phase 3: Hostile Verification (`@armstrong_fault`)
**Role:** Master of Failure Law and adversarial test architect.

`@armstrong_fault` operates independently of the implementation team, acting as an adversary to test the rigidity of the substrate.
- **Hostile Mutation Protocol (Rule 19):** Identifies at least three load-bearing laws per implementation file and creates syntactically plausible mutants (e.g., sign inversion, incorrect mask, dropped factor, bypassed refusal).
- **Typed-Refusal Verification:** Asserts that the mutated implementations either violate a specific postcondition caught by the oracle, or trigger a precise, expected typed refusal (e.g., `Err(StabilityRefusal::ContractionMarginInsufficient)`).
- **Mutant Ledger Registration:** Injects mutants through the real build path and records the exact kill evidence and receipt digest to establish verified test-suite adequacy. 

## Phase 4: Structural Enforcement & Audit (`@turing_machine`)
**Role:** Enforcer of Determinism and structural auditor / merge gatekeeper.

As the final constitutional gate, `@turing_machine` proves that the primitive obeys all structural and object-code laws before admitting it to the substrate.
- **Absolute `CC=1` Enforcement:** Parses the complete transitive authoritative call graph (including private helpers, traits, and macros) to ensure cyclomatic complexity is strictly 1. 
- **Object-Code Audit (Rule 20):** Performs exact production-profile disassembly audits for every supported target. Verifies zero conditional jumps, zero loop backedges, zero panic paths, zero floating-point operations, and zero allocator symbols.
- **Anti-Cheat Verification:** Runs the `bcinr-cheat-scanner` across all source and generated files to detect scanner evasion, magic constants, dead-path compliance, or structural theater.
- **Final Gatekeeping:** Asserts that the entire Mandatory Decomposition Protocol was followed without independence violations. Only upon generating a verified structural artifact and a pristine Substrate Integrity Score (SIS = 100) is the primitive admitted.
