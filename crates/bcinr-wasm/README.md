# bcinr-wasm

WebAssembly bindings for the bcinr library via [wasm-bindgen](https://rustwasm.github.io/wasm-bindgen/).

Exposes POWL workflow compilation, admission context routing, UTF-8 codepoint
counting, and branchless mask operations from the bcinr ecosystem to JavaScript
and any WASM runtime.

## Build

Requires [wasm-pack](https://rustwasm.github.io/wasm-pack/):

```bash
# Already installed: /Users/sac/.cargo/bin/wasm-pack

# Build for browser (ES module)
wasm-pack build crates/bcinr-wasm --target web

# Build for Node.js
wasm-pack build crates/bcinr-wasm --target nodejs

# Build for bundlers (webpack, rollup, etc.)
wasm-pack build crates/bcinr-wasm --target bundler
```

Output in `crates/bcinr-wasm/pkg/`.

## Usage (JavaScript / Browser)

```javascript
import init, {
    compile_sequence,
    admit_context,
    count_codepoints,
    mask_and,
    popcount
} from './bcinr_wasm.js';

await init();

// Compile a POWL sequence
const result = JSON.parse(compile_sequence("A,B,C"));
console.log(result);  // { ok: true, op_count: 3, topology: "Sequence" }

// Admission context routing
const ctx = JSON.parse(admit_context(3, 15, 0, true));
console.log(ctx.topology);  // "Priority"

// UTF-8 codepoint counting (branchless)
console.log(count_codepoints("héllo"));  // 5

// Branchless mask operations (BigInt for u64)
console.log(mask_and(0xFFn, 0x0Fn));   // 15n
console.log(popcount(0xFFn));           // 8
```

## Usage (Node.js)

```javascript
const bcinr = require('./pkg/bcinr_wasm.js');

const result = JSON.parse(bcinr.compile_sequence("A,B,C"));
console.log(result.op_count);  // 3
```

## API

All functions return JSON strings (for complex types) or primitive values.

| Function | Signature | Returns |
|----------|-----------|---------|
| `compile_sequence` | `(labels: string) -> string` | JSON: `{ok, op_count, topology}` |
| `admit_context` | `(tenant_class, urgency_tier, resource_load, has_sla: u32/bool) -> string` | JSON: `{topology, ctx_bits}` |
| `count_codepoints` | `(data: string) -> u32` | Codepoint count |
| `mask_and` | `(a: u64, b: u64) -> u64` | Bitwise AND |
| `mask_or` | `(a: u64, b: u64) -> u64` | Bitwise OR |
| `mask_xor` | `(a: u64, b: u64) -> u64` | Bitwise XOR |
| `popcount` | `(value: u64) -> u32` | Population count |
| `leading_zeros` | `(value: u64) -> u32` | Leading zero count |
| `trailing_zeros` | `(value: u64) -> u32` | Trailing zero count |

## Architecture

This crate is distinct from `playground/src/wasm.rs` (which uses raw C ABI / `no_std`).
`bcinr-wasm` uses `std` + `wasm-bindgen` for JS-friendly exports.

- **wasm-bindgen:** JS/WASM interop (strings, JSON, primitives)
- **Dependencies:** `bcinr-logic` (branchless algorithms), `bcinr-powl` (workflow runtime)
- **Target:** `wasm32-unknown-unknown`
