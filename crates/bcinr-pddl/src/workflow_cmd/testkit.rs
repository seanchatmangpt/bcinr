/// Deterministic effect evidence manufactured by the testkit observer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectObservationRecord {
    pub plan_root: PlanRoot,
    pub execution_root: ExecutionRoot,
    pub tick: u32,
    pub command_index: u32,
    pub effect_root: EffectRoot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectObservationError {
    Serialization(String),
}

impl fmt::Display for EffectObservationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "deterministic effect observer refused: {self:?}")
    }
}

impl std::error::Error for EffectObservationError {}

/// Recording observer for Chicago-style tests and simulations.
///
/// It observes the exact command envelope and manufactures deterministic effect
/// evidence. It does not execute the command or claim a real external effect.
#[derive(Debug, Default)]
pub struct RecordingEffectObserver {
    observations: Vec<EffectObservationRecord>,
}

impl RecordingEffectObserver {
    pub fn observations(&self) -> &[EffectObservationRecord] {
        &self.observations
    }
}

impl<C> EffectObserver<C> for RecordingEffectObserver
where
    C: Serialize,
{
    type Observation = EffectObservationRecord;
    type Error = EffectObservationError;

    fn observe_effect(
        &mut self,
        command: &CommandEnvelope<C>,
    ) -> Result<Self::Observation, Self::Error> {
        let encoded = serde_json::to_vec(command)
            .map_err(|error| EffectObservationError::Serialization(error.to_string()))?;
        let effect_root = EffectRoot::hash_parts(&[
            command.execution_root().as_bytes(),
            &command.tick().to_le_bytes(),
            &command.command_index().to_le_bytes(),
            &encoded,
        ]);
        let observation = EffectObservationRecord {
            plan_root: command.plan_root(),
            execution_root: command.execution_root(),
            tick: command.tick(),
            command_index: command.command_index(),
            effect_root,
        };
        self.observations.push(observation.clone());
        Ok(observation)
    }
}

#[derive(Debug)]
pub enum ScenarioExecutionError {
    MissingCommand { tick: u32, command_index: u32 },
    Observer(EffectObservationError),
    Cursor(CursorError),
}

impl fmt::Display for ScenarioExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "workflow scenario execution refused: {self:?}")
    }
}

impl std::error::Error for ScenarioExecutionError {}

/// Complete the cursor's current admitted tick using deterministic effect evidence.
///
/// This helper drives real public behavior through the broker/cursor boundary,
/// while keeping all external effects simulated and inspectable.
pub fn complete_ready_tick<C>(
    proposal: &DispatchProposal<C>,
    cursor: &mut WorkflowCursor,
    observer: &mut RecordingEffectObserver,
) -> Result<Vec<EffectObservationRecord>, ScenarioExecutionError>
where
    C: Serialize,
{
    let Some(tick) = cursor.next_tick() else {
        return Ok(Vec::new());
    };
    let ready = cursor
        .commands()
        .iter()
        .filter(|command| {
            command.tick == tick
                && matches!(
                    command.progress,
                    CommandProgress::Admitted { .. } | CommandProgress::Attempted { .. }
                )
        })
        .map(|command| (command.tick, command.command_index))
        .collect::<Vec<_>>();
    let mut observations = Vec::with_capacity(ready.len());
    for (tick, command_index) in ready {
        let command = proposal
            .commands()
            .iter()
            .find(|command| {
                command.tick() == tick && command.command_index() == command_index
            })
            .ok_or(ScenarioExecutionError::MissingCommand {
                tick,
                command_index,
            })?;
        let observation = observer
            .observe_effect(command)
            .map_err(ScenarioExecutionError::Observer)?;
        cursor
            .record_effect(tick, command_index, observation.effect_root)
            .map_err(ScenarioExecutionError::Cursor)?;
        observations.push(observation);
    }
    Ok(observations)
}

/// Manufacture the minimal evidence chain for one planning-native scenario.
pub fn scenario_receipt_chain<C, E>(
    prepared: &PreparedWorkflow<C>,
    authorized: &AuthorizedWorkflow<C, E>,
    admission: &DispatchAdmission,
    cursor: &WorkflowCursor,
    effects: &[EffectObservationRecord],
) -> WorkflowReceiptChain {
    let mut chain = WorkflowReceiptChain::default();
    let request = prepared.envelope().request();
    chain.append(ReceiptSubject::Observation(request.observation_root()));
    chain.append(ReceiptSubject::Goal(request.goal_root()));
    chain.append(ReceiptSubject::Plan(prepared.envelope().plan_root()));
    chain.append(ReceiptSubject::Process(prepared.envelope().process_root()));
    chain.append(ReceiptSubject::Execution(prepared.envelope().execution_root()));
    chain.append(ReceiptSubject::Binding(prepared.binding_root()));
    chain.append(ReceiptSubject::Policy(authorized.policy_root()));
    chain.append(ReceiptSubject::Dispatch(admission.dispatch_root()));
    for effect in effects {
        chain.append(ReceiptSubject::Effect(effect.effect_root));
    }
    chain.append(ReceiptSubject::Cursor(cursor.root()));
    chain
}

/// Stable assertions over public behavior without coupling tests to internal layout.
pub struct WorkflowAssertions;

impl WorkflowAssertions {
    pub fn binding_complete(report: &BindingCoverageReport) {
        assert!(
            report.is_complete(),
            "binding coverage incomplete: missing={:?}, extra={:?}",
            report.missing_bindings,
            report.extra_bindings
        );
    }

    pub fn receipt_chain_valid(chain: &WorkflowReceiptChain) {
        chain.verify().expect("receipt chain must replay");
    }

    pub fn cursor_complete(cursor: &WorkflowCursor) {
        assert_eq!(cursor.next_tick(), None, "cursor still contains pending work");
    }

    pub fn same_plan<C, D>(left: &PreparedWorkflow<C>, right: &PreparedWorkflow<D>) {
        assert_eq!(
            left.envelope().plan_root(),
            right.envelope().plan_root(),
            "equivalent compilation inputs must manufacture the same plan root"
        );
    }
}
