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
GOLDEN_CORPUS_DIGEST = 0x436B_6BFF_B836_DBAF   (20 patterns × 4 probes)
```

Any drift in a kernel, the registry, or the evidence wiring changes it (regression lock),
and every other projection target must reproduce it to claim portability.

## Verified legs

| Target | Mechanism | Result |
|---|---|---|
| Rust native | `cargo test -p wasm4games --features std corpus` | `corpus_digest == GOLDEN` ✅ VERIFIED |
| C ABI (executed) | `crates/wasm4games-capi` staticlib + `harness.c` linked with `cc` | digest reproduced ✅ VERIFIED |

C-ABI proof output (`bash crates/wasm4games-capi/portability_proof.sh`):

```text
pattern_count = 20
corpus_digest = 0x436B6BFFB836DBAF (C-ABI execution)
golden_digest = 0x436B6BFFB836DBAF (native Rust oracle)
damage_applied(100,7) = 93
PORTABILITY_OK: C-ABI execution reproduces the native golden digest
```

This is a genuine cross-language receipt: **one ggen-declared pattern law → Rust rlib (tests)
and a C-linked staticlib → byte-identical results.** `wasm4games-capi` is offline-pure (only
depends on the `no_std` `wasm4games` core), so it is a safe workspace member and builds in CI.

## Fenced legs (cannot execute here)

Environment probe: `cc`/`gcc`/`clang` present, `node` present; **`wasm32-unknown-unknown`
target NOT installed**; `wasmtime`/`wasmer` ABSENT.

- **wasm32:** `UNVERIFIED`. The crate is `no_std` + dependency-free and is expected to build
  to `wasm32-unknown-unknown` (`cargo make wasm-check`), but the target is not installed and
  no wasm runtime is available to *execute* and compare the digest. Do not claim it green.
- **Engine adapters (UE5/Unity/Godot/Bevy) and Lua/Luau:** `NOT_STARTED`. These need their
  respective toolchains/engines and belong to the platform-projection roadmap (`ROADMAP.md`).

## Falsifier status

The user's crown benchmark is `20 patterns × {Rust, WASM, C, …} × engines → same
I/O/refusal/OCEL/OTel/receipt/replay`. Discharged so far: **2 targets (Rust native, C),
fully executed and matching.** Remaining (WASM execution, engine adapters, Lua) require an
environment with those runtimes; they are recorded here as UNVERIFIED / NOT_STARTED, not
laundered into a pass.

## Reproduce

```bash
cargo test -p wasm4games --features std corpus      # native oracle
bash crates/wasm4games-capi/portability_proof.sh    # C-ABI cross-language execution
```
