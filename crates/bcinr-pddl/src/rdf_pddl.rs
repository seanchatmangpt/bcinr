//! RDF-shaped-PDDL bridge: triple facts ↔ `Pddl31Domain`/`Pddl31Problem`.
//!
//! Sibling projects (e.g. `mfw`) hand-encode a PDDL domain's structure as
//! RDF triples using a small fixed vocabulary (`pddl:Action`,
//! `pddl:Precondition`, `pddl:AddEffect`, ...) and compile those triples to
//! PDDL text. This module builds the same structural bridge natively in
//! this crate — no RDF/triple-store dependency, just a `Vec<Triple>` — and
//! compiles directly into the real `Pddl31Domain`/`Pddl31Problem` Rust
//! structs, not PDDL text that would need re-parsing.
//!
//! # Vocabulary
//! Every predicate is a `const &str` below. Subjects/objects that name an
//! entity (a domain, action, predicate declaration, type, problem, or
//! object) are minted as scoped IRI-like strings — e.g.
//! `action:<domain>#<name>` — so several domains/problems can share one
//! `FactSet` without their entities colliding. Atoms (preconditions,
//! effects, init/goal facts) are encoded as a single space-joined string
//! `"<predicate> <arg1> <arg2> ..."` rather than reified further; typed
//! names (params, predicate args, object types) are encoded as
//! `"<name>:<type>"`. This keeps the fact set flat and each entity's facts
//! trivially selectable by subject equality.
//!
//! Argument/parameter **order is positional and derives from the order
//! triples appear in the `FactSet`** for a given (subject, predicate) pair
//! — `PDDL_PARAM` and `PDDL_PREDICATE_ARG` are order-sensitive (parameter
//! binding is positional); `PDDL_PRECOND_ATOM`, `PDDL_ADD_EFFECT`,
//! `PDDL_DEL_EFFECT`, `PDDL_INIT_ATOM`, and `PDDL_GOAL_ATOM` are not
//! (conjunction and disjoint add/del effects are order-independent), but
//! `domain_to_facts`/`problem_to_facts` still emit them in a stable order
//! for a deterministic round trip.
//!
//! # Scope (STRIPS + typing only)
//! This bridge round-trips: type hierarchy (`PddlType`), typed predicate
//! declarations, actions with a **flat conjunction of positive atoms** as
//! precondition and flat add/del effect lists, typed objects, init atoms,
//! and a **flat conjunction** goal. It deliberately does **not** support:
//! disjunction, negation, quantifiers (forall/exists), conditional
//! (`when`) or forall-effects, numeric fluents/effects, timed
//! conditions/effects, durative actions, derived predicates, PDDL+
//! processes/events, constraints, preferences, or metrics. This mirrors
//! the sibling project's own stated scope (no OWL-style subsumption or
//! general ontology reasoning — a direct structural isomorphism only) and
//! is a stated limit, not a shortcut: `compile_domain`/`compile_problem`
//! only ever construct what the vocabulary above can express, and
//! `domain_to_facts`/`problem_to_facts` silently drop any richer
//! condition/effect shape that falls outside a flat positive conjunction
//! (see `flatten_conjunction`) rather than guess an encoding for it.

use wasm4pm_compat::pddl::{
    Pddl31Action, Pddl31Domain, Pddl31Problem, Pddl8Atom, PddlCondition, PddlEffect, PddlType,
};

/// One RDF-shaped fact: `(subject, predicate, object)`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Triple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
}

impl Triple {
    pub fn new(
        subject: impl Into<String>,
        predicate: impl Into<String>,
        object: impl Into<String>,
    ) -> Self {
        Self {
            subject: subject.into(),
            predicate: predicate.into(),
            object: object.into(),
        }
    }
}

/// An unordered-in-principle, ordered-in-practice fact set (see module docs
/// on why `PDDL_PARAM`/`PDDL_PREDICATE_ARG` order is significant).
pub type FactSet = Vec<Triple>;

// ---------------------------------------------------------------------------
// Fixed predicate vocabulary
// ---------------------------------------------------------------------------

