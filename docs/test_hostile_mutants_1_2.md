Here is the information regarding the implementations of `mutant_1` and `mutant_2`, including the laws they break and their expected refusal behaviors according to the independent oracle in `hostile_mutants.rs`:

### **Mutant 1: Single Measure Collapse**

* **Implementation Location:** `crates/bcinr-cmca/src/allocator.rs` (around line 1857)
* **Test/Oracle Location:** `crates/bcinr-cmca/tests/hostile_mutants.rs` (around line 467)

**Mathematical Law Broken:**
**Measure Collapse (Index Skew/Dropped Factor).** 
`mutant_1` pins the active measure index (`k_actual`) to `0` for every `k`. This collapses the per-measure canonical-mixing law, which dictates that each of the `K` measures must independently weight the allocation, into a single-measure result.

**Expected Typed Refusal:**
This mutant does **not** produce a typed refusal. As governed by the constitution, when a mutant produces a wrong accepted value rather than a refusal, the test must identify the exact violated postcondition. The oracle asserts that this deterministic corruption produces a specific corrupted allocation array named `WRONG_M1_MEASURE_COLLAPSE` (rather than the `CORRECT_BASELINE`).

---

### **Mutant 2: Q Sign Inversion**

* **Implementation Location:** `crates/bcinr-cmca/src/allocator.rs` (around line 1787)
* **Test/Oracle Location:** `crates/bcinr-cmca/tests/hostile_mutants.rs` (around line 489)

**Mathematical Law Broken:**
**Sign Inversion.** 
`mutant_2` negates each lens's signed `q` value (via `0i32.wrapping_sub(...)`) before it enters the exponential weighting update. This inverts the sign convention that the lens-weighting law requires, where higher payoff must mathematically *increase*, not decrease, relative weight.

**Expected Typed Refusal:**
Similar to `mutant_1`, this mutant does **not** trigger a typed refusal. It evaluates completely but yields a corrupted consequence. The oracle captures the exact violated postcondition by verifying that the output exactly matches the corrupted allocation array named `WRONG_M2_Q_SIGN_INVERSION`.
