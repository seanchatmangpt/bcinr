//! `bcinr-compliance` — Regulatory compliance report generation from WorldManufactureReceipts.
//!
//! Maps a `WorldManufactureReceipt` JSON blob (produced by `manufacture_world` in bcinr-mcp)
//! to structured compliance reports for three frameworks:
//!
//! - **EU AI Act Article 13** (Transparency obligations)
//! - **NIST AI RMF** (Govern / Map / Measure / Manage)
//! - **SOC2 CC6** (Logical and Physical Access Controls)
//!
//! # Quick start
//!
//! ```
//! use bcinr_compliance::{generate_report, ComplianceStandard};
//! let receipt_json = r#"{
//!     "ok": true, "domain_name": "logistics", "problem_name": "p1",
//!     "domain_witness": "aabbcc", "problem_witness": "ddeeff",
//!     "manufacture_chain": "invalid", "plan_chain_hash": "001122",
//!     "admitted": true, "step_count": 3, "makespan": 3.0,
//!     "goal_reached": true, "refusal_reason": null,
//!     "plan_steps": []
//! }"#;
//! let report = generate_report(receipt_json, ComplianceStandard::EuAiAct, &[]).unwrap();
//! assert!(report.eu_ai_act.is_some());
//! ```

pub mod chain;
pub mod policy_bundles;

use serde::{Deserialize, Serialize};

// ── Error type ──────────────────────────────────────────────────────────────

/// Errors that can occur during compliance report generation.
#[derive(Debug)]
pub enum ComplianceError {
    /// The `receipt_data` string is not valid JSON.
    InvalidJson(String),
    /// A required field is absent or has the wrong type in the receipt JSON.
    MissingField(String),
    /// The requested policy bundle name is not recognised.
    UnknownPolicy(String),
}

impl std::fmt::Display for ComplianceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidJson(e) => write!(f, "invalid receipt JSON: {e}"),
            Self::MissingField(field) => write!(f, "missing receipt field: {field}"),
            Self::UnknownPolicy(name) => write!(f, "unknown policy bundle: {name}"),
        }
    }
}

impl std::error::Error for ComplianceError {}

// ── Public enums ────────────────────────────────────────────────────────────

/// Which regulatory framework the report targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceStandard {
    EuAiAct,
    NistAiRmf,
    Soc2,
}

impl ComplianceStandard {
    /// Parse from a case-insensitive string identifier.
    ///
    /// Accepted values: `"EU_AI_ACT"`, `"NIST"`, `"SOC2"`.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_uppercase().as_str() {
            "EU_AI_ACT" | "EU" => Some(Self::EuAiAct),
            "NIST" | "NIST_AI_RMF" => Some(Self::NistAiRmf),
            "SOC2" | "SOC_2" => Some(Self::Soc2),
            _ => None,
        }
    }
}

// ── Sub-report structs ───────────────────────────────────────────────────────

/// EU AI Act Article 13 transparency fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransparencyFields {
    /// Art. 13(1): system identity — `domain_name/problem_name`.
    pub system_description: String,
    /// Art. 13(2)(b): intended purpose derived from domain action names.
    pub intended_purpose: String,
    /// Art. 13(3)(d): human oversight gate result (Prolog8 admission).
    pub human_oversight_gate_passed: bool,
    /// Art. 13(3)(e): goal-achievement evidence.
    pub goal_achieved: bool,
    /// Art. 13(3)(b): BLAKE3 manufacture_chain (cryptographic audit anchor).
    pub audit_trail_hash: String,
    /// Number of plan steps executed.
    pub step_count: u64,
    /// Active policy rule names passed to this report.
    pub policy_rules_active: Vec<String>,
    /// Art. 13(2)(a): refusal traceability — `None` when admitted.
    pub refusal_reason: Option<String>,
    /// Whether the BLAKE3 manufacture chain recomputes correctly.
    pub chain_integrity_valid: bool,
}

/// EU AI Act Article 13 report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EuAiActReport {
    pub article_13_transparency: TransparencyFields,
    /// Determinism evidence: step_count + makespan as a descriptive string.
    pub determinism_evidence: String,
}

/// NIST AI RMF — Govern function fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernFields {
    /// GOVERN 1.1: policies enforced (names of evaluated policy bundles).
    pub policies_enforced: Vec<String>,
}

/// NIST AI RMF — Map function fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapFields {
    /// MAP 1.5: context characterisation.
    pub domain_name: String,
    pub problem_name: String,
    pub step_count: u64,
    pub makespan: f64,
}

