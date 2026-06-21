# wasm4games — Server / Enterprise Architecture (reference, not implemented)

> **Status: forward-guidance reference.** None of this is built in the repo today. The
> current code is the offline `wasm4games` foundry + the verified portability spine (see
> `ROADMAP.md` and `proof/wasm4games/`). This doc records the server-side / solution
> architecture so the direction is on record.

## The shift: ggen as a solution-architecture compiler

Earlier framing was `ggen → wasm4games kernels → engine projection`. The fuller picture is
that ggen can generate the **whole operating surface** from one pattern canon:

```text
ggen
 → client/game pattern code (Rust/WASM/C/Luau)
 → Java game-server code (handlers, protocols, records, sealed interfaces, validators)
 → platform adapters (Roblox/Minecraft/UE5/browser)
 → OCEL/OTel evidence schemas
 → replay/receipt services
 → wasm4pm admission + negative fixtures
 → analytics / NPS / DfLSS pipeline
```

## Layered split (control plane vs law vs projection)

```text
wasm4games        = deterministic pattern law (branchless hot path)
Java servers      = orchestration, sessions, platform coordination, persistence, live ops
wasm4pm           = admission / refusal / replay / conformance
engines/platforms = projection surfaces
ggen              = manufactures all of the above from one source of truth
```

## Java as the command/control plane

Fenced Java facts (as of **2026-06-19**, per OpenJDK JEPs): virtual threads finalized in
**JDK 21** (JEP 444); FFM API finalized in **JDK 22** (JEP 454); **Java 26** is current GA,
**Java 25** is the LTS, **Java 27** is near-future EA. Which JDK to target is governed by
**Chesterton's Fence** — see [Java version fence](#java-version-fence-chestertons-fence)
below: take the newer/riskier surface only when a version-specific capability is the point,
never merely because it is newer. Production baseline is **Java 21/25 LTS**; Java 27 is a
*separate* future-conformance target, not a default.

| Java capability | Use in the game control plane |
|---|---|
| Virtual threads (JEP 444) | one lightweight task per player/session/message; great for I/O-heavy flows |
| Structured concurrency | match ops, save pipelines, external calls as one scoped unit |
| Scoped values | request/session/player/evidence context without ThreadLocal sprawl |
| Records | immutable command/event DTOs |
| Sealed interfaces | closed protocol/message hierarchies |
| Pattern matching / switch | clean routing over generated event types |
| FFM API (JEP 454) | call native wasm4games / C ABI without JNI brittleness |
| Reflection | cold-path discovery, schema loading, generated-module registration |
| MethodHandles / VarHandles | faster generated dispatch than raw reflection on warm paths |

## Critical fence: the Java server is NOT the hot kernel

```text
Java:            session orchestration, routing, platform adapters, persistence, live ops,
                 authorization, social/referral/NPS, evidence shipping
wasm4games/C/WASM: branchless authority — damage/status/collision/tick, receipt append,
                 replay frame, object-event validation
```

Reflection belongs only in the **cold path** (boot-time module discovery, annotation
scanning, handler registration, schema validation, codec wiring, adapter loading, admin/test
tooling). It must never appear per-tick. Hot dispatch is **generated** (switch tables,
`MethodHandle`/`VarHandle`, `enum`/`u16` opcode tables, FFM downcall handles, byte-buffer
codecs).

```text
Reflection discovers the law.   Generated tables actuate the law.
```

## Solution architecture (text diagram)

```text
                         ggen Pattern Source (TTL/SPARQL/Tera/TOML)
                                        |
        +-------------------------------+-------------------------------+
        |                               |                               |
   wasm4games kernels          Java Game Server (gen surface)    Platform Output
   (Rust/WASM/C)               (virtual-thread orchestration)    (Roblox/MC/UE5/Browser)
        |                               |                               |
        +---------------+---------------+---------------+---------------+
                        |                               |
                 OCEL/OTel evidence              Replay / Receipts
                        |                               |
                        +---------------+---------------+
                                        |
                                     wasm4pm
                               (admission / refusal)
```

## Java server bounded contexts

