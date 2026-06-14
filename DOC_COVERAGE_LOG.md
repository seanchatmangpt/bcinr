# Documentation Coverage Log

Tracks bijective coverage: every documented capability has a running example, every example references the doc it demonstrates. Gap map drives iterations; prose alone is never marked ✅.

---

## Iteration 1 — 2026-06-14

**State:** commit `ef3017a`, tree clean, rustc `1.97.0-nightly` (cb40c25f6 2026-05-04)

### Gap Map (at iteration start)

**Documented-but-unexercised (ALL public API surface):**
- `bcinr::mask`: `select_u32`, `select_u64`, `eq_mask_u32`, `is_zero_mask_u32`, `nonzero_mask_u32`, `lt_mask_u32`, `min_u32`, `max_u32`, `abs_i32`
- `bcinr::fix`: `add_sat`, `clamp_u32`, `bucketize_u32`
- `bcinr::int`: `int_phd_gate` (formal anchor), plus functions from `bcinr-api`
- `bcinr::bitset`: `rank_u64`, `select_bit_u64`, `parity_u64_slice`, `jaccard_u64_slices`, `hamming_u64_slices`, `intersect_u64_slices`, `union_u64_slices`, `any_bit_set_u64_slice`
- `bcinr::dfa`: `dfa_advance`, `dfa_run`, `dfa_is_accepting`
- `bcinr::logic::algorithms::*`: 308 algorithm functions (each has `/// # Branchless Contract` doc)
- README Quick Start code (3 doc claims)

**Exercised-but-undocumented:** NONE (no examples/ directory existed)

**False doc claims discovered:**
1. README Quick Start: `use bcinr_core::api::{select_u32, add_sat_u8, clamp_u32}` — `bcinr_core::api` is an empty crate root (lib.rs = `// API exports...`). Correct paths are `bcinr::mask::*` and `bcinr::fix::*`.
2. README Quick Start: `add_sat_u8(200, 100)` — `add_sat_u8` (u8) does not exist in the accessible public surface. Only `add_sat` (u32) is in `bcinr::fix`.
3. README Quick Start: `clamp_u32(150, 0, 100).unwrap()` — `clamp_u32` returns `u32`, not `Result`. `.unwrap()` would fail to compile.

### Triples Closed

**Triple 1 — Mask Primitives ✅**
- **Doc:** `crates/bcinr-logic/src/mask.rs` (rustdoc on all public fns)
- **Example:** `bcinr/examples/mask_primitives.rs`
- **Run output:** `select_u32: mask=0xFFFFFFFF → 10, mask=0x0 → 20` / `eq_mask_u32: 42==42 → 0xffffffff` / `branchless clamp(150, 0, 100) = 100` / `All mask primitive assertions passed.`
- **Exit code: 0** (real exit, not echo)
- **Fail-if-fake:** `assert_eq!(chose_a, 10)` would fail if `select_u32` ignored the mask

**Triple 2 — Saturation Arithmetic ✅**
- **Doc:** `crates/bcinr-logic/src/fix.rs` (rustdoc on `add_sat`, `clamp_u32`, `bucketize_u32`)
- **Example:** `bcinr/examples/saturation_arithmetic.rs`
- **Run output:** `add_sat: 200+100=300, MAX+1=4294967295` / `clamp_u32: 150∈[0,100]=100` / `All saturation arithmetic assertions passed.`
- **Exit code: 0**
- **Fail-if-fake:** `assert_eq!(add_sat(u32::MAX, 1), u32::MAX)` fails if wrapping; `assert_eq!(clamp_u32(150, 0, 100), 100)` fails if passthrough

**Triple 3 — Branchless Pipeline (cross-product) ✅**
- **Doc:** `crates/bcinr-logic/src/mask.rs` + `fix.rs` + `bitset.rs` composition
- **Example:** `bcinr/examples/branchless_pipeline.rs`
- **Run output:** `scores: [10, 50, 30, 50, 20, 50, 70, 90]` / `awarded: [10, 55, 30, 55, 20, 55, 70, 90]` / `high-scorer bitset: 0b11101010` / `hamming distance orig vs clamped: 3` / `All branchless pipeline assertions passed.`
- **Exit code: 0**
- **Fail-if-fake:** Stage 1 `select_u32(hit_mask, with_bonus, s)` assertion fails if mask ignored; Stage 3 `select_bit_u64` assertion fails if wrong position returned

### Also Fixed
- README Quick Start corrected to use real API paths with verified imports
- README Quick Start now links to the 3 new example files

### Queued (next iterations)

**OPEN-documented-unexercised (remaining high-value clusters):**
- `bcinr::dfa` — `dfa_run` / `dfa_advance` / `dfa_is_accepting`: DFA pattern matching, described in rustdoc, no example
- `bcinr::bitset` — standalone cluster: `rank_u64`, `jaccard_u64_slices`, `parity_u64_slice`: bitset algebra described, not demonstrated in isolation
- `bcinr::int` — popcount, leading/trailing zeros: described in rustdoc
- `bcinr::reduce` — `horizontal_and_u32`, `horizontal_or_u32`, `horizontal_xor_u32`: described, no example
- 308 algorithm functions in `bcinr::algorithms` — all have `/// # Branchless Contract` doc, no examples. These are the largest open cluster; a representative cross-section example (e.g. 5 algorithms from different families composing) would demonstrate the surface without enumerating all 308.

