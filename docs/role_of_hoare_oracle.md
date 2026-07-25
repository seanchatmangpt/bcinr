# Role of `@hoare_oracle` and Formal Verification in `bcinr`

## The Role of `@hoare_oracle`
In the deterministic substrate of `bcinr`, `@hoare_oracle` serves as the **Axiomatic proof lead and specification owner**. 

This agent has **exclusive authority** over:
* Preconditions, postconditions, and invariants
* Algebraic laws and admissible domains
* Refusal conditions and proof obligations
* Independent reference semantics

For every primitive, `@hoare_oracle` is required to produce a formal **Hoare contract** ($\{P(x)\} \quad f(x) \quad \{Q(x,f(x))\}$) that explicitly defines the valid input domain, output range, conservation and monotonicity laws, overflow behavior, invalid-input refusal, determinism, state-mutation boundary, and numeric error envelope.

## What is an Independent Oracle?
According to the **Independent Oracle Law** (Section 15), an oracle is a structurally and logically distinct reference used to verify production implementations. 

An oracle is **NOT** independent if it relies on:
* Line-by-line translation of production code
* Reuse of production normalization, lookup tables, or fixed-point helpers
* Identical control structures merely swapped to use `f64` floats
* Importing and wrapping the authoritative function (a "circular oracle")

Instead, **permitted independent forms** include:
* Direct mathematical formulas
* Hoare specifications or abstract state machines
* Symbolic proofs or SAT/SMT bit-vector models
* Arbitrary-precision implementations
* Exhaustive reduced-domain enumerators

## Why `@von_neumann_bypass` Cannot Write Its Own Oracle
The `bcinr` constitution enforces a **mandatory decomposition protocol** requiring absolute independence between mathematical specification and implementation. `@von_neumann_bypass` is the **Architect of Arithmetic Logic** (the implementation owner), responsible for creating the branchless bounded code.

They are strictly forbidden from writing their own oracle because:
1. **No Self-Certification**: An implementation agent may not author its own final oracle to self-certify equivalence. This ensures true structural and mathematical rigor.
2. **Preventing Circular Verification**: Allowing the implementation owner to write the oracle risks creating a "Circular Oracle" (CHEAT-002), where the reference is just a copy of the production implementation rather than an independent mathematical truth.
3. **Separation of Concerns**: The oracle must be reviewed and owned by `@hoare_oracle`, ensuring that the logic is audited from an axiomatic perspective, independent of the implementation's bit-parallel mechanics.

## The Requirement for Full-Domain Standing
In `bcinr`, "covering the entire $2^{64}$ domain" cannot rely on random testing, which is explicitly stated to **never** establish universal standing. It also does not require brute-force enumeration. 

Instead, achieving **Full-Domain Standing** mandates one of the following formal artifacts:
1. A **formal proof**
2. An **exhaustive proof over a finite partition** whose cases strictly cover the entire domain
3. A **SAT/SMT bit-vector solver certificate**
4. An **equivalent bounded theorem artifact**

By satisfying these strict requirements, `bcinr` guarantees mathematically verified, universally valid branchless execution across all admissible inputs.
