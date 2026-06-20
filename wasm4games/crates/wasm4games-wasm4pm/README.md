# wasm4games-wasm4pm

The **online-only admission bridge** between [`wasm4games`](../wasm4games) and the upstream
**wasm4pm** process miner.

## Quick Setup Checklist

- [ ] Pin `wasm4pm` to a specific `rev = "..."` commit in Cargo.toml
- [ ] Pin `wasm4pm-compat` to a specific `rev = "..."` commit in Cargo.toml
- [ ] Replace stub types (`W4pmLogStub`, `W4pmEventStub`, `W4pmReceiptStub`) with real types from `wasm4pm-compat`
- [ ] Run `cargo check -p wasm4games-wasm4pm` to verify the bridge compiles
- [ ] Run `cargo test -p wasm4games-wasm4pm` to verify roundtrip evidence conversion

> Engines project worlds; wasm4games operates patterns; **wasm4pm admits evidence.**

wasm4games emits candidate, object-centric process evidence but never decides admissibility.
This crate carries that evidence across the boundary: it maps wasm4games evidence types onto
the canonical `wasm4pm-compat` types and submits them to wasm4pm for admission, returning a
conformance verdict.

## Excluded from the workspace — and why

This crate is listed under `[workspace] exclude` in the root `Cargo.toml`. It is the **only**
part of the repository that depends on external git repositories, so it is kept out of the
offline workspace build to guarantee that build resolves **zero git dependencies**.

As a result:

- It is **not** built, tested, or linted by the offline workspace commands.
- It **requires network access** to fetch `wasm4pm` and `wasm4pm-compat`.
- It may not compile until the upstream API is resolved and the `// TODO: confirm against
  wasm4pm-compat ...` notes in `src/lib.rs` are reconciled against the real types.

## Before use

1. **Pin the git revisions.** In `Cargo.toml`, replace the branch/tag references with exact
   commit hashes (`rev = "<hash>"`). Tags and branches are mutable; pinning a commit is
   required for reproducible admission.
2. **Reconcile the stubs.** `src/lib.rs` ships dependency-free stand-in types
   (`W4pmLogStub`, `W4pmEventStub`, `W4pmReceiptStub`) and `/* wasm4pm_compat::Type */`
   placeholders so the file parses offline. Replace each with the confirmed upstream path and
   delete the stubs.
3. **Build online.**

   ```bash
   cargo build -p wasm4games-wasm4pm   # requires network
   ```

## Mapping surface

| wasm4games                       | wasm4pm-compat (canonical)        |
| -------------------------------- | --------------------------------- |
| `evidence::ocel::OcelLog`        | `OcelLog` / `LinkedOcel`          |
| `evidence::ocel::OcelEvent`      | `OcelEvent` (+ object edges)      |
| `evidence::receipt::ReceiptEnvelope` | `ReceiptEnvelope` / `ReceiptChain` |
| `compat::Verdict`                | `ConformanceResult`               |
| (typestate)                      | `Evidence<T, State, W>`           |

Upstream, referenced not vendored:

- wasm4pm — <https://github.com/seanchatmangpt/wasm4pm>
- wasm4pm-compat — <https://github.com/seanchatmangpt/wasm4pm-compat>

## License

Dual-licensed under **MIT OR Apache-2.0**.
