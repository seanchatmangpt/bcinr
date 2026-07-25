# Mathematical Opposition: Branchless Masked Commit vs. Speculative Mutation

In the BCINR deterministic substrate, Rule 10 of `AGENTS.md` explicitly forbids speculative mutation in favor of complete admission via a branchless masked commit. This distinction is not merely a design preference but a core mathematical law governing the system's state transitions.

## The Mathematical Opposition

**Speculative Mutation** relies on sequential, branching control flow. The prohibited pattern typically looks like this:
1. Mutate persistent state (e.g., `state.mass[i] = candidate;`).
2. Evaluate validity predicates (e.g., `if invalid`).
3. Branch conditionally to return an error or rollback.

Mathematically, this introduces a discontinuity in the state transition and requires data-dependent branching, violating the Radon Law ($CC=1$). If the operation fails, the system is left in an intermediate state or relies on error-prone rollback mechanisms, which compromise deterministic execution.

**Branchless Masked Commit**, conversely, represents the state transition as a continuous, pure mathematical function without branches:
$$x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t)$$

Here, the state $x_{t+1}$ is computed by applying an admission mask ($m_{\mathrm{admitted}} \in \{0, 2^w-1\}$). If the operation is valid, the mask selects the candidate state; if invalid, it selects the current state $x_t$. This aligns with the Von Neumann Bypass architecture: sequential decisions are transformed into bitwise masks and arithmetic selection, ensuring the instruction stream remains strictly independent of the semantic input.

## The Necessity of Fixed Scratch Space

The BCINR authoritative runtime operates under absolute laws: zero heap allocation (`#![no_std]`), fixed bounded memory access, and fixed bounded execution work. 

Because the runtime cannot dynamically allocate memory ("heap-backed cloning is prohibited"), it must evaluate the candidate state concurrently with retaining the pristine original state ($x_t$). To achieve this without dynamic allocation, the system mandates computing the candidate state in **fixed scratch space**:
- Copying into a fixed-size stack value.
- Utilizing a pre-allocated fixed-size scratch structure.
- Computing the candidate structurally.

This guarantees that the memory footprint and access patterns for generating $x_{\mathrm{candidate}}$ are deterministically bounded and physically impossible to cause allocation-timing side channels or out-of-memory failures during a hot-path transition.

## Preserving Bit-for-Bit State on Rejection

Rule 10 demands that a rejected operation leaves persistent state bit-for-bit unchanged. 

In a system without panic paths, unwinding, or branching rollbacks (Rule 3), partial mutations are fatal. Speculatively modifying a state field before fully admitting the transaction risks state corruption if a subsequent predicate fails.

By enforcing complete evaluation of all predicates *before* deriving the admission mask, the transition logically collapses for invalid inputs to:
$$\operatorname{select}(0, x_{\mathrm{candidate}}, x_t) = x_t$$

This exact bit-for-bit preservation ensures that:
1. **Adversarial inputs** (as tested by `@armstrong_fault`) cannot induce invalid partial states, exploit side-effects, or bypass typed refusals.
2. The mathematical invariants verified by the Hoare oracle ($\{P(x)\} \quad f(x) \quad \{Q(x,f(x))\}$) remain unbroken across all cycles, as the system transitions atomically from one valid state to another, or definitively remains in the prior valid state without leakage.