/// NIST AI RMF — Measure function fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasureFields {
    /// MEASURE 2.5: plan performance metrics.
    pub step_count: u64,
    pub makespan: f64,
    pub goal_reached: bool,
}

/// NIST AI RMF — Manage function fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManageFields {
    /// MANAGE 2.2: incident traceability — full witness chain for forensic replay.
    pub domain_witness: String,
    pub problem_witness: String,
    pub plan_chain_hash: String,
    pub manufacture_chain: String,
}

/// NIST AI RMF report covering all four functions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NistAiRmfReport {
    pub govern: GovernFields,
    pub map_: MapFields,
    pub measure: MeasureFields,
    pub manage: ManageFields,
}

/// SOC2 CC6 report (Logical and Physical Access Controls).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Soc2Report {
    /// CC6.1: access control — Prolog8 admission gate result.
    pub cc6_1_logical_access: bool,
    /// CC6.2: domain credential (BLAKE3 of structural identity).
    pub cc6_2_domain_credential: String,
    /// CC6.3: action count as least-privilege proxy.
    pub cc6_3_action_count: u64,
    /// CC6.6: refusal code if gate rejected.
    pub cc6_6_refusal_code: Option<String>,
    /// CC6.7: manufacture chain integrity (tamper-evident).
    pub cc6_7_chain_valid: bool,
    /// CC6.8: per-step chain hash (malicious software controls proxy).
    pub cc6_8_step_chain_hash: String,
}

/// Per-policy evaluation verdict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyVerdict {
    pub policy_name: String,
    pub passed: bool,
    /// Which receipt fields justified the verdict.
    pub evidence: String,
}

/// Top-level compliance report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    /// Which framework this report covers.
    pub standard: ComplianceStandard,
    /// ISO-8601 timestamp at generation (seconds precision).
    pub generated_at: String,
    /// The `manufacture_chain` hex from the source receipt (primary audit anchor).
    pub receipt_hash: String,
    /// Present when `standard == EuAiAct`.
    pub eu_ai_act: Option<EuAiActReport>,
    /// Present when `standard == NistAiRmf`.
    pub nist_ai_rmf: Option<NistAiRmfReport>,
    /// Present when `standard == Soc2`.
    pub soc2: Option<Soc2Report>,
    /// Policy bundle verdicts (one per named policy passed by the caller).
    pub policy_verdicts: Vec<PolicyVerdict>,
}

// ── Internal helpers ─────────────────────────────────────────────────────────

/// Trim a string to at most `max_len` chars, appending "…" when truncated.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let mut out = s[..max_len].to_string();
        out.push('…');
        out
    }
}

/// Derive a refusal code from the `refusal_reason` text by returning the
/// first whitespace-separated token (e.g. `"STEP_DENIED …"` → `"STEP_DENIED"`).
fn refusal_code(reason: Option<&str>) -> Option<String> {
    reason.map(|r| {
        r.split_whitespace()
            .next()
            .unwrap_or(r)
            .to_string()
    })
}

/// Build a simple ISO-8601 timestamp from `std::time::SystemTime`.
fn now_iso8601() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Convert seconds since epoch to YYYY-MM-DDTHH:MM:SSZ without chrono.
    let s = secs;
    let (y, mo, d, h, mi, sc) = epoch_to_ymd_hms(s);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{sc:02}Z")
}

/// Minimal epoch → calendar conversion (Gregorian, UTC).
fn epoch_to_ymd_hms(secs: u64) -> (u64, u64, u64, u64, u64, u64) {
    let sc = secs % 60;
    let mi = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = secs / 86400;
    // Use the algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days + 719468;
    let era = z / 146097;
    let doe = z % 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mo <= 2 { y + 1 } else { y };
    (y, mo, d, h, mi, sc)
}

