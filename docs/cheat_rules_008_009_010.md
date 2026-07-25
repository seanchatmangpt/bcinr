# Analysis of "Theater" Violations (Rule 16)

This document expands on the "Theater" violations defined in Rule 16 of `AGENTS.md` (`CHEAT-008`, `CHEAT-009`, and `CHEAT-010`). Theater violations occur when compliance with BCINR's deterministic substrate rules is falsified, exaggerated, or artificially engineered to bypass verification gates without actually fulfilling the strict axiomatic requirements.

## CHEAT-008: Benchmark Theater
**Definition:** Benchmarking a stub, constant-folded path, dead result, or reduced problem not equivalent to production.

**What it looks like in practice:**
* **Constant-Folding & Dead Code Elimination:** Writing a benchmark where the inputs are statically known and the output is discarded. Because Rust's LLVM backend is aggressive, it will optimize the entire function away, resulting in a benchmark that reports execution times near `0 ns`. 
  * *Example:* Running `let _ = hot_path_math(5, 10);` instead of using `core::hint::black_box` to force the compiler to perform the actual work (`let res = hot_path_math(black_box(5), black_box(10)); black_box(res);`).
* **Benchmarking Stubs:** Testing an empty or simplified fallback function instead of the heavy, production-ready mathematical routine.
* **Reduced Problem Size:** Benchmarking an array of 2 elements when the production domain mathematically guarantees operations on an array of 4,096 elements. This creates a deceptive narrative about the algorithmic complexity and runtime execution bounds.

## CHEAT-009: Mutant Theater
**Definition:** Creating mutants that cannot compile, are trivially different, or are detected only by `assert_ne!`.

**What it looks like in practice:**
* **Non-compiling Mutants:** Injecting a syntax error or a type mismatch (e.g., changing `let mask: u64 = 0;` to `let mask: u64 = "string";`). The developer claims the mutant was "killed," but in reality, it was just rejected by `rustc` during compilation, not by adversarial tests.
* **Trivially Different Mutants:** Modifying code in a way that doesn't actually test a load-bearing mathematical law. 
  * *Example:* Changing a debug string or modifying `x * 1` to `x + 0`. This checks a box for "mutant created" but fails the requirement of adversarial fault verification.
* **`assert_ne!` Detection:** Writing a test that simply asserts the mutant's output differs from the baseline (`assert_ne!(baseline, mutant_output)`). This is strictly prohibited. The test must definitively prove that the mutated code violates a specific post-condition from the independent oracle or triggers a precise, typed refusal (e.g., `assert_eq!(mutant_output, Err(StabilityRefusal::ContractionMarginInsufficient))`).

## CHEAT-010: Gate-Jurisdiction Theater
**Definition:** Reporting a passing scanner that does not inspect the relevant crate, file, generated output, feature set, or target.

**What it looks like in practice:**
* **Narrowed Scanning:** Running `bcinr-cheat-scanner` or the object-code auditor on a safe, irrelevant directory (like `src/cli/` or the "slow rail") while silently excluding the authoritative hot-path crate where branches are hiding.
* **Ignoring Generated Code:** Using `build.rs` or declarative macros to generate code containing hidden `if` statements or loops, and only running the cheat scanner on the pre-expanded source files, failing to inspect the true source graph.
* **Target and Feature Omission:** Claiming compliance based on a single specific architecture (e.g., `x86_64` with `target-feature=+bmi2` enabled) but failing to run the verification gate on fallback targets (like `wasm32`) where the compiler might emit branching instructions for the exact same Rust source code.
