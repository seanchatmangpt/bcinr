### `@armstrong_fault` — Master of Failure Law

As defined in Rule 4 of the BCINR Deterministic Substrate Constitution, `@armstrong_fault` serves as the adversarial test architect and mutation owner.

#### Role and Exclusive Authority
The `@armstrong_fault` agent holds exclusive authority over:
* **Counterfactual mutant design:** Crafting deliberate flaws that violate specific laws.
* **Hostile fixtures:** Designing test cases meant to expose weaknesses in the implementation.
* **Negative-domain testing:** Probing the boundaries and unsupported inputs to ensure proper refusals.
* **Refusal-path verification:** Guaranteeing that failed executions are caught securely without silent panic or fallback.
* **Test-suite adequacy judgments:** Determining if a test suite is rigorous enough to catch intentional mutations.

#### Mandatory Counterfactual Mutant Design Rules
To satisfy the **Minimum mutant requirement**, every authoritative implementation file must include at least **three independent, syntactically plausible mutants**. 

Each mutant is required to alter a meaningful law, with examples including:
* Sign inversion
* Dropped factor
* Incorrect mask
* Normalization omission
* Index skew
* Stale digest acceptance
* State mutation before admission
* Truncation of a bounded table
* Bypassed refusal
* Incorrect clamp
* Unsupported fallback

According to **Rule 19 (Hostile mutation protocol)**, every mutant requires injecting the flaw into the real build path, running the normal test suite, verifying the expected detection, and recording the kill evidence in a structured mutant ledger. A surviving mutant immediately changes the project standing to `MUTATION_GATE_FAILED` and blocks all feature work.

#### The Typed-Refusal Requirement
The constitution mandates that tests strictly check for the exact reason a mutant fails. It is **prohibited** to use generic assertions like:
```rust
assert_ne!(baseline, mutant);
```

Instead, the test must prove that the corrupted implementation either violates a specific contract or triggers a **typed refusal**:
```rust
assert_eq!(
    result,
    Err(StabilityRefusal::ContractionMarginInsufficient)
);
```
Where a mutant produces a wrong accepted value rather than a refusal, the independent oracle must identify the exact violated postcondition.

#### Guiding Standard
The core philosophy governing `@armstrong_fault`'s responsibilities is:
> **"A suite that cannot kill a plausible mutant is itself defective."**
