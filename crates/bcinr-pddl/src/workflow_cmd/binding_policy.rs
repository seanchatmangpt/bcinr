/// Application binding backend with stable schema identity.
pub trait ActionBinding {
    type Command;
    type Error;

    fn binding_name(&self) -> &str;
    fn supported_actions(&self) -> Vec<&str>;
    fn bind(&self, action: &ActionInvocation) -> Result<Self::Command, Self::Error>;
}

/// Binding-registry construction refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindingRegistryError {
    EmptyName,
    EmptyAction,
    DuplicateAction(String),
}

impl fmt::Display for BindingRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "binding registry refused: {self:?}")
    }
}

impl std::error::Error for BindingRegistryError {}

/// Exact action-catalog coverage report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindingCoverageReport {
    pub missing_bindings: Vec<String>,
    pub extra_bindings: Vec<String>,
}

impl BindingCoverageReport {
    pub fn is_complete(&self) -> bool {
        self.missing_bindings.is_empty() && self.extra_bindings.is_empty()
    }
}

/// Validated binding backend plus a root over the command schema.
#[derive(Debug)]
pub struct BindingRegistry<B> {
    binding: B,
    actions: Vec<String>,
    schema_root: BindingSchemaRoot,
}

impl<B: ActionBinding> BindingRegistry<B> {
    pub fn new(binding: B) -> Result<Self, BindingRegistryError> {
        if binding.binding_name().trim().is_empty() {
            return Err(BindingRegistryError::EmptyName);
        }
        let mut seen = BTreeSet::new();
        let mut actions = Vec::new();
        for action in binding.supported_actions() {
            let action = action.trim();
            if action.is_empty() {
                return Err(BindingRegistryError::EmptyAction);
            }
            if !seen.insert(action.to_string()) {
                return Err(BindingRegistryError::DuplicateAction(action.to_string()));
            }
            actions.push(action.to_string());
        }
        actions.sort();
        let mut parts: Vec<&[u8]> = vec![binding.binding_name().as_bytes()];
        parts.extend(actions.iter().map(String::as_bytes));
        let schema_root = BindingSchemaRoot::hash_parts(&parts);
        Ok(Self {
            binding,
            actions,
            schema_root,
        })
    }

    pub const fn schema_root(&self) -> BindingSchemaRoot {
        self.schema_root
    }

    pub fn supported_actions(&self) -> &[String] {
        &self.actions
    }

    pub fn coverage<I, S>(&self, action_catalog: I) -> BindingCoverageReport
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let catalog = action_catalog
            .into_iter()
            .map(Into::into)
            .collect::<BTreeSet<_>>();
        let bindings = self.actions.iter().cloned().collect::<BTreeSet<_>>();
        BindingCoverageReport {
            missing_bindings: catalog.difference(&bindings).cloned().collect(),
            extra_bindings: bindings.difference(&catalog).cloned().collect(),
        }
    }

    pub fn bind_plan(
        &self,
        verified: &VerifiedWorkflowPlan,
    ) -> Result<TypedWorkflowPlan<B::Command>, B::Error> {
        verified.map_actions(|action| self.binding.bind(action))
    }
}

/// Institutional policy result. Policy produces evidence or an explicit refusal;
/// it never invokes a command handler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyDecision<E, R> {
    Admit(E),
    Refuse(R),
}

pub trait PolicyIdentity {
    fn root(&self) -> PolicySetRoot;
}

pub trait Policy<Input, Context>: PolicyIdentity {
    type Evidence;
    type Refusal;

    fn evaluate(
        &self,
        input: &Input,
        context: &Context,
    ) -> PolicyDecision<Self::Evidence, Self::Refusal>;
}

/// Deterministic conjunction of two policies.
#[derive(Debug, Clone)]
pub struct AllPolicy<P, Q> {
    first: P,
    second: Q,
    root: PolicySetRoot,
}

impl<P, Q> AllPolicy<P, Q>
where
    P: PolicyIdentity,
    Q: PolicyIdentity,
{
    pub fn new(first: P, second: Q) -> Self {
        let root = PolicySetRoot::hash_parts(&[first.root().as_bytes(), second.root().as_bytes()]);
        Self {
            first,
            second,
            root,
        }
    }
}

impl<P, Q> PolicyIdentity for AllPolicy<P, Q> {
    fn root(&self) -> PolicySetRoot {
        self.root
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllPolicyRefusal<R1, R2> {
    First(R1),
    Second(R2),
}

impl<Input, Context, P, Q> Policy<Input, Context> for AllPolicy<P, Q>
where
    P: Policy<Input, Context>,
    Q: Policy<Input, Context>,
{
    type Evidence = (P::Evidence, Q::Evidence);
    type Refusal = AllPolicyRefusal<P::Refusal, Q::Refusal>;

    fn evaluate(
        &self,
        input: &Input,
        context: &Context,
    ) -> PolicyDecision<Self::Evidence, Self::Refusal> {
        let first = match self.first.evaluate(input, context) {
            PolicyDecision::Admit(evidence) => evidence,
            PolicyDecision::Refuse(refusal) => {
                return PolicyDecision::Refuse(AllPolicyRefusal::First(refusal));
            }
        };
        match self.second.evaluate(input, context) {
            PolicyDecision::Admit(second) => PolicyDecision::Admit((first, second)),
            PolicyDecision::Refuse(refusal) => {
                PolicyDecision::Refuse(AllPolicyRefusal::Second(refusal))
            }
        }
    }
}
