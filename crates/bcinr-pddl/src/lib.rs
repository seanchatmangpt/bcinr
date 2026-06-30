//! bcinr-pddl — PDDL8 → POWL tape → Prolog8 admission → OCEL → BLAKE3
//!
//! # BRCE stack position
//! ```text
//! PDDL8   = candidate-future grammar  (parse + ground modules)
//! POWL    = process geometry          (wasm4pm_compat::pddl::Pddl8Tape)
//! Prolog8 = R ⊢ A gate               (execute module)
//! OCEL    = execution trace           (wasm4pm_compat::ocel::OCEL)
//! BLAKE3  = receipt / replay          (wasm4pm_compat::pddl::Pddl8ExecutionReceipt)
//! ```
//!
//! # Isolation guarantee
//! Zero path deps on bcinr-powl or wasm4pm-cognition — the `pddl` parser crate
//! is **only** compiled when this crate is in the dependency graph. PDDL does
//! not bleed into bcinr-powl, wasm4pm, or lsp-max unless they explicitly add
//! bcinr-pddl as a dep.
//!
//! Canonical types live in `wasm4pm_compat::pddl` so any crate can import
//! `Pddl8Tape`, `Pddl8GroundAction`, etc. without pulling in the parser.

pub mod error;
pub mod ground;
pub mod execute;
pub mod parse;
pub mod powl_bridge;
pub mod llm_bridge;
pub use llm_bridge::{AdmittedDomain, AdmittedProblem, WorldManufactureReceipt, admit_candidate_domain, admit_candidate_problem, manufacture_world};

// Re-export canonical types from wasm4pm-compat so callers only need one import.
pub use wasm4pm_compat::pddl::{
    Pddl8ActionSchema, Pddl8Atom, Pddl8Domain, Pddl8ExecutionLog, Pddl8ExecutionReceipt,
    Pddl8GroundAction, Pddl8GroundAtom, Pddl8Problem, Pddl8StepResult, Pddl8Tape, Pddl8TapeOp,
    PDDL8_MAX_ARITY, PDDL8_MAX_CONJUNCTS, PDDL8_MAX_GROUND, PDDL8_MAX_PARAMS,
    PDDL8_MAX_PLAN_DEPTH,
    // New PDDL 3.1 types:
    PddlType, PddlCondition, PddlEffect,
    NumericExpr, NumericOp, NumericEffect, PddlFunction,
    TimeSpecifier, DurationConstraint, DurativeAction,
    TimedLiteral, Metric, MetricDir, MetricExpr,
    TrajectoryConstraint, PddlConstraint, PddlPreference,
    DerivedPredicate, PddlProcess, PddlEvent,
    Pddl31Domain, Pddl31Problem, Pddl31Action,
    TemporalPlanStep, TemporalPlan, TemporalExecutionReceipt,
};
pub use wasm4pm_compat::ocel::{OCEL, OCELEvent};

pub use error::Pddl8Error;
pub use execute::{execute_tape, compute_plan_chain};
pub use ground::{GroundProblem, GroundTemporalProblem, GroundDurativeAction};
pub use parse::{domain_from_pddl, problem_from_pddl, domain31_from_pddl, problem31_from_pddl};
