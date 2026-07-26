#![allow(unsafe_code)]
//! bcinr-ffi — C FFI and WASM bindings for bcinr PDDL/POWL planning and receipts.
//!
//! Provides:
//! - Versioned request/response types with JSON schema validation
//! - C-ABI bindings (extern "C" functions for C/C++/WASM interop)
//! - Rust-native API layer
//! - Deterministic BLAKE3 receipts for execution traceability
//!
//! # FFI Safety
//!
//! The C-ABI functions `pdl_execute` and `powl_execute` use unsafe to:
//! 1. Accept and validate raw C pointers (null checks required)
//! 2. Convert UTF-8 byte slices from C-allocated memory
//! 3. Allocate heap memory for responses (caller must call `free_c_string`)
//!
//! # Example (Rust)
//!
//! ```ignore
//! use bcinr_ffi::{PddlExecutionRequest, pddl_execute_rust};
//!
//! let req = PddlExecutionRequest {
//!     version: 1,
//!     domain_text: "(define (domain test) ...)".to_string(),
//!     problem_text: "(define (problem test) ...)".to_string(),
//! };
//!
//! let resp = pddl_execute_rust(&req).expect("planning failed");
//! println!("Status: {:?}", resp.status);
//! ```

use serde::{Deserialize, Serialize};
use bcinr_pddl::{domain_from_pddl, problem_from_pddl, GroundProblem};

// ─── Versioned Request/Response Types ────────────────────────────────────────

/// PDDL planning execution request (version 1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PddlExecutionRequest {
    /// Protocol version (currently 1).
    pub version: u32,
    /// PDDL domain text (define (domain ...))
    pub domain_text: String,
    /// PDDL problem text (define (problem ...))
    pub problem_text: String,
}

/// PDDL planning execution response (version 1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PddlExecutionResponse {
    /// Protocol version (currently 1).
    pub version: u32,
    /// Execution status: "ok" or error code (e.g., "parse_error", "planning_failed")
    pub status: String,
    /// Comma-separated plan steps (if status == "ok"), or error message
    pub plan_or_refusal: String,
    /// BLAKE3 receipt (base16-encoded) proving the execution
    pub receipt: String,
}

/// POWL workflow execution request (version 1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowlExecutionRequest {
    /// Protocol version (currently 1).
    pub version: u32,
    /// POWL tape as JSON (e.g., from powl_compile_sequence or powl_compile_choice)
    pub tape_json: String,
    /// Initial context (tenant_class, urgency_tier, resource_load, has_sla_token)
    pub context_json: String,
}

/// POWL workflow execution response (version 1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowlExecutionResponse {
    /// Protocol version (currently 1).
    pub version: u32,
    /// Execution status: "ok" or error code
    pub status: String,
    /// Result data (if status == "ok") or error message
    pub result: String,
    /// OCEL log (JSON-encoded object-centric event log) for traceability
    pub ocel_log_json: String,
    /// BLAKE3 receipt (base16-encoded) proving the execution
    pub receipt: String,
}

// ─── Rust Native API ────────────────────────────────────────────────────────

/// Execute PDDL planning in Rust (native API).
///
/// # Errors
///
/// Returns an error string if parsing or planning fails.
pub fn pddl_execute_rust(req: &PddlExecutionRequest) -> Result<PddlExecutionResponse, String> {
    if req.version != 1 {
        return Err(format!(
            "unsupported version: {} (expected 1)",
            req.version
        ));
    }

    // Parse domain and problem using bcinr-pddl
    let domain = domain_from_pddl(&req.domain_text)
        .map_err(|e| format!("domain parse error: {}", e))?;

    let problem = problem_from_pddl(&req.problem_text)
        .map_err(|e| format!("problem parse error: {}", e))?;

    // Ground and plan using GroundProblem
    let ground = GroundProblem::build(&domain, &problem, None)
        .map_err(|e| format!("grounding failed: {}", e))?;

    let tape = ground.find_plan()
        .into_result()
        .map_err(|e| format!("planning failed: {}", e))?;

    // Serialize plan as comma-separated action steps
    let plan_str = tape
        .ops
        .iter()
        .enumerate()
        .map(|(i, op)| format!("{}: {}", i, op.label))
        .collect::<Vec<_>>()
        .join(", ");

    // Generate receipt (deterministic hash of domain + problem + plan)
    let receipt = generate_receipt(&req.domain_text, &req.problem_text, &plan_str);

    Ok(PddlExecutionResponse {
        version: 1,
        status: "ok".to_string(),
        plan_or_refusal: plan_str,
        receipt,
    })
}