**OPEN-doc-substrate (false API claim to resolve):**
- `bcinr_core::api::fix::add_sat_u8` — present in `bcinr-api/src/mod.rs` declaration but `bcinr-api/src/lib.rs` is empty, so the `mod.rs` is dead. If `add_sat_u8` is an intended public API, the `lib.rs` needs to be fixed to declare/re-export the modules from `mod.rs`. Currently the entire `bcinr_core::api` module tree is unreachable.

---

---

## Iteration 2 — 2026-06-14

**State:** commit `2f601db`, tree clean, rustc `1.97.0-nightly` (cb40c25f6 2026-05-04)

### Gap Map (at iteration start — remaining after iteration 1)

**Documented-but-unexercised:**
- `bcinr::int`: `popcount_u64/u32`, `leading_zeros_u64/u32`, `trailing_zeros_u64/u32`, `reverse_bits_u64/u32`, `next_power_of_two_u32`, `is_pow2_u32`, `parity_u32`, `saturating_add/sub/mul_i64`
- `bcinr::dfa`: `dfa_advance`, `dfa_run`, `dfa_is_accepting`
- `bcinr::reduce`: `horizontal_or_u32`, `horizontal_and_u32`, `horizontal_xor_u32`, `horizontal_sum_u8x8`, `horizontal_max_u8x8`, `horizontal_min_u8x8`
- `bcinr::bitset` (standalone): `rank_u64`, `select_bit_u64`, `parity_u64_slice`, `jaccard_u64_slices`
- `bcinr::logic::algorithms::*`: 308 algorithm functions (open)

### Triples Closed

**Triple 4 — Integer Bit Operations ✅**
- **Doc:** `crates/bcinr-logic/src/int.rs` (rustdoc on all public fns)
- **Example:** `bcinr/examples/integer_ops.rs`
- **Run output:** `popcount_u64(0b1011)=3`, `trailing_zeros_u64(0x10)=4`, `next_power_of_two_u32(100)=128`, `saturating_add_i64(MAX,1)=9223372036854775807`, `All integer operation assertions passed.`
- **Exit code: 0**
- **Fail-if-fake:** `reverse_bits_u64(reverse_bits_u64(x)) == x` (double-reverse identity) would fail if the bit-reversal is wrong; parity cross-check against `popcount_u32 & 1` would fail if either diverges

**Triple 5 — DFA Pattern Matching ✅**
- **Doc:** `crates/bcinr-logic/src/dfa.rs` (rustdoc on `dfa_advance`, `dfa_run`, `dfa_is_accepting`)
- **Example:** `bcinr/examples/dfa_matching.rs`
- **Run output:** `dfa_advance: S0+0=0, S0+1=1, S1+1=0`, `dfa_run([1,1,1]) → state 1 (accepted=true)`, `dfa_run([1,1]) → state 0 (accepted=false)`, `All DFA matching assertions passed.`
- **Exit code: 0**
- **Fail-if-fake:** `assert_eq!(s0_on_one, 1, "S0 + 1 → S1")` breaks if advance ignores input; empty-input assertion verifies initial state is preserved

**Triple 6 — Horizontal Reductions ✅ (partial)**
- **Doc:** `crates/bcinr-logic/src/reduce.rs` (rustdoc on all public fns)
- **Example:** `bcinr/examples/horizontal_reductions.rs`
- **Run output:** `horizontal_or_u32([1,2,4])=0b111`, `horizontal_sum_u8x8([1..8])=36`, `flags union=0b1111, intersection=0b1000`, `All horizontal reduction assertions passed.`
- **Exit code: 0**
- **Covered:** `horizontal_or_u32`, `horizontal_and_u32`, `horizontal_xor_u32`, `horizontal_sum_u8x8`

### Defect Found: OPEN-defect — horizontal_max_u8x8 / horizontal_min_u8x8

**Severity:** debug-build panic (release may silently produce wrong results)

