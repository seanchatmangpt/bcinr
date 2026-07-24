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

/// Evidence binding the production compiler-pass chain to the prepared artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowCompilationWitness {
    pub request_root: RequestRoot,
    pub execution_root: ExecutionRoot,
    pub plan_root: PlanRoot,
    pub binding_root: BindingSchemaRoot,
    pub artifact_root: PassRoot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PreparedWorkflowValue<C> {
    envelope: PlanEnvelope,
    typed_plan: TypedWorkflowPlan<C>,
    binding_root: BindingSchemaRoot,
    witness: WorkflowCompilationWitness,
}

/// Compiled application work carried at command-bound standing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedWorkflow<C> {
    artifact: Artifact<PreparedWorkflowValue<C>, BoundToCommands>,
}

impl<C> PreparedWorkflow<C> {
    pub fn envelope(&self) -> &PlanEnvelope {
        &self.artifact.value().envelope
    }

    pub fn typed_plan(&self) -> &TypedWorkflowPlan<C> {
        &self.artifact.value().typed_plan
    }

    pub fn binding_root(&self) -> BindingSchemaRoot {
        self.artifact.value().binding_root
    }

    pub const fn artifact_root(&self) -> PassRoot {
        self.artifact.root()
    }

    pub const fn artifact_ref(&self) -> ArtifactRef<PassRoot> {
        ArtifactRef::new(self.artifact.root())
    }

    pub fn compilation_witness(&self) -> &WorkflowCompilationWitness {
        &self.artifact.value().witness
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AuthorizedWorkflowValue<C, E> {
    evidence: E,
    policy_root: PolicySetRoot,
    proposal: DispatchProposal<C>,
}

/// Evidence for the policy-admitted transition from prepared work to proposal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowAuthorizationWitness {
    pub prepared_root: PassRoot,
    pub policy_root: PolicySetRoot,
    pub dispatch_root: DispatchRoot,
    pub artifact_root: PassRoot,
}

/// Policy evidence paired with the broker proposal it authorized.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizedWorkflow<C, E> {
    artifact: Artifact<AuthorizedWorkflowValue<C, E>, PolicyAdmitted>,
    witness: WorkflowAuthorizationWitness,
}

impl<C, E> AuthorizedWorkflow<C, E> {
    pub fn evidence(&self) -> &E {
        &self.artifact.value().evidence
    }

    pub fn policy_root(&self) -> PolicySetRoot {
        self.artifact.value().policy_root
    }

    pub fn proposal(&self) -> &DispatchProposal<C> {
        &self.artifact.value().proposal
    }

    pub const fn artifact_root(&self) -> PassRoot {
        self.artifact.root()
    }

    pub const fn artifact_ref(&self) -> ArtifactRef<PassRoot> {
        ArtifactRef::new(self.artifact.root())
    }

    pub const fn authorization_witness(&self) -> &WorkflowAuthorizationWitness {
        &self.witness
    }

    pub fn into_parts(self) -> (E, PolicySetRoot, DispatchProposal<C>) {
        let value = self.artifact.value;
        (value.evidence, value.policy_root, value.proposal)
    }
}

fn transition_artifact<T, S>(value: T, root: PassRoot) -> Artifact<T, S> {
    Artifact {
        value,
        root,
        standing: PhantomData,
    }
}

/// Failure while compiling application state into native standing-bearing work.
#[derive(Debug)]
pub enum WorkflowApplicationError<E> {
    Planning(EmbeddedWorkflowError),
    Envelope(PlanEnvelopeError),
    Binding(E),
    PassContinuity {
        first_output: PassRoot,
        second_input: PassRoot,
    },
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

struct WorkflowCompileInput<'a, P: ?Sized, B> {
    workflow: &'a mut EmbeddedWorkflow,
    bindings: &'a BindingRegistry<B>,
    problem: &'a P,
    observation_root: ObservationRoot,
    goal_root: GoalRoot,
    bounds: PlanningBounds,
    search_policy_root: SearchPolicyRoot,
}

struct PlannedCompilation<'a, B> {
    bindings: &'a BindingRegistry<B>,
    verified: VerifiedWorkflowPlan,
    request: PlanRequestEnvelope,
    execution_root: ExecutionRoot,
}

struct EnvelopedCompilation<'a, B> {
    bindings: &'a BindingRegistry<B>,
    verified: VerifiedWorkflowPlan,
    envelope: PlanEnvelope,
}

