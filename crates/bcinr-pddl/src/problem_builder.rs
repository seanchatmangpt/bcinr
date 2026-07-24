//! Deterministic construction utilities for common embedded STRIPS problems.
//!
//! This builder intentionally covers positive atoms, typed objects, initial
//! facts, and conjunctive goals. Richer PDDL conditions remain explicit in the
//! full document API so this convenience layer cannot silently flatten them.

#![cfg(feature = "mfw-planner")]

use std::borrow::Cow;

use crate::embedded::WorkflowProblem;

/// A validated positive PDDL atom.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PddlAtomBuilder {
    predicate: String,
    arguments: Vec<String>,
}

impl PddlAtomBuilder {
    pub fn new<I, S>(
        predicate: impl Into<String>,
        arguments: I,
    ) -> Result<Self, PddlBuildError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let predicate = predicate.into();
        validate_symbol("predicate", &predicate)?;
        let arguments = arguments
            .into_iter()
            .map(Into::into)
            .collect::<Vec<String>>();
        for argument in &arguments {
            validate_symbol("atom argument", argument)?;
        }
        Ok(Self {
            predicate,
            arguments,
        })
    }

    pub fn nullary(predicate: impl Into<String>) -> Result<Self, PddlBuildError> {
        Self::new(predicate, std::iter::empty::<String>())
    }

    pub fn predicate(&self) -> &str {
        &self.predicate
    }

    pub fn arguments(&self) -> &[String] {
        &self.arguments
    }

    pub fn render(&self) -> String {
        if self.arguments.is_empty() {
            format!("({})", self.predicate)
        } else {
            format!("({} {})", self.predicate, self.arguments.join(" "))
        }
    }
}

/// One validated object declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PddlObjectBuilder {
    name: String,
    type_name: Option<String>,
}

impl PddlObjectBuilder {
    pub fn untyped(name: impl Into<String>) -> Result<Self, PddlBuildError> {
        let name = name.into();
        validate_symbol("object", &name)?;
        Ok(Self {
            name,
            type_name: None,
        })
    }

    pub fn typed(
        name: impl Into<String>,
        type_name: impl Into<String>,
    ) -> Result<Self, PddlBuildError> {
        let name = name.into();
        let type_name = type_name.into();
        validate_symbol("object", &name)?;
        validate_symbol("type", &type_name)?;
        Ok(Self {
            name,
            type_name: Some(type_name),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_name(&self) -> Option<&str> {
        self.type_name.as_deref()
    }

    fn render(&self) -> String {
        match &self.type_name {
            Some(type_name) => format!("{} - {}", self.name, type_name),
            None => self.name.clone(),
        }
    }
}

/// Validated, deterministic PDDL problem document.
///
/// The inner document is private so values with this type can only be produced
/// by the validating builder in this module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PddlProblemDocument {
    pddl: String,
}

impl PddlProblemDocument {
    pub fn as_str(&self) -> &str {
        &self.pddl
    }

    pub fn into_string(self) -> String {
        self.pddl
    }
}

impl AsRef<str> for PddlProblemDocument {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl WorkflowProblem for PddlProblemDocument {
    fn to_pddl_problem(&self) -> Cow<'_, str> {
        Cow::Borrowed(self.as_str())
    }
}

/// Builder for the common positive STRIPS/typing application boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StripsProblemBuilder {
    problem_name: String,
    domain_name: String,
    objects: Vec<PddlObjectBuilder>,
    initial_facts: Vec<PddlAtomBuilder>,
    goals: Vec<PddlAtomBuilder>,
}

impl StripsProblemBuilder {
    pub fn new(
        problem_name: impl Into<String>,
        domain_name: impl Into<String>,
    ) -> Result<Self, PddlBuildError> {
        let problem_name = problem_name.into();
        let domain_name = domain_name.into();
        validate_symbol("problem name", &problem_name)?;
        validate_symbol("domain name", &domain_name)?;
        Ok(Self {
            problem_name,
            domain_name,
            objects: Vec::new(),
            initial_facts: Vec::new(),
            goals: Vec::new(),
        })
    }

    pub fn add_object(&mut self, name: impl Into<String>) -> Result<&mut Self, PddlBuildError> {
        self.objects.push(PddlObjectBuilder::untyped(name)?);
        Ok(self)
    }

    pub fn add_typed_object(
        &mut self,
        name: impl Into<String>,
        type_name: impl Into<String>,
    ) -> Result<&mut Self, PddlBuildError> {
        self.objects.push(PddlObjectBuilder::typed(name, type_name)?);
        Ok(self)
    }

    pub fn add_fact<I, S>(
        &mut self,
        predicate: impl Into<String>,
        arguments: I,
    ) -> Result<&mut Self, PddlBuildError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.initial_facts
            .push(PddlAtomBuilder::new(predicate, arguments)?);
        Ok(self)
    }

