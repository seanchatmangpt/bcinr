//! Capability admission: what PDDL features this crate's planners *actually*
//! implement correctly, wired into a real admission gate so a caller-chosen
//! [`CapabilityProfile`] can refuse a domain that needs something the
//! profile marks unsupported instead of silently planning against it anyway
//! and producing a plan that quietly ignores part of the domain's semantics.
//!
//! # Per-feature accounting (the honest part)
//!
//! [`DefaultCapabilityProfile`] is not a guess — every rating below reflects
//! code actually read (and, where noted, actually run) this phase:
//!
//! - [`PddlFeature::Strips`], [`PddlFeature::Typing`] — `Exact`. The
//!   foundation; every passing test in this crate exercises both.
//! - [`PddlFeature::NegativePreconditions`] — `Exact`. `eval_condition`'s
//!   `Not` arm is a straightforward `!eval_condition(inner, ...)`, and it is
//!   genuinely load-bearing in a passing test:
//!   `capability_router::tests::same_file_edit_and_draft_conflict_and_sequence`
//!   only passes because `(at start (not (locked ?f)))` really gates
//!   scheduling.
//! - [`PddlFeature::Disjunction`] — `Exact`. `eval_condition`'s `Or` arm
//!   (`.any(...)`) is equally straightforward and reachable through the same
//!   durative-condition / derived-predicate pipeline already proven live for
//!   `Not`/`Compare`/`Timed`.
//! - [`PddlFeature::Equality`] — `Unsupported`. Nothing in this crate
//!   special-cases the built-in `=` predicate (no equality facts are
//!   synthesized, no identity check exists anywhere in `ground/mod.rs`) — a
//!   domain declaring `:equality` gets `=` treated as an arbitrary
//!   uninterpreted predicate name, which is silently wrong, not merely
//!   incomplete.
//! - [`PddlFeature::ExistentialPreconditions`] — `Unsupported`.
//!   `ground::eval_quantifier`'s `Exists` arm is real and directly
//!   unit-tested (`ground::quantifier_tests::exists_*`), but no parser path
//!   in this crate ever constructs a `PddlCondition::Exists` that reaches
//!   `eval_condition`: the `pddl` crate's durative-action-condition grammar
//!   (`da-GD`) has no `exists` production (only `forall` — see
//!   `src/parse.rs`'s `lower_da_gd`), plain `:action` preconditions and
//!   `:goal` can't carry any `PddlCondition` at all (both are flattened to
//!   `Vec<Pddl8Atom>`), and `ground_derived_schema`'s local `ground_condition`
//!   helper drops `Exists` bodies (`_ => None`). A correct-but-unreachable
//!   evaluator is still `Unsupported` at the admission layer — this is the
//!   explicit, honest choice the mission brief allows for a feature whose
//!   evaluator works but has no way to receive real input.
//! - [`PddlFeature::UniversalPreconditions`] — `Approximate`. The mirror
//!   image: `eval_quantifier`'s `Forall` arm *is* reachable, through exactly
//!   one path — a `:durative-action`'s `:condition` — and that path is
//!   proven correct end-to-end by `tests/durative_quantifiers.rs` (including
//!   the adversarial case the pre-fix stub got wrong: one item not ready).
//!   `Approximate`, not `Exact`, because the same feature in a plain
//!   `:action` precondition or `:goal` still silently vanishes.
//! - [`PddlFeature::ConditionalEffects`] — `Unsupported`. A real bug, not a
//!   gap: `ground::apply_effect_ground`'s `PddlEffect::When { condition,
//!   effects }` arm destructures `condition` with `..` and **never
//!   evaluates it** — the effects fire unconditionally. `PddlEffect::Forall`
//!   effects have the same disease (`vars` is discarded, effects apply once
//!   without any substitution/enumeration over objects). Neither bug was
//!   introduced this phase; both are surfaced here rather than silently
//!   inherited as an `Exact`/`Approximate` rating that would overclaim.
//! - [`PddlFeature::NumericFluents`] — `Approximate`. Numeric comparisons
//!   (`eval_numeric`/`Compare`) are genuinely evaluated and load-bearing
//!   (`capability_router`'s `(>= (attention) 1)` gates every real test in
//!   that module) — but only through the durative-action pipeline. Plain
//!   `:action` preconditions cannot carry a numeric comparison at all
//!   (`Pddl8ActionSchema.preconditions: Vec<Pddl8Atom>` has no slot for one),
//!   so `src/parse.rs`'s `collect_gd` silently drops it instead of
//!   rejecting the domain — directly demonstrated by
//!   `tests/semantic_falsifier.rs`'s `test_numeric_cost`, `#[ignore]`d with
//!   this exact citation rather than left to fail unexplained.
//! - [`PddlFeature::NumericEffects`] — `Exact`. `apply_numeric_effect`
//!   correctly implements `Assign`/`Increase`/`Decrease`/`ScaleUp`/
//!   `ScaleDown`, and it is the only numeric-effect surface `:numeric-fluents`
//!   realistically implies in this crate (paired with `:durative-actions`,
//!   the classical `Pddl8GroundAction` has no numeric-effect field at all —
//!   a structural, advertised STRIPS8 scope limit, not a silent gap).
//! - [`PddlFeature::DurativeActions`] — `Exact`. `GroundTemporalProblem` is
//!   the best-tested part of this crate (`tests/capacity.rs`,
//!   `tests/proposer_substrate.rs`, `capability_router`, the DfCM crown
//!   suite all exercise it).
//! - [`PddlFeature::TimedInitialLiterals`] — `Exact`.
//!   `tests/semantic_falsifier.rs`'s `test_til_schedule` passes and directly
//!   checks TIL-driven makespan values.
//! - [`PddlFeature::DerivedPredicates`] — `Approximate`.
//!   `compute_derived_closure`'s fixpoint iteration is real and
//!   `test_derived_predicates` passes — but `ground_derived_schema`'s
//!   `ground_condition` helper has no `Forall`/`Exists` arm, so a derived
//!   predicate whose body quantifies is silently dropped in full (not
//!   partially evaluated), never appearing in `derived_predicates` at all.
//! - [`PddlFeature::TrajectoryConstraints`] — `Unsupported`.
//!   `crate::parse::problem_from_pddl` and `problem31_from_pddl` both
//!   hardcode `preferences: vec![]` — `(:constraints ...)` is never parsed
//!   into `Pddl8Problem`/`Pddl31Problem` at all, by either function, so
//!   `GroundProblem::build`'s/`GroundTemporalProblem::build`'s
//!   `self.constraints` is always empty regardless of what a domain
//!   declares. Directly demonstrated by `test_trajectory_constraints`,
//!   `#[ignore]`d with this citation.
//! - [`PddlFeature::Preferences`] — `Unsupported`. Same root cause
//!   (`preferences: vec![]`, always), and even setting that aside, nothing
//!   in this crate computes soft-constraint violation cost against a metric.
//! - [`PddlFeature::Metrics`] — `Unsupported`. `problem.metric: Option<Metric>`
//!   is parsed but never consulted: both `GroundProblem::find_plan` and
//!   `GroundTemporalProblem::find_temporal_plan_with_fn_overrides`
//!   hardcode `metric_value: None` on every `TemporalPlan` they return, and
//!   classical `find_plan` has no metric field on `Pddl8Tape` to populate at
//!   all — no plan is ever selected or ranked by a metric.
//!
//! None of the above bugs (`ConditionalEffects`, the two `preferences: vec![]`
//! sites, `metric_value: None`) were introduced by this phase — they're
//! reported here because an honest [`CapabilityProfile`] cannot be built
//! without finding them, and finding them without saying so would be exactly
//! the silent-overclaim this module exists to prevent.

