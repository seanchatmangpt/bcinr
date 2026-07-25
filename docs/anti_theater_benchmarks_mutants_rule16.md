# Anti-Cheat Theater Rules in BCINR

In the BCINR deterministic substrate, "Theater" rules prevent the falsification or misrepresentation of verification evidence, ensuring that compliance with mathematical and structural laws is not an illusion.

## CHEAT-008: Benchmark Theater

**Definition**: Benchmarking a stub, constant-folded path, dead result, or reduced problem not equivalent to production. 

Performance claims cannot override constitutional rules (like `CC=1`). Benchmarks must test the actual, unelided hot path with equivalent complexity and scale to a real production workload.

### How Benchmarking Avoids Constant-Folding
To prevent the LLVM backend from aggressively optimizing away the logic (e.g., dead code elimination, constant folding, branch speculation), benchmarks must strategically use `core::hint::black_box`:
1. **Input Masking**: Variables, masks, or slice references passed into the test primitive must be obscured (e.g., `popcount_u64(black_box(x))`). This forces the CPU to evaluate the function dynamically at runtime.
2. **Result Consumption**: The accumulated output of the benchmark loop must be sunk into a `black_box` return statement (e.g., `black_box(sum)`), ensuring the compiler believes the output is used externally.
3. **Branch Speculation**: In baseline comparisons, predicates are black-boxed so the branch predictor actually has to evaluate them, preventing the compiler from unrolling the loop into a fixed, branch-free pattern.

*Note: `black_box` is only an optimization barrier for benchmarks. It is strictly forbidden to claim that `black_box` guarantees machine-level branchlessness in production code.*

## CHEAT-009: Mutant Theater

**Definition**: Creating mutants that cannot compile, are trivially different, or are detected only by `assert_ne!`.

Hostile mutation testing requires injecting faults into the implementation to verify that the mathematical laws and typed refusals actively enforce correctness. 

### Ensuring Mutants are Syntactically Plausible
A mutant is not valid if it merely breaks the build (caught by `rustc` instead of tests) or alters trivial code (like strings or changing `x * 1` to `x + 0`). A plausible mutant must represent a **meaningful law alteration**, such as:
- Sign inversion
- Dropped factors in arithmetic
- Incorrect masks in state selection
- Index skews
- Bypassed refusals

Furthermore, **detecting mutants lazily with `assert_ne!(baseline, mutant)` is strictly prohibited**. Merely proving the output changed is mathematically insufficient. The test suite must prove the mutant triggered a specific, bounded **typed refusal** (e.g., `assert_eq!(result, Err(StabilityRefusal::ContractionMarginInsufficient))`), or identify the exact violated postcondition via an independent oracle.

## The Anti-Theater Verification Process

The project relies on a mandatory `bcinr-cheat-scanner` (using Abstract Syntax Tree parsing via the `syn` crate and text scanning) to actively block theater violations.

- **Enforcing CHEAT-008**: The scanner examines method calls named `bench_function` or `iter`. If the stringified arguments of the benchmark contain words like `branchless` or `allocate` but lack `black_box`, the scanner flags it as a violation, rejecting the benchmark theater.
- **Enforcing CHEAT-009**: When scanning test files, if a test contains the word `mutant` and uses the `assert_ne!` macro, the scanner demands explicit evidence of a proper verification boundary. It requires the file to contain strings such as `Err(StabilityRefusal::`, `.is_refused()`, `.numeric_faults()`, or `// Named law:`. Without these explicit proofs, the mutant is rejected as theater.

Fabricating verification evidence via these "theaters" is an **absolute failure**, which instantly forces the Substrate Integrity Score to 0 and triggers the quarantine protocol `MaturityScrutiny`.
