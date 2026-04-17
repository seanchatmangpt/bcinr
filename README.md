# bcinr — BranchlessCInRust

Academic-grade Rust library for branchless algorithms, generated and scaffolded with [unrdf](../unrdf). See [docs/bcinr_whitepaper.pdf](./docs/bcinr_whitepaper.pdf) for detailed specifications.

## 12 Algorithm Families

| Family | Description | Symbols |
|--------|-------------|---------|
| **mask** | Bitwise mask calculus | select_u32, select_u64, blend_u8x16 |
| **int** | Integer bitwise | popcount_u32, leading_zeros_u32, next_power_of_two_u32 |
| **fix** | Saturation arithmetic | add_sat_u8, sub_sat_u8, clamp_u32 |
| **network** | Compare-exchange networks | bitonic_sort_8u32, bitonic_sort_16u32, compare_exchange |
| **bitset** | Bitset algebra | set_bit_u64, clear_bit_u64, rank_u64, select_bit_u64 |
| **scan** | Byte/word scanning | first_zero_byte, count_zero_bytes, find_byte |
| **utf8** | UTF-8 calculus | validate_utf8, count_codepoints, next_codepoint_boundary |
| **parse** | Parsing primitives | parse_decimal_u64, parse_hex_u32, skip_whitespace |
| **dfa** | Table-driven automata | dfa_advance, dfa_run, dfa_is_accepting |
| **reduce** | SWAR reductions | horizontal_sum_u8x8, horizontal_max_u8x8, horizontal_min_u8x8 |
| **sketch** | Probabilistic sketches | murmur3_32, xxhash32, fnv1a_64 |
| **simd** | SIMD primitives | splat_u8x16, shuffle_u8x16, movemask_u8x16 |

## Architecture: api/ ↔ logic/

```
src/api/        ← 100% generated (overwrite on every sync)
  └─ mask.rs   └─ thin wrappers calling crate::logic::mask

src/logic/      ← bootstrap-once (skip_existing)
  └─ mask.rs   └─ handwritten performance-critical implementations
```

## Design Philosophy

- **api/** is the public interface — entirely generated from the ontology. Edit the ontology, not the API files.
- **logic/** is the performance substrate — handwritten once, left alone forever (unless you change the symbol names).
- **lib.rs** is bootstrap-once — generated on first sync, never touched again.
- **mod.rs** files are overwritten on every sync to keep module declarations in sync with the ontology.

## Usage

### Build

```bash
cargo check
cargo build --release
```

### Regenerate from Ontology

```bash
cd .. && node ./unrdf/packages/cli/src/cli/main.mjs sync --config bcinr/unrdf.toml
```

Expected output:
- **SKIP** for all `src/logic/` files (bootstrap-once)
- **OK** for all `src/api/` files (overwrite)
- **OK** for `src/lib.rs`, `src/api/mod.rs`, `src/logic/mod.rs`

### Test

```bash
cargo test --lib
```

All 49 tests pass (stubs compile).

## Implementation Roadmap

Once scaffolding is complete, implement each family:

1. Edit `src/logic/FAMILY.rs` (one of: mask, int, fix, network, bitset, scan, utf8, parse, dfa, reduce, sketch, simd)
2. Replace `todo!()` with actual branchless code
3. Add doctests and benchmarks
4. Run `cargo test` to verify

The `api/` layer will automatically expose your implementations without any changes.

## Generated

Generated via `unrdf sync` from:
- `ontology/bcinr.ttl` — RDF ontology with 12 families × 3–5 symbols each
- `templates/*.njk` — Nunjucks templates with Hygen-style front matter

See `unrdf.toml` for the 5 generation rules.