use std::collections::BTreeSet;

use bcinr_mfw_ir::{
    Digest, EpochBounds, InconsistencyWitness, PlannerOutcome, PlanningEpochId, UnsupportedFeature,
};
use wasm4pm_compat::pddl::{Pddl31Domain, Pddl31Problem, Pddl8GroundAction, Pddl8GroundAtom};

use crate::ground::GroundProblem;

/// The sixteen PDDL-requirement-shaped capabilities this crate's planners
/// might need. Not a 1:1 mirror of the `pddl` crate's `Requirement` enum —
/// see [`requirement_implies`] for how the wider requirement vocabulary
/// (`Adl`, `Fluents`, `QuantifiedPreconditions`, `ObjectFluents`, ...) maps
/// onto these sixteen (or, for `ObjectFluents`, is rejected structurally
/// instead, since it has no corresponding `PddlFeature`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PddlFeature {
    Strips,
    Typing,
    NegativePreconditions,
    Disjunction,
    Equality,
    ExistentialPreconditions,
    UniversalPreconditions,
    ConditionalEffects,
    NumericFluents,
    NumericEffects,
    DurativeActions,
    TimedInitialLiterals,
    DerivedPredicates,
    TrajectoryConstraints,
    Preferences,
    Metrics,
}

/// All sixteen [`PddlFeature`] variants, in declaration order — used to
/// iterate the full feature set (e.g. when checking a domain's requirements
/// against a profile).
pub const ALL_PDDL_FEATURES: [PddlFeature; 16] = [
    PddlFeature::Strips,
    PddlFeature::Typing,
    PddlFeature::NegativePreconditions,
    PddlFeature::Disjunction,
    PddlFeature::Equality,
    PddlFeature::ExistentialPreconditions,
    PddlFeature::UniversalPreconditions,
    PddlFeature::ConditionalEffects,
    PddlFeature::NumericFluents,
    PddlFeature::NumericEffects,
    PddlFeature::DurativeActions,
    PddlFeature::TimedInitialLiterals,
    PddlFeature::DerivedPredicates,
    PddlFeature::TrajectoryConstraints,
    PddlFeature::Preferences,
    PddlFeature::Metrics,
];

