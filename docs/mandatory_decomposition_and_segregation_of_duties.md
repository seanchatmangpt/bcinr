# Mandatory Decomposition and Segregation of Duties

The BCINR Deterministic Substrate Constitution enforces absolute segregation of duties to ensure that code is structurally sound, mathematically proven, and free from confirmation bias. This is primarily governed by **Rule 5 (Mandatory decomposition protocol)** and **Rule 27 (No self-certification)**.

## The Four Workstreams and Independent Agents

Every nontrivial task in the BCINR framework must be divided into four strictly independent workstreams, each owned by a specialized agent role:

1. **`@hoare_oracle` (Mathematical Law)**
   - **Role:** Axiomatic proof lead and specification owner.
   - **Responsibility:** Defines preconditions, postconditions, invariants, and independent reference semantics. They author the Hoare contracts and proof obligations.

2. **`@turing_machine` (Structural Enforcement)**
   - **Role:** Structural auditor and merge gatekeeper.
   - **Responsibility:** Enforces the absolute runtime laws (e.g., `CC=1`, zero allocation). They own the source and object-code audit plans, gate-jurisdiction audits, and cheat-scanner policies.

3. **`@armstrong_fault` (Hostile Verification)**
   - **Role:** Adversarial test architect and mutation owner.
   - **Responsibility:** Designs counterfactual mutants, negative-domain tests, and verifies refusal paths. They dictate refusal expectations and evaluate test-suite adequacy.

4. **`@von_neumann_bypass` (Implementation)**
   - **Role:** Authoritative implementation owner.
   - **Responsibility:** Writes the branchless bounded code, translating sequential semantic decisions into bit-parallel masks, arithmetic selection, and fixed-width state transitions.

## Why Self-Certification and Silent Repairs are Fundamentally Illegal

### 1. No Self-Certification (Rule 27 & Rule 15)
It is fundamentally illegal for an implementation agent (`@von_neumann_bypass`) to write their own tests or act as the final approver for mathematical correctness, branchlessness, or mutant adequacy.

- **Confirmation Bias & Circular Oracles:** If the implementer writes the tests or references, they are highly likely to write an oracle that mirrors their own production logic (a "circular oracle," which is an explicit CHEAT-002 violation). To be valid, tests and oracles must be structurally and logically distinct from the implementation.
- **Mechanical Artifact Requirement:** An implementer cannot self-certify equivalence because each approval must come from a different role and be backed by an independent mechanical artifact (a proof, audit, or mutant survival record). An implementer validating their own work degrades the strict verification into mere "agent agreement," which the constitution dictates is *not evidence*.

### 2. No Silent Repairs by Auditors (Rule 5)
It is explicitly prohibited for a structural auditor (`@turing_machine`) to silently repair implementation code and then approve their own repair.

- **Destruction of Independence:** If an auditor fixes the code, they effectively become the implementation agent. They can no longer independently audit the logic since they are now its author.
- **Bypassing the Friction Matrix:** Silent repairs circumvent the required constitutional process. Any code change must go back through the full verification matrix—requiring `@hoare_oracle` to verify the mathematical contracts against the change, `@armstrong_fault` to design new hostile mutants, and `@turing_machine` to audit the final structure. A silent repair nullifies the adversarial guarantees of the substrate.

In the BCINR framework, independence is mandatory. As Rule 27 states, "Five agents repeating the same claim is still one unsupported claim." Every feature must survive the hostile, segregated friction between mathematically pure specification, adversarial mutation, structural audit, and strict implementation.
