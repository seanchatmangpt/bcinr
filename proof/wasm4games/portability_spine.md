# wasm4games — Portability Spine Receipt

> Discharges the offline-verifiable core of the portability falsifier. Status is fenced;
> engine/runtime legs that cannot execute in this environment are recorded UNVERIFIED, not
> claimed green.

```text
wasm4games                = VERIFIED_UNDER_SCOPE
portability (Rust native) = VERIFIED  — golden corpus oracle pinned
portability (C ABI)       = VERIFIED  — cc-linked staticlib reproduces the golden, executed
portability (wasm32)      = UNVERIFIED — target not installed; no wasm runtime in this env
portability (engines/Lua) = NOT_STARTED
bcinr workspace crown     = BLOCKED_BY_PRE_EXISTING_CONTRACT_GATE_RESIDUAL  (unchanged)
```

- **Date:** 2026-06-19 · **Branch:** `claude/determined-hypatia-t8ajak`

## The oracle

`crates/wasm4games/src/corpus.rs` folds every pattern's **kernel output** together with its
**IR + evidence shape** (id, event code, OTEL span, object codes, admission rule) over a
fixed probe set into one rolling FNV receipt. This binds "same output + same OCEL links +
same OTEL span + same receipt" into a single comparable number, frozen as the oracle:

```text
GOLDEN_CORPUS_DIGEST is pinned in crates/wasm4games/src/corpus.rs.
```

`GOLDEN_CORPUS_DIGEST` is pinned in `crates/wasm4games/src/corpus.rs` (re-pinned whenever the
registry changes); the C-ABI leg asserts the C build reproduces whatever value `corpus.rs`
pins, rather than hardcoding it. The registry currently holds **75 patterns** (70 prior + 5
anti-cheat); the digest is re-pinned in `corpus.rs` on every registry change, so this receipt
deliberately does not transcribe its literal value.

Any drift in a kernel, the registry, or the evidence wiring changes the digest (regression
lock), and every other projection target must reproduce it to claim portability.

## Verified legs

| Target | Mechanism | Result |
|---|---|---|
| Rust native | `cargo test -p wasm4games --features std corpus` | `corpus_digest == GOLDEN` ✅ VERIFIED |
| C ABI (executed) | `crates/wasm4games-capi` staticlib + `harness.c` linked with `cc` | digest reproduced ✅ VERIFIED |

C-ABI proof output (`bash crates/wasm4games-capi/portability_proof.sh`). The harness reads the
count and both digests dynamically and passes when the C-ABI digest equals the native golden,
so the literal values below are illustrative of the **shape** only — the live count and digest
track whatever `corpus.rs` pins:

```text
pattern_count = <N>                  # whatever PATTERN_REGISTRY.len() reports (currently 75)
corpus_digest = 0x................ (C-ABI execution)
golden_digest = 0x................ (native Rust oracle)   # equals corpus_digest on pass
damage_applied(100,7) = 93
PORTABILITY_OK: C-ABI execution reproduces the native golden digest
```

This is a genuine cross-language receipt: **one ggen-declared pattern law → Rust rlib (tests)
and a C-linked staticlib → byte-identical results.** The harness compares the C-ABI digest
against the exported native golden (lib-vs-lib), so it stays valid as the registry grows and
never needs editing when a pattern is added. `wasm4games-capi` is offline-pure (only depends
on the `no_std` `wasm4games` core), so it is a safe workspace member and builds in CI.

## Fenced legs (cannot execute here)

Environment probe: `cc`/`gcc`/`clang` present, `node` present; **`wasm32-unknown-unknown`
target NOT installed**; `wasmtime`/`wasmer` ABSENT.

- **wasm32:** `UNVERIFIED`. The crate is `no_std` + dependency-free and is expected to build
  to `wasm32-unknown-unknown` (`cargo make wasm-check`), but the target is not installed and
  no wasm runtime is available to *execute* and compare the digest. Do not claim it green.
- **Engine adapters (UE5/Unity/Godot/Bevy) and Lua/Luau:** `NOT_STARTED`. These need their
  respective toolchains/engines and belong to the platform-projection roadmap (`ROADMAP.md`).

## Falsifier status

The user's crown benchmark is `<all registry patterns> × {Rust, WASM, C, …} × engines → same
I/O/refusal/OCEL/OTel/receipt/replay` (the registry currently holds 75 patterns). Discharged
so far: **2 targets (Rust native, C), fully executed and matching** over the whole registry via
the single corpus digest. Remaining (WASM execution, engine adapters, Lua) require an
environment with those runtimes; they are recorded here as UNVERIFIED / NOT_STARTED, not
laundered into a pass.

## Reproduce

```bash
cargo test -p wasm4games --features std corpus      # native oracle
bash crates/wasm4games-capi/portability_proof.sh    # C-ABI cross-language execution
```
