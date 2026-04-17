# How-To Guides: Branchless Engineering

## How to Guarantee WCET
To guarantee deterministic execution time:
- **Avoid:** `if`, `match`, `while` (with data-dependent termination), and recursive function calls.
- **Use:** Bitwise masks (`select_u32`), lookup tables, and loop unrolling.
- **Verify:** Use `cargo bench` to detect outliers (`target/criterion`).

## How to Debug Side-Channel Leaks
If timing variability is detected:
1.  Isolate the branch in `src/logic/`.
2.  Refactor into a sequence of arithmetic operations (e.g., using `lt_mask` instead of comparison).
3.  Re-run `perf stat` or Criterion to verify variance $\sigma^2 \to 0$.