/// Evaluate a single policy clause predicate against the materialised receipt fields.
///
/// Returns `(passed, evidence_detail)`.
fn eval_clause(
    clause: &str,
    admitted: bool,
    step_count: u64,
    makespan: f64,
    goal_reached: bool,
    chain_valid: bool,
    capability_summary: &str,
    domain_w: &str,
    problem_w: &str,
    plan_chain: &str,
) -> (bool, String) {
    if clause == "admitted" {
        return (admitted, format!("admitted={admitted}"));
    }
    if clause == "goal_reached" {
        return (goal_reached, format!("goal_reached={goal_reached}"));
    }
    if clause == "chain_integrity_valid" {
        return (chain_valid, format!("chain_integrity_valid={chain_valid}"));
    }
    if clause == "all_witnesses_non_empty" {
        let ok = !domain_w.is_empty() && !problem_w.is_empty() && !plan_chain.is_empty();
        return (ok, format!("all_witnesses_non_empty={ok}"));
    }
    if let Some(rest) = clause.strip_prefix("step_count_lte(") {
        if let Some(num_str) = rest.strip_suffix(')') {
            if let Ok(limit) = num_str.parse::<u64>() {
                let ok = step_count <= limit;
                return (ok, format!("step_count={step_count} <= {limit}"));
            }
        }
    }
    if let Some(rest) = clause.strip_prefix("step_count_gte(") {
        if let Some(num_str) = rest.strip_suffix(')') {
            if let Ok(limit) = num_str.parse::<u64>() {
                let ok = step_count >= limit;
                return (ok, format!("step_count={step_count} >= {limit}"));
            }
        }
    }
    if let Some(rest) = clause.strip_prefix("makespan_lte(") {
        if let Some(num_str) = rest.strip_suffix(')') {
            if let Ok(limit) = num_str.parse::<f64>() {
                let ok = makespan <= limit;
                return (ok, format!("makespan={makespan} <= {limit}"));
            }
        }
    }
    if let Some(rest) = clause.strip_prefix("no_action_matching(") {
        if let Some(pattern) = rest.strip_suffix(')') {
            // Simple substring match on capability_summary (no regex in no_std path).
            let needle = pattern.trim_end_matches(".*");
            let matched = capability_summary.contains(needle);
            let ok = !matched;
            return (
                ok,
                format!("no_action_matching({pattern}): capability_summary contains pattern={matched}"),
            );
        }
    }
    // Unknown clause — treat as not-passed with explanation.
    (false, format!("unknown_clause({clause})"))
}

