//! Append-only evidence ledger with a rolling BLAKE3 chain.
//!
//! Every admitted observation is chained: `root(n) = H(root(n-1) || entry(n))`.
//! `verify` recomputes the entire chain from the entries and compares, so a
//! mutated entry, a reordered entry, a deleted entry, or a hand-written root all
//! fail.
//!
//! This exists because `bcinr-powl`'s `OcelLog` cannot serve the purpose:
//! `seal_receipt(&self)` returns a snapshot while leaving the log mutable, and
//! no function anywhere recomputes a sealed digest — so post-seal append is
//! neither detected nor detectable there. A ledger whose seal cannot be checked
//! is a claim, not evidence.

use serde::{Deserialize, Serialize};

use super::{digest_bytes, Observation};

/// Ledger format version, carried in the digest so a format change cannot
/// silently validate against an old root.
pub const EVIDENCE_LEDGER_VERSION: u16 = 1;

/// The all-zero root that precedes the first entry.
pub const GENESIS_ROOT: &str = "0000000000000000000000000000000000000000000000000000000000000000";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub sequence: u64,
    pub observation: Observation,
    pub prior_root: String,
    pub root: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LedgerError {
    /// An entry's recomputed root does not match its recorded one.
    RootMismatch {
        sequence: u64,
        expected: String,
        found: String,
    },
    /// An entry's `prior_root` does not match the previous entry's `root`,
    /// which is what reordering or deletion looks like.
    ChainBroken {
        sequence: u64,
        expected_prior: String,
        found_prior: String,
    },
    /// Sequence numbers are not consecutive from zero.
    SequenceGap { expected: u64, found: u64 },
    /// The ledger's head root disagrees with its last entry.
    HeadMismatch { expected: String, found: String },
}

impl std::fmt::Display for LedgerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RootMismatch {
                sequence,
                expected,
                found,
            } => write!(
                f,
                "evidence ledger entry {sequence} root mismatch: expected {expected}, found {found}"
            ),
            Self::ChainBroken {
                sequence,
                expected_prior,
                found_prior,
            } => write!(
                f,
                "evidence ledger chain broken at {sequence}: prior_root {found_prior} should be {expected_prior}"
            ),
            Self::SequenceGap { expected, found } => {
                write!(f, "evidence ledger sequence gap: expected {expected}, found {found}")
            }
            Self::HeadMismatch { expected, found } => {
                write!(f, "evidence ledger head mismatch: expected {expected}, found {found}")
            }
        }
    }
}

impl std::error::Error for LedgerError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceLedger {
    pub version: u16,
    pub entries: Vec<LedgerEntry>,
    /// Root after the last entry; `GENESIS_ROOT` when empty.
    pub root: String,
}

impl Default for EvidenceLedger {
    fn default() -> Self {
        Self::new()
    }
}

impl EvidenceLedger {
    pub fn new() -> Self {
        Self {
            version: EVIDENCE_LEDGER_VERSION,
            entries: Vec::new(),
            root: GENESIS_ROOT.to_string(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Canonical bytes for one entry. Every field that matters to the verdict
    /// is included, so tampering with any of them breaks the root.
    fn entry_bytes(version: u16, sequence: u64, observation: &Observation) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&version.to_le_bytes());
        buf.extend_from_slice(&sequence.to_le_bytes());
        buf.extend_from_slice(observation.fact.render_atom().as_bytes());
        buf.push(0);
        buf.extend_from_slice(observation.command_digest.as_bytes());
        buf.push(0);
        // The outcome is part of the commitment: a Passed entry must not be
        // rewritable into a Failed one while keeping the same command.
        let outcome = serde_json::to_vec(&observation.outcome).unwrap_or_default();
        buf.extend_from_slice(&outcome);
        buf
    }

    fn fold(prior_root: &str, entry_bytes: &[u8]) -> String {
        let mut buf = Vec::with_capacity(prior_root.len() + 1 + entry_bytes.len());
        buf.extend_from_slice(prior_root.as_bytes());
        buf.push(0);
        buf.extend_from_slice(entry_bytes);
        digest_bytes(&buf)
    }

    /// Append an observation and advance the chain.
    pub fn append(&mut self, observation: Observation) -> &LedgerEntry {
        let sequence = self.entries.len() as u64;
        let prior_root = self.root.clone();
        let bytes = Self::entry_bytes(self.version, sequence, &observation);
        let root = Self::fold(&prior_root, &bytes);

        self.root = root.clone();
        self.entries.push(LedgerEntry {
            sequence,
            observation,
            prior_root,
            root,
        });
        self.entries.last().expect("just pushed")
    }

    /// Recompute the whole chain and compare against what is recorded.
    pub fn verify(&self) -> Result<(), LedgerError> {
        let mut prior = GENESIS_ROOT.to_string();

        for (index, entry) in self.entries.iter().enumerate() {
            let expected_sequence = index as u64;
            if entry.sequence != expected_sequence {
                return Err(LedgerError::SequenceGap {
                    expected: expected_sequence,
                    found: entry.sequence,
                });
            }
            if entry.prior_root != prior {
                return Err(LedgerError::ChainBroken {
                    sequence: entry.sequence,
                    expected_prior: prior,
                    found_prior: entry.prior_root.clone(),
                });
            }
            let bytes = Self::entry_bytes(self.version, entry.sequence, &entry.observation);
            let recomputed = Self::fold(&prior, &bytes);
            if recomputed != entry.root {
                return Err(LedgerError::RootMismatch {
                    sequence: entry.sequence,
                    expected: recomputed,
                    found: entry.root.clone(),
                });
            }
            prior = entry.root.clone();
        }

        if self.root != prior {
            return Err(LedgerError::HeadMismatch {
                expected: prior,
                found: self.root.clone(),
            });
        }
        Ok(())
    }

    /// Every admitted fact, as PDDL atoms for the controller problem's `:init`.
    pub fn admitted_atoms(&self) -> Vec<String> {
        self.entries
            .iter()
            .map(|e| e.observation.fact.render_atom())
            .collect()
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }
}
