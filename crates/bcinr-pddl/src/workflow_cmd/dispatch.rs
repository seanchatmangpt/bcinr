/// Stable idempotency identity supplied by the application.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    pub fn new(value: impl Into<String>) -> Result<Self, DispatchProposalError> {
        let value = value.into();
        if value.trim().is_empty() {
            Err(DispatchProposalError::EmptyIdempotencyKey)
        } else {
            Ok(Self(value))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One typed command tied to its planning evidence and scheduler position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CommandEnvelope<C> {
    plan_root: PlanRoot,
    execution_root: ExecutionRoot,
    binding_root: BindingSchemaRoot,
    policy_root: Option<PolicySetRoot>,
    tick: u32,
    command_index: u32,
    command: C,
}

impl<C> CommandEnvelope<C> {
    pub const fn plan_root(&self) -> PlanRoot {
        self.plan_root
    }

    pub const fn execution_root(&self) -> ExecutionRoot {
        self.execution_root
    }

    pub const fn binding_root(&self) -> BindingSchemaRoot {
        self.binding_root
    }

    pub const fn policy_root(&self) -> Option<PolicySetRoot> {
        self.policy_root
    }

    pub const fn tick(&self) -> u32 {
        self.tick
    }

    pub const fn command_index(&self) -> u32 {
        self.command_index
    }

    pub const fn command(&self) -> &C {
        &self.command
    }
}

/// Standing-bearing broker proposal. It is serializable for audit but not
/// deserializable as authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DispatchProposal<C> {
    root: DispatchRoot,
    plan_root: PlanRoot,
    execution_root: ExecutionRoot,
    idempotency: IdempotencyKey,
    commands: Vec<CommandEnvelope<C>>,
}

/// Dispatch manufacture refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DispatchProposalError {
    EmptyIdempotencyKey,
    StandingMismatch,
    ExecutionRootMismatch,
    Serialization(String),
}

impl fmt::Display for DispatchProposalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "dispatch proposal refused: {self:?}")
    }
}

impl std::error::Error for DispatchProposalError {}

impl<C> DispatchProposal<C>
where
    C: Clone + Serialize,
{
    pub fn from_typed_plan(
        typed: &TypedWorkflowPlan<C>,
        envelope: &PlanEnvelope,
        binding_root: BindingSchemaRoot,
        policy_root: Option<PolicySetRoot>,
        idempotency: IdempotencyKey,
    ) -> Result<Self, DispatchProposalError> {
        if typed.standing() != envelope.standing() {
            return Err(DispatchProposalError::StandingMismatch);
        }
        let typed_execution: ExecutionRoot =
            typed
                .execution_root()
                .parse()
                .map_err(|error: RootParseError| {
                    DispatchProposalError::Serialization(error.to_string())
                })?;
        if typed_execution != envelope.execution_root() {
            return Err(DispatchProposalError::ExecutionRootMismatch);
        }
        let mut commands = Vec::new();
        for batch in typed.batches() {
            for (command_index, command) in batch.actions().iter().cloned().enumerate() {
                commands.push(CommandEnvelope {
                    plan_root: envelope.plan_root(),
                    execution_root: envelope.execution_root(),
                    binding_root,
                    policy_root,
                    tick: batch.tick(),
                    command_index: command_index as u32,
                    command,
                });
            }
        }
        let encoded = serde_json::to_vec(&commands)
            .map_err(|error| DispatchProposalError::Serialization(error.to_string()))?;
        let policy_identity = policy_root.unwrap_or(PolicySetRoot::ZERO);
        let root = DispatchRoot::hash_parts(&[
            envelope.plan_root().as_bytes(),
            binding_root.as_bytes(),
            policy_identity.as_bytes(),
            idempotency.as_str().as_bytes(),
            &encoded,
        ]);
        Ok(Self {
            root,
            plan_root: envelope.plan_root(),
            execution_root: envelope.execution_root(),
            idempotency,
            commands,
        })
    }

    pub const fn root(&self) -> DispatchRoot {
        self.root
    }

    pub const fn plan_root(&self) -> PlanRoot {
        self.plan_root
    }

    pub const fn execution_root(&self) -> ExecutionRoot {
        self.execution_root
    }

    pub fn idempotency(&self) -> &IdempotencyKey {
        &self.idempotency
    }

    pub fn commands(&self) -> &[CommandEnvelope<C>] {
        &self.commands
    }
}

/// Broker admission evidence. It proves proposal admission, not command effect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DispatchAdmission {
    dispatch_root: DispatchRoot,
    receipt_root: ReceiptRoot,
}

impl DispatchAdmission {
    pub const fn dispatch_root(&self) -> DispatchRoot {
        self.dispatch_root
    }

    pub const fn receipt_root(&self) -> ReceiptRoot {
        self.receipt_root
    }
}

pub trait BatchBroker<C> {
    type Admission;
    type Refusal;

    fn admit_batch(
        &mut self,
        proposal: &DispatchProposal<C>,
    ) -> Result<Self::Admission, Self::Refusal>;
}

pub trait EffectObserver<C> {
    type Observation;
    type Error;

    fn observe_effect(
        &mut self,
        command: &CommandEnvelope<C>,
    ) -> Result<Self::Observation, Self::Error>;
}

/// Deterministic in-memory broker fake for Chicago-style application tests. It
/// records real proposals and refuses duplicate idempotency keys.
#[derive(Debug, Default)]
pub struct RecordingBroker {
    admitted: Vec<DispatchRoot>,
    idempotency: BTreeSet<IdempotencyKey>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordingBrokerRefusal {
    DuplicateIdempotency(IdempotencyKey),
}

impl RecordingBroker {
    pub fn admitted_roots(&self) -> &[DispatchRoot] {
        &self.admitted
    }
}

impl<C> BatchBroker<C> for RecordingBroker {
    type Admission = DispatchAdmission;
    type Refusal = RecordingBrokerRefusal;

    fn admit_batch(
        &mut self,
        proposal: &DispatchProposal<C>,
    ) -> Result<Self::Admission, Self::Refusal> {
        if !self.idempotency.insert(proposal.idempotency.clone()) {
            return Err(RecordingBrokerRefusal::DuplicateIdempotency(
                proposal.idempotency.clone(),
            ));
        }
        self.admitted.push(proposal.root);
        let receipt_root = ReceiptRoot::hash_parts(&[
            proposal.root.as_bytes(),
            proposal.idempotency.as_str().as_bytes(),
        ]);
        Ok(DispatchAdmission {
            dispatch_root: proposal.root,
            receipt_root,
        })
    }
}
