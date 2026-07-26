use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const PROFILE_SCHEMA_VERSION: u16 = 2;
pub const RECEIPT_SCHEMA_VERSION: u16 = 2;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseProfile {
    pub schema_version: u16,
    pub release: String,
    pub repository: String,
    pub output_directory: String,
    #[serde(default = "default_max_log_bytes")]
    pub max_log_bytes: u64,
    #[serde(default)]
    pub inherited_environment: Vec<String>,
    pub rails: Vec<RailSpec>,
    #[serde(default)]
    pub artifacts: Vec<ArtifactSpec>,
    #[serde(default)]
    pub byte_identity: Vec<ByteIdentitySpec>,
}

const fn default_max_log_bytes() -> u64 {
    16 * 1024 * 1024
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RailSpec {
    pub id: String,
    pub phase: String,
    pub program: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "default_working_directory")]
    pub working_directory: String,
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u64,
    #[serde(default = "required_by_default")]
    pub required: bool,
    #[serde(default)]
    pub environment: BTreeMap<String, String>,
}

fn default_working_directory() -> String {
    ".".to_owned()
}

const fn default_timeout_seconds() -> u64 {
    900
}

const fn required_by_default() -> bool {
    true
}

const fn default_minimum_files() -> usize {
    1
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactSpec {
    pub id: String,
    pub path: String,
    #[serde(default = "required_by_default")]
    pub required: bool,
    #[serde(default)]
    pub recursive: bool,
    #[serde(default)]
    pub suffix: Option<String>,
    #[serde(default = "default_minimum_files")]
    pub minimum_files: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ByteIdentitySpec {
    pub id: String,
    pub left: String,
    pub right: String,
    #[serde(default = "required_by_default")]
    pub required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Standing {
    Alive,
    PartialAlive,
    Blocked,
    BuildBroken,
    #[expect(
        dead_code,
        reason = "reserved receipt standing for incomplete observations"
    )]
    Unknown,
    #[expect(
        dead_code,
        reason = "reserved receipt standing for unsupported verifier boundaries"
    )]
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RailOutcome {
    Passed,
    ExitNonzero,
    TimedOut,
    SpawnRefused,
    EvidenceFailure,
    ExecutableChanged,
    RepositoryMutated,
    SkippedAfterRefusal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IssueCode {
    HeadMismatch,
    HeadUnknown,
    RepositoryMismatch,
    RemoteUnknown,
    TreeDirtyBeforeVerification,
    TreeDirtyAfterVerification,
    HeadChangedDuringVerification,
    ProvenanceIncomplete,
    ExecutableChanged,
    RepositoryMutated,
}

#[derive(Debug, Clone, Serialize)]
pub struct Provenance {
    pub head_sha: Option<String>,
    pub object_format: Option<String>,
    pub commit_unix_seconds: Option<u64>,
    pub branch: Option<String>,
    pub remote_url: Option<String>,
    pub normalized_repository: Option<String>,
    pub dirty: bool,
    pub status: Vec<String>,
    pub submodules: Vec<String>,
    pub rustc: Option<String>,
    pub cargo: Option<String>,
    pub operating_system: String,
    pub architecture: String,
    pub capture_errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogReceipt {
    pub path: String,
    pub bytes_observed: u64,
    pub bytes_retained: u64,
    pub truncated: bool,
    pub blake3: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RailReceipt {
    pub id: String,
    pub phase: String,
    pub command: Vec<String>,
    pub working_directory: String,
    pub required: bool,
    pub inherited_environment: Vec<String>,
    pub explicit_environment: Vec<String>,
    pub executable_path: Option<String>,
    pub executable_target: Option<String>,
    pub executable_blake3_before: Option<String>,
    pub executable_blake3_after: Option<String>,
    pub repository_state_before: Option<String>,
    pub repository_state_after: Option<String>,
    pub started_unix_ms: u128,
    pub duration_ms: u128,
    pub exit_code: Option<i32>,
    pub signal: Option<i32>,
    pub timed_out: bool,
    pub error: Option<String>,
    pub stdout: LogReceipt,
    pub stderr: LogReceipt,
    pub outcome: RailOutcome,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArtifactEntryReceipt {
    pub path: String,
    pub size_bytes: u64,
    pub blake3: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArtifactReceipt {
    pub id: String,
    pub path: String,
    pub required: bool,
    pub recursive: bool,
    pub minimum_files: usize,
    pub present: bool,
    pub entries: Vec<ArtifactEntryReceipt>,
    pub aggregate_blake3: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ByteIdentityReceipt {
    pub id: String,
    pub left: String,
    pub right: String,
    pub required: bool,
    pub identical: bool,
    pub left_size_bytes: Option<u64>,
    pub right_size_bytes: Option<u64>,
    pub left_blake3: Option<String>,
    pub right_blake3: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdmissionIssue {
    pub code: IssueCode,
    pub message: String,
    pub blocking: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReleaseReceipt {
    pub schema_version: u16,
    pub release: String,
    pub repository: String,
    pub profile_path: String,
    pub profile_blake3: String,
    pub verifier_path: String,
    pub verifier_blake3: String,
    pub expected_head_sha: String,
    pub output_directory: String,
    pub started_unix_ms: u128,
    pub finished_unix_ms: u128,
    pub duration_ms: u128,
    pub provenance_before: Provenance,
    pub provenance_after: Provenance,
    pub rails: Vec<RailReceipt>,
    pub artifacts: Vec<ArtifactReceipt>,
    pub byte_identity: Vec<ByteIdentityReceipt>,
    pub issues: Vec<AdmissionIssue>,
    pub evidence_root: String,
    pub standing: Standing,
}