/// `(domain_iri, PDDL_REQUIREMENT, ":strips" | ":typing" | ...)`
pub const PDDL_REQUIREMENT: &str = "pddl:requirement";
/// `(domain_iri, PDDL_TYPE, type_iri)` — declares `type_iri` a member of the domain's type hierarchy.
pub const PDDL_TYPE: &str = "pddl:type";
/// `(type_iri, PDDL_TYPE_PARENT, parent_type_name)`
pub const PDDL_TYPE_PARENT: &str = "pddl:typeParent";
/// `(domain_iri, PDDL_PREDICATE, pred_iri)`
pub const PDDL_PREDICATE: &str = "pddl:predicate";
/// `(pred_iri, PDDL_PREDICATE_ARG, "varname:typename")`, one per positional argument.
pub const PDDL_PREDICATE_ARG: &str = "pddl:predicateArg";
/// `(domain_iri, PDDL_ACTION, action_iri)`
pub const PDDL_ACTION: &str = "pddl:action";
/// `(action_iri, PDDL_PARAM, "varname:typename")`, one per positional parameter.
pub const PDDL_PARAM: &str = "pddl:param";
/// `(action_iri, PDDL_PRECOND_ATOM, "predicate arg1 arg2 ...")`, one per precondition conjunct.
pub const PDDL_PRECOND_ATOM: &str = "pddl:preconditionAtom";
/// `(action_iri, PDDL_ADD_EFFECT, "predicate arg1 arg2 ...")`
pub const PDDL_ADD_EFFECT: &str = "pddl:addEffect";
/// `(action_iri, PDDL_DEL_EFFECT, "predicate arg1 arg2 ...")`
pub const PDDL_DEL_EFFECT: &str = "pddl:delEffect";
/// `(problem_iri, PDDL_PROBLEM_DOMAIN, domain_name)` — the problem's `:domain` reference.
pub const PDDL_PROBLEM_DOMAIN: &str = "pddl:problemDomain";
/// `(problem_iri, PDDL_OBJECT, object_iri)`
pub const PDDL_OBJECT: &str = "pddl:object";
/// `(object_iri, PDDL_OBJECT_TYPE, type_name)`
pub const PDDL_OBJECT_TYPE: &str = "pddl:objectType";
/// `(problem_iri, PDDL_INIT_ATOM, "predicate arg1 arg2 ...")`
pub const PDDL_INIT_ATOM: &str = "pddl:initAtom";
/// `(problem_iri, PDDL_GOAL_ATOM, "predicate arg1 arg2 ...")`, one per goal conjunct.
pub const PDDL_GOAL_ATOM: &str = "pddl:goalAtom";

/// The complete fixed vocabulary — every predicate a well-formed fact set may use.
const KNOWN_PREDICATES: &[&str] = &[
    PDDL_REQUIREMENT,
    PDDL_TYPE,
    PDDL_TYPE_PARENT,
    PDDL_PREDICATE,
    PDDL_PREDICATE_ARG,
    PDDL_ACTION,
    PDDL_PARAM,
    PDDL_PRECOND_ATOM,
    PDDL_ADD_EFFECT,
    PDDL_DEL_EFFECT,
    PDDL_PROBLEM_DOMAIN,
    PDDL_OBJECT,
    PDDL_OBJECT_TYPE,
    PDDL_INIT_ATOM,
    PDDL_GOAL_ATOM,
];

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Typed compilation failure for the facts → `Pddl31*` direction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RdfPddlError {
    /// A fact a construction requires (e.g. the problem's domain link) is
    /// absent from the fact set.
    MissingRequiredFact { subject: String, predicate: String },
    /// A triple's object could not be decoded into the structured value its
    /// predicate promises (e.g. an empty atom encoding, or a typed-name pair
    /// missing its `:` separator).
    MalformedTriple {
        subject: String,
        predicate: String,
        object: String,
        detail: String,
    },
    /// A triple predicate outside the fixed RDF-shaped-PDDL vocabulary.
    UnknownPredicate { predicate: String },
    /// The problem's declared domain (`PDDL_PROBLEM_DOMAIN`) does not match
    /// the domain name the caller requested.
    DomainMismatch { declared: String, requested: String },
}

impl std::fmt::Display for RdfPddlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingRequiredFact { subject, predicate } => write!(
                f,
                "missing required fact: no triple ({subject}, {predicate}, ?) in fact set"
            ),
            Self::MalformedTriple {
                subject,
                predicate,
                object,
                detail,
            } => write!(f, "malformed triple ({subject}, {predicate}, {object}): {detail}"),
            Self::UnknownPredicate { predicate } => write!(
                f,
                "unknown predicate '{predicate}' is outside the RDF-shaped-PDDL vocabulary"
            ),
            Self::DomainMismatch { declared, requested } => write!(
                f,
                "problem declares domain '{declared}' but caller requested domain '{requested}'"
            ),
        }
    }
}

