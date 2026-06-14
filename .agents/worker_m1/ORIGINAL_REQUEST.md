## 2026-06-12T19:31:02Z
You are a Worker agent. Your working directory is `/Users/sac/bcinr/.agents/worker_m1/`.
Your task is to implement Milestone 1:
1. Fix the line length assertions in `/Users/sac/bcinr/tools/u64_audit.py`. Change `assert len(lines) == 36` to `assert len(lines) == 34` in `build_target_c`, and change `assert len(lines) == 39` to `assert len(lines) == 34` in `build_target_d`.
2. Inspect the 307 algorithm files in `crates/bcinr-logic/src/algorithms/` (excluding `mod.rs`). Some may have parameter names other than `(val: u64, aux: u64)`. Refactor both their public implementation functions and reference functions (under `mod tests`) to use the parameter names `val` and `aux` and type `u64`, in order to satisfy the interface contract: `pub fn <name>(val: u64, aux: u64) -> u64`.
3. Run `tools/u64_audit.py` to update the references, doc comments, proofs, and padding.
4. Refactor the implementation bodies of all 307 algorithm files to match the updated references branchlessly. The implementation must not contain `if`, `match`, or loops (to satisfy Radon Law CC=1). For Category F, which has a reference implementation containing an `if` expression, implement the select branchlessly using bitwise operations (e.g., mask-based selection).
5. Verify that `cargo test -p bcinr-logic --lib` compiles and passes all unit and proptest equivalence tests.
6. Write a handoff report at `/Users/sac/bcinr/.agents/worker_m1/handoff.md` summarizing the changes, verification results, and any warnings.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.