/// Evaluate all clauses in a named policy bundle, returning a `PolicyVerdict`.
fn eval_policy(
    policy_name: &str,
    clauses: &[&str],
    admitted: bool,
    step_count: u64,
    makespan: f64,
    goal_reached: bool,
    chain_valid: bool,
    capability_summary: &str,
    domain_w: &str,
    problem_w: &str,
    plan_chain: &str,
) -> PolicyVerdict {
    let mut all_passed = true;
    let mut evidence_parts = Vec::new();
    for clause in clauses {
        let (ok, detail) = eval_clause(
            clause,
            admitted,
            step_count,
            makespan,
            goal_reached,
            chain_valid,
            capability_summary,
            domain_w,
            problem_w,
            plan_chain,
        );
        if !ok {
            all_passed = false;
        }
        evidence_parts.push(detail);
    }
    PolicyVerdict {
        policy_name: policy_name.to_string(),
        passed: all_passed,
        evidence: evidence_parts.join("; "),
    }
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Generate a compliance report from a `WorldManufactureReceipt` JSON string.
///
/// # Arguments
///
/// * `receipt_json` — The raw JSON output of `manufacture_world`.
/// * `standard` — Which regulatory framework to populate.
/// * `policies` — Named policy bundles to evaluate (e.g. `["gdpr_data_minimisation"]`).
///
/// # Errors
///
/// Returns [`ComplianceError::InvalidJson`] if `receipt_json` is not valid JSON.
/// Returns [`ComplianceError::UnknownPolicy`] if any name in `policies` is not recognised.
///
/// # Examples
///
/// ```
/// use bcinr_compliance::{generate_report, ComplianceStandard};
/// let receipt = r#"{
///     "ok": true, "domain_name": "logistics", "problem_name": "p1",
///     "domain_witness": "aa", "problem_witness": "bb",
///     "manufacture_chain": "xx", "plan_chain_hash": "cc",
///     "admitted": true, "step_count": 2, "makespan": 2.0,
///     "goal_reached": true, "refusal_reason": null, "plan_steps": []
/// }"#;
/// let r = generate_report(receipt, ComplianceStandard::EuAiAct, &[]).unwrap();
/// assert!(r.eu_ai_act.is_some());
/// assert_eq!(r.standard, ComplianceStandard::EuAiAct);
/// ```
pub fn generate_report(
    receipt_json: &str,
    standard: ComplianceStandard,
    policies: &[&str],
) -> Result<ComplianceReport, ComplianceError> {
    let data: serde_json::Value = serde_json::from_str(receipt_json)
        .map_err(|e| ComplianceError::InvalidJson(e.to_string()))?;

    // ── Extract receipt fields ───────────────────────────────────────────────
    let domain_name = data
        .get("domain_name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let problem_name = data
        .get("problem_name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let domain_w = data
        .get("domain_witness")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let problem_w = data
        .get("problem_witness")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let plan_chain = data
        .get("plan_chain_hash")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let manufacture_chain = data
        .get("manufacture_chain")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let admitted = data
        .get("admitted")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let step_count = data
        .get("step_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let makespan = data
        .get("makespan")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let goal_reached = data
        .get("goal_reached")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let refusal_reason: Option<String> = data
        .get("refusal_reason")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Build capability summary from plan_steps[*].action_name.
    let capability_summary: String = {
        let names: Vec<&str> = data
            .get("plan_steps")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|s| s.get("action_name").and_then(|v| v.as_str()))
                    .collect()
            })
            .unwrap_or_default();
        truncate(&names.join(", "), 256)
    };

    // Recompute BLAKE3 manufacture chain.
    let chain_valid = chain::verify_manufacture_chain(
        domain_w,
        problem_w,
        plan_chain,
        manufacture_chain,
    );

    // ── Validate & resolve policies ──────────────────────────────────────────
    for name in policies {
        if policy_bundles::lookup(name).is_none() {
            return Err(ComplianceError::UnknownPolicy(name.to_string()));
        }
    }

    // ── Build policy verdicts ────────────────────────────────────────────────
    let policy_verdicts: Vec<PolicyVerdict> = policies
        .iter()
        .map(|name| {
            let clauses = policy_bundles::lookup(name).unwrap_or(&[]);
            eval_policy(
                name,
                clauses,
                admitted,
                step_count,
                makespan,
                goal_reached,
                chain_valid,
                &capability_summary,
                domain_w,
                problem_w,
                plan_chain,
            )
        })
        .collect();

    let receipt_hash = manufacture_chain.to_string();
    let generated_at = now_iso8601();

    // ── Populate framework-specific report ───────────────────────────────────
    let (eu_ai_act, nist_ai_rmf, soc2) = match standard {
        ComplianceStandard::EuAiAct => {
            let policy_names: Vec<String> = policies.iter().map(|s| s.to_string()).collect();
            let fields = TransparencyFields {
                system_description: format!("{domain_name}/{problem_name}"),
                intended_purpose: if capability_summary.is_empty() {
                    "no actions in plan".to_string()
                } else {
                    capability_summary.clone()
                },
                human_oversight_gate_passed: admitted,
                goal_achieved: goal_reached,
                audit_trail_hash: manufacture_chain.to_string(),
                step_count,
                policy_rules_active: policy_names,
                refusal_reason: refusal_reason.clone(),
                chain_integrity_valid: chain_valid,
            };
            let report = EuAiActReport {
                article_13_transparency: fields,
                determinism_evidence: format!(
                    "step_count={step_count}, makespan={makespan:.3}, branchless O(1)/O(log n)"
                ),
            };
            (Some(report), None, None)
        }

        ComplianceStandard::NistAiRmf => {
            let policy_names: Vec<String> = policies.iter().map(|s| s.to_string()).collect();
            let report = NistAiRmfReport {
                govern: GovernFields {
                    policies_enforced: policy_names,
                },
                map_: MapFields {
                    domain_name: domain_name.to_string(),
                    problem_name: problem_name.to_string(),
                    step_count,
                    makespan,
                },
                measure: MeasureFields {
                    step_count,
                    makespan,
                    goal_reached,
                },
                manage: ManageFields {
                    domain_witness: domain_w.to_string(),
                    problem_witness: problem_w.to_string(),
                    plan_chain_hash: plan_chain.to_string(),
                    manufacture_chain: manufacture_chain.to_string(),
                },
            };
            (None, Some(report), None)
        }

        ComplianceStandard::Soc2 => {
            let report = Soc2Report {
                cc6_1_logical_access: admitted,
                cc6_2_domain_credential: domain_w.to_string(),
                cc6_3_action_count: step_count,
                cc6_6_refusal_code: refusal_code(refusal_reason.as_deref()),
                cc6_7_chain_valid: chain_valid,
                cc6_8_step_chain_hash: plan_chain.to_string(),
            };
            (None, None, Some(report))
        }
    };

    Ok(ComplianceReport {
        standard,
        generated_at,
        receipt_hash,
        eu_ai_act,
        nist_ai_rmf,
        soc2,
        policy_verdicts,
    })
}