#[derive(Debug)]
enum PlanningPassRefusal {
    Planning(EmbeddedWorkflowError),
    Envelope(PlanEnvelopeError),
}

#[derive(Debug, Clone, Copy)]
struct PlanWorkflowPass;

impl<'a, P, B> PlanPass<WorkflowCompileInput<'a, P, B>> for PlanWorkflowPass
where
    P: WorkflowProblem + ?Sized,
    B: ActionBinding,
{
    type Output = PlannedCompilation<'a, B>;
    type Witness = (RequestRoot, ExecutionRoot);
    type Refusal = PlanningPassRefusal;

    fn apply(
        &self,
        input: WorkflowCompileInput<'a, P, B>,
    ) -> Result<PassOutput<Self::Output, Self::Witness>, Self::Refusal> {
        let domain_source_root = input
            .workflow
            .domain_source_root()
            .parse()
            .map_err(|error| PlanningPassRefusal::Envelope(PlanEnvelopeError::DomainRoot(error)))?;
        let request = PlanRequestEnvelope::manufacture(
            domain_source_root,
            input.observation_root,
            input.goal_root,
            input.bounds,
            input.search_policy_root,
        );
        let request_root = request.request_root();
        let input_root = PassRoot::hash_parts(&[b"request", request_root.as_bytes()]);
        let verified = input
            .workflow
            .plan(input.problem)
            .map_err(PlanningPassRefusal::Planning)?;
        let execution_root = verified.execution_root().parse().map_err(|error| {
            PlanningPassRefusal::Envelope(PlanEnvelopeError::ExecutionRoot(error))
        })?;
        let output_root = PassRoot::hash_parts(&[b"execution", execution_root.as_bytes()]);
        Ok(PassOutput {
            value: PlannedCompilation {
                bindings: input.bindings,
                verified,
                request,
                execution_root,
            },
            witness: (request_root, execution_root),
            input_root,
            output_root,
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct ManufactureEnvelopePass;

impl<'a, B> PlanPass<PlannedCompilation<'a, B>> for ManufactureEnvelopePass {
    type Output = EnvelopedCompilation<'a, B>;
    type Witness = (ExecutionRoot, PlanRoot);
    type Refusal = std::convert::Infallible;

    fn apply(
        &self,
        input: PlannedCompilation<'a, B>,
    ) -> Result<PassOutput<Self::Output, Self::Witness>, Self::Refusal> {
        let input_root = PassRoot::hash_parts(&[b"execution", input.execution_root.as_bytes()]);
        let standing = input.verified.standing();
        let envelope = PlanEnvelope::manufacture(input.request, standing, input.execution_root);
        let output_root = PassRoot::hash_parts(&[b"plan", envelope.plan_root().as_bytes()]);
        Ok(PassOutput {
            witness: (input.execution_root, envelope.plan_root()),
            value: EnvelopedCompilation {
                bindings: input.bindings,
                verified: input.verified,
                envelope,
            },
            input_root,
            output_root,
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct BindCommandsPass;

impl<'a, B> PlanPass<EnvelopedCompilation<'a, B>> for BindCommandsPass
where
    B: ActionBinding,
{
    type Output = PreparedWorkflow<B::Command>;
    type Witness = (PlanRoot, BindingSchemaRoot, PassRoot);
    type Refusal = B::Error;

    fn apply(
        &self,
        input: EnvelopedCompilation<'a, B>,
    ) -> Result<PassOutput<Self::Output, Self::Witness>, Self::Refusal> {
        let input_root = PassRoot::hash_parts(&[b"plan", input.envelope.plan_root().as_bytes()]);
        let typed_plan = input.bindings.bind_plan(&input.verified)?;
        let binding_root = input.bindings.schema_root();
        let artifact_root = PassRoot::hash_parts(&[
            b"bound-workflow",
            input.envelope.plan_root().as_bytes(),
            binding_root.as_bytes(),
        ]);
        let witness = WorkflowCompilationWitness {
            request_root: input.envelope.request().request_root(),
            execution_root: input.envelope.execution_root(),
            plan_root: input.envelope.plan_root(),
            binding_root,
            artifact_root,
        };
        let plan_root = input.envelope.plan_root();
        let prepared = PreparedWorkflow {
            artifact: transition_artifact(
                PreparedWorkflowValue {
                    envelope: input.envelope,
                    typed_plan,
                    binding_root,
                    witness,
                },
                artifact_root,
            ),
        };
        Ok(PassOutput {
            value: prepared,
            witness: (plan_root, binding_root, artifact_root),
            input_root,
            output_root: artifact_root,
        })
    }
}

type WorkflowPipelineRefusal<E> = PassChainRefusal<
    PassChainRefusal<PlanningPassRefusal, std::convert::Infallible>,
    E,
>;

fn map_pipeline_refusal<E>(error: WorkflowPipelineRefusal<E>) -> WorkflowApplicationError<E> {
    match error {
        PassChainRefusal::First(PassChainRefusal::First(
            PlanningPassRefusal::Planning(error),
        )) => WorkflowApplicationError::Planning(error),
        PassChainRefusal::First(PassChainRefusal::First(
            PlanningPassRefusal::Envelope(error),
        )) => WorkflowApplicationError::Envelope(error),
        PassChainRefusal::First(PassChainRefusal::Second(never)) => match never {},
        PassChainRefusal::First(PassChainRefusal::RootDiscontinuity {
            first_output,
            second_input,
        })
        | PassChainRefusal::RootDiscontinuity {
            first_output,
            second_input,
        } => WorkflowApplicationError::PassContinuity {
            first_output,
            second_input,
        },
        PassChainRefusal::Second(error) => WorkflowApplicationError::Binding(error),
    }
}

/// High-level compiler host for one resident domain and one native command schema.
///
/// The facade is the production `PlanPass` composition path. It performs only
/// reversible manufacture: planning, verification, envelope construction, and
/// command binding. It never invokes a command handler.
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
        self.compile_problem(
            problem,
            observation.root(),
            goal.root(),
            bounds,
            search_policy_root,
        )
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
        self.compile_problem(
            &problem,
            observation.root(),
            goal.root(),
            bounds,
            search_policy_root,
        )
    }

    fn compile_problem<P>(
        &mut self,
        problem: &P,
        observation_root: ObservationRoot,
        goal_root: GoalRoot,
        bounds: PlanningBounds,
        search_policy_root: SearchPolicyRoot,
    ) -> Result<PreparedWorkflow<B::Command>, WorkflowApplicationError<B::Error>>
    where
        P: WorkflowProblem + ?Sized,
    {
        let input = WorkflowCompileInput {
            workflow: &mut self.workflow,
            bindings: &self.bindings,
            problem,
            observation_root,
            goal_root,
            bounds,
            search_policy_root,
        };
        PlanWorkflowPass
            .then(ManufactureEnvelopePass)
            .then(BindCommandsPass)
            .apply(input)
            .map(|output| output.value)
            .map_err(map_pipeline_refusal)
    }
}

impl<C> PreparedWorkflow<C>
where
    C: Clone + Serialize,
{
    /// Evaluate institutional policy and manufacture a broker proposal.
    ///
    /// Admission remains a typestate transition; this method does not call a broker.
    pub fn authorize_and_propose<P, Context>(
        &self,
        policy: &P,
        context: &Context,
        idempotency: IdempotencyKey,
    ) -> Result<AuthorizedWorkflow<C, P::Evidence>, AuthorizationProposalError<P::Refusal>>
    where
        P: Policy<TypedWorkflowPlan<C>, Context>,
    {
        let evidence = match policy.evaluate(self.typed_plan(), context) {
            PolicyDecision::Admit(evidence) => evidence,
            PolicyDecision::Refuse(refusal) => {
                return Err(AuthorizationProposalError::Policy(refusal));
            }
        };
        let policy_root = policy.root();
        let proposal = DispatchProposal::from_typed_plan(
            self.typed_plan(),
            self.envelope(),
            self.binding_root(),
            Some(policy_root),
            idempotency,
        )
        .map_err(AuthorizationProposalError::Dispatch)?;
        let dispatch_root = proposal.root();
        let artifact_root = PassRoot::hash_parts(&[
            b"policy-admitted-workflow",
            self.artifact_root().as_bytes(),
            policy_root.as_bytes(),
            dispatch_root.as_bytes(),
        ]);
        Ok(AuthorizedWorkflow {
            artifact: transition_artifact(
                AuthorizedWorkflowValue {
                    evidence,
                    policy_root,
                    proposal,
                },
                artifact_root,
            ),
            witness: WorkflowAuthorizationWitness {
                prepared_root: self.artifact_root(),
                policy_root,
                dispatch_root,
                artifact_root,
            },
        })
    }
}
