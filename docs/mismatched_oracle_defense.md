# Mismatched Oracle Defense: Avoiding CHEAT-002

In the BCINR Deterministic Substrate, the integrity of the mathematical contracts is paramount. To guarantee mathematical correctness, every branchless production primitive must be verified against an independent source of truth.

## Rule 15: The Independent Oracle Law

Rule 15 dictates that an oracle must be **structurally and logically distinct** from the authoritative production code. Placing a test reference in a separate file (e.g., `tests/reference.rs`) does not inherently make it independent. The oracle must serve as an axiomatic specification that mathematically defines the desired behavior without being constrained by the hot-path execution laws (such as branchless $CC=1$ or zero allocation). 

Furthermore, the independent oracle must be designed and reviewed exclusively by the `@hoare_oracle` role, enforcing strict separation of concerns from the implementation owner (`@von_neumann_bypass`). This ensures compliance with Rule 27 (No Self-Certification).

## CHEAT-002: The Circular Oracle Anti-Pattern

Under Rule 16 (Anti-Cheat Manifesto), **CHEAT-002** explicitly prohibits "Circular Oracles." A circular oracle occurs when the reference implementation is copied from or structurally mirrors the production implementation.

Specifically, the following practices are banned when constructing an oracle:
- **Line-by-line translation** of production code.
- **Identical control structures**, even if swapped to use higher-precision types (e.g., using `f64` but keeping the same structural logic).
- **Reuse of production components**, such as lookup tables, normalization steps, or fixed-point helpers.
- **Directly importing and wrapping** the authoritative production function.

### Why is Shared Logic or Translation Prohibited?

Sharing logic or directly translating production code into the test oracle is prohibited because it **defeats verification**. 

If the oracle shares the same structural steps, algorithmic approximations, or fixed-point mechanics as the production code, any logical flaws, mathematical oversights, or masking errors in the implementation will simply be replicated in the reference. This creates a scenario where the test suite falsely passes because both sides made the exact same error, merely proving that "the code does what the code does."

To genuinely verify that a branchless, deterministic implementation correctly models the true mathematical intent, the oracle must arrive at the result through completely different, declarative, or rigorous mathematical pathways. Any detection of a circular oracle is an absolute failure that instantly drops the Substrate Integrity Score (SIS) to 0.

## Defining a Valid, Independent Structural Oracle

To avoid CHEAT-002, a valid structural oracle must enforce the pre-conditions, post-conditions, and invariants defined by `@hoare_oracle`. Permitted forms of independent oracles include:

- **Direct mathematical formula:** Implementing the pure, exact mathematical equations without being bound by performance or branchless constraints.
- **Hoare specification:** Validating precise pre- and post-conditions mathematically.
- **Abstract state machine:** Defining valid state transitions in an abstract, formal model.
- **Symbolic proof:** Utilizing formal verification and symbolic methods to prove mathematical equivalence.
- **Arbitrary-precision implementation:** Using unconstrained numeric types (e.g., big integers or big decimals) to verify exact mathematical boundaries and avoid fixed-width precision or fixed-point artifacts.
- **SAT/SMT bit-vector model:** Using formal solvers (like Z3) to rigorously exhaust domain bounds or formally prove equivalence.
- **Exhaustive reduced-domain enumerator:** Testing all possible inputs within a mathematically valid partition.

By strictly utilizing these independent forms, BCINR guarantees an uncompromised structural firewall between pure algorithmic truth and deterministic execution mechanics.
