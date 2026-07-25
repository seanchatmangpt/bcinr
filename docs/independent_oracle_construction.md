# Independent Oracle Construction Example (Rule 15)

Rule 15 states that an oracle is not independent merely because it is in a test file. It prohibits line-by-line translation of production code, reuse of production normalization, lookup tables, or fixed-point helpers. It requires the oracle to be structurally and algorithmically distinct, using mathematical formulas, abstract state machines, or arbitrary-precision/floating-point implementations.

### Independent Oracle Example: `q_lens` / `fixed-point` verification

In `crates/bcinr-cmca/tests/reference.rs`, an independent `f64` oracle is constructed to verify the branchless fixed-point production code for the `allocate_f64` algorithm. 

Here is the markdown documentation of its construction:

```rust
// This file is the independent f64 oracle required by
// `.claude/rules/cmca/verification.md` Invariant 2: it must remain structurally and
// algorithmically distinct from the production fixed-point implementation, so its
// control-flow shape (explicit index loops, manual min/max clamps mirroring the
// authoritative function's own parameter list) is left as originally written rather
// than rewritten to satisfy production-code style lints. Documented allow per
// AGENTS.md's "no undocumented allow" rule.
#![allow(
    clippy::needless_range_loop,
    clippy::manual_clamp,
    clippy::too_many_arguments
)]

use bcinr_cmca::generated::case_studies::{LensSpec, PackedSemanticState, K, N, Q};

pub fn compute_measures_f64(state: &PackedSemanticState) -> [f64; K] {
    let factors: Vec<f64> = state
        .factors
        .iter()
        .map(|f| f.value_bits() as f64 / 65536.0)
        .collect();

    let recomp = factors[0];
    let verify = factors[1];
    let standing = factors[2];
    let access = factors[4];
    let search = factors[5];
    let retrieval = factors[6];
    let sched = factors[7];
    let bval = factors[8];
    let conseq = factors[9];

    let m0 = (recomp * 5.0 + verify) * access * standing;
    let m1 = (bval + conseq) * search * standing;
    let m2 = bval * retrieval;
    let m3 = bval * sched;

    [m0, m1, m2, m3]
}

pub fn allocate_f64(...) -> [f64; N] {
    // 1. Identify leaves (using branching and distinct control flow)
    let mut is_leaf = [true; N];
    for i in 0..N {
        for j in 0..N {
            if parent[j] == i as i32 {
                is_leaf[i] = false;
            }
        }
    }

    // ...

    // 4. Overwrite parent masses with q-norm aggregation
    let mut node_masses = raw_masses;

    // Manual Clamp masses instead of production branchless SWAR clamp
    for k in 0..K {
        for i in 0..N {
            if node_masses[k][i] < 0.0001 {
                node_masses[k][i] = 0.0001;
            }
            if node_masses[k][i] > 1000.0 {
                node_masses[k][i] = 1000.0;
            }
        }
    }

    // ... (Floating point logic with `.powf()` and `.exp()` standard libraries 
    // rather than the production environment's strict `no_std` bitwise arithmetic)
```

### Key Elements of Construction
1. **Mathematical Divergence (f64 vs. Fixed-Point):** It uses standard double-precision floating-point arithmetic (`f64`), `.powf()`, and `.exp()` rather than the fixed-width bit-parallel fixed-point arithmetic (`#![no_std]` 0-allocation bounds) seen in the production hot path.
2. **Control-Flow Independence:** It relies on explicit loop indexing, nested `for` statements, and manual clamping with `if/else`, which are fundamentally prohibited (via `CC=1` and branchlessness) in the authoritative code. 
3. **Intentional Exemption from Linting:** The oracle uses `#![allow(clippy::...)]` specifically to avoid being refactored into idiomatic or "clever" Rust that might mirror the production pipeline, fulfilling the "structurally distinct" requirement of Rule 15.
4. **Independent Variable Scaling:** E.g., converting bits to floats by explicitly dividing by `65536.0` instead of using the production bitwise shift or fixed-point helper structures.
