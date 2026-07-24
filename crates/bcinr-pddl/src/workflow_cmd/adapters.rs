/// Content identity for a runtime-neutral adapter projection.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AdapterProjectionRoot([u8; 32]);

impl AdapterProjectionRoot {
    pub fn hash_parts(parts: &[&[u8]]) -> Self {
        Self(hash_domain("bcinr:workflow:adapter-projection:v1", parts))
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Display for AdapterProjectionRoot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&hex_encode(&self.0))
    }
}

impl fmt::Debug for AdapterProjectionRoot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AdapterProjectionRoot({self})")
    }
}

/// Pure projection from a dispatch proposal into an execution-runtime value.
pub trait DispatchAdapter<C> {
    type Output;
    type Error;

    fn project(&self, proposal: &DispatchProposal<C>) -> Result<Self::Output, Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterProjectionError {
    EmptyDestination,
    EmptyReason,
    Serialization(String),
}

impl fmt::Display for AdapterProjectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "dispatch adapter projection refused: {self:?}")
    }
}

impl std::error::Error for AdapterProjectionError {}

/// One semantically admitted scheduler tick for a local task-group runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TaskBatch<C> {
    root: AdapterProjectionRoot,
    dispatch_root: DispatchRoot,
    tick: u32,
    commands: Vec<CommandEnvelope<C>>,
}

impl<C> TaskBatch<C> {
    pub const fn root(&self) -> AdapterProjectionRoot {
        self.root
    }

    pub const fn dispatch_root(&self) -> DispatchRoot {
        self.dispatch_root
    }

    pub const fn tick(&self) -> u32 {
        self.tick
    }

    pub fn commands(&self) -> &[CommandEnvelope<C>] {
        &self.commands
    }
}

/// Reference projection for Tokio task groups, structured concurrency scopes,
/// or any local executor. It does not spawn tasks.
#[derive(Debug, Clone, Copy, Default)]
pub struct TaskGroupAdapter;

impl<C> DispatchAdapter<C> for TaskGroupAdapter
where
    C: Clone + Serialize,
{
    type Output = Vec<TaskBatch<C>>;
    type Error = AdapterProjectionError;

    fn project(&self, proposal: &DispatchProposal<C>) -> Result<Self::Output, Self::Error> {
        let mut by_tick = BTreeMap::<u32, Vec<CommandEnvelope<C>>>::new();
        for command in proposal.commands() {
            by_tick
                .entry(command.tick())
                .or_default()
                .push(command.clone());
        }
        by_tick
            .into_iter()
            .map(|(tick, commands)| {
                let encoded = serde_json::to_vec(&commands)
                    .map_err(|error| AdapterProjectionError::Serialization(error.to_string()))?;
                let root = AdapterProjectionRoot::hash_parts(&[
                    proposal.root().as_bytes(),
                    &tick.to_le_bytes(),
                    &encoded,
                ]);
                Ok(TaskBatch {
                    root,
                    dispatch_root: proposal.root(),
                    tick,
                    commands,
                })
            })
            .collect()
    }
}

/// Durable outbox row manufactured without choosing a database or transaction API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OutboxRecord<C> {
    root: AdapterProjectionRoot,
    destination: String,
    message_key: String,
    dispatch_root: DispatchRoot,
    plan_root: PlanRoot,
    execution_root: ExecutionRoot,
    tick: u32,
    command_index: u32,
    command: C,
}

impl<C> OutboxRecord<C> {
    pub const fn root(&self) -> AdapterProjectionRoot {
        self.root
    }

    pub fn destination(&self) -> &str {
        &self.destination
    }

    pub fn message_key(&self) -> &str {
        &self.message_key
    }

    pub const fn dispatch_root(&self) -> DispatchRoot {
        self.dispatch_root
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

/// Transactional-outbox projection. Persistence and publication remain application-owned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboxAdapter {
    destination: String,
}

impl OutboxAdapter {
    pub fn new(destination: impl Into<String>) -> Result<Self, AdapterProjectionError> {
        let destination = destination.into();
        if destination.trim().is_empty() {
            Err(AdapterProjectionError::EmptyDestination)
        } else {
            Ok(Self { destination })
        }
    }

    pub fn destination(&self) -> &str {
        &self.destination
    }
}

impl<C> DispatchAdapter<C> for OutboxAdapter
where
    C: Clone + Serialize,
{
    type Output = Vec<OutboxRecord<C>>;
    type Error = AdapterProjectionError;

    fn project(&self, proposal: &DispatchProposal<C>) -> Result<Self::Output, Self::Error> {
        proposal
            .commands()
            .iter()
            .map(|envelope| {
                let message_key = format!(
                    "{}:{}:{}",
                    proposal.root(),
                    envelope.tick(),
                    envelope.command_index()
                );
                let command = envelope.command().clone();
                let encoded = serde_json::to_vec(&command)
                    .map_err(|error| AdapterProjectionError::Serialization(error.to_string()))?;
                let root = AdapterProjectionRoot::hash_parts(&[
                    proposal.root().as_bytes(),
                    self.destination.as_bytes(),
                    message_key.as_bytes(),
                    &encoded,
                ]);
                Ok(OutboxRecord {
                    root,
                    destination: self.destination.clone(),
                    message_key,
                    dispatch_root: proposal.root(),
                    plan_root: envelope.plan_root(),
                    execution_root: envelope.execution_root(),
                    tick: envelope.tick(),
                    command_index: envelope.command_index(),
                    command,
                })
            })
            .collect()
    }
}

/// Human-review packet containing the same typed commands and semantic roots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApprovalPacket<C> {
    root: AdapterProjectionRoot,
    reason: String,
    dispatch_root: DispatchRoot,
    plan_root: PlanRoot,
    commands: Vec<CommandEnvelope<C>>,
}

impl<C> ApprovalPacket<C> {
    pub const fn root(&self) -> AdapterProjectionRoot {
        self.root
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }

    pub const fn dispatch_root(&self) -> DispatchRoot {
        self.dispatch_root
    }

    pub const fn plan_root(&self) -> PlanRoot {
        self.plan_root
    }

    pub fn commands(&self) -> &[CommandEnvelope<C>] {
        &self.commands
    }
}

/// Projection for manual approval, dual control, or policy escalation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalAdapter {
    reason: String,
}

impl ApprovalAdapter {
    pub fn new(reason: impl Into<String>) -> Result<Self, AdapterProjectionError> {
        let reason = reason.into();
        if reason.trim().is_empty() {
            Err(AdapterProjectionError::EmptyReason)
        } else {
            Ok(Self { reason })
        }
    }
}

impl<C> DispatchAdapter<C> for ApprovalAdapter
where
    C: Clone + Serialize,
{
    type Output = ApprovalPacket<C>;
    type Error = AdapterProjectionError;

    fn project(&self, proposal: &DispatchProposal<C>) -> Result<Self::Output, Self::Error> {
        let commands = proposal.commands().to_vec();
        let encoded = serde_json::to_vec(&commands)
            .map_err(|error| AdapterProjectionError::Serialization(error.to_string()))?;
        let root = AdapterProjectionRoot::hash_parts(&[
            proposal.root().as_bytes(),
            self.reason.as_bytes(),
            &encoded,
        ]);
        Ok(ApprovalPacket {
            root,
            reason: self.reason.clone(),
            dispatch_root: proposal.root(),
            plan_root: proposal.plan_root(),
            commands,
        })
    }
}
