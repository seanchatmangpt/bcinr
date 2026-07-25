/// Cursor-visible command state. Completed effects remain distinct from
/// admission and attempt evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandProgress {
    Pending,
    Admitted { dispatch_root: DispatchRoot },
    Attempted { attempt: u32 },
    EffectObserved { effect_root: EffectRoot },
    Refused { refusal_root: ReceiptRoot },
    Compensated { effect_root: EffectRoot },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CursorCommand {
    pub tick: u32,
    pub command_index: u32,
    pub progress: CommandProgress,
}

/// Explicit application cursor. It is not hidden engine state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkflowCursor {
    plan_root: PlanRoot,
    dispatch_root: DispatchRoot,
    next_tick: Option<u32>,
    generation: u32,
    superseded_by: Option<PlanRoot>,
    commands: Vec<CursorCommand>,
    root: CursorRoot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CursorError {
    DispatchRootMismatch,
    CommandNotFound { tick: u32, command_index: u32 },
    InvalidTransition,
    PlanAlreadySuperseded,
}

impl fmt::Display for CursorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "workflow cursor refused: {self:?}")
    }
}

impl std::error::Error for CursorError {}

impl WorkflowCursor {
    pub fn from_proposal<C>(proposal: &DispatchProposal<C>) -> Self {
        let commands = proposal
            .commands
            .iter()
            .map(|command| CursorCommand {
                tick: command.tick,
                command_index: command.command_index,
                progress: CommandProgress::Pending,
            })
            .collect::<Vec<_>>();
        let next_tick = commands.iter().map(|command| command.tick).min();
        let mut cursor = Self {
            plan_root: proposal.plan_root,
            dispatch_root: proposal.root,
            next_tick,
            generation: 0,
            superseded_by: None,
            commands,
            root: CursorRoot::ZERO,
        };
        cursor.recompute_root();
        cursor
    }

    pub const fn root(&self) -> CursorRoot {
        self.root
    }

    pub const fn plan_root(&self) -> PlanRoot {
        self.plan_root
    }

    pub const fn dispatch_root(&self) -> DispatchRoot {
        self.dispatch_root
    }

    pub const fn next_tick(&self) -> Option<u32> {
        self.next_tick
    }

    pub const fn generation(&self) -> u32 {
        self.generation
    }

    pub const fn superseded_by(&self) -> Option<PlanRoot> {
        self.superseded_by
    }

    pub fn commands(&self) -> &[CursorCommand] {
        &self.commands
    }

    pub fn next_ready(&self) -> Vec<(u32, u32)> {
        let Some(tick) = self.next_tick else {
            return Vec::new();
        };
        self.commands
            .iter()
            .filter(|command| {
                command.tick == tick && matches!(command.progress, CommandProgress::Pending)
            })
            .map(|command| (command.tick, command.command_index))
            .collect()
    }

    pub fn record_admission(&mut self, admission: &DispatchAdmission) -> Result<(), CursorError> {
        if admission.dispatch_root != self.dispatch_root {
            return Err(CursorError::DispatchRootMismatch);
        }
        let Some(tick) = self.next_tick else {
            return Ok(());
        };
        for command in self
            .commands
            .iter_mut()
            .filter(|command| command.tick == tick)
        {
            if matches!(command.progress, CommandProgress::Pending) {
                command.progress = CommandProgress::Admitted {
                    dispatch_root: admission.dispatch_root,
                };
            }
        }
        self.recompute_root();
        Ok(())
    }

    pub fn record_attempt(
        &mut self,
        tick: u32,
        command_index: u32,
        attempt: u32,
    ) -> Result<(), CursorError> {
        let command = self.command_mut(tick, command_index)?;
        if !matches!(command.progress, CommandProgress::Admitted { .. }) {
            return Err(CursorError::InvalidTransition);
        }
        command.progress = CommandProgress::Attempted { attempt };
        self.recompute_root();
        Ok(())
    }

