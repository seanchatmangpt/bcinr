# ROADMAP — wasm4games

> **Status: future guidance, not a commitment and not implemented.** This file records
> direction so it isn't lost. What actually exists today is listed under "Current state";
> everything below "Vision" is aspirational and fenced.

## Current state (admitted)

- `crates/wasm4games` — `no_std`, offline-pure, branchless game-pattern foundry on
  `bcinr-logic`. 20 generated kernels across four families, Pattern IR, byte-class status
  vocabulary, evidence layer (OCEL events, OTEL span codes, FNV receipt chains, replay),
  in-crate `verify` self-checks, dependency-free `compat` mirror of `wasm4pm-compat` shapes.
- `crates/wasm4games/ggen/` — RDF + SPARQL + Tera input surface (`ggen sync`) that
  regenerates the committed kernels. Docs are ggen-first (the GGEN-ONLY covenant).
- `crates/wasm4games-wasm4pm` — workspace-excluded, online-only admission bridge to the
  external `wasm4pm` / `wasm4pm-compat` repos (referenced, not vendored).
- `crates/wasm4games-capi` — C-ABI staticlib; the cross-language **portability spine** is
  verified (Rust native + C reproduce one golden corpus digest). See
  `proof/wasm4games/portability_spine.md`.
- **Law-state:** `wasm4games = VERIFIED_UNDER_SCOPE`;
  `bcinr workspace crown = BLOCKED_BY_PRE_EXISTING_CONTRACT_GATE_RESIDUAL` (17 files, see
  `proof/wasm4games/residual_contract_gate.md`).

## Near-term checkpoints (engineering, in scope)

1. **GATE-W4G-001 — ggen byte-stable reproduction.** Wire `ggen sync` + `git diff
   --exit-code` as a gate so committed kernels are *receipted* as reproducible, not just
   asserted. Currently UNVERIFIED here (`ggen` not installed).
2. **Portability legs — wasm32 + a second engine.** Execute the corpus digest under a wasm
   runtime (target + `wasmtime`/`node`), then one engine adapter (Bevy or Godot), proving
   `same input → same output → same OCEL/OTEL/receipt` across ≥3 targets.
3. **Residual-clear (separate).** Make the 17 contract-gate-flagged `bcinr-logic`
   algorithms genuinely branchless, or justify each gate. Distinct from the wasm4games work.

## Vision — "project the canon, don't port the game"

The product is not one game ported to many engines. It is **one canon / law stack projected
into platform-native games**, each exploring the part of the world that platform is best at,
all emitting compatible evidence/receipts into the same admitted universe.

> Different games, same admitted universe.
> Roblox trains pilots, Minecraft manufactures parts, UE5 stages the war, browser/WASM holds
> the proofs, and wasm4pm preserves the receipts.

### Platform grammar (illustrative — e.g. a "Gundam Nexus" canon)

| Platform | Natural form | Why |
|---|---|---|
| Roblox | tactics / squad / arena / pilot academy | social, fast sessions, identity |
| Minecraft | part manufacture / logistics / foundry | crafting, process, supply chains |
| UE5 | premium cinematic mech combat / cockpit | fidelity, animation, spectacle |
| Browser/WASM | instant demo / verifier HUD / receipt viewer | distribution, proof, shareability |
| Godot/Bevy | open technical prototype / systems lab | fast iteration, Rust/open arch |
| Mobile | companion: pilot record / build planner | retention, identity, async play |

### Shared canon objects (small, strict)

`Pilot, Frame, Part, Weapon, Material, Factory, Mission, Battle, Squad, Replay, Achievement,
ReceiptChain` — each platform specializes the same object (e.g. `Part` = equipment modifier
in Roblox tactics, a manufactured object in Minecraft, a visual+combat subsystem in UE5),
and cross-platform continuity flows through `ReceiptChain`.

### ggen platform targets (one canon, many projections)

```text
ggen target roblox_tactics     → Luau modules, ability/turn tables, compact evidence export
ggen target minecraft_foundry  → Fabric/Bedrock script, recipe/process graphs, part QA gates
ggen target ue5_frontline      → C ABI / plugin glue, Blueprint-callable projection + replay
ggen target browser_proof      → WASM module, TypeScript wrapper, receipt viewer
```

The existing `crates/wasm4games-capi` (C ABI) and the `ggen/` surface are the seed of these
targets; `wasm4pm` admits the evidence each platform emits.

### Cross-platform feedback loops (the "civilization" layer)

- Minecraft manufactures `Part` (passes a quality gate, emits receipt) → Roblox tactical unit
  equips it (movement range +1; match replay links the part receipt).
- Roblox verified maneuver → UE5 unlocks a cinematic sortie ("Doctrine: Encirclement admitted").
- UE5 physics feat (e.g. overheating under high-thrust) → Minecraft unlocks a heat-sink recipe.

### Server / enterprise layer (Java)

ggen can also generate the server **control plane**, not just kernels: Java virtual-thread
orchestration, platform gateways, session/match services, evidence/receipt services, and
wasm4pm admission packs — `wasm4games` owns the branchless authority, Java owns coordination.
JDK selection follows Chesterton's Fence: a **Java 21/25 LTS** baseline plus a separate
`java-server-27` future-conformance target justified by Java-27-only fences (chiefly **JEP 527
PQ-hybrid TLS** for evidence transport), not by virtual threads/FFM (which are older). Full
detail: [Server / Enterprise Architecture](docs/diataxis/explanation/wasm4games-server-architecture.md).

### Commercial posture

Fund the **canon/law stack**, not one game: it projects into whichever platform an audience
already uses (social players → Roblox; builders → Minecraft; premium → UE5; funders →
browser proof; educators → physics/manufacturing mode; AI researchers → agent benchmark
worlds).

## Doctrine

```text
Do not port the game. Project the canon.
Each platform gets the game it is naturally good at.
Each platform emits evidence back into the same admitted universe.
Engines project worlds; wasm4games operates patterns; wasm4pm admits evidence;
ggen manufactures the law.
```

## Exclusions (do not over-claim)

wasm4games does not replace engines; branchless is not always faster; WASM does not solve all
portability; receipts are not unforgeable; generated code is not automatically correct. The
admitted claim is only: *wasm4games can manufacture portable, branchless-first,
evidence-emitting game patterns that engines project and wasm4pm verifies.*