/// How faithfully this crate's planners implement a given [`PddlFeature`].
///
/// This is a closed, four-way classification along the same "never round up
/// silently" discipline as `bcinr_mfw_ir::FormalStanding`:
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticSupport {
    /// Genuinely correct wherever it is reachable, and reachable through
    /// every path a domain declaring the corresponding requirement would
    /// realistically use.
    Exact,
    /// Genuinely correct, but only within a caller-supplied structural
    /// bound (e.g. a search/grounding limit) — not used by
    /// [`DefaultCapabilityProfile`] today (no feature in this crate is
    /// bound-limited rather than simply reachable or not), but part of the
    /// closed vocabulary for profiles built by later phases.
    BoundedExact,
    /// Genuinely correct wherever it is reachable, but reachable through
    /// only part of the surface a domain declaring the requirement might
    /// use (see the module doc comment for exactly which paths, per
    /// feature).
    Approximate,
    /// Not implemented, silently wrong if assumed, or real but practically
    /// unreachable from any parser entry point — never claim a plan
    /// respected this feature.
    Unsupported,
}

/// A policy: what level of support this planner instance claims for each
/// [`PddlFeature`]. Implementations other than [`DefaultCapabilityProfile`]
/// let a caller be *more* conservative (e.g. downgrade `Approximate` to
/// `Unsupported` for a safety-critical deployment) — [`admit_planning_task`]
/// never grants more trust than the profile it is given.
pub trait CapabilityProfile {
    fn support(&self, feature: PddlFeature) -> SemanticSupport;
}

/// The profile reflecting what this crate's planners *actually* implement
/// today, per the module doc comment's per-feature accounting.
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultCapabilityProfile;

impl CapabilityProfile for DefaultCapabilityProfile {
    fn support(&self, feature: PddlFeature) -> SemanticSupport {
        use SemanticSupport::{Approximate, Exact, Unsupported};
        match feature {
            PddlFeature::Strips => Exact,
            PddlFeature::Typing => Exact,
            PddlFeature::NegativePreconditions => Exact,
            PddlFeature::Disjunction => Exact,
            PddlFeature::Equality => Unsupported,
            PddlFeature::ExistentialPreconditions => Unsupported,
            PddlFeature::UniversalPreconditions => Approximate,
            PddlFeature::ConditionalEffects => Unsupported,
            PddlFeature::NumericFluents => Approximate,
            PddlFeature::NumericEffects => Exact,
            PddlFeature::DurativeActions => Exact,
            PddlFeature::TimedInitialLiterals => Exact,
            PddlFeature::DerivedPredicates => Approximate,
            PddlFeature::TrajectoryConstraints => Unsupported,
            PddlFeature::Preferences => Unsupported,
            PddlFeature::Metrics => Unsupported,
        }
    }
}

