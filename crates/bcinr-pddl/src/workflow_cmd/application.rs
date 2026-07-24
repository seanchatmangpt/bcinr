/// Application state capable of manufacturing a planning problem for an exact goal.
pub trait GoalDirectedWorkflowProblem<G> {
    fn to_pddl_problem_for_goal<'a>(
        &'a self,
        goal: &'a G,
    ) -> std::borrow::Cow<'a, str>;
}

struct GoalDirectedProblem<'a, O, G> {
    observation: &'a O,
    goal: &'a G,
}

impl<O, G> WorkflowProblem for GoalDirectedProblem<'_, O, G>
where
    O: GoalDirectedWorkflowProblem<G>,
{
    fn to_pddl_problem(&self) -> std::borrow::Cow<'_, str> {
        self.observation.to_pddl_problem_for_goal(self.goal)
    }
}

/// Compiled application work: verified process metadata plus native Rust commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedWorkflow<C> {
    envelope: PlanEnvelope,
    typed_plan: TypedWorkflowPlan<C>,
    binding_root: BindingSchemaRoot,
}

impl<C> PreparedWorkflow<C> {
    pub const fn envelope(&self) -> &PlanEnvelope {
        &self.envelope
    }

    pub const fn typed_plan(&self) -> &TypedWorkflowPlan<C> {
        &self.typed_plan
    }

    pub const fn binding_root(&self) -> BindingSchemaRoot {
        self.binding_root
    }
}

/// Policy evidence paired with the broker proposal it authorized.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizedWorkflow<C, E> {
    evidence: E,
    policy_root: PolicySetRoot,
    proposal: DispatchProposal<C>,
}

impl<C, E> AuthorizedWorkflow<C, E> {
    pub const fn evidence(&self) -> &E {
        &self.evidence
    }

    pub const fn policy_root(&self) -> PolicySetRoot {
        self.policy_root
    }

    pub const fn proposal(&self) -> &DispatchProposal<C> {
        &self.proposal
    }

    pub fn into_parts(self) -> (E, PolicySetRoot, DispatchProposal<C>) {
        (self.evidence, self.policy_root, self.proposal)
    }
}

/// Failure while compiling application state into native standing-bearing work.
#[derive(Debug)]
pub enum WorkflowApplicationError<E> {
    Planning(EmbeddedWorkflowError),
    Envelope(PlanEnvelopeError),
    Binding(E),
}

impl<E: fmt::Debug> fmt::Display for WorkflowApplicationError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "planning-native application compilation refused: {self:?}")
    }
}

impl<E: fmt::Debug> std::error::Error for WorkflowApplicationError<E> {}

/// Refusal while turning prepared commands into an authorized dispatch proposal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthorizationProposalError<R> {
    Policy(R),
    Dispatch(DispatchProposalError),
}

impl<R: fmt::Debug> fmt::Display for AuthorizationProposalError<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "workflow authorization proposal refused: {self:?}")
    }
}

impl<R: fmt::Debug> std::error::Error for AuthorizationProposalError<R> {}

/// High-level compiler host for one resident domain and one native command schema.
///
/// This facade performs only reversible manufacture: planning, verification,
/// envelope construction, and command binding. It never invokes a command handler.
pub struct WorkflowApplication<B> {
    workflow: EmbeddedWorkflow,
    bindings: BindingRegistry<B>,
}

impl<B: ActionBinding> WorkflowApplication<B> {
    pub fn new(workflow: EmbeddedWorkflow, binding: B) -> Result<Self, BindingRegistryError> {
        Ok(Self {
            workflow,
            bindings: BindingRegistry::new(binding)?,
        })
    }

    pub const fn workflow(&self) -> &EmbeddedWorkflow {
        &self.workflow
    }

    pub const fn bindings(&self) -> &BindingRegistry<B> {
        &self.bindings
    }

    pub fn workflow_mut(&mut self) -> &mut EmbeddedWorkflow {
        &mut self.workflow
    }

