/// All errors from the PDDL8 → receipt pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlannerOutcome<T> {
    Found(T),
    Exhausted,
    Bounded(usize),
    Unsupported(String),
    Inconsistent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pddl8Error {
    /// PDDL text could not be parsed.
    ParseError(String),
    /// A structural bound was exceeded (arity, body atoms, variables, depth).
    BoundExceeded {
        what: &'static str,
        limit: u8,
        got: usize,
    },
    /// An action schema references an unknown predicate or type.
    UnknownPredicate(String),
    /// Grounding produced zero applicable actions — plan search space is empty.
    EmptyGrounding,
    /// The planner exhausted bounded search without reaching the goal.
    NoAdmittedPlan,
    /// Prolog8 admission kernel rejected a rule at load time.
    AdmissionLoadError(String),
    /// An op fired but Prolog8 denied it at runtime.
    StepDenied { op_index: u8, reason: String },
    /// Goal was not reached after executing all admitted steps.
    GoalNotReached,
    /// Receipt chain integrity failure.
    ReceiptIntegrity(String),
    /// case_id contains disallowed characters or is out of the 1-64 char range.
    InvalidCaseId(String),
}

impl std::fmt::Display for Pddl8Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError(s) => write!(f, "PDDL parse error: {s}"),
            Self::BoundExceeded { what, limit, got } => {
                write!(f, "PDDL8 bound exceeded: {what} limit={limit} got={got}")
            }
            Self::UnknownPredicate(p) => write!(f, "unknown predicate: {p}"),
            Self::EmptyGrounding => write!(f, "grounding produced no applicable actions"),
            Self::NoAdmittedPlan => write!(f, "bounded plan search exhausted without goal"),
            Self::AdmissionLoadError(s) => write!(f, "Prolog8 admission load error: {s}"),
            Self::StepDenied { op_index, reason } => write!(f, "step {op_index} denied: {reason}"),
            Self::GoalNotReached => write!(f, "goal not reached after plan execution"),
            Self::ReceiptIntegrity(s) => write!(f, "receipt integrity failure: {s}"),
            Self::InvalidCaseId(s) => write!(f, "invalid case_id: {s}"),
        }
    }
}

impl std::error::Error for Pddl8Error {}

impl<T> PlannerOutcome<T> {
    pub fn into_result(self) -> Result<T, Pddl8Error> {
        match self {
            Self::Found(t) => Ok(t),
            Self::Exhausted => Err(Pddl8Error::NoAdmittedPlan),
            Self::Bounded(f) => Err(Pddl8Error::BoundExceeded {
                what: "frontier",
                limit: 0,
                got: f,
            }),
            Self::Unsupported(_) => Err(Pddl8Error::NoAdmittedPlan),
            Self::Inconsistent => Err(Pddl8Error::NoAdmittedPlan),
        }
    }
}
