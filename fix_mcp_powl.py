import re

with open('crates/bcinr-mcp/src/main.rs', 'r') as f:
    main = f.read()

# powl_plan_to_tape
old_p = """    async fn powl_plan_to_tape(&self, Parameters(input): Parameters<PlanInput>) -> String {
        use bcinr_pddl::powl_bridge::temporal_plan_to_powl_tape;
        use bcinr_pddl::{
            domain_from_pddl, problem_from_pddl, GroundProblem, GroundTemporalProblem,
            TemporalPlan, TemporalPlanStep,
        };

        let domain = match domain_from_pddl(&input.domain_text) {
            Err(e) => {
                return serde_json::json!({ "ok": false, "error": e.to_string() }).to_string()
            }
            Ok(d) => d,
        };
        let problem = match problem_from_pddl(&input.problem_text) {
            Err(e) => {
                return serde_json::json!({ "ok": false, "error": e.to_string() }).to_string()
            }
            Ok(p) => p,
        };

        let temporal_plan = if !domain.durative_actions.is_empty() {
            // Real temporal planning for domains with durative actions.
            let ground = match GroundTemporalProblem::build(&domain, &problem) {
                Err(e) => {
                    return serde_json::json!({ "ok": false, "error": e.to_string() }).to_string()
                }
                Ok(g) => g,
            };
            let plan = match bcinr_pddl::temporal_plan(&ground) {
                Err(e) => {
                    return serde_json::json!({ "ok": false, "error": e.to_string() }).to_string()
                }
                Ok(p) => p,
            };
            plan
        } else {
            // STRIPS fallback: mock temporal plan where each step takes 1.0 time.
            let ground = match GroundProblem::build(&domain, &problem) {
                Err(e) => {
                    return serde_json::json!({ "ok": false, "error": e.to_string() }).to_string()
                }
                Ok(g) => g,
            };
            let plan = match bcinr_pddl::strips_plan(&ground) {
                Err(e) => {
                    return serde_json::json!({ "ok": false, "error": e.to_string() }).to_string()
                }
                Ok(p) => p,
            };
            let steps = plan
                .steps
                .into_iter()
                .enumerate()
                .map(|(i, action)| TemporalPlanStep {
                    start_time: i as f64,
                    action,
                    duration: 1.0,
                })
                .collect();
            TemporalPlan { steps }
        };

        let tape = temporal_plan_to_powl_tape(&temporal_plan);
        serde_json::json!({
            "ok": true,
            "op_count": tape.len(),
            "ops": tape,
        })
        .to_string()
    }"""
new_p = """    async fn powl_plan_to_tape(&self, Parameters(input): Parameters<PlanInput>) -> String {
        Self::powl_plan_to_tape_impl(&input.domain_text, &input.problem_text)
    }
}
impl BcinrMcpServer {
    fn powl_plan_to_tape_impl(domain_text: &str, problem_text: &str) -> String {
        use bcinr_pddl::powl_bridge::temporal_plan_to_powl_tape;
        use bcinr_pddl::{
            domain_from_pddl, problem_from_pddl, GroundProblem, GroundTemporalProblem,
            TemporalPlan, TemporalPlanStep,
        };

        let domain = match domain_from_pddl(domain_text) {
            Err(e) => return serde_json::json!({ "ok": false, "error": e.to_string() }).to_string(),
            Ok(d) => d,
        };
        let problem = match problem_from_pddl(problem_text) {
            Err(e) => return serde_json::json!({ "ok": false, "error": e.to_string() }).to_string(),
            Ok(p) => p,
        };

        let temporal_plan = if !domain.durative_actions.is_empty() {
            let ground = match GroundTemporalProblem::build(&domain, &problem) {
                Err(e) => return serde_json::json!({ "ok": false, "error": e.to_string() }).to_string(),
                Ok(g) => g,
            };
            match bcinr_pddl::temporal_plan(&ground) {
                Err(e) => return serde_json::json!({ "ok": false, "error": e.to_string() }).to_string(),
                Ok(p) => p,
            }
        } else {
            let ground = match GroundProblem::build(&domain, &problem) {
                Err(e) => return serde_json::json!({ "ok": false, "error": e.to_string() }).to_string(),
                Ok(g) => g,
            };
            let plan = match bcinr_pddl::strips_plan(&ground) {
                Err(e) => return serde_json::json!({ "ok": false, "error": e.to_string() }).to_string(),
                Ok(p) => p,
            };
            let steps = plan.steps.into_iter().enumerate().map(|(i, action)| TemporalPlanStep {
                start_time: i as f64,
                action,
                duration: 1.0,
            }).collect();
            TemporalPlan { steps }
        };

        let tape = temporal_plan_to_powl_tape(&temporal_plan);
        serde_json::json!({ "ok": true, "op_count": tape.len(), "ops": tape }).to_string()
    }"""
main = main.replace(old_p, new_p)

with open('crates/bcinr-mcp/src/main.rs', 'w') as f:
    f.write(main)
