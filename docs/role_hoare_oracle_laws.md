Based on `AGENTS.md` (specifically **Rule 4: Roster of Transcendent Constructs**), the `@hoare_oracle` acts as the **Oracle of Invariants**. Their core role is the **"Axiomatic proof lead and specification owner."** 

Under this mandate, they possess exclusive authority to define the strict mathematical bounds of the system, including **algebraic laws** and **admissible domains**.

### Authority over Algebraic Laws
The `@hoare_oracle` establishes the mathematical constraints and algebraic properties that all deterministic operations must follow. 
* **The Hoare Contract:** They must output a rigorous mathematical contract ( $\{P(x)\} \quad f(x) \quad \{Q(x,f(x))\}$ ) for every authoritative primitive.
* **Included Laws:** This contract must explicitly formalize algebraic behaviors such as the **conservation law** and, where applicable, the **monotonicity law**.
* **Strict Definition:** A core standard they enforce is that *"If a property cannot be stated precisely, it is not yet law."*
* **Independent Oracle Review:** Under Rule 15, any independent test oracle constructed must be reviewed by `@hoare_oracle`, not the implementation owner, to ensure algebraic independence.

### Authority over Admissible Domains
The `@hoare_oracle` rigidly controls which inputs are mathematically valid for the deterministic substrate and how the system proves correctness across that domain space.
* **Domain Definition:** The oracle specifies the exact **"valid input domain"**, its corresponding **"output range,"** and the deterministic **"invalid-input refusal"** conditions for edge cases (e.g., overflow behavior).
* **The Full-Domain Requirement:** The oracle strictly enforces that verification covers the entire mathematical domain (e.g., all $2^{64}$ values). Since "random testing alone never establishes universal standing," they must furnish rigorous evidence to achieve full-domain standing using one of the following:
  1. A formal mathematical proof.
  2. An exhaustive proof over a finite partition whose cases completely cover the domain.
  3. A bit-vector solver certificate (e.g., SAT/SMT).
  4. An equivalent bounded theorem artifact.

Additionally, under the repository's mandatory decomposition and write-isolation protocols (Rules 5 and 26), `@hoare_oracle` holds **exclusive write ownership** over all contracts, proof obligations, and independent reference semantics. No implementation agent is permitted to self-certify equivalence or dictate these mathematical bounds.
