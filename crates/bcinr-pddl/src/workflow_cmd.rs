//! Design-for-Combinatorial-Maximalism application contracts.
//!
//! This module turns the embedded planning facade into a compiler-shaped
//! application boundary. It contains value objects and contracts only: no
//! command handler is invoked and no external side effect is performed here.

#![cfg(feature = "mfw-planner")]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{
    ActionInvocation, CognitiveExecutionStanding, EmbeddedWorkflow, TypedWorkflowPlan,
    VerifiedWorkflowPlan,
};

fn hash_domain(context: &str, parts: &[&[u8]]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(context.as_bytes());
    hasher.update(&[0]);
    for part in parts {
        hasher.update(&(part.len() as u64).to_le_bytes());
        hasher.update(part);
    }
    *hasher.finalize().as_bytes()
}

fn hex_encode(bytes: &[u8; 32]) -> String {
    let mut out = String::with_capacity(64);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(out, "{byte:02x}");
    }
    out
}

fn hex_decode(value: &str) -> Result<[u8; 32], RootParseError> {
    if value.len() != 64 {
        return Err(RootParseError::InvalidLength {
            actual: value.len(),
        });
    }
    let mut bytes = [0u8; 32];
    for (index, slot) in bytes.iter_mut().enumerate() {
        let start = index * 2;
        *slot = u8::from_str_radix(&value[start..start + 2], 16)
            .map_err(|_| RootParseError::InvalidHex { index: start })?;
    }
    Ok(bytes)
}

/// Refusal to parse a typed semantic root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RootParseError {
    InvalidLength { actual: usize },
    InvalidHex { index: usize },
}

impl fmt::Display for RootParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength { actual } => {
                write!(f, "semantic root has length {actual}; expected 64 hex characters")
            }
            Self::InvalidHex { index } => {
                write!(f, "semantic root contains invalid hexadecimal data at byte {index}")
            }
        }
    }
}

impl std::error::Error for RootParseError {}

macro_rules! semantic_root {
    ($name:ident, $context:literal) => {
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        pub struct $name([u8; 32]);

        impl $name {
            pub const ZERO: Self = Self([0u8; 32]);

            pub fn hash(bytes: &[u8]) -> Self {
                Self(hash_domain($context, &[bytes]))
            }

            pub fn hash_parts(parts: &[&[u8]]) -> Self {
                Self(hash_domain($context, parts))
            }

            pub const fn from_bytes(bytes: [u8; 32]) -> Self {
                Self(bytes)
            }

            pub const fn as_bytes(&self) -> &[u8; 32] {
                &self.0
            }

            pub fn to_hex(&self) -> String {
                hex_encode(&self.0)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&hex_encode(&self.0))
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), self)
            }
        }

        impl FromStr for $name {
            type Err = RootParseError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                hex_decode(value).map(Self)
            }
        }
    };
}

semantic_root!(DomainSourceRoot, "bcinr:workflow:domain-source:v1");
semantic_root!(CompiledDomainRoot, "bcinr:workflow:compiled-domain:v1");
semantic_root!(ObservationRoot, "bcinr:workflow:observation:v1");
semantic_root!(GoalRoot, "bcinr:workflow:goal:v1");
semantic_root!(BoundsRoot, "bcinr:workflow:bounds:v1");
semantic_root!(SearchPolicyRoot, "bcinr:workflow:search-policy:v1");
semantic_root!(RequestRoot, "bcinr:workflow:request:v1");
semantic_root!(PlanRoot, "bcinr:workflow:plan:v1");
semantic_root!(ProcessRoot, "bcinr:workflow:process:v1");
semantic_root!(ExecutionRoot, "bcinr:workflow:execution:v1");
semantic_root!(BindingSchemaRoot, "bcinr:workflow:binding-schema:v1");
semantic_root!(PolicySetRoot, "bcinr:workflow:policy-set:v1");
semantic_root!(DispatchRoot, "bcinr:workflow:dispatch:v1");
semantic_root!(EffectRoot, "bcinr:workflow:effect:v1");
semantic_root!(CursorRoot, "bcinr:workflow:cursor:v1");
semantic_root!(ReceiptRoot, "bcinr:workflow:receipt:v1");
semantic_root!(PassRoot, "bcinr:workflow:pass:v1");

/// Semantic version carried by a compiled action theory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DomainVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl DomainVersion {
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub const fn is_backward_compatible_with(self, required: Self) -> bool {
        self.major == required.major
            && (self.minor > required.minor
                || (self.minor == required.minor && self.patch >= required.patch))
    }
}

/// Compiled domain metadata plus application-owned compiled representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionedDomain<D> {
    source_root: DomainSourceRoot,
    compiled_root: CompiledDomainRoot,
    version: DomainVersion,
    actions: BTreeSet<String>,
    inner: D,
}

