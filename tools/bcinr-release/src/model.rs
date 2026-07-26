use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseProfile {
    pub schema_version: u16,
    pub release: String,
    pub repository: String,
    pub output_directory: String,
    pub rails: Vec<RailSpec>,
    #[serde(default)]
    pub artifacts: Vec<ArtifactSpec>,
    #[serde(default)]
    pub byte_identity: Vec<ByteIdentitySpec>,
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

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactSpec {
    pub id: String,
    pub path: String,
    #[serde(default = "required_by_default")]
    pub required: bool,
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
}

#[derive(Debug, Clone, Serialize)]
pub struct Provenance {
    pub head_sha: Option<String>,
    pub branch: Option<String>,
    pub remote_url: Option<String>,
    pub dirty: bool,
    pub status: Vec<String>,
    pub rustc: Option<String>,
    pub cargo: Option<String>,
    pub operating_system: String,
    pub architecture: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RailReceipt {
    pub id: String,
    pub phase: String,
    pub command: Vec<String>,
    pub working_directory: String,
    pub required: bool,
    pub started_unix_ms: u128,
    pub duration_ms: u128,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub spawn_error: Option<String>,
    pub stdout_log: String,
    pub stderr_log: String,
    pub stdout_blake3: Option<String>,
    pub stderr_blake3: Option<String>,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArtifactReceipt {
    pub id: String,
    pub path: String,
    pub required: bool,
    pub present: bool,
    pub size_bytes: Option<u64>,
    pub blake3: Option<String>,
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
    pub code: String,
    pub message: String,
    pub blocking: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReleaseReceipt {
    pub schema_version: u16,
    pub release: String,
    pub repository: String,
    pub profile_path: String,
    pub expected_head_sha: Option<String>,
    pub started_unix_ms: u128,
    pub finished_unix_ms: u128,
    pub duration_ms: u128,
    pub provenance_before: Provenance,
    pub provenance_after: Provenance,
    pub rails: Vec<RailReceipt>,
    pub artifacts: Vec<ArtifactReceipt>,
    pub byte_identity: Vec<ByteIdentityReceipt>,
    pub issues: Vec<AdmissionIssue>,
    pub standing: Standing,
}