/// True iff `req` (a `Pddl31Domain::requirements` entry — the `Debug`
/// spelling of the `pddl` crate's `Requirement` enum, e.g.
/// `"NumericFluents"`, **not** the PDDL surface syntax's `:kebab-case`
/// spelling; see `src/parse.rs`'s `domain31_from_pddl`, which populates this
/// field via `format!("{r:?}")`) implies `feature`.
///
/// Shorthand requirements (`Adl`, `Fluents`, `QuantifiedPreconditions`) are
/// expanded to the same constituent set the `pddl` crate's own
/// `Requirement::expand()` uses. `ObjectFluents`/`DurationInequalities`/
/// `ContinuousEffects`/`ActionCosts` never imply any [`PddlFeature`] here —
/// `ObjectFluents` is rejected structurally by [`admit_planning_task`]
/// instead (see that function), and the other three have no corresponding
/// feature in this crate's admission vocabulary at all (this crate does not
/// implement continuous effects or duration inequalities beyond the
/// min/max bounds `resolve_duration` already resolves, and `ActionCosts`'s
/// restricted-metric semantics fall under [`PddlFeature::Metrics`], which is
/// already `Unsupported`).
fn requirement_implies(req: &str, feature: PddlFeature) -> bool {
    match req {
        "Strips" => feature == PddlFeature::Strips,
        "Typing" => feature == PddlFeature::Typing,
        "NegativePreconditions" => feature == PddlFeature::NegativePreconditions,
        "DisjunctivePreconditions" => feature == PddlFeature::Disjunction,
        "Equality" => feature == PddlFeature::Equality,
        "ExistentialPreconditions" => feature == PddlFeature::ExistentialPreconditions,
        "UniversalPreconditions" => feature == PddlFeature::UniversalPreconditions,
        "QuantifiedPreconditions" => matches!(
            feature,
            PddlFeature::ExistentialPreconditions | PddlFeature::UniversalPreconditions
        ),
        "ConditionalEffects" => feature == PddlFeature::ConditionalEffects,
        "Fluents" | "NumericFluents" => {
            matches!(
                feature,
                PddlFeature::NumericFluents | PddlFeature::NumericEffects
            )
        }
        "Adl" => matches!(
            feature,
            PddlFeature::Strips
                | PddlFeature::Typing
                | PddlFeature::NegativePreconditions
                | PddlFeature::Disjunction
                | PddlFeature::Equality
                | PddlFeature::ExistentialPreconditions
                | PddlFeature::UniversalPreconditions
                | PddlFeature::ConditionalEffects
        ),
        "DurativeActions" => feature == PddlFeature::DurativeActions,
        "DerivedPredicates" => feature == PddlFeature::DerivedPredicates,
        // TimedInitialLiterals implies DurativeActions per the `pddl` crate's
        // own doc comment on `Requirement::TimedInitialLiterals`.
        "TimedInitialLiterals" => matches!(
            feature,
            PddlFeature::TimedInitialLiterals | PddlFeature::DurativeActions
        ),
        "Preferences" => feature == PddlFeature::Preferences,
        "Constraints" => feature == PddlFeature::TrajectoryConstraints,
        "ActionCosts" => feature == PddlFeature::Metrics,
        _ => false,
    }
}

/// A domain + problem that passed [`admit_planning_task`]'s structural and
/// capability checks. Cheap to construct further planning stages from —
/// `theory_digest` content-addresses exactly the structural identity used
/// to compute it, matching `crate::llm_bridge::compute_domain_witness`/
/// `compute_problem_witness`'s existing witnessing style but folded into one
/// `bcinr_mfw_ir::Digest` instead of two separate hex strings.
#[derive(Debug, Clone)]
pub struct AdmittedPlanningTask {
    pub domain: Pddl31Domain,
    pub problem: Pddl31Problem,
    pub theory_digest: Digest,
}

