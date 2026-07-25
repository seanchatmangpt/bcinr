# The `@hoare_oracle` Full-Domain Requirement

According to **Rule 4: Roster of Transcendent Constructs** in the `AGENTS.md` constitution, the `@hoare_oracle` role is responsible for the mathematical contracts and proofs of primitives. A key part of this is the **Full-domain requirement**.

### Why Brute-Force Enumeration is Insufficient

The constitution explicitly states that "“Covers the entire (2^{64}) domain” does not mean brute-force enumeration of (2^{64}) values." Brute-force enumeration and random testing are insufficient because they lack a structural, mathematical guarantee. As the constitution notes, "Random testing alone never establishes universal standing." The substrate requires rigorous mathematical proof artifacts rather than relying on raw computation or enumeration to guarantee that an invariant holds universally across the domain.

### Acceptable Forms of Proving Full-Domain Standing

To satisfy the full-domain standing requirement, one of the following four rigorous proof artifacts must be provided:

1. A **formal proof**.
2. An **exhaustive proof over a finite partition** whose cases cover the domain.
3. A **bit-vector solver certificate**.
4. An **equivalent bounded theorem artifact**.
