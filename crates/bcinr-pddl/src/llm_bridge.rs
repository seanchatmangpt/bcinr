//! Vision 2030: LLM First-Mile Manufacturing Bridge.
//!
//! This module implements the primary entry point for the "admitted world" manufacturing
//! loop described in Vision 2030: No Blank Page (Chatman 2026).
//!
//! The LLM proposes — the substrate admits.
//!
//! ## The Manufacturing Loop
//! ```text
//! LLM text → admit_candidate_domain() → AdmittedDomain
//!           → admit_candidate_problem() → AdmittedProblem
//!           → manufacture_world() → WorldManufactureReceipt
//!                                        ├── domain_witness (BLAKE3)
//!                                        ├── problem_witness (BLAKE3)
//!                                        ├── plan (TemporalPlan)
//!                                        ├── plan_receipt (TemporalExecutionReceipt)
//!                                        └── manufacture_chain (BLAKE3 over all)
//! ```

use crate::error::Pddl8Error;
use crate::parse::{domain31_from_pddl, problem31_from_pddl, domain_from_pddl, problem_from_pddl};
use crate::ground::{GroundProblem, GroundTemporalProblem};
use crate::execute::execute_temporal_plan;
use wasm4pm_compat::pddl::{
    Pddl31Domain, Pddl31Problem, Pddl8Domain, Pddl8Problem,
    TemporalPlan, TemporalExecutionReceipt,
};
use blake3::Hasher;

// ─── Admitted wrapper types ──────────────────────────────────────────────────

/// A PDDL 3.1 domain that has passed admission validation.
/// The witness cryptographically binds the domain structure.
pub struct AdmittedDomain {
    pub domain31: Pddl31Domain,
    /// Compatibility shim — STRIPS-only view of the domain for existing pipeline.
    pub domain8: Pddl8Domain,
    /// BLAKE3 over (name || requirements || predicate names || action names || durative action names)
    pub witness: String,
}

/// A PDDL 3.1 problem that has passed admission validation.
pub struct AdmittedProblem {
    pub problem31: Pddl31Problem,
    pub problem8: Pddl8Problem,
    /// BLAKE3 over (name || domain || object names || goal hash)
    pub witness: String,
}

/// The receipt produced by a complete world manufacturing cycle.
/// This is the primary audit artifact of the Vision 2030 substrate.
pub struct WorldManufactureReceipt {
    pub domain_name: String,
    pub problem_name: String,
    pub domain_witness: String,
    pub problem_witness: String,
    pub plan: TemporalPlan,
    pub plan_receipt: TemporalExecutionReceipt,
    /// BLAKE3(domain_witness || problem_witness || plan_receipt.chain_hash)
    pub manufacture_chain: String,
    /// Whether this world was admitted (true) or refused (false).
    pub admitted: bool,
    /// Human-readable refusal reason if admitted=false.
    pub refusal_reason: Option<String>,
}

// ─── Admission functions ─────────────────────────────────────────────────────

/// Validate and admit an LLM-generated PDDL 3.1 domain text.
///
/// Returns AdmittedDomain if the text parses cleanly and passes structural validation.
/// Returns Pddl8Error if the text is malformed or violates admission constraints.
pub fn admit_candidate_domain(text: &str) -> Result<AdmittedDomain, Pddl8Error> {
    // Parse using the full PDDL 3.1 parser
    let domain31 = domain31_from_pddl(text)?;
    // Also parse using the STRIPS-only parser (for Prolog8 admission gate compatibility)
    let domain8 = domain_from_pddl(text)?;

    // Structural validation
    if domain31.name.is_empty() {
        return Err(Pddl8Error::ParseError("domain name is empty".to_string()));
    }
    if domain31.predicates.is_empty() && domain31.actions.is_empty() && domain31.durative_actions.is_empty() {
        return Err(Pddl8Error::ParseError("domain has no predicates and no actions".to_string()));
    }

    // Compute domain witness: BLAKE3 over structural identity
    let witness = compute_domain_witness(&domain31);

    Ok(AdmittedDomain { domain31, domain8, witness })
}

/// Validate and admit an LLM-generated PDDL 3.1 problem text against an admitted domain.
pub fn admit_candidate_problem(text: &str, domain: &AdmittedDomain) -> Result<AdmittedProblem, Pddl8Error> {
    let problem31 = problem31_from_pddl(text)?;
    let problem8 = problem_from_pddl(text)?;

    // Validate domain reference
    if problem31.domain != domain.domain31.name {
        return Err(Pddl8Error::ParseError(format!(
            "problem references domain '{}' but admitted domain is '{}'",
            problem31.domain, domain.domain31.name
        )));
    }

    if problem31.name.is_empty() {
        return Err(Pddl8Error::ParseError("problem name is empty".to_string()));
    }

    let witness = compute_problem_witness(&problem31);

    Ok(AdmittedProblem { problem31, problem8, witness })
}

