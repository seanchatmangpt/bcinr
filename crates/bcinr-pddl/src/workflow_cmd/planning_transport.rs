/// Bounds are part of request identity and application policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanningBounds {
    pub max_observation_bytes: usize,
    pub max_facts: usize,
    pub max_ground_actions: usize,
    pub max_plan_depth: usize,
    pub max_search_states: usize,
    pub max_process_nodes: usize,
    pub max_receipt_bytes: usize,
}

impl PlanningBounds {
    pub const fn interactive() -> Self {
        Self {
            max_observation_bytes: 256 * 1024,
            max_facts: 16_384,
            max_ground_actions: 20_000,
            max_plan_depth: 32,
            max_search_states: 250_000,
            max_process_nodes: 64,
            max_receipt_bytes: 1024 * 1024,
        }
    }

    pub fn root(&self) -> BoundsRoot {
        let bytes = serde_json::to_vec(self).expect("PlanningBounds serialization is infallible");
        BoundsRoot::hash(&bytes)
    }
}

impl Default for PlanningBounds {
    fn default() -> Self {
        Self::interactive()
    }
}

/// Explicit identity for an application planning request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlanRequestEnvelope {
    version: u16,
    domain_source_root: DomainSourceRoot,
    observation_root: ObservationRoot,
    goal_root: GoalRoot,
    bounds: PlanningBounds,
    bounds_root: BoundsRoot,
    search_policy_root: SearchPolicyRoot,
    request_root: RequestRoot,
}

impl PlanRequestEnvelope {
    pub fn manufacture(
        domain_source_root: DomainSourceRoot,
        observation_root: ObservationRoot,
        goal_root: GoalRoot,
        bounds: PlanningBounds,
        search_policy_root: SearchPolicyRoot,
    ) -> Self {
        let bounds_root = bounds.root();
        let request_root = RequestRoot::hash_parts(&[
            domain_source_root.as_bytes(),
            observation_root.as_bytes(),
            goal_root.as_bytes(),
            bounds_root.as_bytes(),
            search_policy_root.as_bytes(),
        ]);
        Self {
            version: 1,
            domain_source_root,
            observation_root,
            goal_root,
            bounds,
            bounds_root,
            search_policy_root,
            request_root,
        }
    }

    pub const fn request_root(&self) -> RequestRoot {
        self.request_root
    }

    pub const fn domain_source_root(&self) -> DomainSourceRoot {
        self.domain_source_root
    }

    pub const fn observation_root(&self) -> ObservationRoot {
        self.observation_root
    }

    pub const fn goal_root(&self) -> GoalRoot {
        self.goal_root
    }

    pub const fn bounds(&self) -> PlanningBounds {
        self.bounds
    }

    pub const fn bounds_root(&self) -> BoundsRoot {
        self.bounds_root
    }

    pub const fn search_policy_root(&self) -> SearchPolicyRoot {
        self.search_policy_root
    }
}

/// Standing-bearing plan metadata. This type is serializable for inspection but
/// intentionally not deserializable as verified standing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlanEnvelope {
    version: u16,
    request: PlanRequestEnvelope,
    standing: CognitiveExecutionStanding,
    execution_root: ExecutionRoot,
    process_root: ProcessRoot,
    plan_root: PlanRoot,
}

impl PlanEnvelope {
    pub fn manufacture(
        request: PlanRequestEnvelope,
        standing: CognitiveExecutionStanding,
        execution_root: ExecutionRoot,
    ) -> Self {
        let standing_tag: &[u8] = match standing {
            CognitiveExecutionStanding::WitnessedConcurrentStrips => b"witnessed-concurrent-strips",
            CognitiveExecutionStanding::ExactSequentialClassical => b"exact-sequential-classical",
        };
        // The current downstream facade exposes one execution root that already
        // binds semantic input, selected plan, POWL choices, and execution. Until
        // a separate process root is exposed, derive a typed process identity
        // from that exact root rather than inventing independent evidence.
        let process_root = ProcessRoot::hash(execution_root.as_bytes());
        let plan_root = PlanRoot::hash_parts(&[
            request.request_root().as_bytes(),
            standing_tag,
            process_root.as_bytes(),
            execution_root.as_bytes(),
        ]);
        Self {
            version: 1,
            request,
            standing,
            execution_root,
            process_root,
            plan_root,
        }
    }

    pub const fn request(&self) -> &PlanRequestEnvelope {
        &self.request
    }

    pub const fn standing(&self) -> CognitiveExecutionStanding {
        self.standing
    }

    pub const fn execution_root(&self) -> ExecutionRoot {
        self.execution_root
    }

    pub const fn process_root(&self) -> ProcessRoot {
        self.process_root
    }

    pub const fn plan_root(&self) -> PlanRoot {
        self.plan_root
    }

    pub fn erase_for_transport(&self) -> UntrustedPlanEnvelope {
        UntrustedPlanEnvelope {
            version: self.version,
            request: (&self.request).into(),
            standing: self.standing,
            execution_root: self.execution_root,
            process_root: self.process_root,
            claimed_plan_root: self.plan_root,
        }
    }
}

impl ContentAddressed for PlanEnvelope {
    type Root = PlanRoot;

    fn content_root(&self) -> Self::Root {
        self.plan_root
    }
}

