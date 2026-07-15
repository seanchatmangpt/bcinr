//! Bounds enforcement — Need9 detection and PDDL8 bound checks.
//!
//! ANDON law: checks_run = ∅ → ANDON. No silent empties.
//! A BoundReport with no checks_run is not PASS — it is ANDON.

use serde::{Deserialize, Serialize};

pub const MAX_ARITY: usize = 8;
pub const MAX_PARAMS: usize = 8;
pub const MAX_CONJUNCTS: usize = 8;
pub const MAX_PLAN_DEPTH: usize = 64;
pub const MAX_GROUND: usize = 4096;
pub const MAX_WORK_UNIT_TASKS: usize = 8;
pub const MAX_ACTION_PRECONDITIONS: usize = 8;
pub const MAX_ACTION_EFFECTS: usize = 8;
pub const MAX_ACTION_PARAMS: usize = 8;
pub const MAX_GOAL_ATOMS: usize = 8;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BoundReportStatus {
    /// All required checks ran and found no violations.
    Pass,
    /// At least one bound was exceeded.
    Refused,
    /// No checks ran — the report is not trustworthy.
    Andon,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BoundKind {
    WorkUnitTasks,
    ActionParameters,
    ActionPreconditions,
    ActionEffects,
    GoalAtoms,
    PredicateArity,
    PlanDepth,
    GroundActions,
    BuildConcurrency,
    ResourceEnvelope,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundViolation {
    pub kind: BoundKind,
    pub actual: usize,
    pub limit: usize,
    pub name: String,
}

impl BoundViolation {
    pub fn is_need9(&self) -> bool {
        matches!(self.kind, BoundKind::WorkUnitTasks)
    }

    pub fn diagnostic_code(&self) -> &'static str {
        match self.kind {
            BoundKind::WorkUnitTasks => "WORK_UNIT_NEED9",
            BoundKind::ActionParameters => "ACTION_PARAMETER_OVERFLOW",
            BoundKind::ActionPreconditions => "ACTION_PRECONDITION_OVERFLOW",
            BoundKind::ActionEffects => "ACTION_EFFECT_OVERFLOW",
            BoundKind::GoalAtoms => "GOAL_ATOM_OVERFLOW",
            BoundKind::PredicateArity => "PREDICATE_ARITY_OVERFLOW",
            BoundKind::PlanDepth => "PLAN_DEPTH_OVERFLOW",
            BoundKind::GroundActions => "GROUND_ACTION_OVERFLOW",
            BoundKind::BuildConcurrency => "BUILD_CONCURRENCY_NEED9",
            BoundKind::ResourceEnvelope => "RESOURCE_ENVELOPE_EXCEEDED",
        }
    }

    pub fn message(&self) -> String {
        format!(
            "{} in '{}': {} (limit {}). Split or decompose.",
            self.diagnostic_code(), self.name, self.actual, self.limit,
        )
    }
}

/// Aggregated bound report with evidence that checks actually ran.
///
/// ANDON law: status = Andon when checks_run is empty.
/// A clean check requires checks_run non-empty AND violations empty.
#[derive(Debug, Clone)]
pub struct BoundReport {
    pub status: BoundReportStatus,
    /// Names of checks that executed (evidence the check ran).
    pub checks_run: Vec<String>,
    pub violations: Vec<BoundViolation>,
    /// Required checks that were not executed (missing coverage).
    pub missing_checks: Vec<String>,
}

impl Default for BoundReport {
    fn default() -> Self {
        // Default is ANDON — no checks ran
        Self {
            status: BoundReportStatus::Andon,
            checks_run: vec![],
            violations: vec![],
            missing_checks: vec!["no_checks_registered".into()],
        }
    }
}

impl BoundReport {
    pub fn is_clean(&self) -> bool {
        self.status == BoundReportStatus::Pass
    }

    pub fn has_need9(&self) -> bool {
        self.violations.iter().any(|v| v.is_need9())
    }

    /// Compute status from checks_run and violations.
    pub fn finalize(checks_run: Vec<String>, violations: Vec<BoundViolation>) -> Self {
        let status = if checks_run.is_empty() {
            BoundReportStatus::Andon
        } else if violations.is_empty() {
            BoundReportStatus::Pass
        } else {
            BoundReportStatus::Refused
        };
        Self { status, checks_run, violations, missing_checks: vec![] }
    }

    pub fn andon(reason: &str) -> Self {
        Self {
            status: BoundReportStatus::Andon,
            checks_run: vec![],
            violations: vec![],
            missing_checks: vec![reason.to_string()],
        }
    }
}

/// Check a work-unit task count for Need9.
pub fn check_work_unit(name: &str, task_count: usize) -> Option<BoundViolation> {
    if task_count > MAX_WORK_UNIT_TASKS {
        Some(BoundViolation {
            kind: BoundKind::WorkUnitTasks,
            actual: task_count,
            limit: MAX_WORK_UNIT_TASKS,
            name: name.to_string(),
        })
    } else {
        None
    }
}

/// Check concurrent build count against MAX_HEAVY_SLOTS.
pub fn check_build_concurrency(name: &str, concurrent_count: usize, max_slots: usize) -> Option<BoundViolation> {
    if concurrent_count > max_slots {
        Some(BoundViolation {
            kind: BoundKind::BuildConcurrency,
            actual: concurrent_count,
            limit: max_slots,
            name: name.to_string(),
        })
    } else {
        None
    }
}

/// Check a parsed PDDL8 domain for bound violations.
///
/// Runs: preconditions, effects, params, goal atoms per action.
/// Returns ANDON if the domain has zero actions (empty domain is a defect).
pub fn check_domain(domain: &wasm4pm_compat::pddl::Pddl8Domain) -> BoundReport {
    if domain.actions.is_empty() {
        return BoundReport::andon("LIFECYCLE_DOMAIN_EMPTY: domain has 0 actions");
    }

    let mut checks_run: Vec<String> = Vec::new();
    let mut violations: Vec<BoundViolation> = Vec::new();

    for action in &domain.actions {
        // Check preconditions
        checks_run.push(format!("preconditions:{}", action.name));
        if action.preconditions.len() > MAX_ACTION_PRECONDITIONS {
            violations.push(BoundViolation {
                kind: BoundKind::ActionPreconditions,
                actual: action.preconditions.len(),
                limit: MAX_ACTION_PRECONDITIONS,
                name: action.name.clone(),
            });
        }

        // Check add effects
        checks_run.push(format!("add_effects:{}", action.name));
        if action.add_effects.len() > MAX_ACTION_EFFECTS {
            violations.push(BoundViolation {
                kind: BoundKind::ActionEffects,
                actual: action.add_effects.len(),
                limit: MAX_ACTION_EFFECTS,
                name: action.name.clone(),
            });
        }

        // Check params
        checks_run.push(format!("params:{}", action.name));
        if action.params.len() > MAX_ACTION_PARAMS {
            violations.push(BoundViolation {
                kind: BoundKind::ActionParameters,
                actual: action.params.len(),
                limit: MAX_ACTION_PARAMS,
                name: action.name.clone(),
            });
        }
    }

    // Check predicate arity
    for (pred_name, arity) in &domain.predicates {
        checks_run.push(format!("predicate_arity:{pred_name}"));
        if *arity as usize > MAX_ARITY {
            violations.push(BoundViolation {
                kind: BoundKind::PredicateArity,
                actual: *arity as usize,
                limit: MAX_ARITY,
                name: pred_name.clone(),
            });
        }
    }

    BoundReport::finalize(checks_run, violations)
}

/// Check a PDDL8 problem for bound violations (goal atoms).
pub fn check_problem(problem: &wasm4pm_compat::pddl::Pddl8Problem) -> BoundReport {
    if problem.goal.is_empty() {
        return BoundReport::andon("PDDL_GOAL_EMPTY: problem has no goal atoms");
    }
    if problem.init.is_empty() {
        return BoundReport::andon("PDDL_INIT_EMPTY: problem has no init atoms");
    }

    let mut checks_run = vec!["goal_atoms".to_string(), "init_atoms".to_string()];
    let mut violations = vec![];

    if problem.goal.len() > MAX_GOAL_ATOMS {
        violations.push(BoundViolation {
            kind: BoundKind::GoalAtoms,
            actual: problem.goal.len(),
            limit: MAX_GOAL_ATOMS,
            name: problem.name.clone(),
        });
    }

    BoundReport::finalize(checks_run, violations)
}

/// Check the PDDL8 lifecycle domain for bound violations.
///
/// Parses the generated domain and runs real precondition/effect/param checks.
/// Returns ANDON (not PASS) if the domain cannot be parsed or has 0 actions.
pub fn check_lifecycle_domain() -> BoundReport {
    use bcinr_pddl::domain_from_pddl;
    let domain_text = crate::projection::emit_domain();
    match domain_from_pddl(&domain_text) {
        Ok(domain) => check_domain(&domain),
        Err(e) => BoundReport::andon(&format!("LIFECYCLE_DOMAIN_PARSE_FAILED: {e:?}")),
    }
}

/// Check a domain from raw PDDL text.
///
/// If the parser itself enforces a bound (BoundExceeded), that counts as a
/// check running and a violation being found — result is REFUSED, not ANDON.
pub fn check_domain_text(text: &str) -> BoundReport {
    use bcinr_pddl::{domain_from_pddl, error::Pddl8Error};
    match domain_from_pddl(text) {
        Ok(domain) => check_domain(&domain),
        Err(Pddl8Error::BoundExceeded { what, limit, got }) => {
            // Parser enforced the bound — this IS a check result, not a stub.
            let kind = if what.contains("precondition") { BoundKind::ActionPreconditions }
                else if what.contains("effect") { BoundKind::ActionEffects }
                else if what.contains("param") { BoundKind::ActionParameters }
                else if what.contains("goal") { BoundKind::GoalAtoms }
                else { BoundKind::ActionPreconditions };
            BoundReport::finalize(
                vec![format!("parser_bound_check:{what}")],
                vec![BoundViolation {
                    kind,
                    actual: got,
                    limit,
                    name: format!("parsed:{what}"),
                }],
            )
        }
        Err(e) => BoundReport::andon(&format!("DOMAIN_PARSE_FAILED: {e:?}")),
    }
}
