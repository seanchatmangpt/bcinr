/// Deterministic logical time supplied by the application boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LogicalTime(pub u64);

/// Version of the application state source used for stale-observation checks.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SourceVersion(pub String);

/// Canonical bounded observation value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ObservationSnapshot<O> {
    observed_at: LogicalTime,
    source_version: SourceVersion,
    root: ObservationRoot,
    value: O,
}

impl<O: Serialize> ObservationSnapshot<O> {
    pub fn manufacture(
        observed_at: LogicalTime,
        source_version: SourceVersion,
        value: O,
    ) -> Result<Self, serde_json::Error> {
        let encoded = serde_json::to_vec(&value)?;
        let root = ObservationRoot::hash_parts(&[
            &observed_at.0.to_le_bytes(),
            source_version.0.as_bytes(),
            &encoded,
        ]);
        Ok(Self {
            observed_at,
            source_version,
            root,
            value,
        })
    }
}

impl<O> ObservationSnapshot<O> {
    pub const fn root(&self) -> ObservationRoot {
        self.root
    }

    pub const fn observed_at(&self) -> LogicalTime {
        self.observed_at
    }

    pub fn source_version(&self) -> &SourceVersion {
        &self.source_version
    }

    pub const fn value(&self) -> &O {
        &self.value
    }
}

/// Root-aware observation delta. Applying the delta remains front-end-specific.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservationDelta<D> {
    pub base: ObservationRoot,
    pub root: ObservationRoot,
    pub delta: D,
}

impl<D: Serialize> ObservationDelta<D> {
    pub fn manufacture(base: ObservationRoot, delta: D) -> Result<Self, serde_json::Error> {
        let encoded = serde_json::to_vec(&delta)?;
        let root = ObservationRoot::hash_parts(&[base.as_bytes(), &encoded]);
        Ok(Self { base, root, delta })
    }
}

/// Domain-neutral goal expression. A semantic rail must still explicitly admit
/// each constructor it supports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoalExpr<A, N> {
    Atom(A),
    Not(Box<Self>),
    All(Vec<Self>),
    Any(Vec<Self>),
    Exists { variable: String, body: Box<Self> },
    ForAll { variable: String, body: Box<Self> },
    Numeric(N),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GoalPriority(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoalPolicy {
    Hard,
    Soft,
}

/// Goal plus its exact identity and application policy metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GoalEnvelope<G> {
    goal: G,
    priority: GoalPriority,
    deadline: Option<LogicalTime>,
    policy: GoalPolicy,
    root: GoalRoot,
}

impl<G: Serialize> GoalEnvelope<G> {
    pub fn manufacture(
        goal: G,
        priority: GoalPriority,
        deadline: Option<LogicalTime>,
        policy: GoalPolicy,
    ) -> Result<Self, serde_json::Error> {
        let encoded = serde_json::to_vec(&(&goal, priority, deadline, policy))?;
        let root = GoalRoot::hash(&encoded);
        Ok(Self {
            goal,
            priority,
            deadline,
            policy,
            root,
        })
    }
}

impl<G> GoalEnvelope<G> {
    pub const fn root(&self) -> GoalRoot {
        self.root
    }

    pub const fn goal(&self) -> &G {
        &self.goal
    }

    pub const fn priority(&self) -> GoalPriority {
        self.priority
    }

    pub const fn deadline(&self) -> Option<LogicalTime> {
        self.deadline
    }

    pub const fn policy(&self) -> GoalPolicy {
        self.policy
    }
}

/// A value that exposes content-addressed identity without implying verification.
pub trait ContentAddressed {
    type Root: Copy + Eq;

    fn content_root(&self) -> Self::Root;
}

/// Reference to a semantic artifact. Possessing a reference does not imply
/// possession of the artifact or restoration of its standing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRef<R> {
    root: R,
}

impl<R: Copy> ArtifactRef<R> {
    pub const fn new(root: R) -> Self {
        Self { root }
    }

    pub const fn root(&self) -> R {
        self.root
    }
}

/// Candidate standing. The artifact has not yet earned semantic admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Candidate;
/// Admitted standing. The front end and capability profile accepted the artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Admitted;
/// Planned standing. Search selected a candidate course of action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Planned;
/// Process-verified standing. Process execution and replay succeeded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessVerified;
/// Command-bound standing. Symbolic actions were converted to native commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundToCommands;
/// Policy-admitted standing. Institutional policy admitted the typed work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PolicyAdmitted;
/// Dispatch-admitted standing. A broker accepted an attempt proposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DispatchAdmitted;
/// Effect-observed standing. An application observer recorded a consequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectObserved;

/// Generic standing-bearing value for compiler rails and facade transitions.
///
/// `WorkflowApplication` uses this ladder internally for its concrete aggregates:
/// `PreparedWorkflow` carries `BoundToCommands` standing and `AuthorizedWorkflow`
/// carries `PolicyAdmitted` standing. Custom `SemanticRail` implementations use the
/// same primitive, and `erase_for_transport` explicitly removes standing. Named
/// facade aggregates preserve domain-specific accessors while the shared artifact
/// type prevents candidate, bound, authorized, and observed values from collapsing.
///
/// Construction is intentionally narrow; transport erases standing and lawful
/// transitions manufacture a new root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact<T, S> {
    value: T,
    root: PassRoot,
    standing: PhantomData<S>,
}

impl<T> Artifact<T, Candidate> {
    pub fn candidate(value: T, root: PassRoot) -> Self {
        Self {
            value,
            root,
            standing: PhantomData,
        }
    }
}

impl<T, S> Artifact<T, S> {
    pub fn value(&self) -> &T {
        &self.value
    }

    pub const fn root(&self) -> PassRoot {
        self.root
    }

    /// Representation-only mapping. The caller is responsible for using this
    /// only when the semantic content is unchanged.
    pub fn map_value<U>(self, map: impl FnOnce(T) -> U) -> Artifact<U, S> {
        Artifact {
            value: map(self.value),
            root: self.root,
            standing: PhantomData,
        }
    }

    pub fn try_transform<U, S2, W, E>(
        self,
        transform: impl FnOnce(T) -> Result<(U, PassRoot, W), E>,
    ) -> Result<(Artifact<U, S2>, W), E> {
        let (value, root, witness) = transform(self.value)?;
        Ok((
            Artifact {
                value,
                root,
                standing: PhantomData,
            },
            witness,
        ))
    }

    pub fn erase_for_transport(self) -> UntrustedArtifact<T> {
        UntrustedArtifact {
            version: 1,
            value: self.value,
            claimed_root: self.root,
        }
    }
}

/// Transport shape that explicitly carries no standing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UntrustedArtifact<T> {
    pub version: u16,
    pub value: T,
    pub claimed_root: PassRoot,
}
