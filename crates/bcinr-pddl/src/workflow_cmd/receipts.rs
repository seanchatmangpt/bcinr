/// Receipt subject with typed semantic identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReceiptSubject {
    Observation(ObservationRoot),
    Goal(GoalRoot),
    Plan(PlanRoot),
    Process(ProcessRoot),
    Execution(ExecutionRoot),
    Binding(BindingSchemaRoot),
    Policy(PolicySetRoot),
    Dispatch(DispatchRoot),
    Effect(EffectRoot),
    Cursor(CursorRoot),
}

impl ReceiptSubject {
    fn tag(self) -> &'static [u8] {
        match self {
            Self::Observation(_) => b"observation",
            Self::Goal(_) => b"goal",
            Self::Plan(_) => b"plan",
            Self::Process(_) => b"process",
            Self::Execution(_) => b"execution",
            Self::Binding(_) => b"binding",
            Self::Policy(_) => b"policy",
            Self::Dispatch(_) => b"dispatch",
            Self::Effect(_) => b"effect",
            Self::Cursor(_) => b"cursor",
        }
    }

    fn bytes(self) -> [u8; 32] {
        match self {
            Self::Observation(root) => *root.as_bytes(),
            Self::Goal(root) => *root.as_bytes(),
            Self::Plan(root) => *root.as_bytes(),
            Self::Process(root) => *root.as_bytes(),
            Self::Execution(root) => *root.as_bytes(),
            Self::Binding(root) => *root.as_bytes(),
            Self::Policy(root) => *root.as_bytes(),
            Self::Dispatch(root) => *root.as_bytes(),
            Self::Effect(root) => *root.as_bytes(),
            Self::Cursor(root) => *root.as_bytes(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowReceiptRecord {
    pub sequence: u64,
    pub subject: ReceiptSubject,
    pub parent: Option<ReceiptRoot>,
    pub root: ReceiptRoot,
}

/// Typed, parent-linked receipt chain.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct WorkflowReceiptChain {
    records: Vec<WorkflowReceiptRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReceiptChainError {
    SequenceMismatch { expected: u64, actual: u64 },
    ParentMismatch { sequence: u64 },
    RootMismatch { sequence: u64 },
}

impl fmt::Display for ReceiptChainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "workflow receipt chain refused: {self:?}")
    }
}

impl std::error::Error for ReceiptChainError {}

impl WorkflowReceiptChain {
    pub fn append(&mut self, subject: ReceiptSubject) -> ReceiptRoot {
        let sequence = self.records.len() as u64;
        let parent = self.records.last().map(|record| record.root);
        let subject_bytes = subject.bytes();
        let parent_bytes = parent.unwrap_or(ReceiptRoot::ZERO);
        let root = ReceiptRoot::hash_parts(&[
            &sequence.to_le_bytes(),
            subject.tag(),
            &subject_bytes,
            parent_bytes.as_bytes(),
        ]);
        self.records.push(WorkflowReceiptRecord {
            sequence,
            subject,
            parent,
            root,
        });
        root
    }

    pub fn records(&self) -> &[WorkflowReceiptRecord] {
        &self.records
    }

    pub fn root(&self) -> Option<ReceiptRoot> {
        self.records.last().map(|record| record.root)
    }

    pub fn verify(&self) -> Result<(), ReceiptChainError> {
        let mut parent = None;
        for (index, record) in self.records.iter().enumerate() {
            let expected_sequence = index as u64;
            if record.sequence != expected_sequence {
                return Err(ReceiptChainError::SequenceMismatch {
                    expected: expected_sequence,
                    actual: record.sequence,
                });
            }
            if record.parent != parent {
                return Err(ReceiptChainError::ParentMismatch {
                    sequence: record.sequence,
                });
            }
            let subject_bytes = record.subject.bytes();
            let parent_bytes = parent.unwrap_or(ReceiptRoot::ZERO);
            let expected = ReceiptRoot::hash_parts(&[
                &record.sequence.to_le_bytes(),
                record.subject.tag(),
                &subject_bytes,
                parent_bytes.as_bytes(),
            ]);
            if expected != record.root {
                return Err(ReceiptChainError::RootMismatch {
                    sequence: record.sequence,
                });
            }
            parent = Some(record.root);
        }
        Ok(())
    }
}