    pub fn record_effect(
        &mut self,
        tick: u32,
        command_index: u32,
        effect_root: EffectRoot,
    ) -> Result<(), CursorError> {
        let command = self.command_mut(tick, command_index)?;
        if !matches!(
            command.progress,
            CommandProgress::Admitted { .. } | CommandProgress::Attempted { .. }
        ) {
            return Err(CursorError::InvalidTransition);
        }
        command.progress = CommandProgress::EffectObserved { effect_root };
        self.advance_completed_ticks();
        self.recompute_root();
        Ok(())
    }

    pub fn supersede(&mut self, replacement: PlanRoot) -> Result<(), CursorError> {
        if self.superseded_by.is_some() {
            return Err(CursorError::PlanAlreadySuperseded);
        }
        self.superseded_by = Some(replacement);
        self.generation = self.generation.saturating_add(1);
        self.recompute_root();
        Ok(())
    }

    pub fn erase_for_transport(&self) -> WorkflowCursorTransport {
        WorkflowCursorTransport {
            version: 1,
            plan_root: self.plan_root,
            dispatch_root: self.dispatch_root,
            next_tick: self.next_tick,
            generation: self.generation,
            superseded_by: self.superseded_by,
            commands: self.commands.clone(),
            claimed_root: self.root,
        }
    }

    fn command_mut(
        &mut self,
        tick: u32,
        command_index: u32,
    ) -> Result<&mut CursorCommand, CursorError> {
        self.commands
            .iter_mut()
            .find(|command| command.tick == tick && command.command_index == command_index)
            .ok_or(CursorError::CommandNotFound {
                tick,
                command_index,
            })
    }

    fn advance_completed_ticks(&mut self) {
        loop {
            let Some(tick) = self.next_tick else {
                return;
            };
            let mut current = self.commands.iter().filter(|command| command.tick == tick);
            let Some(first) = current.next() else {
                return;
            };
            let complete = matches!(
                first.progress,
                CommandProgress::EffectObserved { .. } | CommandProgress::Compensated { .. }
            ) && current.all(|command| {
                matches!(
                    command.progress,
                    CommandProgress::EffectObserved { .. }
                        | CommandProgress::Compensated { .. }
                )
            });
            if !complete {
                return;
            }
            self.next_tick = self
                .commands
                .iter()
                .filter(|command| command.tick > tick)
                .map(|command| command.tick)
                .min();
        }
    }

    fn recompute_root(&mut self) {
        #[derive(Serialize)]
        struct CursorRootMaterial<'a> {
            plan_root: PlanRoot,
            dispatch_root: DispatchRoot,
            next_tick: Option<u32>,
            generation: u32,
            superseded_by: Option<PlanRoot>,
            commands: &'a [CursorCommand],
        }
        let material = CursorRootMaterial {
            plan_root: self.plan_root,
            dispatch_root: self.dispatch_root,
            next_tick: self.next_tick,
            generation: self.generation,
            superseded_by: self.superseded_by,
            commands: &self.commands,
        };
        let encoded = serde_json::to_vec(&material).expect("cursor serialization is infallible");
        self.root = CursorRoot::hash(&encoded);
    }
}

/// Deserializable cursor proposal with no restored standing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowCursorTransport {
    pub version: u16,
    pub plan_root: PlanRoot,
    pub dispatch_root: DispatchRoot,
    pub next_tick: Option<u32>,
    pub generation: u32,
    pub superseded_by: Option<PlanRoot>,
    pub commands: Vec<CursorCommand>,
    pub claimed_root: CursorRoot,
}

impl WorkflowCursorTransport {
    pub fn replay_and_restore(
        self,
        trusted_plan_root: PlanRoot,
        trusted_dispatch_root: DispatchRoot,
    ) -> Result<WorkflowCursor, CursorError> {
        if self.plan_root != trusted_plan_root || self.dispatch_root != trusted_dispatch_root {
            return Err(CursorError::DispatchRootMismatch);
        }
        let claimed_root = self.claimed_root;
        let mut cursor = WorkflowCursor {
            plan_root: self.plan_root,
            dispatch_root: self.dispatch_root,
            next_tick: self.next_tick,
            generation: self.generation,
            superseded_by: self.superseded_by,
            commands: self.commands,
            root: CursorRoot::ZERO,
        };
        cursor.recompute_root();
        if cursor.root != claimed_root {
            return Err(CursorError::InvalidTransition);
        }
        Ok(cursor)
    }
}
