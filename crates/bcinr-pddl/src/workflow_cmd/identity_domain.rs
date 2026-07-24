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
                write!(
                    f,
                    "semantic root has length {actual}; expected 64 hex characters"
                )
            }
            Self::InvalidHex { index } => {
                write!(
                    f,
                    "semantic root contains invalid hexadecimal data at byte {index}"
                )
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
        use std::collections::btree_map::Entry;

        match self.domains.entry(domain.source_root) {
            Entry::Vacant(entry) => {
                entry.insert(domain);
                Ok(())
            }
            Entry::Occupied(entry) => Err(DomainRegistryError::DuplicateSourceRoot(*entry.key())),
        }
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
