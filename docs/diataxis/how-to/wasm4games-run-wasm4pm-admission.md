# How-To: Run Real wasm4pm Admission

> How-to — a practical recipe for getting a *real* admissibility verdict on `wasm4games`
> evidence using the external **wasm4pm** authority. For why admission is external in the
> first place, see the [foundry overview](../explanation/wasm4games-overview.md#the-wasm4pm-relationship-referenced-not-vendored).

## Goal

Take candidate evidence emitted by `wasm4games` and have the *real* `wasm4pm` authority
**admit** the good evidence and **refuse** the bad. Specifically, you will confirm that
the negative fixtures produced by `verify::negative_fixture` are *refused* by a genuine
authority — not merely flagged by the in-crate offline proxy.

## Before you start: offline vs. real admission

`wasm4games` deliberately does **not** decide admissibility on its own:

- **Offline (what ships in the workspace):** the crate emits candidate evidence and
  self-checks it with the in-crate [`verify`] module. These checks are an *admissibility
  proxy*, not the authority — they confirm status codes are in-bounds, replays are
  deterministic, and they *construct* negative fixtures that a real authority ought to
  refuse. No network, no git dependency; this is what bcinr's offline `-D warnings` CI
  runs.
- **Real (what this recipe sets up):** the external `wasm4pm` engine renders an actual
  **admit / refuse** verdict. This lives behind the **workspace-excluded**
  `crates/wasm4games-wasm4pm` bridge crate, which depends on the real repositories and is
  built **online**.

> ⚠️ This recipe **requires network access and pinned git revisions.** It is intentionally
> outside the default workspace so the offline build never pulls these dependencies. Do not
> add the bridge crate to the workspace `members` list.

The two halves connect through `crates/wasm4games/src/compat.rs`, whose `EvidenceState`,
`Verdict`, and receipt mirror types are shaped 1:1 onto `wasm4pm-compat`, so the bridge can
map emitted evidence onto the real authority's types without translation drift.

## Prerequisites

- The `bcinr` workspace, with `crates/wasm4games` present.
- The excluded bridge crate `crates/wasm4games-wasm4pm` (it is *excluded* from the
  workspace, not absent — it has its own `Cargo.toml`).
- Network access and the ability to pin git dependencies.

## Step 1 — Pin the external authorities

The bridge depends on the two real repositories. **Pin them to exact revisions** so the
verdict is reproducible — never track a moving branch for an authority you rely on.

```toml
# crates/wasm4games-wasm4pm/Cargo.toml  (this crate is EXCLUDED from the workspace)
[package]
name = "wasm4games-wasm4pm"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
wasm4games = { path = "../wasm4games", features = ["alloc"] }

# Referenced, not vendored — pinned to exact revs (replace with the revs you vet).
wasm4pm = { git = "https://github.com/seanchatmangpt/wasm4pm", rev = "<PINNED_REV>" }
wasm4pm-compat = { git = "https://github.com/seanchatmangpt/wasm4pm-compat", rev = "<PINNED_REV>" }
```

If you prefer working from local clones, clone and check out the pinned revs, then point
the dependencies at the paths instead:

```bash
git clone https://github.com/seanchatmangpt/wasm4pm        ../wasm4pm
git clone https://github.com/seanchatmangpt/wasm4pm-compat ../wasm4pm-compat
git -C ../wasm4pm        checkout <PINNED_REV>
git -C ../wasm4pm-compat checkout <PINNED_REV>
```

## Step 2 — Build the bridge (online)

Build the excluded bridge crate directly (not through `cargo make`, which targets the
offline workspace). This step fetches the pinned git dependencies, so it needs network:

```bash
cargo build --manifest-path crates/wasm4games-wasm4pm/Cargo.toml
```

A successful build means the bridge has linked the *real* `wasm4pm` authority against the
`wasm4games` evidence types via the `compat` mirrors.

## Step 3 — Emit candidate evidence and submit an OcelLog

In the bridge, drive `wasm4games` to produce candidate evidence — an `OcelLog` of
`OcelEvent`s with their object links — then hand it to the real authority for a verdict.
The `alloc` feature gives you the growable `OcelLog`:

```rust
use wasm4games::prelude::*;
use wasm4games::evidence::ocel::{OcelEvent, OcelLog};

// 1. Emit candidate evidence offline (here, a single admitted event for illustration).
let mut log = OcelLog::new();
let mut ev = OcelEvent::new(0x000A, /* activity */ 10, /* tick */ 1, status::ADMITTED);
ev.objects.push(1, /* attacker id */ 42);
ev.objects.push(2, /* target id   */ 7);
log.push(ev);

// 2. Hand the log to the REAL authority via the bridge. `admit` is the bridge function
//    that maps `OcelLog` onto wasm4pm-compat types and returns a real verdict.
let verdict = wasm4games_wasm4pm::admit(&log);
assert!(matches!(verdict, wasm4games::compat::Verdict::Admitted));
```

The bridge converts each `OcelEvent` into the canonical `wasm4pm-compat` object-centric
form (its `objects` links survive the mapping, preserving object-centricity) and runs the
authority's conformance check.

## Step 4 — Confirm negative fixtures are REFUSED

This is the load-bearing check. The offline proxy only *constructs* a negative fixture; a
real authority must actually *refuse* it. For every pattern in the registry, build its
negative fixture (an event stamped with the pattern's `refusal_status`) and assert the
authority rejects it:

```rust
use wasm4games::patterns::PATTERN_REGISTRY;
use wasm4games::verify::negative_fixture;
use wasm4games::evidence::ocel::OcelLog;
use wasm4games::compat::Verdict;

for spec in PATTERN_REGISTRY {
    // A deliberately-bad event carrying this pattern's refusal status.
    let bad = negative_fixture(spec);

    let mut log = OcelLog::new();
    log.push(bad);

    // The REAL authority must refuse it. Offline `verify` cannot prove this;
    // only wasm4pm can render the actual refusal verdict.
    match wasm4games_wasm4pm::admit(&log) {
        Verdict::Refused(code) => assert_eq!(code, spec.admission.refusal_status),
        other => panic!(
            "expected refusal for pattern {} (id {}), got {:?}",
            spec.name, spec.id.0, other
        ),
    }
}
```

If every negative fixture comes back `Verdict::Refused` with the matching code, you have a
*real* admissibility result: the authority admits well-formed evidence and refuses the
fixtures the foundry flagged offline. Run it as a bridge test:

```bash
cargo test --manifest-path crates/wasm4games-wasm4pm/Cargo.toml
```

## Troubleshooting

- **It tries to build during offline CI.** Then the bridge leaked into the workspace.
  Confirm `crates/wasm4games-wasm4pm` is in the workspace `exclude` list (or simply not in
  `members`); the offline build must never see it.
- **Dependency fetch fails / no network.** This recipe genuinely needs network and pinned
  revs. There is no offline path to a *real* verdict — that is the design. Use the
  [`verify`] proxy for offline confidence instead.
- **A fixture is admitted instead of refused.** Either the pinned `wasm4pm` rev disagrees
  with the pattern's `AdmissionRule`, or the pattern's `refusalStatus` in `patterns.ttl` is
  wrong. Fix it in the **ontology** and regenerate (see the
  [add-a-pattern tutorial](../tutorials/wasm4games-add-a-pattern.md)) — do not patch the
  generated kernel.
- **Type mismatch in the bridge.** The `compat` mirrors drift from `wasm4pm-compat` only if
  the pinned rev changed shape. Re-pin to a vetted rev, or update the mirror types in
  `crates/wasm4games/src/compat.rs` to match (a substrate change, allowed by hand).

## Summary

`wasm4games` emits candidate, object-centric evidence and self-checks it offline; **real
admission and refusal come only from the external `wasm4pm`** via the workspace-excluded
`wasm4games-wasm4pm` bridge. Pin the authorities, build the bridge online, submit an
`OcelLog`, and prove that `verify::negative_fixture` events are genuinely refused.
