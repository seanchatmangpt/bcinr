# Hardware-Level Verification: The `perf-branch-gate`

Within the `bcinr` (BranchlessCInRust) deterministic substrate, the architectural mandate is defined by the **Radon Law ($CC=1$)**: no authoritative primitive may contain conditional logic. All execution in the hot path must be branchless, allocation-free, and mathematically bounded, converting control flow into bitwise polynomials and masked state selection.

While the project enforces this at the source level via `bcinr-cheat-scanner` (AST parsing) and at the compiled level via `audit-object-code` (static disassembly), these gates are necessary but insufficient. The **ultimate arbiter of determinism is the physical silicon**, verified dynamically through the `perf-branch-gate`.

## Beyond Source Code and Disassembly

Source code is merely a suggestion to the compiler, and object code (disassembly) is a request to the processor. Statically verifying the absence of conditional jump instructions (`je`, `jne`, etc.) via `objdump` or `otool` guarantees that the compiler didn't emit explicit branches, but it cannot account for:

1. **Microarchitectural Behavior**: Complex compiler intrinsics, microcoded instructions, or standard library calls might hide data-dependent latency or internal branching within the CPU itself.
2. **Hidden Speculation**: Hardware branch predictors might still engage in speculative execution due to dynamic indirect calls or unseen pipeline behavior, introducing potential timing side-channels.
3. **The Slow Rail Infection**: The risk that non-authoritative branching logic (like `std` library allocations or string parsing) inadvertently leaks into the hot path.

To prove that the execution substrate is mathematically branchless, `bcinr` measures the undeniable physical reality of the CPU using hardware performance counters.

## The Silicon Ground Truth: `perf stat`

The `perf-branch-gate` is a critical CI pipeline step (specifically running on Linux within `Makefile.toml`) that executes the `bcinr-powl` benchmark suite (`scheduler_bench`) under hardware profiling:

```bash
perf stat -e instructions,branch-misses -- "$BENCH_BIN"
```

By extracting the exact number of retired **`instructions`** and dynamic **`branch-misses`**, the pipeline measures how the CPU's branch predictor reacted to the executed workload. 

## The 0.1% Strict Tolerance Threshold

The gate calculates the branch misprediction rate in basis points. The constitutional law dictates that **hot paths must incur < 0.1% (10 basis points) branch misses of total instructions.**

If the branch miss rate exceeds 10 basis points, the pipeline mechanically fails (`GATE FAIL`). A rate this low mathematically proves that:

1. **The Hot Path is Pure Arithmetic**: The logic is executed as predicated masks and bit-parallel fixed-width operations. The CPU is not making semantic, data-dependent execution decisions.
2. **Negligible Overhead**: The few branch misses that do occur are statistically negligible—attributed solely to the static loop backedges of the benchmark orchestration itself or initial program setup, not the authoritative algorithm.

## Why it is the Ultimate Arbiter

By gating the build matrix on actual silicon performance counters, `bcinr` eliminates the gap between theoretical determinism and physical execution. If an implementation deviates from the branchless contract—even if it tricks the static analyzer or looks clean in assembly—the hardware branch predictor will expose the data-dependent execution.

The `perf-branch-gate` ensures that $CC=1$ is not just an abstract source-code metric, but an unbreakable physical property. It guarantees that `bcinr` operates in constant $O(1)$ time, immune to timing side-channels, and perfectly predictable across civilizational-scale systems.
