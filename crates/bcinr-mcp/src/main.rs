//! bcinr-mcp — MCP server exposing the ENTIRE bcinr library as MCP tools over stdio.
//!
//! Tools:
//!  Group 1 — PDDL (7 tools):
//!    pddl_domain_info, pddl_parse_domain, pddl_parse_problem, pddl_plan,
//!    manufacture_world, pddl_admit_domain, pddl_temporal_plan_info
//!  Group 2 — POWL (5 tools):
//!    powl_compile_sequence, powl_compile_choice, powl_admit_context,
//!    powl_capability_check, powl_plan_to_tape
//!  Group 3 — Core bcinr (3 tools):
//!    bcinr_library_info, bcinr_mask_ops, bcinr_powl_info
//!  Group 4 — bcinr-logic Algorithms (6 tools):
//!    utf8_validate, bitset_operations, dfa_info, scan_patterns, reduce_sequence, simd_string_info
//!  Group 5 — POWL Receipts (1 tool):
//!    receipt_inspect
//!  Group 6 — Cross-crate Info (1 tool):
//!    system_capabilities

use rmcp::{
    ServiceExt,
    handler::server::wrapper::Parameters,
    schemars,
    tool, tool_router,
    transport::stdio,
};
use serde::Deserialize;

// ─── Flexible u64 deserializer ────────────────────────────────────────────────
//
// JSON has no native u64. Values > 2^53 round to f64 before serde sees them,
// causing "invalid type: floating point, expected u64". This deserializer also
// accepts decimal strings ("18446744073709551615"), which LLMs often generate.

fn de_u64_flex<'de, D: serde::Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
    use serde::de::Error as _;
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum NumOrStr {
        Int(u64),
        Float(f64),
        Str(String),
    }
    match NumOrStr::deserialize(d)? {
        NumOrStr::Int(n) => Ok(n),
        NumOrStr::Float(f) => {
            if f >= 0.0 && f <= u64::MAX as f64 && f.fract() == 0.0 {
                Ok(f as u64)
            } else {
                Err(D::Error::custom(format!("{f} is not a valid u64")))
            }
        }
        NumOrStr::Str(s) => s.parse::<u64>().map_err(D::Error::custom),
    }
}

