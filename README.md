# unibit POC

`unibit` is a private Rust proof-of-concept for a pinned, bit-native execution substrate.

The POC has one job:

> Prove that a nightly Rust program can allocate, pin, validate, lock, and execute over a fixed L1 region while preserving a strict hot-path contract.

No ontology generation yet.  
No public API polish yet.  
No process intelligence yet.  
No `unios` yet.  
No `dteam` yet.

First: build the machine-shaped fact.

---

## Status

Current phase:

```text
Phase 0: nightly smoke

Target green bar:

nightly compiles
generic const arithmetic compiles
L1 region validates
truth region position validates
scratch region position validates
pinning validates
OS lock attempt exists
inline assembly smoke passes
assembly emission works
lexicon check passes
```

## Core law

The POC is built around one resident L1 pair:

```text
truth  = current operational state
scratch = bounded motion workspace
```

The active truth region is:

```text
U_{1,262144}
262,144 bits
[u64; 4096]
```

The scratch region is the same shape:

```text
U_{1,262144}
262,144 bits
[u64; 4096]
```

The contract:

```text
truth stays resident
motion happens in scratch
```

Hot execution receives:

```text
pinned truth
pinned scratch
compiled masks
raw kernel
```

Hot execution does not receive:

```text
planner
policy engine
allocator
parser
graph walker
dashboard state
process model interpreter
logging framework
```

## Work and memory

unibit separates kinetic work from resident memory.

```text
work   = 8^n
memory = 64^n
```

Work tiers:

| Tier | Bits | Role | Target |
|---|---|---|---|
| `8^1` | 8 | local flag atom | 2 ns |
| `8^2` | 64 | word atom | 10 ns |
| `8^3` | 512 | semantic cache line | 100 ns |
| `8^4` | 4,096 | attention block | 200 ns |
| `8^5` | 32,768 | active tile | 500 ns |

Memory tiers:

| Tier | Bits | Role |
|---|---|---|
| `64^1` | 64 | word memory |
| `64^2` | 4,096 | attention memory |
| `64^3` | 262,144 | active universe |
| `64^4` | 16,777,216 | meaning field |

Rule:

> the planner chooses the smallest `8^n` work tier inside the required `64^n` memory tier

## Lexicon law

Documentation and abstractions must remain bit-native.

Allowed forms:

- `8^n`
- `64^n`
- `U_{1,n}`
- exact bit counts

Examples:

```text
U_{1,64}
U_{1,512}
U_{1,4096}
U_{1,32768}
U_{1,262144}
U_{1,16777216}
```

The forbidden ordinary storage noun is not allowed anywhere in source, docs, comments, tests, scripts, or generated output.

The lexicon checker fails the build if it appears.

```bash
node bin/check-lexicon.mjs
```

This is intentional.

If a code agent writes that word, it has likely left the unibit ontology and started using the wrong abstraction.

## Quickstart

Install nightly:

```bash
rustup toolchain install nightly
```

Run the smoke:

```bash
cargo +nightly run
```

Expected shape:

```text
nightly hello world passed
generic_const_exprs passed
pinned L1 position validated
inline asm smoke passed
base    = 0x...
truth   = 0x... offset=0
scratch = 0x... offset=32768
```
