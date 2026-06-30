//! bcinr-wasm — WebAssembly bindings for bcinr via wasm-bindgen.
//!
//! Exposes core bcinr primitives (POWL workflow compilation, admission context
//! routing, UTF-8 codepoint counting, branchless mask operations) to JavaScript
//! and any WASM runtime.
//!
//! # Build
//!
//! ```text
//! wasm-pack build crates/bcinr-wasm --target web
//! # Output: crates/bcinr-wasm/pkg/
//! ```
//!
//! # Usage (JavaScript)
//!
//! ```javascript
//! import init, { compile_sequence, admit_context, count_codepoints, mask_and }
//!   from './bcinr_wasm.js';
//!
//! await init();
//! const result = JSON.parse(compile_sequence("A,B,C"));
//! console.log(result.ok, result.op_count);
//! console.log(count_codepoints("héllo"));  // 5
//! console.log(mask_and(0xFFn, 0x0Fn));    // 15n
//! ```

use wasm_bindgen::prelude::*;

// ── compile_sequence ─────────────────────────────────────────────────────────

/// Compile a comma-separated list of activity labels into a POWL sequence tape.
///
/// Returns a JSON string: `{"ok":true,"op_count":3,"topology":"Sequence"}`.
#[wasm_bindgen]
pub fn compile_sequence(labels: &str) -> String {
    use bcinr_powl::compiler::{compile_powl, CompileError, PowlAstNode};

    let atoms: Vec<PowlAstNode<'_>> = labels
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(PowlAstNode::Atom)
        .collect();

    if atoms.is_empty() {
        return r#"{"ok":false,"op_count":0,"topology":"empty_labels","error":"labels must be non-empty"}"#
            .to_string();
    }

    let node = PowlAstNode::Sequence(atoms);
    match compile_powl(&node) {
        Ok(tape) => {
            format!(
                r#"{{"ok":true,"op_count":{},"topology":"Sequence"}}"#,
                tape.len
            )
        }
        Err(e) => {
            let reason = match e {
                CompileError::TapeFull => "tape_full",
                CompileError::EmptySequence => "empty_sequence",
                CompileError::EmptyChoice => "empty_choice",
                CompileError::EmptyPartialOrder => "empty_partial_order",
                CompileError::Cycle => "cycle",
                CompileError::InvalidEdge { .. } => "invalid_edge",
            };
            format!(r#"{{"ok":false,"op_count":0,"topology":"{}"}}"#, reason)
        }
    }
}

// ── admit_context ─────────────────────────────────────────────────────────────

/// Determine the process topology for a POWL admission context.
///
/// Returns a JSON string: `{"topology":"Priority","ctx_bits":4369}`.
///
/// Topology values: `"Priority"`, `"Standard"`, `"Background"`, `"Quarantine"`.
#[wasm_bindgen]
pub fn admit_context(
    tenant_class: u32,
    urgency_tier: u32,
    resource_load: u32,
    has_sla_token: bool,
) -> String {
    use bcinr_powl::admit::{admit, ProcessTopology};

    let ctx: u64 = ((tenant_class & 0xF) as u64)
        | (((urgency_tier & 0xF) as u64) << 4)
        | (((resource_load & 0xF) as u64) << 8)
        | ((has_sla_token as u64) << 12);

    let topology = admit(ctx);
    let topo_str = match topology {
        ProcessTopology::Priority => "Priority",
        ProcessTopology::Standard => "Standard",
        ProcessTopology::Background => "Background",
        ProcessTopology::Quarantine => "Quarantine",
    };

    format!(
        r#"{{"topology":"{}","ctx_bits":{}}}"#,
        topo_str, ctx
    )
}

// ── count_codepoints ─────────────────────────────────────────────────────────

/// Count the number of Unicode codepoints in a UTF-8 string.
///
/// Uses the branchless bcinr-logic implementation.
#[wasm_bindgen]
pub fn count_codepoints(data: &str) -> u32 {
    bcinr_logic::utf8::count_codepoints(data.as_bytes()) as u32
}

// ── mask operations ───────────────────────────────────────────────────────────

/// Branchless bitwise AND of two u64 values.
#[wasm_bindgen]
pub fn mask_and(a: u64, b: u64) -> u64 {
    a & b
}

/// Branchless bitwise OR of two u64 values.
#[wasm_bindgen]
pub fn mask_or(a: u64, b: u64) -> u64 {
    a | b
}

/// Branchless bitwise XOR of two u64 values.
#[wasm_bindgen]
pub fn mask_xor(a: u64, b: u64) -> u64 {
    a ^ b
}

/// Count set bits (population count) of a u64 value.
#[wasm_bindgen]
pub fn popcount(value: u64) -> u32 {
    value.count_ones()
}

/// Leading zeros count of a u64 value.
#[wasm_bindgen]
pub fn leading_zeros(value: u64) -> u32 {
    value.leading_zeros()
}

/// Trailing zeros count of a u64 value.
#[wasm_bindgen]
pub fn trailing_zeros(value: u64) -> u32 {
    value.trailing_zeros()
}