1. **Platform Gateway** — converts Roblox/Minecraft/UE5/browser/mobile messages into
   canonical generated commands (`PatternCommand`, `OcelCandidateEvent`, `ReceiptCandidate`).
2. **Session / Match Service** — one virtual thread per connection/message; structured task
   scope per match op; scoped values for player/session/receipt context.
3. **Pattern Authority Service** — calls generated wasm4games kernels via FFM (native C ABI),
   a WASM sidecar, or a generated pure-Java fallback for non-crown paths.
4. **Manufacturing / Parts Service** (Minecraft foundry) — workflow-heavy, persistent,
   object-centric: `OreExtracted → MaterialRefined → PartManufactured → PartInspected →
   PartAdmitted → FrameAssembled`.
5. **Tactics / Match Service** (Roblox) — `TurnStarted → ActionSubmitted → AbilityAdmitted →
   DamageApplied → StatusEffectTicked → VictoryResolved → ReplayFrameRecorded`.
6. **Evidence Service** — appends OCEL events, OTel spans, receipt chains, replay frames,
   negative fixtures.
7. **Promotion / NPS Service** — generated from DfLSS CTQs: `MasteryMomentDetected →
   ShareArtifactGenerated → ReferralInviteCreated → NpsPromptGated → NpsScoreSubmitted`.

## Java version fence (Chesterton's Fence)

The standard mirrors `wasm4pm-compat` forcing nightly: *take a newer / runtime-risk surface
only when it gives a capability that is unavailable, materially different, or strategically
future-conformant.* Most Java capabilities above are therefore **not** Java-27 reasons.

**Not a Java 27 fence** (use because they are good; target the LTS): virtual threads (J21),
records (J16), sealed classes (J17), pattern matching for `switch` over reference types (J21),
FFM (J22), reflection, scoped values. None of these justify Java 27.

**Java-27-specific reasons** (dated 2026-06-19; fenced as noted):

| JEP | Capability | Relevance | Fence |
|---|---|---|---|
| 527 | Post-Quantum Hybrid Key Exchange for TLS 1.3 | PQ-ready transport by default for receipt/evidence/gateway channels | **Strong, completed** |
| 532 | Primitive types in patterns / `instanceof` / `switch` (5th preview) | cleaner generated routers over primitive event/status/opcode classes | useful, **preview** |
| 537/508 | Vector API (incubator) | Java-side SIMD for server *evidence scans* (not for replacing crown kernels) | useful, **incubator** |
| 531 | Lazy Constants (3rd preview) | lazy-but-foldable generated registries/tables/adapters | useful, **preview** |
| 538 | PEM Encodings of Cryptographic Objects (preview) | keys/certs/CRLs for receipt signing, TLS material, verifier fixtures | useful, **preview** |
| 534 | Compact Object Headers by default | server density for many small records/commands/events (96→64-bit headers) | runtime-density |
| 523 | G1 default in all environments | predictable GC across constrained deployments | deployment posture |

The decisive one is **JEP 527**: these servers are *evidence-transport nodes*, so PQ-hybrid
TLS by default (hybrid groups `X25519MLKEM768`, `SecP256r1MLKEM768`, `SecP384r1MLKEM1024`,
with `X25519MLKEM768` first) is a real, completed Java-27 fence for the receipt / telemetry /
replay / gateway transport surface. Preview/incubator items stay behind the `java-server-27`
build profile only.

> Rule: generate Java 21/25-compatible code by default; emit the Java-27 target only where the
> fence above is the point — not because virtual threads, records, sealed types, or FFM exist
> (they are all older).

### JEP 527 as a platform moat (fenced)

JEP 527 adds hybrid post-quantum key exchange to TLS 1.3 (a PQ algorithm combined with a
traditional one); apps using `javax.net.ssl` benefit **by default, without code changes**. For
a normal game server that is a security upgrade. For this architecture it is a category move:
the **transport** becomes future-conformant at the same moment the **gameplay** layer becomes
generated, receipted, replayable, and process-admitted.

