I have reviewed `AGENTS.md` regarding **CHEAT-008 — Benchmark theater**.

Under **Rule 16 (Anti-cheat manifesto)**, "Benchmark theater" (CHEAT-008) is explicitly defined as:

> *Benchmarking a stub, constant-folded path, dead result, or reduced problem not equivalent to production.*

In the context of the BCINR (BranchlessCInRust) deterministic substrate, this anti-cheat rule exists to prevent reporting invalid or misleading performance metrics that do not reflect true authoritative execution. Because BCINR enforces strict branchless (`CC=1`), zero-allocation, straight-line arithmetic mechanics (relying heavily on SWAR and bit-parallel execution), the runtime is highly susceptible to aggressive compiler optimizations.

Specifically, in BCINR, **Benchmark theater** means falling into one of these traps:

1. **Constant-folded path**: If benchmark inputs are static or known at compile-time, the Rust compiler (via LLVM) will often compute the entire branchless algebraic equation during compilation. The benchmark then falsely measures a zero-cost execution instead of the actual dynamic instruction workload.
2. **Dead result**: Because authoritative code in BCINR forbids side effects (no heap allocations, no unwinding, no I/O), a computation whose result is not explicitly observed or returned in the benchmark can be entirely stripped away as "dead code" by the optimizer. 
3. **Stub**: Benchmarking a non-authoritative dummy primitive or simple fallback rather than the full, rigorous masking logic that passes the disassembly/object-code gates.
4. **Reduced problem**: Testing over a simplistic or narrow domain partition instead of the exact, fixed-width structural state transitions and fixed-bounded execution work required for the production environment (which enforces full-domain 2^64 validity).

In summary, to comply with the BCINR constitution, benchmarks must force the execution of the actual verified object code using opaque inputs and observable outputs, rather than measuring compiler illusions.