impl std::error::Error for RdfPddlError {}

// ---------------------------------------------------------------------------
// IRI helpers — deterministic, scoped, and trivially invertible.
// ---------------------------------------------------------------------------

fn domain_iri(domain_name: &str) -> String {
    format!("domain:{domain_name}")
}

fn type_iri(domain_name: &str, type_name: &str) -> String {
    format!("type:{domain_name}#{type_name}")
}

fn predicate_iri(domain_name: &str, pred_name: &str) -> String {
    format!("pred:{domain_name}#{pred_name}")
}

fn action_iri(domain_name: &str, action_name: &str) -> String {
    format!("action:{domain_name}#{action_name}")
}

fn problem_iri(problem_name: &str) -> String {
    format!("problem:{problem_name}")
}

fn object_iri(problem_name: &str, object_name: &str) -> String {
    format!("object:{problem_name}#{object_name}")
}

/// Recover the plain name minted into an entity IRI's `#`-suffix (or the
/// whole string, for IRIs with no `#`).
fn local_name(iri: &str) -> &str {
    iri.rsplit('#').next().unwrap_or(iri)
}

// ---------------------------------------------------------------------------
// Atom / typed-name encoding — the only two scalar codecs the vocabulary needs.
// ---------------------------------------------------------------------------

fn encode_atom(atom: &Pddl8Atom) -> String {
    let mut parts = Vec::with_capacity(1 + atom.args.len());
    parts.push(atom.pred.as_str());
    parts.extend(atom.args.iter().map(String::as_str));
    parts.join(" ")
}

fn decode_atom(triple: &Triple) -> Result<Pddl8Atom, RdfPddlError> {
    let mut tokens = triple.object.split_whitespace();
    let pred = tokens
        .next()
        .ok_or_else(|| RdfPddlError::MalformedTriple {
            subject: triple.subject.clone(),
            predicate: triple.predicate.clone(),
            object: triple.object.clone(),
            detail: "atom encoding is empty; expected 'predicate arg1 arg2 ...'".to_string(),
        })?
        .to_string();
    Ok(Pddl8Atom {
        pred,
        args: tokens.map(str::to_string).collect(),
    })
}

fn encode_typed(name: &str, typ: &str) -> String {
    format!("{name}:{typ}")
}

fn decode_typed(triple: &Triple) -> Result<(String, String), RdfPddlError> {
    triple
        .object
        .split_once(':')
        .map(|(name, typ)| (name.to_string(), typ.to_string()))
        .ok_or_else(|| RdfPddlError::MalformedTriple {
            subject: triple.subject.clone(),
            predicate: triple.predicate.clone(),
            object: triple.object.clone(),
            detail: "typed-name encoding must be 'name:type'".to_string(),
        })
}

/// Extract the flat positive-atom conjuncts a `PddlCondition` expresses in
/// this bridge's scope. A bare `Atom` counts as a one-conjunct conjunction;
/// anything else that appears *inside* an `And` (or as the condition itself)
/// — `Not`, `Or`, `Forall`, `Exists`, `Imply`, `Timed`, `Compare` — falls
/// outside the STRIPS+typing scope and is dropped rather than guessed at.
fn flatten_conjunction(condition: &PddlCondition) -> Vec<&Pddl8Atom> {
    match condition {
        PddlCondition::And(items) => items
            .iter()
            .filter_map(|item| match item {
                PddlCondition::Atom(atom) => Some(atom),
                _ => None,
            })
            .collect(),
        PddlCondition::Atom(atom) => vec![atom],
        _ => Vec::new(),
    }
}