Why it matters for *games* specifically — **harvest-now, decrypt-later**. The durable value
here is not only accounts / payments / match state; it is the player's authored history: feats,
receipts, replay proofs, manufactured parts, pilot record, cross-platform progression, and the
social / promotion trail. If game evidence becomes long-lived identity / economy / community
data, protecting it with PQ-hybrid transport is foundational, not premature.

The moat is the *combination*, not the cryptography alone:

```text
Java 27 PQ-hybrid TLS (default transport)
+ generated server protocols + platform gateways (ggen)
+ branchless wasm4games kernels
+ OCEL/OTel evidence + receipt chains
+ wasm4pm admission
= an evidence-secure, post-quantum-transported game platform
```

**Fence (do not over-claim):** competitors *can* eventually flip on PQ TLS. The edge is not
"nobody can compete" — it is that PQ transport arrives *inside* a generated, evidence-native,
cross-platform game-law architecture from the start, and that retrofitting trust (TLS / cert /
key workflows, evidence transport, telemetry integrity, replay integrity, gateways, audit
story) onto years of engine-specific, telemetry-string systems is expensive. Designing around
trust from the beginning is the moat.

## ggen server-side target set (tiered)

```text
ggen target java-server-lts    → Java 21/25 baseline: records, sealed interfaces, virtual
                                 threads, FFM, structured concurrency, routers, codecs
ggen target java-server-27     → future-conformance: PQ-hybrid TLS (JEP 527),
                                 primitive-pattern switch (preview), Vector API (incubator),
                                 Lazy Constants (preview), PEM crypto (preview), compact
                                 object-header assumptions
ggen target java-server-ffm    → FFM downcalls to native wasm4games / C ABI (version-agnostic)
ggen target java-platform-roblox    → Roblox gateway DTOs + Luau protocol spec
ggen target java-platform-minecraft → Fabric/Bukkit/Bedrock command/event bridge
ggen target java-evidence      → OCEL/OTel/receipt Java models
ggen target java-verifier      → negative fixtures + replay validators
ggen target java-dflss-nps     → CTQ state machines + prompt gating
```

The existing `crates/wasm4games-capi` (C ABI staticlib, portability-verified) is the seed of
the `java-server-ffm` downcall target. JDK selection per the fence above.

## Generated Java shape (illustrative)

```java
public sealed interface GameCommand
    permits InputAdmittedCommand, DamageAppliedCommand,
            PartManufacturedCommand, NpsPromptGatedCommand {}

public record DamageAppliedCommand(
    long playerId, long weaponId, long targetId,
    int weaponClass, int armorClass, int receiptSequence) implements GameCommand {}

public final class GeneratedPatternRouter {
    public PatternResult route(GameCommand command) {
        return switch (command) {
            case DamageAppliedCommand c     -> damageApplied.handle(c);
            case PartManufacturedCommand c  -> partManufactured.handle(c);
            case NpsPromptGatedCommand c    -> npsPromptGated.handle(c);
            default                          -> PatternResult.refused(RefusalCode.UNKNOWN_COMMAND);
        };
    }
}
```

(Java 27 preview features such as primitive types in patterns/switch should be fenced behind
a build profile, not assumed.)

## Virtual-threads fit

```text
✅ WebSocket message handling, save/load, platform API calls, match orchestration,
   evidence shipping, NPS/referral/notification flows (all I/O-heavy)
❌ CPU-heavy collision/damage loops, massive tick simulation
   → use wasm4games / Rust / SIMD / native kernels (fixed worker/core pools or FFM)
```

Doctrine: *virtual threads for orchestration; branchless kernels for authority; structured
concurrency for bounded workflows; wasm4pm for standing.*

## The category line

> ggen generates the complete solution architecture — game clients, server protocols, Java
> virtual-thread orchestration, branchless authority kernels, platform adapters, evidence
> schemas, replay receipts, and wasm4pm admission packs — from one pattern canon.

## Exclusions

Reference only. This does not exist in the repo yet, makes no claim that the generators are
written, and does not assert any Java version beyond the dated facts above. When built, each
generated surface must carry the same fences as the Rust side (no hot-path reflection, no
laundered evidence, receipts are telemetry-grade unless signed).
