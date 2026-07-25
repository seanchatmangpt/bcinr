Based on my research of the `AGENTS.md` and `docs/branchless_contract_failed_refusal.md` files, here are the details regarding the `BranchlessContractFailed` typed refusal:

### Purpose of `BranchlessContractFailed`

In the `bcinr` deterministic substrate, `BranchlessContractFailed` is a strictly bounded **Typed Refusal** that acts as the structural fail-safe for the runtime's axiomatic calculus. Its primary purposes are:

1. **Enforcing the Radon Law ($CC=1$)**: It ensures that constraint failures are handled without any control-flow branching. Instead of using early `return`s, `if` statements, or `panic!`, failures evaluate as boolean constraints reduced to bitmasks (e.g., evaluating to `0`).
2. **Bit-Parallel Accumulation**: These bitmasks are seamlessly accumulated into a `RefusalSet` and mathematically resolved as the `BranchlessContractFailed` state when hitting a strict Envelope Boundary.
3. **Zero-Allocation Guarantee**: It helps maintain a zero-allocation boundary by signaling failures deterministically and purely through bitwise polynomial math.
4. **Preventing Silent Fallbacks**: If an unsupported condition or edge case occurs, the runtime is strictly prohibited from bypassing branchless fixed-point constraints by falling back to simpler, branching, or floating-point algorithms. It must yield `BranchlessContractFailed` instead.

### Why It Must Be Formally Bound

Formal binding of this typed refusal is non-negotiable for preserving the substrate's integrity and achieving a perfect Substrate Integrity Score (SIS). It must be formally bound to enforce the following:

- **Mask-Based State Isolation**: Because state is never mutated speculatively, the refusal mechanically drives execution logic via fixed-width mathematical selection ($x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t)$). If the Hoare contract evaluates to a `BranchlessContractFailed` equivalent mask ($m_{\mathrm{admitted}} = 0$), the candidate mutation is cleanly discarded bit-for-bit, keeping persistent state intact.
- **Hostile Mutant Verification**: When adversarial negative domain tests are injected (`@armstrong_fault`), the protocol demands absolute typed proof of failure. Tests must explicitly assert `assert_eq!(result, Err(StabilityRefusal::BranchlessContractFailed))`. Generic `assert_ne!` assertions or bounds-check panics are strictly prohibited.
- **Ensuring Branchlessness**: By ensuring that *mathematical rejection never necessitates control-flow rejection*, it protects the whole authoritative call graph from branching. Any attempt to use unwrap checks or conditional jumps is flagged immediately by the `bcinr-cheat-scanner` and object-code disassembler, automatically blocking the merge.