fn validate_predicates(facts: &FactSet) -> Result<(), RdfPddlError> {
    for triple in facts {
        if !KNOWN_PREDICATES.contains(&triple.predicate.as_str()) {
            return Err(RdfPddlError::UnknownPredicate {
                predicate: triple.predicate.clone(),
            });
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// facts -> Pddl31Domain / Pddl31Problem
// ---------------------------------------------------------------------------

/// Walk `facts` for the domain named `domain_name` and construct a real
/// `Pddl31Domain`. Entities the domain declares nothing about (e.g. zero
/// types) simply contribute no facts and yield the corresponding empty
/// `Vec`, matching `Pddl31Domain::default()`.
pub fn compile_domain(facts: &FactSet, domain_name: &str) -> Result<Pddl31Domain, RdfPddlError> {
    validate_predicates(facts)?;
    let d_iri = domain_iri(domain_name);

    let mut domain = Pddl31Domain {
        name: domain_name.to_string(),
        ..Pddl31Domain::default()
    };

    for triple in facts
        .iter()
        .filter(|t| t.subject == d_iri && t.predicate == PDDL_REQUIREMENT)
    {
        domain.requirements.push(triple.object.clone());
    }

    for triple in facts
        .iter()
        .filter(|t| t.subject == d_iri && t.predicate == PDDL_TYPE)
    {
        let type_iri = &triple.object;
        let parent = facts
            .iter()
            .find(|p| &p.subject == type_iri && p.predicate == PDDL_TYPE_PARENT)
            .map(|p| p.object.clone());
        domain.types.push(PddlType {
            name: local_name(type_iri).to_string(),
            parent,
        });
    }

    for triple in facts
        .iter()
        .filter(|t| t.subject == d_iri && t.predicate == PDDL_PREDICATE)
    {
        let pred_iri = &triple.object;
        let params = facts
            .iter()
            .filter(|p| &p.subject == pred_iri && p.predicate == PDDL_PREDICATE_ARG)
            .map(decode_typed)
            .collect::<Result<Vec<_>, _>>()?;
        domain.predicates.push((local_name(pred_iri).to_string(), params));
    }

    for triple in facts
        .iter()
        .filter(|t| t.subject == d_iri && t.predicate == PDDL_ACTION)
    {
        let action_iri = &triple.object;
        let params = facts
            .iter()
            .filter(|p| &p.subject == action_iri && p.predicate == PDDL_PARAM)
            .map(decode_typed)
            .collect::<Result<Vec<_>, _>>()?;

        let precond_atoms = facts
            .iter()
            .filter(|p| &p.subject == action_iri && p.predicate == PDDL_PRECOND_ATOM)
            .map(decode_atom)
            .collect::<Result<Vec<_>, _>>()?;
        let precondition =
            PddlCondition::And(precond_atoms.into_iter().map(PddlCondition::Atom).collect());

        let mut effect = Vec::new();
        for p in facts
            .iter()
            .filter(|p| &p.subject == action_iri && p.predicate == PDDL_ADD_EFFECT)
        {
            effect.push(PddlEffect::Add(decode_atom(p)?));
        }
        for p in facts
            .iter()
            .filter(|p| &p.subject == action_iri && p.predicate == PDDL_DEL_EFFECT)
        {
            effect.push(PddlEffect::Del(decode_atom(p)?));
        }

        domain.actions.push(Pddl31Action {
            name: local_name(action_iri).to_string(),
            params,
            precondition,
            effect,
        });
    }

    Ok(domain)
}

/// Walk `facts` for the problem named `problem_name` (declaring domain
/// `domain_name`) and construct a real `Pddl31Problem`. Errors with
/// `MissingRequiredFact` if the problem→domain link is absent, and
/// `DomainMismatch` if it names a different domain than `domain_name`.
pub fn compile_problem(
    facts: &FactSet,
    problem_name: &str,
    domain_name: &str,
) -> Result<Pddl31Problem, RdfPddlError> {
    validate_predicates(facts)?;
    let p_iri = problem_iri(problem_name);

    let domain_ref = facts
        .iter()
        .find(|t| t.subject == p_iri && t.predicate == PDDL_PROBLEM_DOMAIN)
        .ok_or_else(|| RdfPddlError::MissingRequiredFact {
            subject: p_iri.clone(),
            predicate: PDDL_PROBLEM_DOMAIN.to_string(),
        })?;
    if domain_ref.object != domain_name {
        return Err(RdfPddlError::DomainMismatch {
            declared: domain_ref.object.clone(),
            requested: domain_name.to_string(),
        });
    }

    let mut problem = Pddl31Problem {
        name: problem_name.to_string(),
        domain: domain_name.to_string(),
        ..Pddl31Problem::default()
    };

    for triple in facts
        .iter()
        .filter(|t| t.subject == p_iri && t.predicate == PDDL_OBJECT)
    {
        let object_iri = &triple.object;
        let obj_type = facts
            .iter()
            .find(|p| &p.subject == object_iri && p.predicate == PDDL_OBJECT_TYPE)
            .map(|p| p.object.clone())
            .unwrap_or_else(|| "object".to_string());
        problem.objects.push((local_name(object_iri).to_string(), obj_type));
    }

    for triple in facts
        .iter()
        .filter(|t| t.subject == p_iri && t.predicate == PDDL_INIT_ATOM)
    {
        problem.init_atoms.push(decode_atom(triple)?);
    }

    let goal_atoms = facts
        .iter()
        .filter(|t| t.subject == p_iri && t.predicate == PDDL_GOAL_ATOM)
        .map(decode_atom)
        .collect::<Result<Vec<_>, _>>()?;
    problem.goal = PddlCondition::And(goal_atoms.into_iter().map(PddlCondition::Atom).collect());

    Ok(problem)
}

// ---------------------------------------------------------------------------
// Pddl31Domain / Pddl31Problem -> facts (the inverse bridge)
// ---------------------------------------------------------------------------

/// Emit `domain` as facts under the given `domain_name`. Preconditions and
/// effects that are not a flat conjunction of positive atoms (see
/// `flatten_conjunction`) are dropped rather than encoded lossily — the
/// caller is expected to only round-trip domains already within this
/// bridge's STRIPS+typing scope (which `compile_domain` above never
/// produces anything outside of).
pub fn domain_to_facts(domain: &Pddl31Domain, domain_name: &str) -> FactSet {
    let mut facts = FactSet::new();
    let d_iri = domain_iri(domain_name);

    for requirement in &domain.requirements {
        facts.push(Triple::new(&d_iri, PDDL_REQUIREMENT, requirement));
    }

    for ty in &domain.types {
        let t_iri = type_iri(domain_name, &ty.name);
        facts.push(Triple::new(&d_iri, PDDL_TYPE, &t_iri));
        if let Some(parent) = &ty.parent {
            facts.push(Triple::new(&t_iri, PDDL_TYPE_PARENT, parent));
        }
    }

    for (pred_name, params) in &domain.predicates {
        let p_iri = predicate_iri(domain_name, pred_name);
        facts.push(Triple::new(&d_iri, PDDL_PREDICATE, &p_iri));
        for (var, typ) in params {
            facts.push(Triple::new(&p_iri, PDDL_PREDICATE_ARG, encode_typed(var, typ)));
        }
    }

    for action in &domain.actions {
        let a_iri = action_iri(domain_name, &action.name);
        facts.push(Triple::new(&d_iri, PDDL_ACTION, &a_iri));
        for (var, typ) in &action.params {
            facts.push(Triple::new(&a_iri, PDDL_PARAM, encode_typed(var, typ)));
        }
        for atom in flatten_conjunction(&action.precondition) {
            facts.push(Triple::new(&a_iri, PDDL_PRECOND_ATOM, encode_atom(atom)));
        }
        for effect in &action.effect {
            match effect {
                PddlEffect::Add(atom) => {
                    facts.push(Triple::new(&a_iri, PDDL_ADD_EFFECT, encode_atom(atom)))
                }
                PddlEffect::Del(atom) => {
                    facts.push(Triple::new(&a_iri, PDDL_DEL_EFFECT, encode_atom(atom)))
                }
                // Numeric/timed/forall/conditional effects are outside this
                // bridge's scope (see module docs) and are dropped.
                PddlEffect::Numeric(_)
                | PddlEffect::Timed(_, _)
                | PddlEffect::Forall { .. }
                | PddlEffect::When { .. } => {}
            }
        }
    }

    facts
}

/// Emit `problem` as facts under `problem_name`, declaring domain
/// `domain_name`. As with `domain_to_facts`, a goal outside a flat positive
/// conjunction is dropped rather than encoded lossily.
pub fn problem_to_facts(
    problem: &Pddl31Problem,
    problem_name: &str,
    domain_name: &str,
) -> FactSet {
    let mut facts = FactSet::new();
    let p_iri = problem_iri(problem_name);

    facts.push(Triple::new(&p_iri, PDDL_PROBLEM_DOMAIN, domain_name));

    for (obj_name, obj_type) in &problem.objects {
        let o_iri = object_iri(problem_name, obj_name);
        facts.push(Triple::new(&p_iri, PDDL_OBJECT, &o_iri));
        facts.push(Triple::new(&o_iri, PDDL_OBJECT_TYPE, obj_type));
    }

    for atom in &problem.init_atoms {
        facts.push(Triple::new(&p_iri, PDDL_INIT_ATOM, encode_atom(atom)));
    }

    for atom in flatten_conjunction(&problem.goal) {
        facts.push(Triple::new(&p_iri, PDDL_GOAL_ATOM, encode_atom(atom)));
    }

    facts
}