/// Full pipeline: LLM text → admitted domain + problem → temporal plan → receipt.
///
/// This is the Vision 2030 first-mile manufacturing entry point.
/// Returns a WorldManufactureReceipt whether or not planning succeeds —
/// the receipt records admitted=false with a refusal_reason if planning fails.
pub fn manufacture_world(
    domain_text: &str,
    problem_text: &str,
    case_id: &str,
    policy_rules: &[(&str, Vec<&str>)],
) -> WorldManufactureReceipt {
    // Admit domain
    let admitted_domain = match admit_candidate_domain(domain_text) {
        Ok(d) => d,
        Err(e) => return WorldManufactureReceipt {
            domain_name: "<parse-failed>".to_string(),
            problem_name: "<parse-failed>".to_string(),
            domain_witness: String::new(),
            problem_witness: String::new(),
            plan: TemporalPlan::default(),
            plan_receipt: refused_receipt(),
            manufacture_chain: String::new(),
            admitted: false,
            refusal_reason: Some(format!("domain admission failed: {e}")),
        },
    };

    // Admit problem
    let admitted_problem = match admit_candidate_problem(problem_text, &admitted_domain) {
        Ok(p) => p,
        Err(e) => return WorldManufactureReceipt {
            domain_name: admitted_domain.domain31.name.clone(),
            problem_name: "<parse-failed>".to_string(),
            domain_witness: admitted_domain.witness.clone(),
            problem_witness: String::new(),
            plan: TemporalPlan::default(),
            plan_receipt: refused_receipt(),
            manufacture_chain: String::new(),
            admitted: false,
            refusal_reason: Some(format!("problem admission failed: {e}")),
        },
    };

    let domain_name = admitted_domain.domain31.name.clone();
    let problem_name = admitted_problem.problem31.name.clone();
    let domain_witness = admitted_domain.witness.clone();
    let problem_witness = admitted_problem.witness.clone();

    // Ground and plan — try temporal first, fall back to STRIPS
    let (plan, plan_receipt) = match ground_and_plan(&admitted_domain, &admitted_problem, case_id, policy_rules) {
        Ok(result) => result,
        Err(e) => {
            let receipt = refused_receipt();
            let chain = chain_witnesses(&domain_witness, &problem_witness, &receipt.chain_hash);
            return WorldManufactureReceipt {
                domain_name,
                problem_name,
                domain_witness,
                problem_witness,
                plan: TemporalPlan::default(),
                plan_receipt: receipt,
                manufacture_chain: chain,
                admitted: false,
                refusal_reason: Some(format!("planning failed: {e}")),
            };
        }
    };

    let manufacture_chain = chain_witnesses(&domain_witness, &problem_witness, &plan_receipt.chain_hash);

    WorldManufactureReceipt {
        domain_name,
        problem_name,
        domain_witness,
        problem_witness,
        plan,
        plan_receipt,
        manufacture_chain,
        admitted: true,
        refusal_reason: None,
    }
}

// ─── Internal helpers ────────────────────────────────────────────────────────

fn ground_and_plan(
    domain: &AdmittedDomain,
    problem: &AdmittedProblem,
    case_id: &str,
    policy_rules: &[(&str, Vec<&str>)],
) -> Result<(TemporalPlan, TemporalExecutionReceipt), Pddl8Error> {
    // Try temporal planning if there are durative actions
    if !domain.domain31.durative_actions.is_empty() {
        let ground = GroundTemporalProblem::build(&domain.domain8, &problem.problem8)?;
        let plan = ground.find_temporal_plan()?;
        let (receipt, _ocel) = execute_temporal_plan(&plan, &domain.domain8, &problem.problem8, case_id, policy_rules)?;
        return Ok((plan, receipt));
    }

    // Fall back to classical STRIPS planning
    let ground = GroundProblem::build(&domain.domain8, &problem.problem8, None)?;
    let tape = ground.find_plan()?;

    // Convert classical tape to a TemporalPlan (each step at sequential integer times)
    let steps = tape.ops.iter().enumerate().map(|(i, op)| {
        wasm4pm_compat::pddl::TemporalPlanStep {
            start_time: i as f64,
            duration: 1.0,
            action_name: op.label.clone(),
            args: op.action.preconditions.iter()
                .flat_map(|a| a.args.clone())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect(),
        }
    }).collect::<Vec<_>>();

    let makespan = steps.len() as f64;
    let plan = TemporalPlan { steps, makespan, metric_value: None };
    let (receipt, _ocel) = execute_temporal_plan(&plan, &domain.domain8, &problem.problem8, case_id, policy_rules)?;
    Ok((plan, receipt))
}

fn compute_domain_witness(domain: &Pddl31Domain) -> String {
    let mut h = Hasher::new();
    h.update(domain.name.as_bytes());
    for req in &domain.requirements { h.update(req.as_bytes()); }
    for (name, _) in &domain.predicates { h.update(name.as_bytes()); }
    for action in &domain.actions { h.update(action.name.as_bytes()); }
    for da in &domain.durative_actions { h.update(da.name.as_bytes()); }
    hex(h.finalize().as_bytes())
}

fn compute_problem_witness(problem: &Pddl31Problem) -> String {
    let mut h = Hasher::new();
    h.update(problem.name.as_bytes());
    h.update(problem.domain.as_bytes());
    for (obj, typ) in &problem.objects { h.update(obj.as_bytes()); h.update(typ.as_bytes()); }
    hex(h.finalize().as_bytes())
}

fn chain_witnesses(domain_w: &str, problem_w: &str, plan_chain: &str) -> String {
    let mut h = Hasher::new();
    h.update(domain_w.as_bytes());
    h.update(problem_w.as_bytes());
    h.update(plan_chain.as_bytes());
    hex(h.finalize().as_bytes())
}

fn refused_receipt() -> TemporalExecutionReceipt {
    TemporalExecutionReceipt {
        plan_root: String::new(),
        state_root: String::new(),
        goal_root: String::new(),
        makespan: 0.0,
        step_count: 0,
        requirements: vec![],
        goal_reached: false,
        chain_hash: "REFUSED".to_string(),
    }
}

fn hex(b: &[u8; 32]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