impl<D> VersionedDomain<D> {
    pub fn new(
        source_root: DomainSourceRoot,
        compiled_root: CompiledDomainRoot,
        version: DomainVersion,
        actions: impl IntoIterator<Item = String>,
        inner: D,
    ) -> Self {
        Self {
            source_root,
            compiled_root,
            version,
            actions: actions.into_iter().collect(),
            inner,
        }
    }

    pub const fn source_root(&self) -> DomainSourceRoot {
        self.source_root
    }

    pub const fn compiled_root(&self) -> CompiledDomainRoot {
        self.compiled_root
    }

    pub const fn version(&self) -> DomainVersion {
        self.version
    }

    pub fn actions(&self) -> &BTreeSet<String> {
        &self.actions
    }

    pub const fn inner(&self) -> &D {
        &self.inner
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainRegistryError {
    DuplicateSourceRoot(DomainSourceRoot),
    MissingDomain(DomainSourceRoot),
    IncompatibleVersion {
        available: DomainVersion,
        required: DomainVersion,
    },
}

impl fmt::Display for DomainRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "domain registry refused: {self:?}")
    }
}

impl std::error::Error for DomainRegistryError {}

/// Registry of immutable semantic artifacts. It never stores execution cursors.
#[derive(Debug, Clone, Default)]
pub struct DomainRegistry<D> {
    domains: BTreeMap<DomainSourceRoot, VersionedDomain<D>>,
}

impl<D> DomainRegistry<D> {
    pub fn insert(&mut self, domain: VersionedDomain<D>) -> Result<(), DomainRegistryError> {
        if self.domains.contains_key(&domain.source_root) {
            return Err(DomainRegistryError::DuplicateSourceRoot(domain.source_root));
        }
        self.domains.insert(domain.source_root, domain);
        Ok(())
    }

    pub fn get(&self, root: DomainSourceRoot) -> Result<&VersionedDomain<D>, DomainRegistryError> {
        self.domains
            .get(&root)
            .ok_or(DomainRegistryError::MissingDomain(root))
    }

    pub fn require_compatible(
        &self,
        root: DomainSourceRoot,
        required: DomainVersion,
    ) -> Result<&VersionedDomain<D>, DomainRegistryError> {
        let domain = self.get(root)?;
        if domain.version.is_backward_compatible_with(required) {
            Ok(domain)
        } else {
            Err(DomainRegistryError::IncompatibleVersion {
                available: domain.version,
                required,
            })
        }
    }

    pub fn len(&self) -> usize {
        self.domains.len()
    }

    pub fn is_empty(&self) -> bool {
        self.domains.is_empty()
    }
}

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