/// Structurally validate `domain`/`problem` and check every requirement
/// `domain` declares against `profile`, refusing (`PlannerOutcome::Unsupported`)
/// rather than silently proceeding for anything `profile` marks
/// `SemanticSupport::Unsupported`.
///
/// This does **not** ground or search — it is the admission gate that runs
/// before either, so a domain requiring an unsupported feature never reaches
/// `GroundProblem::build`/`GroundTemporalProblem::build` at all.
pub fn admit_planning_task(
    domain: &Pddl31Domain,
    problem: &Pddl31Problem,
    profile: &dyn CapabilityProfile,
) -> PlannerOutcome<AdmittedPlanningTask> {
    if domain.name.is_empty() {
        return PlannerOutcome::Inconsistent(InconsistencyWitness {
            kind_name: "empty-domain-name".to_string(),
            context: "admit_planning_task: domain.name must be non-empty".to_string(),
            digest: Digest::ZERO,
        });
    }
    if domain.predicates.is_empty()
        && domain.actions.is_empty()
        && domain.durative_actions.is_empty()
    {
        return PlannerOutcome::Inconsistent(InconsistencyWitness {
            kind_name: "empty-domain-structure".to_string(),
            context: "admit_planning_task: domain has no predicates and no actions".to_string(),
            digest: Digest::ZERO,
        });
    }
    if problem.domain != domain.name {
        return PlannerOutcome::Inconsistent(InconsistencyWitness {
            kind_name: "domain-problem-mismatch".to_string(),
            context: format!(
                "admit_planning_task: problem references domain '{}' but admitted domain is '{}'",
                problem.domain, domain.name
            ),
            digest: Digest::ZERO,
        });
    }

    // Structural rejection for `:object-fluents`: no `PddlFeature` variant
    // represents it (see `requirement_implies`'s doc comment) because no
    // object-fluent representation exists anywhere in this grounder —
    // supersedes the old, dead `ground::check_capabilities`, which checked
    // this same requirement but against the wrong string format (`:kebab-case`
    // against a `PascalCase`-populated field, so it could never actually
    // fire) and had no callers.
    if domain.requirements.iter().any(|r| r == "ObjectFluents") {
        return PlannerOutcome::Unsupported(UnsupportedFeature {
            feature_name: "object-fluents".to_string(),
            context: "PDDL 3.1 object-valued fluents have no representation anywhere in this \
                      grounder (only numeric fluents are modeled) — not one of the sixteen \
                      PddlFeature variants because there is nothing partial to rate; the domain \
                      is refused outright."
                .to_string(),
        });
    }

    for req in &domain.requirements {
        for &feature in &ALL_PDDL_FEATURES {
            if requirement_implies(req, feature)
                && profile.support(feature) == SemanticSupport::Unsupported
            {
                return PlannerOutcome::Unsupported(UnsupportedFeature {
                    feature_name: format!("{feature:?}"),
                    context: format!(
                        "domain requirement {req:?} implies PddlFeature::{feature:?}, which the \
                         given CapabilityProfile marks Unsupported"
                    ),
                });
            }
        }
    }

    let theory_digest = domain_problem_digest(domain, problem);
    PlannerOutcome::Found(AdmittedPlanningTask {
        domain: domain.clone(),
        problem: problem.clone(),
        theory_digest,
    })
}

/// Content-addressed digest of `domain`'s + `problem`'s structural identity —
/// same fields `crate::llm_bridge::compute_domain_witness`/
/// `compute_problem_witness` hash, just mixed into one `Digest` via
/// `Digest::mix` instead of two separate hex strings.
fn domain_problem_digest(domain: &Pddl31Domain, problem: &Pddl31Problem) -> Digest {
    let mut dbuf = Vec::new();
    dbuf.extend_from_slice(domain.name.as_bytes());
    for req in &domain.requirements {
        dbuf.extend_from_slice(req.as_bytes());
    }
    for (name, _) in &domain.predicates {
        dbuf.extend_from_slice(name.as_bytes());
    }
    for a in &domain.actions {
        dbuf.extend_from_slice(a.name.as_bytes());
    }
    for da in &domain.durative_actions {
        dbuf.extend_from_slice(da.name.as_bytes());
    }
    let domain_digest = Digest::hash(&dbuf);

    let mut pbuf = Vec::new();
    pbuf.extend_from_slice(problem.name.as_bytes());
    pbuf.extend_from_slice(problem.domain.as_bytes());
    for (obj, typ) in &problem.objects {
        pbuf.extend_from_slice(obj.as_bytes());
        pbuf.extend_from_slice(typ.as_bytes());
    }
    let problem_digest = Digest::hash(&pbuf);

    domain_digest.mix(&problem_digest)
}