    pub fn into_parts(self) -> (EmbeddedWorkflow, BindingRegistry<B>) {
        (self.workflow, self.bindings)
    }

    /// Compile from an explicitly supplied problem document.
    ///
    /// This lower-level path is useful for existing PDDL integrations. The caller
    /// is responsible for ensuring the supplied problem corresponds to the
    /// receipted observation and goal. New applications should prefer
    /// [`Self::compile_goal_directed`].
    pub fn compile<P, O, G>(
        &mut self,
        problem: &P,
        observation: &ObservationSnapshot<O>,
        goal: &GoalEnvelope<G>,
        bounds: PlanningBounds,
        search_policy_root: SearchPolicyRoot,
    ) -> Result<PreparedWorkflow<B::Command>, WorkflowApplicationError<B::Error>>
    where
        P: WorkflowProblem + ?Sized,
    {
        let verified = self
            .workflow
            .plan(problem)
            .map_err(WorkflowApplicationError::Planning)?;
        self.finish_compilation(verified, observation.root(), goal.root(), bounds, search_policy_root)
    }

    /// Compile a planning problem manufactured from the exact observation value
    /// and goal value whose roots enter the plan envelope.
    pub fn compile_goal_directed<O, G>(
        &mut self,
        observation: &ObservationSnapshot<O>,
        goal: &GoalEnvelope<G>,
        bounds: PlanningBounds,
        search_policy_root: SearchPolicyRoot,
    ) -> Result<PreparedWorkflow<B::Command>, WorkflowApplicationError<B::Error>>
    where
        O: GoalDirectedWorkflowProblem<G>,
    {
        let problem = GoalDirectedProblem {
            observation: observation.value(),
            goal: goal.goal(),
        };
        let verified = self
            .workflow
            .plan(&problem)
            .map_err(WorkflowApplicationError::Planning)?;
        self.finish_compilation(verified, observation.root(), goal.root(), bounds, search_policy_root)
    }

    fn finish_compilation(
        &self,
        verified: VerifiedWorkflowPlan,
        observation_root: ObservationRoot,
        goal_root: GoalRoot,
        bounds: PlanningBounds,
        search_policy_root: SearchPolicyRoot,
    ) -> Result<PreparedWorkflow<B::Command>, WorkflowApplicationError<B::Error>> {
        let envelope = self
            .workflow
            .manufacture_plan_envelope(
                &verified,
                observation_root,
                goal_root,
                bounds,
                search_policy_root,
            )
            .map_err(WorkflowApplicationError::Envelope)?;
        let typed_plan = self
            .bindings
            .bind_plan(&verified)
            .map_err(WorkflowApplicationError::Binding)?;
        Ok(PreparedWorkflow {
            envelope,
            typed_plan,
            binding_root: self.bindings.schema_root(),
        })
    }
}

impl<C> PreparedWorkflow<C>
where
    C: Clone + Serialize,
{
    /// Evaluate institutional policy and manufacture a broker proposal.
    ///
    /// Admission remains a value transition; this method does not call a broker.
    pub fn authorize_and_propose<P, Context>(
        &self,
        policy: &P,
        context: &Context,
        idempotency: IdempotencyKey,
    ) -> Result<AuthorizedWorkflow<C, P::Evidence>, AuthorizationProposalError<P::Refusal>>
    where
        P: Policy<TypedWorkflowPlan<C>, Context>,
    {
        let evidence = match policy.evaluate(&self.typed_plan, context) {
            PolicyDecision::Admit(evidence) => evidence,
            PolicyDecision::Refuse(refusal) => {
                return Err(AuthorizationProposalError::Policy(refusal));
            }
        };
        let policy_root = policy.root();
        let proposal = DispatchProposal::from_typed_plan(
            &self.typed_plan,
            &self.envelope,
            self.binding_root,
            Some(policy_root),
            idempotency,
        )
        .map_err(AuthorizationProposalError::Dispatch)?;
        Ok(AuthorizedWorkflow {
            evidence,
            policy_root,
            proposal,
        })
    }
}
