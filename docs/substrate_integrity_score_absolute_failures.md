# Substrate Integrity Score (SIS) and Absolute Failures

## The SIS Formula
According to Rule 24 of the BCINR Deterministic Substrate Constitution (`AGENTS.md`), the Substrate Integrity Score (SIS) is defined mathematically as:

$$SIS = 100 - \sum_i w_i V_i$$

Where:
* **$V_i$** represents verified violations.
* **$w_i > 0$** represents the weight assigned to each respective violation.

## Absolute Failures (SIS = 0)
Regardless of the weighted score calculated above, certain foundational violations are considered "absolute failures." Any single occurrence of these forces **$SIS = 0$** instantly and triggers the `MaturityScrutiny` protocol. 

The specific absolute failures listed in the constitution are:
* Hidden authoritative branch
* Allocation in the hot path
* Unwitnessed mutation
* Surviving mutant
* Circular oracle
* Scanner evasion
* Stale certificate acceptance
* State mutation after refusal
* Gate-jurisdiction omission
* Fabricated verification evidence

## Why "No weighted average may conceal a constitutional violation"?
The constitution dictates this rule because BCINR acts as a deterministic computational substrate that relies on absolute, mathematically verifiable guarantees. Its core invariants—such as strict branchlessness ($CC=1$), total lack of heap allocation in the hot path, and immutable state upon admission refusal—are binary: they are either mathematically proven or completely broken. 

Allowing a weighted average to mask a violation would mean that a component could fail a critical structural law (e.g., concealing a data-dependent branch) but still pass repository gates simply because it scored well in other areas (e.g., well-documented code or a lack of scanner evasion). In an adversarial, high-stakes system demanding mathematically verifiable determinism, foundational flaws cannot be offset by minor successes. A single foundational breach completely invalidates the deterministic runtime and breaks the project's core mathematical contract.