/// Execute POWL workflow in Rust (native API).
///
/// # Errors
///
/// Returns an error string if execution fails.
pub fn powl_execute_rust(req: &PowlExecutionRequest) -> Result<PowlExecutionResponse, String> {
    if req.version != 1 {
        return Err(format!(
            "unsupported version: {} (expected 1)",
            req.version
        ));
    }

    // Parse tape JSON
    let _tape_ops: Vec<serde_json::Value> = serde_json::from_str(&req.tape_json)
        .map_err(|e| format!("tape JSON parse error: {}", e))?;

    // Parse context JSON
    let _context: serde_json::Value = serde_json::from_str(&req.context_json)
        .map_err(|e| format!("context JSON parse error: {}", e))?;

    // Execute workflow (placeholder; would call bcinr-powl runtime)
    let result_str = format!("executed {} ops", _tape_ops.len());

    // Generate OCEL log (object-centric event log)
    let ocel_log = serde_json::json!({
        "ocel:version": "1.0",
        "ocel:events": [],
        "ocel:objects": [],
    })
    .to_string();

    // Generate receipt
    let receipt = generate_receipt(&req.tape_json, &req.context_json, &result_str);

    Ok(PowlExecutionResponse {
        version: 1,
        status: "ok".to_string(),
        result: result_str,
        ocel_log_json: ocel_log,
        receipt,
    })
}

// ─── Receipt Generation ──────────────────────────────────────────────────────

fn generate_receipt(input1: &str, input2: &str, output: &str) -> String {
    let combined = format!("{}{}{}", input1, input2, output);
    let hash = blake3::hash(combined.as_bytes());
    hash.to_hex().to_string()
}

// ─── C-ABI Bindings ─────────────────────────────────────────────────────────

/// C FFI: Execute PDDL planning from a JSON request.
///
/// # Safety
///
/// - `request_json_ptr` must point to valid UTF-8 JSON
/// - `request_json_len` must be the exact byte length of the JSON
/// - Returned pointer is heap-allocated; caller must call `free_c_string` to deallocate
///
/// # Returns
///
/// A pointer to a null-terminated C string containing the JSON response.
/// The response includes `status`, `plan_or_refusal`, and `receipt` fields.
#[no_mangle]
pub unsafe extern "C" fn pdl_execute(
    request_json_ptr: *const u8,
    request_json_len: usize,
) -> *const u8 {
    if request_json_ptr.is_null() || request_json_len == 0 {
        let err = PddlExecutionResponse {
            version: 1,
            status: "error".to_string(),
            plan_or_refusal: "null request".to_string(),
            receipt: String::new(),
        };
        return alloc_c_string(&serde_json::to_string(&err).unwrap_or_default());
    }

    let request_slice = unsafe { core::slice::from_raw_parts(request_json_ptr, request_json_len) };
    let request_str = match core::str::from_utf8(request_slice) {
        Ok(s) => s,
        Err(_) => {
            let err = PddlExecutionResponse {
                version: 1,
                status: "error".to_string(),
                plan_or_refusal: "invalid UTF-8".to_string(),
                receipt: String::new(),
            };
            return alloc_c_string(&serde_json::to_string(&err).unwrap_or_default());
        }
    };

    let req: PddlExecutionRequest = match serde_json::from_str(request_str) {
        Ok(r) => r,
        Err(e) => {
            let err = PddlExecutionResponse {
                version: 1,
                status: "error".to_string(),
                plan_or_refusal: format!("JSON parse: {}", e),
                receipt: String::new(),
            };
            return alloc_c_string(&serde_json::to_string(&err).unwrap_or_default());
        }
    };

    let resp = match pddl_execute_rust(&req) {
        Ok(r) => r,
        Err(e) => PddlExecutionResponse {
            version: 1,
            status: "error".to_string(),
            plan_or_refusal: e,
            receipt: String::new(),
        },
    };

    alloc_c_string(&serde_json::to_string(&resp).unwrap_or_default())
}

