#![allow(unsafe_code)]
//! Fail-closed C/WASM boundary for admitted BCINR planning operations.

use std::ffi::{c_char, CString};

use bcinr_pddl::{
    admit_planning_task, domain31_from_pddl, domain_from_pddl, problem31_from_pddl,
    problem_from_pddl, DefaultCapabilityProfile, GroundProblem,
};
use serde::{Deserialize, Serialize};

const VERSION: u32 = 1;
const PDDL_RECEIPT_DOMAIN: &[u8] = b"bcinr.ffi.pddl-execution-receipt.v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PddlExecutionRequest {
    pub version: u32,
    pub domain_text: String,
    pub problem_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PddlExecutionResponse {
    pub version: u32,
    pub status: String,
    pub plan_or_refusal: String,
    pub receipt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowlExecutionRequest {
    pub version: u32,
    pub tape_json: String,
    pub context_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowlExecutionResponse {
    pub version: u32,
    pub status: String,
    pub result: String,
    pub ocel_log_json: String,
    pub receipt: String,
}

fn require_version(version: u32) -> Result<(), String> {
    if version == VERSION {
        Ok(())
    } else {
        Err(format!("unsupported version: {version} (expected {VERSION})"))
    }
}

/// Parse, admit, ground, and plan. Unsupported PDDL semantics never reach the
/// bounded planner.
pub fn pddl_execute_rust(req: &PddlExecutionRequest) -> Result<PddlExecutionResponse, String> {
    require_version(req.version)?;

    let domain31 = domain31_from_pddl(&req.domain_text)
        .map_err(|e| format!("domain admission parse error: {e}"))?;
    let problem31 = problem31_from_pddl(&req.problem_text)
        .map_err(|e| format!("problem admission parse error: {e}"))?;
    let admitted = admit_planning_task(&domain31, &problem31, &DefaultCapabilityProfile)
        .into_result()
        .map_err(|e| format!("planning admission refused: {e}"))?;

    let domain = domain_from_pddl(&req.domain_text)
        .map_err(|e| format!("domain execution parse error: {e}"))?;
    let problem = problem_from_pddl(&req.problem_text)
        .map_err(|e| format!("problem execution parse error: {e}"))?;
    let ground = GroundProblem::build(&domain, &problem, None)
        .map_err(|e| format!("grounding failed: {e}"))?;
    let tape = ground
        .find_plan()
        .into_result()
        .map_err(|e| format!("planning failed: {e}"))?;

    let plan = tape
        .ops
        .iter()
        .enumerate()
        .map(|(i, op)| format!("{i}: {}", op.label))
        .collect::<Vec<_>>()
        .join(", ");
    let receipt = receipt(
        PDDL_RECEIPT_DOMAIN,
        &[
            (b"domain-text", req.domain_text.as_bytes()),
            (b"problem-text", req.problem_text.as_bytes()),
            (b"admitted-theory", admitted.theory_digest.as_bytes()),
            (b"plan", plan.as_bytes()),
        ],
    );
    Ok(PddlExecutionResponse {
        version: VERSION,
        status: "ok".into(),
        plan_or_refusal: plan,
        receipt,
    })
}

/// Refuse the legacy JSON surface until it constructs the actual versioned
/// POWL tape, scheduler, OCEL trace, and replay receipt.
pub fn powl_execute_rust(req: &PowlExecutionRequest) -> Result<PowlExecutionResponse, String> {
    require_version(req.version)?;
    let _: serde_json::Value = serde_json::from_str(&req.tape_json)
        .map_err(|e| format!("tape JSON parse error: {e}"))?;
    let _: serde_json::Value = serde_json::from_str(&req.context_json)
        .map_err(|e| format!("context JSON parse error: {e}"))?;
    Err("unsupported_execution_rail: protocol v1 has no canonical POWL tape decoder; no workflow was executed and no receipt may be issued".into())
}

fn receipt(domain: &[u8], fields: &[(&[u8], &[u8])]) -> String {
    let mut h = blake3::Hasher::new();
    frame(&mut h, domain);
    for (label, value) in fields {
        frame(&mut h, label);
        frame(&mut h, value);
    }
    h.finalize().to_hex().to_string()
}

fn frame(h: &mut blake3::Hasher, bytes: &[u8]) {
    h.update(&(bytes.len() as u64).to_le_bytes());
    h.update(bytes);
}

fn pddl_error(message: impl Into<String>) -> PddlExecutionResponse {
    PddlExecutionResponse {
        version: VERSION,
        status: "error".into(),
        plan_or_refusal: message.into(),
        receipt: String::new(),
    }
}

fn powl_error(message: impl Into<String>) -> PowlExecutionResponse {
    PowlExecutionResponse {
        version: VERSION,
        status: "error".into(),
        result: message.into(),
        ocel_log_json: String::new(),
        receipt: String::new(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn pdl_execute(ptr: *const u8, len: usize) -> *const u8 {
    if ptr.is_null() || len == 0 {
        return c_string(&serde_json::to_string(&pddl_error("null request")).unwrap_or_default());
    }
    let bytes = unsafe { core::slice::from_raw_parts(ptr, len) };
    let response = match core::str::from_utf8(bytes) {
        Err(_) => pddl_error("invalid UTF-8"),
        Ok(text) => match serde_json::from_str::<PddlExecutionRequest>(text) {
            Err(e) => pddl_error(format!("JSON parse: {e}")),
            Ok(request) => pddl_execute_rust(&request).unwrap_or_else(pddl_error),
        },
    };
    c_string(&serde_json::to_string(&response).unwrap_or_default())
}

#[no_mangle]
pub unsafe extern "C" fn powl_execute(ptr: *const u8, len: usize) -> *const u8 {
    if ptr.is_null() || len == 0 {
        return c_string(&serde_json::to_string(&powl_error("null request")).unwrap_or_default());
    }
    let bytes = unsafe { core::slice::from_raw_parts(ptr, len) };
    let response = match core::str::from_utf8(bytes) {
        Err(_) => powl_error("invalid UTF-8"),
        Ok(text) => match serde_json::from_str::<PowlExecutionRequest>(text) {
            Err(e) => powl_error(format!("JSON parse: {e}")),
            Ok(request) => powl_execute_rust(&request).unwrap_or_else(powl_error),
        },
    };
    c_string(&serde_json::to_string(&response).unwrap_or_default())
}

#[no_mangle]
pub unsafe extern "C" fn free_c_string(ptr: *mut u8) {
    if !ptr.is_null() {
        drop(unsafe { CString::from_raw(ptr.cast::<c_char>()) });
    }
}

fn c_string(value: &str) -> *const u8 {
    CString::new(value.replace('\0', "\\u0000"))
        .expect("sanitized string")
        .into_raw()
        .cast::<u8>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn framing_is_unambiguous() {
        assert_ne!(
            receipt(b"test", &[(b"a", b"bc")]),
            receipt(b"test", &[(b"ab", b"c")])
        );
    }

    #[test]
    fn placeholder_powl_refuses() {
        let request = PowlExecutionRequest {
            version: VERSION,
            tape_json: "[]".into(),
            context_json: "{}".into(),
        };
        assert!(powl_execute_rust(&request)
            .unwrap_err()
            .starts_with("unsupported_execution_rail:"));
    }
}
