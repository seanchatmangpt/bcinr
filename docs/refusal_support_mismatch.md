# SupportMismatch in BCINR

## Definition

In the BCINR deterministic substrate, `SupportMismatch` is a mandatory typed refusal mandated by **Rule 18** of the repository constitution (`AGENTS.md`). It is triggered when an operation, bounded data structure, or requested topological layout deviates from mathematically proven operational boundaries or lacks admitted hardware/software support.

Because the system strictly adheres to the **Radon Law ($CC=1$)**, `SupportMismatch` cannot be implemented using control flow like `if !valid { return Err(SupportMismatch); }`. Instead, it is defined within the authoritative hot-path as a constant branchless bitflag inside a fault accumulator (e.g., `pub const SUPPORT_MISMATCH: Self = Self(1 << 5);` inside `NumericFaultSet` or `TopologyFaultSet`).

Only when execution safely exits the $CC=1$ hot-path boundary is this accumulated fault bit unpacked and translated into the user-facing enumeration `Err(StabilityRefusal::SupportMismatch)`.

## Branchless Mathematical Condition

The condition that triggers a `SupportMismatch` is enforced through bitwise polynomials and SWAR (SIMD Within A Register) masking. The mathematical sequence operates continuously without early returns:

### 1. Mask Derivation
A mismatch occurs mathematically when the requested operational bits fall outside the pre-certified support mask. This is calculated in bit-parallel:

$$ out\_of\_bounds = requested\_nodes \land \neg admitted\_support\_mask $$
$$ has\_mismatch = (out\_of\_bounds \neq 0) $$

This boolean equivalent is immediately transformed into a full-width **Canonical Mask** ($0xFFFFFFFFFFFFFFFF$ for true, $0x0000000000000000$ for false) using wrapping subtraction:
$$ mismatch\_mask = 0 - has\_mismatch $$

### 2. Sticky Fault Accumulation (Join-Semilattice)
To guarantee constant-time execution, the substrate does not short-circuit. Instead, it accumulates faults as a join-semilattice under bitwise union:

$$ e = \operatorname{select}(mismatch\_mask, SUPPORT\_MISMATCH, \emptyset) $$
$$ faults_{t+1} = faults_{t} \cup e $$

Concurrently, the computational pipeline routes to a safe fallback state via a branchless multiplexer ($\operatorname{select}(mismatch\_mask, safe\_fallback, computed\_state)$) to complete execution without triggering hardware panics.

### 3. Masked State Rejection
At the outer boundary of the authoritative hot-path, the presence of the `SUPPORT_MISMATCH` bit in the fault set dictates the final transaction mask ($m_{admitted}$). The entire state commit is predicated on this mask:

$$ x_{t+1} = \operatorname{select}(m_{admitted}, x_{candidate}, x_t) $$

If a mismatch occurred, $m_{admitted}$ is $0$, safely rejecting the transition bit-for-bit while strictly avoiding data-dependent branching.