    pub fn add_nullary_fact(
        &mut self,
        predicate: impl Into<String>,
    ) -> Result<&mut Self, PddlBuildError> {
        self.initial_facts.push(PddlAtomBuilder::nullary(predicate)?);
        Ok(self)
    }

    pub fn add_goal<I, S>(
        &mut self,
        predicate: impl Into<String>,
        arguments: I,
    ) -> Result<&mut Self, PddlBuildError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.goals.push(PddlAtomBuilder::new(predicate, arguments)?);
        Ok(self)
    }

    pub fn add_nullary_goal(
        &mut self,
        predicate: impl Into<String>,
    ) -> Result<&mut Self, PddlBuildError> {
        self.goals.push(PddlAtomBuilder::nullary(predicate)?);
        Ok(self)
    }

    pub fn build(self) -> Result<PddlProblemDocument, PddlBuildError> {
        if self.goals.is_empty() {
            return Err(PddlBuildError::MissingGoal);
        }

        let objects = if self.objects.is_empty() {
            String::new()
        } else {
            format!(
                "\n  (:objects {})",
                self.objects
                    .iter()
                    .map(PddlObjectBuilder::render)
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        };
        let initial = self
            .initial_facts
            .iter()
            .map(PddlAtomBuilder::render)
            .collect::<Vec<_>>()
            .join(" ");
        let goals = self
            .goals
            .iter()
            .map(PddlAtomBuilder::render)
            .collect::<Vec<_>>();
        let goal = if goals.len() == 1 {
            goals[0].clone()
        } else {
            format!("(and {})", goals.join(" "))
        };
        let pddl = format!(
            "(define (problem {})\n  (:domain {}){}\n  (:init {})\n  (:goal {}))",
            self.problem_name, self.domain_name, objects, initial, goal
        );
        Ok(PddlProblemDocument { pddl })
    }
}

/// Refusal from deterministic problem construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PddlBuildError {
    InvalidSymbol {
        kind: &'static str,
        value: String,
    },
    MissingGoal,
}

impl std::fmt::Display for PddlBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSymbol { kind, value } => write!(
                f,
                "invalid PDDL {kind} {value:?}: expected an ASCII letter followed by letters, digits, '-' or '_'"
            ),
            Self::MissingGoal => write!(f, "a PDDL problem requires at least one goal atom"),
        }
    }
}

impl std::error::Error for PddlBuildError {}

fn validate_symbol(kind: &'static str, value: &str) -> Result<(), PddlBuildError> {
    let mut chars = value.chars();
    let valid = chars
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic())
        && chars.all(|character| {
            character.is_ascii_alphanumeric() || character == '-' || character == '_'
        });
    if valid {
        Ok(())
    } else {
        Err(PddlBuildError::InvalidSymbol {
            kind,
            value: value.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CognitiveExecutionStanding, EmbeddedWorkflow};

    #[test]
    fn builder_renders_deterministic_typed_problem() {
        let mut builder = StripsProblemBuilder::new("order-42", "fulfillment").unwrap();
        builder
            .add_typed_object("order-42", "order")
            .unwrap()
            .add_fact("paid", ["order-42"])
            .unwrap()
            .add_goal("reserved", ["order-42"])
            .unwrap()
            .add_goal("notified", ["order-42"])
            .unwrap();
        let document = builder.build().unwrap();
        assert_eq!(
            document.as_str(),
            "(define (problem order-42)\n  (:domain fulfillment)\n  (:objects order-42 - order)\n  (:init (paid order-42))\n  (:goal (and (reserved order-42) (notified order-42))))"
        );
    }

    #[test]
    fn built_problem_runs_inside_embedded_workflow() {
        let mut builder = StripsProblemBuilder::new("job-1", "jobs").unwrap();
        builder
            .add_nullary_fact("ready")
            .unwrap()
            .add_nullary_goal("done")
            .unwrap();
        let problem = builder.build().unwrap();
        let mut workflow = EmbeddedWorkflow::new(
            "(define (domain jobs) (:requirements :strips) \
             (:predicates (ready) (done)) \
             (:action finish :parameters () :precondition (ready) :effect (done)))",
        );
        let plan = workflow.plan(&problem).unwrap();
        assert_eq!(
            plan.standing(),
            CognitiveExecutionStanding::WitnessedConcurrentStrips
        );
    }

    #[test]
    fn builder_refuses_ambiguous_symbols_and_missing_goal() {
        assert!(StripsProblemBuilder::new("42-order", "fulfillment").is_err());
        assert!(StripsProblemBuilder::new("order-42", "fulfillment")
            .unwrap()
            .build()
            .is_err());
    }
}