`horizontal_max_u8x8` and `horizontal_min_u8x8` in `crates/bcinr-logic/src/reduce.rs:57` use plain `+` for a SWAR byte-lane comparison, causing integer overflow in debug builds (Rust's overflow checks fire). The intermediate `(v2 & mask) + (mask ^ (v & mask))` can produce values where byte-lane carries propagate across u64 word boundaries, corrupting adjacent lanes.

**Fix needed:** Replace `+` with `wrapping_add` at minimum (prevents panic); a correct fix also needs to prevent cross-lane carry (mask inputs to 7 bits or restructure the SWAR comparison). These two functions are marked OPEN-defective until repaired.

### Also Noted

- `dfa.rs` and `reduce.rs` contain residual "Padding Line N" and boilerplate comments not caught by `strip_boilerplate.py` (different sentinel text). Tracked for a future strip pass.
- `dfa.rs` module tests contain a vacuous `dfa_reference(val,aux) = val^aux` with no relation to DFA semantics — leftover scaffolding, not tested here.

### Queued (next iterations)

**OPEN-documented-unexercised:**
- `bcinr::bitset` standalone: `rank_u64`, `select_bit_u64`, `parity_u64_slice`, `jaccard_u64_slices`, `hamming_u64_slices`, `intersect_u64_slices`, `union_u64_slices`
- `bcinr::reduce::horizontal_max_u8x8` / `horizontal_min_u8x8` — OPEN-defective (need impl fix before example can witness)
- `bcinr::scan`: all scan functions (unexamined)
- `bcinr::utf8`: all UTF-8 functions (unexamined)
- `bcinr::sketch`: HyperLogLog / Bloom filter API (unexamined)
- `bcinr::algorithms::*` — 308 algorithms, each with `/// # Branchless Contract`: a representative cross-section example covering one from each difficulty tier (1-100, 101-200, 201-300) is the right approach rather than 308 individual examples


---

## Iteration 3 — 2026-06-14

**State:** commit `d125c49`, tree clean, rustc `1.97.0-nightly`

### Gap Map (at iteration start — remaining after iteration 2)

**Documented-but-unexercised:**
- `bcinr::scan`: `find_byte_mask`, `skip_spaces`, `is_ascii_u64_slice`
- `bcinr::utf8`: `count_codepoints`
- `bcinr::sketch`: `count_min_sketch_update`
- `bcinr::algorithms::*`: 308 algorithm functions (representative cross-section still needed)
- `bcinr::bitset` standalone (queued)
- `bcinr::reduce::horizontal_max_u8x8` / `horizontal_min_u8x8` (OPEN-defective)

### Triples Closed

**Triple 7 — Scan Primitives ✅**
- **Doc:** `crates/bcinr-logic/src/scan.rs`
- **Example:** `bcinr/examples/scan_primitives.rs`
- **Run output:** `find_byte_mask(b"hello world", b'l') = 0b01000001100`, `skip_spaces(b"   hello") = 3`, `tokenizer: skipped 3 spaces, token=token`, `All scan primitive assertions passed.`
- **Exit code: 0**
- **Fail-if-fake:** bit-position assertions on the mask fail if `find_byte_mask` miscounts; the tokenizer cross-product composition fails end-to-end if either scan function is wrong

**Triple 8 — UTF-8 Codepoints + Count-Min Sketch ✅**
- **Doc:** `crates/bcinr-logic/src/utf8.rs` + `src/sketch.rs`
- **Example:** `bcinr/examples/utf8_and_sketch.rs`
- **Run output:** `count_codepoints(héllo bytes=6)=5`, `count_min_sketch: 3/3 rows have max=2`, `saturation: updating MAX cells stays at u32::MAX = true`, `All UTF-8 and sketch assertions passed.`
- **Exit code: 0**
- **Fail-if-fake:** `count_codepoints(héllo) != héllo.len()` assertion catches raw byte-counting; saturation test fails if `saturating_add` wraps

**Triple 9 — Algorithm Cross-Section (5 functions, composition pipeline) ✅**
- **Doc:** `crates/bcinr-logic/src/algorithms/` — `abs_diff_u64`, `rotate_left_u64`, `gcd_u64_branchless`, `popcount_u128`, `leb128_decode_u64`
- **Example:** `bcinr/examples/algorithms_cross_section.rs`
- **Run output:** `gcd(12,8)=4`, `leb128_decode(0x0180)=128`, `pipeline: gcd=12, normalized=(4,3), combined_bits=3, rotate=4096`, `All algorithm cross-section assertions passed.`
- **Exit code: 0**
- **Composition proof:** GCD normalization pipeline chains gcd → abs_diff → rotate_left → popcount; any broken link fails the final `rotate=4096` assertion

### Queued (next iterations)

**OPEN-documented-unexercised (remaining):**
- `bcinr::bitset` standalone: `rank_u64`, `select_bit_u64`, `parity_u64_slice`, `jaccard_u64_slices`, `hamming_u64_slices`, `intersect_u64_slices`, `union_u64_slices` — used only as sub-operations in iteration 1's pipeline; need a dedicated standalone example
- `bcinr::reduce::horizontal_max_u8x8` / `horizontal_min_u8x8` — OPEN-defective (SWAR overflow bug)
- `bcinr::algorithms::*` — 308 total functions; 5 demonstrated in triple 9; ~303 not individually witnessed. The cross-section proves the surface pattern; exhaustive 308-example coverage is a separate (very large) task.
- `bcinr::network` — not yet examined
- `bcinr::parse` — not yet examined

**Approximate coverage state after 3 iterations (9 triples):**
- Core modules (mask, fix, int, bitset, dfa, reduce, scan, utf8, sketch): ~80% of public fns have running witnesses
- Algorithm module (308 fns): 5/308 individually witnessed; the shared branchless contract pattern is documented in the cross-section
- Defects found: 1 (horizontal_max/min_u8x8 SWAR overflow); 3 README false claims corrected