/// Untrusted plan transport. Verification against trusted evidence is required
/// before it can become a `PlanEnvelope` again.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UntrustedPlanEnvelope {
    pub version: u16,
    pub request: PlanRequestEnvelopeTransport,
    pub standing: CognitiveExecutionStanding,
    pub execution_root: ExecutionRoot,
    pub process_root: ProcessRoot,
    pub claimed_plan_root: PlanRoot,
}

/// Deserializable request metadata used only inside untrusted transport.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanRequestEnvelopeTransport {
    pub version: u16,
    pub domain_source_root: DomainSourceRoot,
    pub observation_root: ObservationRoot,
    pub goal_root: GoalRoot,
    pub bounds: PlanningBounds,
    pub bounds_root: BoundsRoot,
    pub search_policy_root: SearchPolicyRoot,
    pub request_root: RequestRoot,
}

impl From<PlanRequestEnvelope> for PlanRequestEnvelopeTransport {
    fn from(value: PlanRequestEnvelope) -> Self {
        Self {
            version: value.version,
            domain_source_root: value.domain_source_root,
            observation_root: value.observation_root,
            goal_root: value.goal_root,
            bounds: value.bounds,
            bounds_root: value.bounds_root,
            search_policy_root: value.search_policy_root,
            request_root: value.request_root,
        }
    }
}

impl From<&PlanRequestEnvelope> for PlanRequestEnvelopeTransport {
    fn from(value: &PlanRequestEnvelope) -> Self {
        value.clone().into()
    }
}

impl From<PlanRequestEnvelopeTransport> for PlanRequestEnvelope {
    fn from(value: PlanRequestEnvelopeTransport) -> Self {
        Self {
            version: value.version,
            domain_source_root: value.domain_source_root,
            observation_root: value.observation_root,
            goal_root: value.goal_root,
            bounds: value.bounds,
            bounds_root: value.bounds_root,
            search_policy_root: value.search_policy_root,
            request_root: value.request_root,
        }
    }
}

/// Trust refusal for transported planning evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportTrustError {
    VersionMismatch,
    RequestRootMismatch,
    BoundsRootMismatch,
    ProcessRootMismatch,
    PlanRootMismatch,
    TrustedEvidenceMismatch,
}

impl fmt::Display for TransportTrustError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "untrusted workflow transport refused: {self:?}")
    }
}

impl std::error::Error for TransportTrustError {}

impl UntrustedPlanEnvelope {
    pub fn verify_against(
        self,
        trusted: &PlanEnvelope,
    ) -> Result<PlanEnvelope, TransportTrustError> {
        if self.version != 1 || self.version != trusted.version {
            return Err(TransportTrustError::VersionMismatch);
        }
        if self.request.bounds.root() != self.request.bounds_root {
            return Err(TransportTrustError::BoundsRootMismatch);
        }
        let recomputed_request = RequestRoot::hash_parts(&[
            self.request.domain_source_root.as_bytes(),
            self.request.observation_root.as_bytes(),
            self.request.goal_root.as_bytes(),
            self.request.bounds_root.as_bytes(),
            self.request.search_policy_root.as_bytes(),
        ]);
        if recomputed_request != self.request.request_root {
            return Err(TransportTrustError::RequestRootMismatch);
        }
        if ProcessRoot::hash(self.execution_root.as_bytes()) != self.process_root {
            return Err(TransportTrustError::ProcessRootMismatch);
        }
        let request: PlanRequestEnvelope = self.request.into();
        let recomputed = PlanEnvelope::manufacture(request, self.standing, self.execution_root);
        if recomputed.plan_root != self.claimed_plan_root {
            return Err(TransportTrustError::PlanRootMismatch);
        }
        if &recomputed != trusted {
            return Err(TransportTrustError::TrustedEvidenceMismatch);
        }
        Ok(recomputed)
    }
}

/// Failure while manufacturing an application plan envelope from the current
/// embedded facade.
#[derive(Debug)]
pub enum PlanEnvelopeError {
    DomainRoot(RootParseError),
    ExecutionRoot(RootParseError),
}

impl fmt::Display for PlanEnvelopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DomainRoot(error) => write!(f, "installed domain root is invalid: {error}"),
            Self::ExecutionRoot(error) => write!(f, "verified execution root is invalid: {error}"),
        }
    }
}

impl std::error::Error for PlanEnvelopeError {}

impl EmbeddedWorkflow {
    /// Bind a verified plan to explicit observation, goal, bounds, and search
    /// policy identities. This is pure metadata manufacture.
    pub fn manufacture_plan_envelope(
        &self,
        verified: &VerifiedWorkflowPlan,
        observation_root: ObservationRoot,
        goal_root: GoalRoot,
        bounds: PlanningBounds,
        search_policy_root: SearchPolicyRoot,
    ) -> Result<PlanEnvelope, PlanEnvelopeError> {
        let domain_source_root = self
            .domain_source_root()
            .parse()
            .map_err(PlanEnvelopeError::DomainRoot)?;
        let execution_root = verified
            .execution_root()
            .parse()
            .map_err(PlanEnvelopeError::ExecutionRoot)?;
        let request = PlanRequestEnvelope::manufacture(
            domain_source_root,
            observation_root,
            goal_root,
            bounds,
            search_policy_root,
        );
        Ok(PlanEnvelope::manufacture(
            request,
            verified.standing(),
            execution_root,
        ))
    }
}
