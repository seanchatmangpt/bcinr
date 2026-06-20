# Tutorial: Add a Pattern to wasm4games (the ggen way)

> Tutorial — learn by doing. By the end you will have added one new pattern,
> `DamageApplied`, to `crates/wasm4games` **without hand-writing a single kernel**. If you
> have not yet read why that matters, read the
> [GGEN-Only User Surface covenant](../explanation/ggen-only-user-surface.md) first.

## What you will build

A `DamageApplied` pattern: when a combat hit lands, it applies damage to an entity's
health byte class using *saturating* arithmetic (health never wraps below zero), and emits
an object-centric event linking the attacker and the target.

The important thing about this tutorial is what you will **not** do: you will not open
`src/patterns/` and type a function. You will *declare* the pattern in RDF and let `ggen`
generate the kernel for you. That is the entire point of the foundry.

## Prerequisites

- The `bcinr` workspace checked out, with `crates/wasm4games` present.
- `ggen` installed from <https://github.com/seanchatmangpt/ggen> (the generator; only
  needed at development time, never to *build* the crate).
- A Rust toolchain (edition 2021, MSRV 1.70).

## Step 1 — Declare the pattern in the ontology

Open the ontology and add one `w4g:Pattern` individual. This is the *only* file you author
to define the behavior.

```bash
$EDITOR crates/wasm4games/ggen/schema/patterns.ttl
```

Append a new individual. Pick the next free `w4g:id` (here `0x000A`/`10`; choose whatever
is unused in your tree) and describe the pattern as data:

```turtle
@prefix w4g: <https://bcinr.dev/wasm4games#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

w4g:DamageApplied a w4g:Pattern ;
    w4g:id            10 ;                       # stable PatternId; drives ORDER BY ?id
    w4g:name          "damage_applied" ;         # snake_case kernel fn + file stem
    w4g:event         "DamageApplied" ;          # PascalCase OCEL activity name
    w4g:eventCode     "0x000A"^^xsd:hexBinary ;  # EventKind::code
    w4g:lowering      w4g:Saturating ;           # LoweringKind::Saturating
    w4g:primitive     "bcinr_logic::int::sub_sat_u8" ;  # the substrate it lowers onto
    w4g:stateCard     256 ;                       # health byte class cardinality
    w4g:otelSpan      "0x000A"^^xsd:hexBinary ;  # PatternSpec::otel_span
    # Object-centricity: every event links to the objects it touches.
    w4g:touchesObject [ w4g:objectCode 1 ; w4g:objectName "attacker" ] ;
    w4g:touchesObject [ w4g:objectCode 2 ; w4g:objectName "target" ] ;
    # Admissibility contract (mirrors wasm4pm-compat semantics).
    w4g:requiredStatus 4 ;   # status::ADMITTED — must be admitted to count
    w4g:refusalStatus  7 .   # status::REFUSED — emitted when the rule is violated
```

A few notes that connect this back to the real API:

- `w4g:lowering w4g:Saturating` selects `ir::LoweringKind::Saturating`, which tells `ggen`
  to use the saturating-arithmetic kernel template — so `DamageApplied` lowers onto
  `bcinr_logic`'s saturating ops rather than an `if health < 0` branch.
- `w4g:requiredStatus`/`w4g:refusalStatus` become the kernel's `ir::AdmissionRule`. The
  `7` (`status::REFUSED`) is what `verify::negative_fixture` will stamp onto a deliberately
  bad event so a real authority can be tested for refusal.
- The two `w4g:touchesObject` blobs become the `ir::ObjectKind` slice on the spec, which is
  what makes the emitted `OcelEvent` object-centric: it links to *attacker* and *target*,
  not to nothing.

## Step 2 — Generate the kernel

Run the generator. It reads `ggen.toml`, runs the SPARQL queries (each ending in
`ORDER BY ?id` for deterministic emission), renders the Tera templates, and writes the
committed `.rs` files under `src/patterns/`.

```bash
cd crates/wasm4games
ggen sync
```

This is the moment the kernel *comes into existence*. You did not write it; `ggen`
manufactured it from your declaration. Expect `git status` to show a new
`src/patterns/damage_applied.rs` and a modified `src/patterns/mod.rs` (the
`PATTERN_REGISTRY` grew by one entry).

## Step 3 — Format the generated code

Template output is correct Rust but not guaranteed `rustfmt`-clean, and the workspace
enforces formatting. Run the single required post-sync step:

```bash
cargo fmt -p wasm4games
```

That is the whole ritual. There is no further hand-finishing — if you find yourself
wanting to *edit* `damage_applied.rs`, stop and change the TTL or the template instead
(see the [covenant](../explanation/ggen-only-user-surface.md)).

## Step 4 — Verify

Build and run the crate's tests with the full feature set. The `verify` self-checks iterate
`PATTERN_REGISTRY`, so your new pattern is exercised automatically: its status codes are
bounds-checked, and its negative fixture is confirmed to carry the refusal status and link
to the activity.

```bash
cargo test -p wasm4games --features std
```

You should see the registry self-consistency test pass with your new pattern included. If
it fails, the fix is almost always in the **ontology** (a bad `w4g:id`, an out-of-range
status, a missing object) or the **template** — never a hand-edit of the kernel.

## Step 5 — Inspect what ggen produced

Now read the generated artifacts to confirm they say what you declared. Open the kernel:

```bash
$EDITOR crates/wasm4games/src/patterns/damage_applied.rs
```

You will find the generated banner at the top:

```rust
// GENERATED BY ggen — DO NOT EDIT
// Source: crates/wasm4games/ggen/schema/patterns.ttl
// Regenerate with: ggen sync   (then: cargo fmt -p wasm4games)
```

…followed by a branchless `damage_applied(...)` kernel that lowers onto the saturating
primitive you named, and emits an `OcelEvent` linked to `attacker` and `target`. Then open
`src/patterns/mod.rs` and find the new `PATTERN_REGISTRY` entry, which will look like:

```rust
// GENERATED BY ggen — DO NOT EDIT
PatternSpec {
    id: PatternId(10),
    name: "damage_applied",
    lowering: LoweringKind::Saturating,
    state_card: 255,                 // clamped into [0, card)
    event: EventKind { code: 0x000A, name: "DamageApplied" },
    objects: &[
        ObjectKind { code: 1, name: "attacker" },
        ObjectKind { code: 2, name: "target" },
    ],
    admission: AdmissionRule { required_status: 4, refusal_status: 7 },
    otel_span: 0x000A,
},
```

Notice that **every field traces straight back to a triple in `patterns.ttl`**. The
registry is a faithful projection of the ontology — which is exactly the property the
covenant exists to protect.

## What you learned

- Patterns are *declared*, not coded: one `w4g:Pattern` individual in `patterns.ttl`.
- `ggen sync` manufactures the branchless kernel and registers it; you never open
  `src/patterns/*` to write logic.
- The single post-sync step is `cargo fmt -p wasm4games`; verification is just the normal
  test run, because `verify` iterates the generated registry.
- The generated banner and the id-sorted registry are the contract that keeps the code an
  honest projection of the ontology.

## Next steps

- Browse the full catalog in the
  [pattern reference](../reference/wasm4games-patterns.md).
- When you are ready for a *real* admissibility verdict (not the offline proxy), follow
  [Run real wasm4pm admission](../how-to/wasm4games-run-wasm4pm-admission.md).