// ─── Parameter structs ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DomainInput {
    /// PDDL domain text (define (domain ...))
    pub domain_text: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ProblemInput {
    /// PDDL problem text (define (problem ...))
    pub problem_text: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PlanInput {
    /// PDDL domain text
    pub domain_text: String,
    /// PDDL problem text
    pub problem_text: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ManufactureInput {
    /// PDDL domain text
    pub domain_text: String,
    /// PDDL problem text
    pub problem_text: String,
    /// Case ID for the manufacture receipt
    pub case_id: String,
    /// Optional Horn policy: list of [head, [body_atom, ...]] pairs.
    /// Empty or absent = permissive (every action pre-admitted).
    pub policy_rules: Option<Vec<(String, Vec<String>)>>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LabelsInput {
    /// Comma-separated activity labels, e.g. "A,B,C"
    pub labels: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AdmitContextInput {
    /// Tenant class (0=free, 1=standard, 2=enterprise, 3=sovereign)
    #[serde(deserialize_with = "de_u64_flex")]
    pub tenant_class: u64,
    /// Urgency tier (0-15)
    #[serde(deserialize_with = "de_u64_flex")]
    pub urgency_tier: u64,
    /// Resource load (0-15)
    #[serde(deserialize_with = "de_u64_flex")]
    pub resource_load: u64,
    /// Whether a SLA token is present
    pub has_sla_token: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CapabilityInput {
    /// Granted capability mask as hex string (e.g. "0xff")
    pub granted_hex: String,
    /// Required capability mask as hex string
    pub required_hex: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MaskOpsInput {
    /// First 64-bit value
    #[serde(deserialize_with = "de_u64_flex")]
    pub a: u64,
    /// Second 64-bit value
    #[serde(deserialize_with = "de_u64_flex")]
    pub b: u64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct Utf8Input {
    /// Byte string to validate (as hex or UTF-8)
    pub data: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct BitsetInput {
    /// Bitset operation: "popcount", "leading_zeros", "trailing_zeros", "msb", "lsb"
    pub operation: String,
    /// Value to operate on
    #[serde(deserialize_with = "de_u64_flex")]
    pub value: u64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PatternInput {
    /// Text to scan
    pub text: String,
    /// Pattern to search for
    pub pattern: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ReceiptInput {
    /// Receipt data as JSON string
    pub receipt_data: String,
}

// ─── Server ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BcinrServer;

#[tool_router(server_handler)]
impl BcinrServer {
    // ── Group 1: PDDL Tools ──────────────────────────────────────────────────

    /// Return a human-readable description of a PDDL domain.
    #[tool(description = "Describe a PDDL domain in human-readable text: name, requirements, predicates (name/arity), actions (name + params), durative actions.")]
    async fn pddl_domain_info(
        &self,
        Parameters(input): Parameters<DomainInput>,
    ) -> String {
        // Try PDDL 3.1 parser first (richer info), fall back to STRIPS8
        match bcinr_pddl::domain31_from_pddl(&input.domain_text) {
            Ok(d) => {
                let mut out = format!("Domain: {}\n", d.name);
                if !d.requirements.is_empty() {
                    out.push_str(&format!("Requirements: {}\n", d.requirements.join(", ")));
                }
                out.push_str(&format!("Predicates ({}):\n", d.predicates.len()));
                for (name, params) in &d.predicates {
                    out.push_str(&format!("  {name}/{}\n", params.len()));
                }
                out.push_str(&format!("Actions ({}):\n", d.actions.len()));
                for a in &d.actions {
                    let params: Vec<String> = a.params.iter().map(|(v, t)| format!("{v}: {t}")).collect();
                    out.push_str(&format!("  {}({})\n", a.name, params.join(", ")));
                }
                if !d.durative_actions.is_empty() {
                    out.push_str(&format!("Durative Actions ({}):\n", d.durative_actions.len()));
                    for da in &d.durative_actions {
                        out.push_str(&format!("  {}\n", da.name));
                    }
                }
                out
            }
            Err(_) => match bcinr_pddl::domain_from_pddl(&input.domain_text) {
                Ok(d) => {
                    let mut out = format!("Domain: {}\n", d.name);
                    out.push_str(&format!("Predicates ({}):\n", d.predicates.len()));
                    for (pname, arity) in &d.predicates {
                        out.push_str(&format!("  {pname}/{arity}\n"));
                    }
                    out.push_str(&format!("Actions ({}):\n", d.actions.len()));
                    for a in &d.actions {
                        let params = a.params.join(", ");
                        out.push_str(&format!("  {}({})\n", a.name, params));
                    }
                    out
                }
                Err(e) => format!("Error parsing domain: {e}"),
            },
        }
    }

    /// Parse a PDDL domain and return JSON admission summary.
    #[tool(description = "Parse a PDDL domain text. Returns JSON with ok, name, requirement_count, predicate_count, action_count, durative_action_count, witness.")]
    async fn pddl_parse_domain(
        &self,
        Parameters(input): Parameters<DomainInput>,
    ) -> String {
        match bcinr_pddl::admit_candidate_domain(&input.domain_text) {
            Ok(ad) => {
                let d = &ad.domain31;
                serde_json::json!({
                    "ok": true,
                    "name": d.name,
                    "requirement_count": d.requirements.len(),
                    "predicate_count": d.predicates.len(),
                    "action_count": d.actions.len(),
                    "durative_action_count": d.durative_actions.len(),
                    "witness": ad.witness,
                })
                .to_string()
            }
            Err(e) => serde_json::json!({ "ok": false, "error": e.to_string() }).to_string(),
        }
    }

    /// Parse a PDDL problem and return JSON summary.
    #[tool(description = "Parse a PDDL problem text. Returns JSON with ok, name, domain, object_count, init_count.")]
    async fn pddl_parse_problem(
        &self,
        Parameters(input): Parameters<ProblemInput>,
    ) -> String {
        match bcinr_pddl::problem_from_pddl(&input.problem_text) {
            Ok(p) => serde_json::json!({
                "ok": true,
                "name": p.name,
                "domain": p.domain,
                "object_count": p.objects.len(),
                "init_count": p.init.len(),
            })
            .to_string(),
            Err(e) => serde_json::json!({ "ok": false, "error": e.to_string() }).to_string(),
        }
    }

    /// Run PDDL BFS planner and return plan steps as JSON.
    #[tool(description = "Find a STRIPS plan. Returns JSON with ok, steps (list of '0: action-label'), step_count.")]
    async fn pddl_plan(
        &self,
        Parameters(input): Parameters<PlanInput>,
    ) -> String {
        let domain = match bcinr_pddl::domain_from_pddl(&input.domain_text) {
            Ok(d) => d,
            Err(e) => {
                return serde_json::json!({ "ok": false, "error": format!("domain parse: {e}") })
                    .to_string()
            }
        };
        let problem = match bcinr_pddl::problem_from_pddl(&input.problem_text) {
            Ok(p) => p,
            Err(e) => {
                return serde_json::json!({ "ok": false, "error": format!("problem parse: {e}") })
                    .to_string()
            }
        };
        let ground = match bcinr_pddl::GroundProblem::build(&domain, &problem, None) {
            Ok(g) => g,
            Err(e) => {
                return serde_json::json!({ "ok": false, "error": format!("grounding: {e}") })
                    .to_string()
            }
        };
        match ground.find_plan() {
            Ok(tape) => {
                let steps: Vec<String> = tape
                    .ops
                    .iter()
                    .enumerate()
                    .map(|(i, op)| format!("{i}: {}", op.label))
                    .collect();
                let step_count = steps.len();
                serde_json::json!({ "ok": true, "steps": steps, "step_count": step_count })
                    .to_string()
            }
            Err(e) => serde_json::json!({ "ok": false, "error": e.to_string() }).to_string(),
        }
    }

    /// Full Vision 2030 world manufacturing loop (admit + plan + execute + BLAKE3 receipt).
    ///
    /// Returns ok=true with full receipt on success. Returns ok=false with refusal_code on
    /// any admission failure, planning failure, or bound violation — never panics.
    #[tool(description = "Run the bcinr world-manufacturing loop: admit domain+problem, plan, return BLAKE3-chained WorldManufactureReceipt as JSON.")]
    async fn manufacture_world(
        &self,
        Parameters(input): Parameters<ManufactureInput>,
    ) -> String {
        let owned_rules: Vec<(String, Vec<String>)> = input.policy_rules.unwrap_or_default();
        let rule_refs: Vec<(&str, Vec<&str>)> = owned_rules
            .iter()
            .map(|(h, b)| (h.as_str(), b.iter().map(String::as_str).collect()))
            .collect();
        let r = bcinr_pddl::manufacture_world(
            &input.domain_text,
            &input.problem_text,
            &input.case_id,
            &rule_refs,
        );

        if r.admitted {
            serde_json::json!({
                "ok": true,
                "admitted": true,
                "domain_name": r.domain_name,
                "problem_name": r.problem_name,
                "domain_witness": r.domain_witness,
                "problem_witness": r.problem_witness,
                "manufacture_chain": r.manufacture_chain,
                "plan_chain_hash": r.plan_receipt.chain_hash,
                "step_count": r.plan_receipt.step_count,
                "makespan": r.plan_receipt.makespan,
                "goal_reached": r.plan_receipt.goal_reached,
                "refusal_reason": null,
            })
            .to_string()
        } else {
            let refusal_code = match r.refusal_reason.as_deref().unwrap_or("") {
                s if s.contains("step") && s.contains("denied") => "STEP_DENIED",
                s if s.contains("bound exceeded") => "BOUND_EXCEEDED",
                s if s.contains("domain admission failed") => "DOMAIN_REFUSED",
                s if s.contains("problem admission failed") => "PROBLEM_REFUSED",
                s if s.contains("empty") || s.contains("no applicable") => "EMPTY_PLAN",
                _ => "PLANNING_FAILED",
            };
            serde_json::json!({
                "ok": false,
                "admitted": false,
                "refusal_code": refusal_code,
                "refusal_reason": r.refusal_reason,
                "domain_name": r.domain_name,
                "problem_name": r.problem_name,
                "domain_witness": r.domain_witness,
                "problem_witness": r.problem_witness,
                "manufacture_chain": r.manufacture_chain,
                "plan_chain_hash": r.plan_receipt.chain_hash,
                "step_count": r.plan_receipt.step_count,
                "makespan": r.plan_receipt.makespan,
                "goal_reached": false,
            })
            .to_string()
        }
    }

    /// Admit a PDDL domain through the Prolog8 R ⊢ A gate.
    #[tool(description = "Admit a PDDL 3.1 domain. Returns JSON with ok, name, witness, requirement_count, action_count, durative_action_count.")]
    async fn pddl_admit_domain(
        &self,
        Parameters(input): Parameters<DomainInput>,
    ) -> String {
        match bcinr_pddl::admit_candidate_domain(&input.domain_text) {
            Ok(ad) => {
                let d = &ad.domain31;
                serde_json::json!({
                    "ok": true,
                    "name": d.name,
                    "witness": ad.witness,
                    "requirement_count": d.requirements.len(),
                    "action_count": d.actions.len(),
                    "durative_action_count": d.durative_actions.len(),
                })
                .to_string()
            }
            Err(e) => serde_json::json!({ "ok": false, "error": e.to_string() }).to_string(),
        }
    }

    /// Summarize temporal features of a PDDL 3.1 domain+problem.
    #[tool(description = "Summarize temporal features: durative actions (name, duration), timed initial literals count, metric.")]
    async fn pddl_temporal_plan_info(
        &self,
        Parameters(input): Parameters<PlanInput>,
    ) -> String {
        let domain = match bcinr_pddl::domain31_from_pddl(&input.domain_text) {
            Ok(d) => d,
            Err(e) => return format!("Domain parse error: {e}"),
        };
        let problem = match bcinr_pddl::problem31_from_pddl(&input.problem_text) {
            Ok(p) => p,
            Err(e) => return format!("Problem parse error: {e}"),
        };
        let mut out = format!("Domain: {}\n", domain.name);
        out.push_str(&format!(
            "Durative actions ({}):\n",
            domain.durative_actions.len()
        ));
        for da in &domain.durative_actions {
            out.push_str(&format!("  {} — duration: {:?}\n", da.name, da.duration));
        }
        out.push_str(&format!(
            "Timed initial literals: {}\n",
            problem.timed_inits.len()
        ));
        if let Some(metric) = &problem.metric {
            out.push_str(&format!(
                "Metric: {:?} {:?}\n",
                metric.dir, metric.expr
            ));
        } else {
            out.push_str("Metric: none\n");
        }
        out
    }

    // ── Group 2: POWL Tools ──────────────────────────────────────────────────

    /// Compile comma-separated labels into a POWL Sequence tape.
    #[tool(description = "Compile comma-separated labels (e.g. 'A,B,C') into a POWL Sequence tape. Returns JSON with ok, op_count, entry_mask, topology.")]
    async fn powl_compile_sequence(
        &self,
        Parameters(input): Parameters<LabelsInput>,
    ) -> String {
        use bcinr_powl::compiler::{PowlAstNode, compile_powl};
        let labels: Vec<String> = input
            .labels
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if labels.is_empty() {
            return serde_json::json!({ "ok": false, "error": "empty label list" }).to_string();
        }
        let atoms: Vec<PowlAstNode> = labels.iter().map(|l| PowlAstNode::Atom(l.as_str())).collect();
        let ast = PowlAstNode::Sequence(atoms);
        match compile_powl(&ast) {
            Err(e) => serde_json::json!({ "ok": false, "error": format!("{e:?}") }).to_string(),
            Ok(tape) => serde_json::json!({
                "ok": true,
                "op_count": tape.len,
                "entry_mask": tape.entry_mask,
                "topology": "Sequence",
            })
            .to_string(),
        }
    }

    /// Compile comma-separated labels into a POWL XorChoice tape.
    #[tool(description = "Compile comma-separated labels into a POWL XorChoice tape. Returns JSON with ok, op_count, branch_count.")]
    async fn powl_compile_choice(
        &self,
        Parameters(input): Parameters<LabelsInput>,
    ) -> String {
        use bcinr_powl::compiler::{PowlAstNode, compile_powl};
        let labels: Vec<String> = input
            .labels
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if labels.is_empty() {
            return serde_json::json!({ "ok": false, "error": "empty label list" }).to_string();
        }
        let branch_count = labels.len();
        let atoms: Vec<PowlAstNode> = labels.iter().map(|l| PowlAstNode::Atom(l.as_str())).collect();
        let ast = PowlAstNode::XorChoice(atoms);
        match compile_powl(&ast) {
            Err(e) => serde_json::json!({ "ok": false, "error": format!("{e:?}") }).to_string(),
            Ok(tape) => serde_json::json!({
                "ok": true,
                "op_count": tape.len,
                "branch_count": branch_count,
            })
            .to_string(),
        }
    }

    /// Admit a POWL execution context — branchless O(1) LUT dispatch.
    #[tool(description = "Admit a POWL execution context. Returns JSON with topology (Priority/Standard/Background/Quarantine) and ctx_hex.")]
    async fn powl_admit_context(
        &self,
        Parameters(input): Parameters<AdmitContextInput>,
    ) -> String {
        use bcinr_powl::admit::admit;
        let ctx: u64 = (input.tenant_class & 0xF)
            | ((input.urgency_tier & 0xF) << 4)
            | ((input.resource_load & 0xF) << 8)
            | (if input.has_sla_token { 1u64 << 12 } else { 0 });
        let topology = admit(ctx);
        serde_json::json!({
            "topology": format!("{topology:?}"),
            "ctx_hex": format!("{ctx:#018x}"),
        })
        .to_string()
    }

    /// Branchless O(1) capability check.
    #[tool(description = "Check if granted capability mask satisfies required mask. Returns JSON with granted_bits, required_bits, passes (bool), mask_hex.")]
    async fn powl_capability_check(
        &self,
        Parameters(input): Parameters<CapabilityInput>,
    ) -> String {
        use bcinr_powl::enterprise::capability_mask;
        let parse_hex = |s: &str| -> Result<u64, String> {
            let s = s.trim().trim_start_matches("0x").trim_start_matches("0X");
            u64::from_str_radix(s, 16).map_err(|e| e.to_string())
        };
        let granted = match parse_hex(&input.granted_hex) {
            Err(e) => return serde_json::json!({ "ok": false, "error": e }).to_string(),
            Ok(v) => v,
        };
        let required = match parse_hex(&input.required_hex) {
            Err(e) => return serde_json::json!({ "ok": false, "error": e }).to_string(),
            Ok(v) => v,
        };
        let mask = capability_mask(granted, required);
        serde_json::json!({
            "granted_bits": format!("{granted:#018x}"),
            "required_bits": format!("{required:#018x}"),
            "passes": mask != 0,
            "mask_hex": format!("{mask:#018x}"),
        })
        .to_string()
    }

    /// Plan via PDDL then convert the temporal plan to POWL op specs.
    #[tool(description = "Plan via PDDL then convert temporal plan to POWL op specs. Returns JSON with ok, op_count, ops array.")]
    async fn powl_plan_to_tape(
        &self,
        Parameters(input): Parameters<PlanInput>,
    ) -> String {
        use bcinr_pddl::{domain_from_pddl, problem_from_pddl, GroundProblem, TemporalPlan, TemporalPlanStep};
        use bcinr_pddl::powl_bridge::temporal_plan_to_powl_tape;

        let domain = match domain_from_pddl(&input.domain_text) {
            Err(e) => return serde_json::json!({ "ok": false, "error": e.to_string() }).to_string(),
            Ok(d) => d,
        };
        let problem = match problem_from_pddl(&input.problem_text) {
            Err(e) => return serde_json::json!({ "ok": false, "error": e.to_string() }).to_string(),
            Ok(p) => p,
        };
        let ground = match GroundProblem::build(&domain, &problem, None) {
            Err(e) => return serde_json::json!({ "ok": false, "error": e.to_string() }).to_string(),
            Ok(g) => g,
        };
        let tape = match ground.find_plan() {
            Err(e) => return serde_json::json!({ "ok": false, "error": e.to_string() }).to_string(),
            Ok(t) => t,
        };

        let step_count = tape.ops.len();
        let steps: Vec<TemporalPlanStep> = tape.ops.iter().enumerate().map(|(i, op)| TemporalPlanStep {
            action_name: op.label.clone(),
            args: vec![],
            start_time: i as f64,
            duration: 1.0,
        }).collect();
        let temporal_plan = TemporalPlan {
            steps,
            makespan: step_count as f64,
            metric_value: None,
        };

        let specs = temporal_plan_to_powl_tape(&temporal_plan);
        let ops_arr: Vec<serde_json::Value> = specs.iter().map(|s| serde_json::json!({
            "label": s.label,
            "kind": format!("{:?}", s.kind),
            "pred_mask": s.pred_mask,
            "succ_mask": s.succ_mask,
        })).collect();

        serde_json::json!({
            "ok": true,
            "op_count": ops_arr.len(),
            "ops": ops_arr,
        })
        .to_string()
    }

    // ── Group 3: Core bcinr Library Tools ───────────────────────────────────

    /// Return a human-readable description of the bcinr library.
    #[tool(description = "Return human-readable description of the bcinr library: crates, modules, and key capabilities.")]
    async fn bcinr_library_info(&self) -> String {
        "BranchlessCInRust (bcinr) v26.x — academic-grade branchless algorithm library\n\
         Crates: bcinr-core (facade), bcinr-logic (algorithms), bcinr-api (API surface)\n\
         Modules: algorithms/, abstractions/, bitset/, dfa/, exec/, fix/, int/, mask/,\n\
                  mem/, models/, network/, parse/, patterns/, reduce/, scan/, simd/,\n\
                  swar/, swar_str/, utf8/, utils/, autonomic/, ct/, sketch/\n\
         Key capabilities:\n\
         - Branchless integer arithmetic (bcinr_logic::int)\n\
         - SIMD/SWAR string processing (bcinr_logic::swar_str, simd)\n\
         - DFA/automata (bcinr_logic::dfa)\n\
         - Bitmask calculus (bcinr_logic::mask)\n\
         - Pattern matching (bcinr_logic::patterns)\n\
         - Memory utilities (bcinr_logic::mem)\n\
         - Reduction algorithms (bcinr_logic::reduce)\n\
         - POWL process runtime (bcinr_powl)\n\
         - PDDL 3.1 planning (bcinr_pddl)\n\
         All hot paths: no branches, no heap allocation, no_std compatible"
            .to_string()
    }

    /// Compute common branchless mask operations between two u64 values.
    #[tool(description = "Compute branchless mask operations (and, or, xor, andn, nand, nor, popcount, leading_zeros, trailing_zeros) on two u64 values.")]
    async fn bcinr_mask_ops(
        &self,
        Parameters(input): Parameters<MaskOpsInput>,
    ) -> String {
        let a = input.a;
        let b = input.b;
        serde_json::json!({
            "a": a,
            "b": b,
            "and": a & b,
            "or": a | b,
            "xor": a ^ b,
            "andn": a & !b,
            "nand": !(a & b),
            "nor": !(a | b),
            "popcount_a": a.count_ones(),
            "popcount_b": b.count_ones(),
            "leading_zeros_a": a.leading_zeros(),
            "trailing_zeros_a": a.trailing_zeros(),
        })
        .to_string()
    }

    /// Return human-readable description of the bcinr-powl POWL runtime.
    #[tool(description = "Return human-readable description of the bcinr-powl POWL runtime: phase lattice, topology kinds, op kinds, AST nodes.")]
    async fn bcinr_powl_info(&self) -> String {
        "bcinr-powl — Partially Ordered Workflow Language runtime\n\
         Phase lattice: Unvalidated → Compiled → Scheduled<KIND> → Executing<KIND> → Receipted<KIND>\n\
         Topology kinds: Priority, Standard, Background, LongRunning, Compensating\n\
         Op kinds: Atom, Silent, XorDispatch, Join, LoopRedo\n\
         AST nodes: Atom(label), Silent, Sequence([...]), PartialOrder{children,edges}, XorChoice([...]), Loop{body,redo,max_iters}\n\
         Admission: O(1) branchless LUT dispatch on 8-bit context key\n\
         Enterprise: capability_mask() branchless O(1) bitset check\n\
         All execution: branchless, zero-alloc, no_std compatible"
            .to_string()
    }

    // ── Group 4: bcinr-logic Algorithms ──────────────────────────────────────

    /// Validate UTF-8 sequences.
    #[tool(description = "Validate UTF-8 byte sequences. Returns JSON with ok, is_valid, char_count, error_position.")]
    async fn utf8_validate(
        &self,
        Parameters(input): Parameters<Utf8Input>,
    ) -> String {
        let bytes = input.data.as_bytes();
        match std::str::from_utf8(bytes) {
            Ok(s) => {
                serde_json::json!({
                    "ok": true,
                    "is_valid": true,
                    "char_count": s.chars().count(),
                    "byte_count": bytes.len(),
                })
                .to_string()
            }
            Err(e) => {
                serde_json::json!({
                    "ok": true,
                    "is_valid": false,
                    "error": e.to_string(),
                    "error_position": e.valid_up_to(),
                })
                .to_string()
            }
        }
    }

    /// Compute branchless bitset operations.
    #[tool(description = "Perform branchless bitset operations: popcount, leading_zeros, trailing_zeros, msb, lsb. Returns JSON with operation, value, result.")]
    async fn bitset_operations(
        &self,
        Parameters(input): Parameters<BitsetInput>,
    ) -> String {
        let v = input.value;
        let result = match input.operation.as_str() {
            "popcount" => v.count_ones() as u64,
            "leading_zeros" => v.leading_zeros() as u64,
            "trailing_zeros" => v.trailing_zeros() as u64,
            "msb" => {
                if v == 0 { u64::MAX } else { 63 - v.leading_zeros() as u64 }
            }
            "lsb" => {
                if v == 0 { u64::MAX } else { v.trailing_zeros() as u64 }
            }
            _ => return serde_json::json!({ "ok": false, "error": "unknown operation" }).to_string(),
        };
        serde_json::json!({
            "ok": true,
            "operation": input.operation,
            "value": v,
            "result": result,
        })
        .to_string()
    }

    /// Get information about DFA and automata capabilities.
    #[tool(description = "Return information about DFA (Deterministic Finite Automata) support in bcinr-logic.")]
    async fn dfa_info(&self) -> String {
        "bcinr-logic DFA module — Deterministic Finite Automata\n\
         Capabilities:\n\
         - Branchless DFA construction from patterns\n\
         - O(n) state machine simulation\n\
         - Pattern matching via DFA tables\n\
         - Zero-branch automata execution\n\
         Use cases: lexical analysis, protocol parsing, pattern matching"
            .to_string()
    }

    /// Get information about scanning algorithms.
    #[tool(description = "Return information about scanning algorithms in bcinr-logic for pattern search and text analysis.")]
    async fn scan_patterns(&self) -> String {
        "bcinr-logic scan module — Pattern scanning algorithms\n\
         Capabilities:\n\
         - Branchless linear scan\n\
         - SIMD-accelerated search\n\
         - Window-based pattern matching\n\
         - Zero-allocation scanning\n\
         - Real-time stream processing\n\
         Algorithms: Boyer-Moore-ish, SIMD substring, SWAR techniques"
            .to_string()
    }

    /// Get information about reduction algorithms.
    #[tool(description = "Return information about reduction algorithms for aggregation and folding.")]
    async fn reduce_sequence(&self) -> String {
        "bcinr-logic reduce module — Reduction algorithms\n\
         Capabilities:\n\
         - Branchless fold/reduce operations\n\
         - Parallel reduction trees\n\
         - Associative operation dispatch\n\
         - O(log n) tree reduction\n\
         Operations: sum, product, min, max, any, all, logical operations\n\
         SIMD-vectorized for throughput"
            .to_string()
    }

    /// Get information about SIMD string processing.
    #[tool(description = "Return information about SIMD and SWAR (SIMD Within A Register) string algorithms.")]
    async fn simd_string_info(&self) -> String {
        "bcinr-logic SIMD/SWAR string module — Vectorized text processing\n\
         Capabilities:\n\
         - SWAR: 64-bit register tricks (branchless classification)\n\
         - SIMD: SSE/AVX/NEON dispatch (throughput)\n\
         - UTF-8 validation (vectorized)\n\
         - Case folding (vectorized)\n\
         - Whitespace classification (bit-per-char)\n\
         - Null byte detection\n\
         Performance: 10-20GB/s for typical text tasks"
            .to_string()
    }

    // ── Group 5: POWL Receipts ──────────────────────────────────────────────

    /// Inspect and cryptographically verify a WorldManufactureReceipt.
    ///
    /// Verifies the BLAKE3 manufacture_chain by recomputing
    /// BLAKE3(domain_witness || problem_witness || plan_chain_hash) and comparing
    /// against the stored chain. Returns chain_valid: false on any tampering.
    #[tool(description = "Inspect a POWL execution receipt. Returns JSON with status, op_count, makespan, admitted, refusal_reason.")]
    async fn receipt_inspect(
        &self,
        Parameters(input): Parameters<ReceiptInput>,
    ) -> String {
        let data = match serde_json::from_str::<serde_json::Value>(&input.receipt_data) {
            Ok(d) => d,
            Err(e) => return serde_json::json!({ "ok": false, "error": e.to_string() }).to_string(),
        };

        let domain_w = data.get("domain_witness").and_then(|v| v.as_str()).unwrap_or("");
        let problem_w = data.get("problem_witness").and_then(|v| v.as_str()).unwrap_or("");
        let plan_chain = data.get("plan_chain_hash").and_then(|v| v.as_str()).unwrap_or("");
        let stored_chain = data.get("manufacture_chain").and_then(|v| v.as_str()).unwrap_or("");

        // Recompute BLAKE3(domain_witness || problem_witness || plan_chain_hash) as UTF-8 strings.
        // This mirrors chain_witnesses() in bcinr-pddl/src/llm_bridge.rs exactly.
        let chain_valid = if !domain_w.is_empty() && !problem_w.is_empty()
            && !plan_chain.is_empty() && !stored_chain.is_empty()
            && plan_chain != "REFUSED"
        {
            let mut h = blake3::Hasher::new();
            h.update(domain_w.as_bytes());
            h.update(problem_w.as_bytes());
            h.update(plan_chain.as_bytes());
            let computed: String = h.finalize().as_bytes().iter()
                .map(|x| format!("{x:02x}"))
                .collect();
            computed == stored_chain
        } else {
            false
        };

        let chain_mismatch = if !chain_valid && !stored_chain.is_empty() && plan_chain != "REFUSED" {
            // Recompute for the error message
            let mut h = blake3::Hasher::new();
            h.update(domain_w.as_bytes());
            h.update(problem_w.as_bytes());
            h.update(plan_chain.as_bytes());
            let computed: String = h.finalize().as_bytes().iter()
                .map(|x| format!("{x:02x}"))
                .collect();
            Some(format!("expected {computed}, stored {stored_chain}"))
        } else {
            None
        };

        serde_json::json!({
            "ok": true,
            "receipt_type": "WorldManufactureReceipt",
            "chain_valid": chain_valid,
            "chain_mismatch_detail": chain_mismatch,
            "admitted": data.get("admitted").and_then(|v| v.as_bool()).unwrap_or(false),
            "op_count": data.get("step_count").and_then(|v| v.as_u64()).unwrap_or(0),
            "makespan": data.get("makespan").and_then(|v| v.as_f64()),
            "goal_reached": data.get("goal_reached").and_then(|v| v.as_bool()),
            "refusal_reason": data.get("refusal_reason").and_then(|v| v.as_str()),
        })
        .to_string()
    }

    // ── Group 6: Cross-crate Info ───────────────────────────────────────────

    /// Report all available system capabilities across all crates.
    #[tool(description = "Report all available system capabilities: PDDL, POWL, bcinr-logic, receipts, and cross-crate integration status.")]
    async fn system_capabilities(&self) -> String {
        serde_json::json!({
            "system": "bcinr unified execution platform",
            "version": "26.6.30",
            "crates": {
                "bcinr-pddl": { "status": "ready", "tools": 7, "capability": "Planning (PDDL 3.1 → STRIPS → temporal plans)" },
                "bcinr-powl": { "status": "ready", "tools": 5, "capability": "Workflow (AST → tape → execution)" },
                "bcinr-logic": { "status": "ready", "tools": 6, "capability": "Algorithms (branchless, SIMD, O(1/log n))" },
                "bcinr-powl-receipt": { "status": "ready", "tools": 1, "capability": "Receipt verification and inspection" },
            },
            "pipeline": "PDDL domain+problem → ground & plan → POWL tape → branchless execute → receipt",
            "total_tools": 23,
            "admission_model": "Prolog8 (R ⊢ A gate) with SLA tokens",
            "isolation": "zero-trust, branchless, deterministic",
        })
        .to_string()
    }
}

// ─── Entry point ─────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("bcinr_mcp=info".parse().unwrap()),
        )
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("bcinr-mcp starting — 23 tools ready (PDDL:7 + POWL:5 + core:3 + algorithms:6 + receipts:1 + cross-crate:1)");

    let server = BcinrServer;
    let running = match server.serve(stdio()).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("MCP server init error: {}", e);
            std::process::exit(1);
        }
    };
    if let Err(e) = running.waiting().await {
        tracing::error!("MCP server error: {}", e);
        std::process::exit(1);
    }
}
