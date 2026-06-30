//! bcinr-py — PyO3 Python bindings for the bcinr library.
//!
//! Exposes core bcinr primitives (POWL workflow compilation, admission
//! context routing, UTF-8 codepoint counting, branchless mask operations)
//! to Python via maturin.
//!
//! # Build
//!
//! ```text
//! cargo install maturin
//! cd crates/bcinr-py && maturin develop
//! ```
//!
//! # Usage
//!
//! ```python
//! import bcinr
//!
//! result = bcinr.compile_sequence("A,B,C")
//! ctx    = bcinr.admit_context(tenant_class=2, urgency_tier=8,
//!                              resource_load=4, has_sla_token=True)
//! print(bcinr.count_codepoints("hello"))
//! print(bcinr.mask_and(0xFF, 0x0F))
//! ```

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

// ── compile_sequence ─────────────────────────────────────────────────────────

/// Compile a comma-separated list of activity labels into a POWL sequence tape.
///
/// Returns a dict with keys: ``ok`` (bool), ``op_count`` (int), ``topology`` (str).
#[pyfunction]
fn compile_sequence(py: Python<'_>, labels: String) -> PyResult<Py<PyDict>> {
    use bcinr_powl::compiler::{compile_powl, CompileError, PowlAstNode};

    let label_vec: Vec<String> = labels
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if label_vec.is_empty() {
        return Err(PyValueError::new_err("labels must be a non-empty comma-separated list"));
    }

    let owned_atoms: Vec<PowlAstNode<'_>> =
        label_vec.iter().map(|s| PowlAstNode::Atom(s.as_str())).collect();
    let node = PowlAstNode::Sequence(owned_atoms);
    let result = compile_powl(&node);

    let dict = PyDict::new_bound(py);
    match result {
        Ok(tape) => {
            dict.set_item("ok", true)?;
            dict.set_item("op_count", tape.len as i64)?;
            dict.set_item("topology", "Sequence")?;
        }
        Err(e) => {
            dict.set_item("ok", false)?;
            dict.set_item("op_count", 0i64)?;
            let reason = match e {
                CompileError::TapeFull => "tape_full",
                CompileError::EmptySequence => "empty_sequence",
                CompileError::EmptyChoice => "empty_choice",
                CompileError::EmptyPartialOrder => "empty_partial_order",
                CompileError::Cycle => "cycle",
                CompileError::InvalidEdge { .. } => "invalid_edge",
            };
            dict.set_item("topology", reason)?;
        }
    }
    Ok(dict.unbind())
}

// ── admit_context ─────────────────────────────────────────────────────────────

/// Determine the process topology for a POWL admission context.
///
/// Returns a dict with keys: ``topology`` (str), ``ctx_bits`` (int).
///
/// Topology values: ``"Priority"``, ``"Standard"``, ``"Background"``, ``"Quarantine"``.
#[pyfunction]
fn admit_context(
    py: Python<'_>,
    tenant_class: u64,
    urgency_tier: u64,
    resource_load: u64,
    has_sla_token: bool,
) -> PyResult<Py<PyDict>> {
    use bcinr_powl::admit::{admit, ProcessTopology};

    let ctx: u64 = (tenant_class & 0xF)
        | ((urgency_tier & 0xF) << 4)
        | ((resource_load & 0xF) << 8)
        | ((has_sla_token as u64) << 12);

    let topology = admit(ctx);
    let topo_str = match topology {
        ProcessTopology::Priority => "Priority",
        ProcessTopology::Standard => "Standard",
        ProcessTopology::Background => "Background",
        ProcessTopology::Quarantine => "Quarantine",
    };

    let dict = PyDict::new_bound(py);
    dict.set_item("topology", topo_str)?;
    dict.set_item("ctx_bits", ctx)?;
    Ok(dict.unbind())
}

// ── count_codepoints ─────────────────────────────────────────────────────────

/// Count the number of Unicode codepoints in a UTF-8 string.
#[pyfunction]
fn count_codepoints(data: &str) -> PyResult<usize> {
    Ok(bcinr_logic::utf8::count_codepoints(data.as_bytes()))
}

// ── mask operations ───────────────────────────────────────────────────────────

/// Branchless bitwise AND of two u64 values.
#[pyfunction]
fn mask_and(a: u64, b: u64) -> PyResult<u64> {
    Ok(a & b)
}

/// Branchless bitwise OR of two u64 values.
#[pyfunction]
fn mask_or(a: u64, b: u64) -> PyResult<u64> {
    Ok(a | b)
}

/// Branchless bitwise XOR of two u64 values.
#[pyfunction]
fn mask_xor(a: u64, b: u64) -> PyResult<u64> {
    Ok(a ^ b)
}

/// Count set bits (population count) of a u64 value.
#[pyfunction]
fn popcount(value: u64) -> PyResult<u32> {
    Ok(value.count_ones())
}

// ── module ────────────────────────────────────────────────────────────────────

/// bcinr Python bindings — branchless algorithms, POWL workflows, and admission control.
#[pymodule]
fn bcinr(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compile_sequence, m)?)?;
    m.add_function(wrap_pyfunction!(admit_context, m)?)?;
    m.add_function(wrap_pyfunction!(count_codepoints, m)?)?;
    m.add_function(wrap_pyfunction!(mask_and, m)?)?;
    m.add_function(wrap_pyfunction!(mask_or, m)?)?;
    m.add_function(wrap_pyfunction!(mask_xor, m)?)?;
    m.add_function(wrap_pyfunction!(popcount, m)?)?;
    Ok(())
}