/// One bounded PDDL grounding + search run — the PDDL-shaped
/// `GroundedPlanningEpoch` `bcinr_mfw_ir::epoch`'s module doc comment
/// explicitly deferred to this crate ("`GroundedPlanningEpoch` itself...
/// is PDDL-shaped and is left to `bcinr-pddl` to define"), built from the
/// generic bound-tracking primitives [`EpochBounds`]/`bcinr_mfw_ir::DescentMeter`.
#[derive(Debug, Clone)]
pub struct GroundedPlanningEpoch {
    pub id: PlanningEpochId,
    pub theory_digest: Digest,
    pub initial_state: BTreeSet<Pddl8GroundAtom>,
    pub goal: Vec<Pddl8GroundAtom>,
    pub actions: Vec<Pddl8GroundAction>,
    pub bounds: EpochBounds,
}

impl GroundedPlanningEpoch {
    /// Build a `GroundedPlanningEpoch` from an already-grounded classical
    /// `GroundProblem` (`crate::ground::GroundProblem::build`) plus a
    /// caller-chosen `theory_digest` (typically
    /// `AdmittedPlanningTask::theory_digest`) and `bounds`.
    ///
    /// `id` is derived deterministically from `theory_digest`'s first 16
    /// bytes rather than a counter or random nonce: the same domain+problem
    /// (same `theory_digest`) always produces the same epoch id, which is
    /// exactly the property `crate::cache::StandingConsequenceCache` needs
    /// for a cache-hit to mean "the same planning epoch, not just a
    /// coincidentally-equal state."
    pub fn from_ground_problem(
        gp: &GroundProblem,
        theory_digest: Digest,
        bounds: EpochBounds,
    ) -> Self {
        let mut id_bytes = [0u8; 16];
        id_bytes.copy_from_slice(&theory_digest.as_bytes()[..16]);
        Self {
            id: PlanningEpochId(u128::from_le_bytes(id_bytes)),
            theory_digest,
            initial_state: gp.initial_state.clone(),
            goal: gp.goal.clone(),
            actions: gp.actions.clone(),
            bounds,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::{domain31_from_pddl, problem31_from_pddl};

    const STRIPS_DOMAIN: &str = "(define (domain d) (:requirements :strips) (:predicates (p)) \
                                  (:action a :parameters () :precondition (p) :effect (not (p))))";
    const STRIPS_PROBLEM: &str = "(define (problem pr) (:domain d) (:init (p)) (:goal (not (p))))";

    #[test]
    fn strips_domain_is_admitted_under_default_profile() {
        let domain = domain31_from_pddl(STRIPS_DOMAIN).unwrap();
        let problem = problem31_from_pddl(STRIPS_PROBLEM).unwrap();
        let outcome = admit_planning_task(&domain, &problem, &DefaultCapabilityProfile);
        assert!(outcome.is_found());
    }

    #[test]
    fn equality_requirement_is_refused_as_unsupported() {
        let domain = domain31_from_pddl(
            "(define (domain d) (:requirements :strips :equality) (:predicates (p)) \
             (:action a :parameters () :precondition (p) :effect (not (p))))",
        )
        .unwrap();
        let problem = problem31_from_pddl(STRIPS_PROBLEM).unwrap();
        let outcome = admit_planning_task(&domain, &problem, &DefaultCapabilityProfile);
        match outcome {
            PlannerOutcome::Unsupported(u) => assert_eq!(u.feature_name, "Equality"),
            other => panic!("expected Unsupported(Equality), got {other:?}"),
        }
    }

    #[test]
    fn existential_preconditions_requirement_is_refused_as_unsupported() {
        let domain = domain31_from_pddl(
            "(define (domain d) (:requirements :strips :existential-preconditions) \
             (:predicates (p)) (:action a :parameters () :precondition (p) :effect (not (p))))",
        )
        .unwrap();
        let problem = problem31_from_pddl(STRIPS_PROBLEM).unwrap();
        let outcome = admit_planning_task(&domain, &problem, &DefaultCapabilityProfile);
        match outcome {
            PlannerOutcome::Unsupported(u) => {
                assert_eq!(u.feature_name, "ExistentialPreconditions")
            }
            other => panic!("expected Unsupported(ExistentialPreconditions), got {other:?}"),
        }
    }

    #[test]
    fn universal_preconditions_requirement_is_admitted_as_approximate_not_refused() {
        let domain = domain31_from_pddl(
            "(define (domain d) (:requirements :strips :universal-preconditions) \
             (:predicates (p)) (:action a :parameters () :precondition (p) :effect (not (p))))",
        )
        .unwrap();
        let problem = problem31_from_pddl(STRIPS_PROBLEM).unwrap();
        let outcome = admit_planning_task(&domain, &problem, &DefaultCapabilityProfile);
        assert!(
            outcome.is_found(),
            "Approximate must still admit — only Unsupported refuses"
        );
    }

    #[test]
    fn object_fluents_requirement_is_refused_structurally() {
        let domain = domain31_from_pddl(
            "(define (domain d) (:requirements :strips :object-fluents) (:predicates (p)) \
             (:action a :parameters () :precondition (p) :effect (not (p))))",
        )
        .unwrap();
        let problem = problem31_from_pddl(STRIPS_PROBLEM).unwrap();
        let outcome = admit_planning_task(&domain, &problem, &DefaultCapabilityProfile);
        match outcome {
            PlannerOutcome::Unsupported(u) => assert_eq!(u.feature_name, "object-fluents"),
            other => panic!("expected Unsupported(object-fluents), got {other:?}"),
        }
    }

    #[test]
    fn empty_domain_name_is_inconsistent_not_unsupported() {
        // Structural invalidity is a different failure kind from a
        // capability refusal — Inconsistent, not Unsupported.
        let mut domain = domain31_from_pddl(STRIPS_DOMAIN).unwrap();
        domain.name = String::new();
        let problem = problem31_from_pddl(STRIPS_PROBLEM).unwrap();
        let outcome = admit_planning_task(&domain, &problem, &DefaultCapabilityProfile);
        assert!(matches!(outcome, PlannerOutcome::Inconsistent(_)));
    }

    #[test]
    fn theory_digest_is_deterministic_for_the_same_domain_and_problem() {
        let domain = domain31_from_pddl(STRIPS_DOMAIN).unwrap();
        let problem = problem31_from_pddl(STRIPS_PROBLEM).unwrap();
        let a = admit_planning_task(&domain, &problem, &DefaultCapabilityProfile);
        let b = admit_planning_task(&domain, &problem, &DefaultCapabilityProfile);
        let (PlannerOutcome::Found(a), PlannerOutcome::Found(b)) = (a, b) else {
            panic!("expected both admissions to succeed");
        };
        assert_eq!(a.theory_digest, b.theory_digest);
    }

    #[test]
    fn a_less_permissive_custom_profile_can_refuse_what_default_admits() {
        struct StrictProfile;
        impl CapabilityProfile for StrictProfile {
            fn support(&self, feature: PddlFeature) -> SemanticSupport {
                match feature {
                    // Downgrade Approximate -> Unsupported for everything;
                    // otherwise defer to the default.
                    _ if DefaultCapabilityProfile.support(feature)
                        == SemanticSupport::Approximate =>
                    {
                        SemanticSupport::Unsupported
                    }
                    other => DefaultCapabilityProfile.support(other),
                }
            }
        }
        let domain = domain31_from_pddl(
            "(define (domain d) (:requirements :strips :universal-preconditions) \
             (:predicates (p)) (:action a :parameters () :precondition (p) :effect (not (p))))",
        )
        .unwrap();
        let problem = problem31_from_pddl(STRIPS_PROBLEM).unwrap();
        let default_outcome = admit_planning_task(&domain, &problem, &DefaultCapabilityProfile);
        let strict_outcome = admit_planning_task(&domain, &problem, &StrictProfile);
        assert!(default_outcome.is_found());
        assert!(matches!(strict_outcome, PlannerOutcome::Unsupported(_)));
    }

    #[test]
    fn grounded_planning_epoch_builds_from_a_ground_problem() {
        let domain = crate::parse::domain_from_pddl(STRIPS_DOMAIN).unwrap();
        let problem = crate::parse::problem_from_pddl(STRIPS_PROBLEM).unwrap();
        let gp = GroundProblem::build(&domain, &problem, None).unwrap();
        let bounds = EpochBounds {
            max_ground_actions: 64,
            max_plan_depth: 64,
            max_search_steps: 1_000,
            max_partition_boxes: 8,
        };
        let epoch =
            GroundedPlanningEpoch::from_ground_problem(&gp, Digest::hash(b"theory"), bounds);
        assert_eq!(epoch.actions.len(), gp.actions.len());
        assert_eq!(epoch.initial_state, gp.initial_state);
    }
}