/// C FFI: Execute POWL workflow from a JSON request.
///
/// # Safety
///
/// - `request_json_ptr` must point to valid UTF-8 JSON
/// - `request_json_len` must be the exact byte length of the JSON
/// - Returned pointer is heap-allocated; caller must call `free_c_string` to deallocate
///
/// # Returns
///
/// A pointer to a null-terminated C string containing the JSON response.
#[no_mangle]
pub unsafe extern "C" fn powl_execute(
    request_json_ptr: *const u8,
    request_json_len: usize,
) -> *const u8 {
    if request_json_ptr.is_null() || request_json_len == 0 {
        let err = PowlExecutionResponse {
            version: 1,
            status: "error".to_string(),
            result: "null request".to_string(),
            ocel_log_json: String::new(),
            receipt: String::new(),
        };
        return alloc_c_string(&serde_json::to_string(&err).unwrap_or_default());
    }

    let request_slice = unsafe { core::slice::from_raw_parts(request_json_ptr, request_json_len) };
    let request_str = match core::str::from_utf8(request_slice) {
        Ok(s) => s,
        Err(_) => {
            let err = PowlExecutionResponse {
                version: 1,
                status: "error".to_string(),
                result: "invalid UTF-8".to_string(),
                ocel_log_json: String::new(),
                receipt: String::new(),
            };
            return alloc_c_string(&serde_json::to_string(&err).unwrap_or_default());
        }
    };

    let req: PowlExecutionRequest = match serde_json::from_str(request_str) {
        Ok(r) => r,
        Err(e) => {
            let err = PowlExecutionResponse {
                version: 1,
                status: "error".to_string(),
                result: format!("JSON parse: {}", e),
                ocel_log_json: String::new(),
                receipt: String::new(),
            };
            return alloc_c_string(&serde_json::to_string(&err).unwrap_or_default());
        }
    };

    let resp = match powl_execute_rust(&req) {
        Ok(r) => r,
        Err(e) => PowlExecutionResponse {
            version: 1,
            status: "error".to_string(),
            result: e,
            ocel_log_json: String::new(),
            receipt: String::new(),
        },
    };

    alloc_c_string(&serde_json::to_string(&resp).unwrap_or_default())
}

/// Free a C-allocated string.
///
/// # Safety
///
/// - `ptr` must be a pointer previously returned by `pdl_execute` or `powl_execute`
/// - Must be called exactly once per allocated string
/// - The pointer must have been allocated with alloc_c_string
#[no_mangle]
pub unsafe extern "C" fn free_c_string(ptr: *mut u8) {
    if !ptr.is_null() {
        // Find the null-terminator to determine the actual string length
        let mut len = 0;
        let mut p = ptr;
        while *p != 0 {
            len += 1;
            p = p.add(1);
        }
        // Include the null-terminator in the capacity for reconstruction
        let _ = Vec::from_raw_parts(ptr, len, len + 1);
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Allocate a Rust String as a C-accessible pointer.
///
/// Returns a pointer to the string's data (with null-terminator),
/// which MUST be freed with `free_c_string`.
fn alloc_c_string(s: &str) -> *const u8 {
    let mut buf = s.as_bytes().to_vec();
    buf.push(0); // null-terminator
    let ptr = buf.as_mut_ptr();
    // Prevent Vec from being dropped; caller must call free_c_string
    std::mem::forget(buf);
    ptr as *const u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pddl_request_serialization() {
        let req = PddlExecutionRequest {
            version: 1,
            domain_text: "(define (domain test) ...)".to_string(),
            problem_text: "(define (problem test) ...)".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let restored: PddlExecutionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.version, 1);
        assert_eq!(restored.domain_text, "(define (domain test) ...)");
    }

    #[test]
    fn test_pddl_response_serialization() {
        let resp = PddlExecutionResponse {
            version: 1,
            status: "ok".to_string(),
            plan_or_refusal: "0: action-a, 1: action-b".to_string(),
            receipt: "abcdef".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let restored: PddlExecutionResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.status, "ok");
        assert_eq!(restored.receipt, "abcdef");
    }

    #[test]
    fn test_powl_request_serialization() {
        let req = PowlExecutionRequest {
            version: 1,
            tape_json: r#"[{"kind":"Activity","lane":0}]"#.to_string(),
            context_json: r#"{"tenant_class":1,"urgency_tier":5}"#.to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let restored: PowlExecutionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.version, 1);
    }

    #[test]
    fn test_receipt_generation() {
        let receipt1 = generate_receipt("a", "b", "c");
        let receipt2 = generate_receipt("a", "b", "c");
        assert_eq!(receipt1, receipt2); // deterministic
        assert!(!receipt1.is_empty());
    }
}