/// Standing-bearing value. Construction is intentionally narrow; transport
/// erases standing and transitions manufacture a new root.
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
        let catalog = action_catalog.into_iter().map(Into::into).collect::<BTreeSet<_>>();
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
        let typed_execution: ExecutionRoot = typed
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
        let root = DispatchRoot::hash_parts(&[
            envelope.plan_root().as_bytes(),
            binding_root.as_bytes(),
            policy_root.unwrap_or(PolicySetRoot::ZERO).as_bytes(),
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

    pub fn record_admission(
        &mut self,
        admission: &DispatchAdmission,
    ) -> Result<(), CursorError> {
        if admission.dispatch_root != self.dispatch_root {
            return Err(CursorError::DispatchRootMismatch);
        }
        let Some(tick) = self.next_tick else {
            return Ok(());
        };
        for command in self.commands.iter_mut().filter(|command| command.tick == tick) {
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
            let current = self
                .commands
                .iter()
                .filter(|command| command.tick == tick)
                .collect::<Vec<_>>();
            let complete = !current.is_empty()
                && current.iter().all(|command| {
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

/// Residual reconciliation input. This utility decides whether an existing
/// suffix remains eligible for verification; it does not pretend to manufacture
/// a replacement plan without invoking a semantic rail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResidualRequest {
    pub original_plan: PlanRoot,
    pub original_observation: ObservationRoot,
    pub current_observation: ObservationRoot,
    pub next_tick: Option<u32>,
    pub goal_already_satisfied: bool,
    pub generation: u32,
    pub max_generations: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplanDecision {
    KeepSuffix { from_tick: Option<u32> },
    ReplaceRequired {
        previous_observation: ObservationRoot,
        current_observation: ObservationRoot,
    },
    GoalAlreadySatisfied,
    Refuse { reason: ReplanRefusal },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplanRefusal {
    GenerationBoundExceeded { limit: u32 },
}

pub fn reconcile_residual(request: &ResidualRequest) -> ReplanDecision {
    if request.generation >= request.max_generations {
        return ReplanDecision::Refuse {
            reason: ReplanRefusal::GenerationBoundExceeded {
                limit: request.max_generations,
            },
        };
    }
    if request.goal_already_satisfied {
        return ReplanDecision::GoalAlreadySatisfied;
    }
    if request.original_observation == request.current_observation {
        ReplanDecision::KeepSuffix {
            from_tick: request.next_tick,
        }
    } else {
        ReplanDecision::ReplaceRequired {
            previous_observation: request.original_observation,
            current_observation: request.current_observation,
        }
    }
}

/// Compiler-style plan/process transformation.
pub trait PlanPass<I> {
    type Output;
    type Witness;
    type Refusal;

    fn apply(
        &self,
        input: I,
    ) -> Result<PassOutput<Self::Output, Self::Witness>, Self::Refusal>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassOutput<T, W> {
    pub value: T,
    pub witness: W,
    pub input_root: PassRoot,
    pub output_root: PassRoot,
}

#[derive(Debug, Clone)]
pub struct Then<A, B> {
    first: A,
    second: B,
}

pub trait PlanPassExt<I>: PlanPass<I> + Sized {
    fn then<B>(self, second: B) -> Then<Self, B> {
        Then {
            first: self,
            second,
        }
    }
}

impl<I, P> PlanPassExt<I> for P where P: PlanPass<I> {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PassChainRefusal<A, B> {
    First(A),
    Second(B),
    RootDiscontinuity {
        first_output: PassRoot,
        second_input: PassRoot,
    },
}

impl<I, A, B> PlanPass<I> for Then<A, B>
where
    A: PlanPass<I>,
    B: PlanPass<A::Output>,
{
    type Output = B::Output;
    type Witness = (A::Witness, B::Witness);
    type Refusal = PassChainRefusal<A::Refusal, B::Refusal>;

    fn apply(
        &self,
        input: I,
    ) -> Result<PassOutput<Self::Output, Self::Witness>, Self::Refusal> {
        let first = self
            .first
            .apply(input)
            .map_err(PassChainRefusal::First)?;
        let first_input = first.input_root;
        let first_output = first.output_root;
        let first_witness = first.witness;
        let second = self
            .second
            .apply(first.value)
            .map_err(PassChainRefusal::Second)?;
        if second.input_root != first_output {
            return Err(PassChainRefusal::RootDiscontinuity {
                first_output,
                second_input: second.input_root,
            });
        }
        Ok(PassOutput {
            value: second.value,
            witness: (first_witness, second.witness),
            input_root: first_input,
            output_root: second.output_root,
        })
    }
}

/// Language-independent semantic rail contract. Implementations must publish
/// their own capability/refusal types and may not hide non-capability failures
/// behind fallback success.
pub trait SemanticRail<Request> {
    type Candidate;
    type Standing;
    type Refusal;

    fn rail_root(&self) -> SearchPolicyRoot;
    fn admit_and_plan(
        &mut self,
        request: &Request,
    ) -> Result<Artifact<Self::Candidate, Self::Standing>, Self::Refusal>;
}

/// Stable refusal categories for application policy and recovery routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowRefusalCode {
    SourceParse,
    Canonicalization,
    UnsupportedCapability,
    InconsistentTheory,
    InvalidObservation,
    InvalidGoal,
    BoundExhaustion,
    SearchExhaustion,
    PlanValidation,
    CausalAnalysis,
    ConcurrencyWitness,
    ProcessProjection,
    ProcessValidation,
    SchedulerDeadlock,
    ReplayMismatch,
    ActionLabel,
    CommandBinding,
    Policy,
    StaleObservation,
    BrokerAdmission,
    IdempotencyConflict,
    Actuation,
    EffectObservation,
    CursorMismatch,
    Replan,
    ReceiptMismatch,
    TransportTrust,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roots_are_domain_separated_and_roundtrip() {
        let bytes = b"same material";
        let plan = PlanRoot::hash(bytes);
        let effect = EffectRoot::hash(bytes);
        assert_ne!(plan.as_bytes(), effect.as_bytes());
        assert_eq!(plan.to_string().parse::<PlanRoot>().unwrap(), plan);
    }

    #[test]
    fn pass_chain_refuses_root_discontinuity() {
        struct First;
        struct Second;

        impl PlanPass<&'static str> for First {
            type Output = String;
            type Witness = &'static str;
            type Refusal = ();

            fn apply(
                &self,
                input: &'static str,
            ) -> Result<PassOutput<Self::Output, Self::Witness>, Self::Refusal> {
                Ok(PassOutput {
                    value: input.to_uppercase(),
                    witness: "upper",
                    input_root: PassRoot::hash(input.as_bytes()),
                    output_root: PassRoot::hash(input.to_uppercase().as_bytes()),
                })
            }
        }

        impl PlanPass<String> for Second {
            type Output = usize;
            type Witness = &'static str;
            type Refusal = ();

            fn apply(
                &self,
                input: String,
            ) -> Result<PassOutput<Self::Output, Self::Witness>, Self::Refusal> {
                Ok(PassOutput {
                    value: input.len(),
                    witness: "length",
                    input_root: PassRoot::hash(b"wrong"),
                    output_root: PassRoot::hash(&input.len().to_le_bytes()),
                })
            }
        }

        let result = First.then(Second).apply("hello");
        assert!(matches!(
            result,
            Err(PassChainRefusal::RootDiscontinuity { .. })
        ));
    }
}
