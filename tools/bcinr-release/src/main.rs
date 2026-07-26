mod config;
mod executor;
mod model;
mod provenance;

use crate::config::{load_profile, validate_output_directory};
use crate::executor::{compare_files, execute_rail, hash_file, unix_time_ms};
use crate::model::{
    AdmissionIssue, ArtifactReceipt, ByteIdentityReceipt, ReleaseReceipt, Standing,
};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

struct Cli {
    profile: PathBuf,
    repository_root: PathBuf,
    expected_head: Option<String>,
    output_override: Option<String>,
}

fn main() -> ExitCode {
    match run() {
        Ok(standing) => ExitCode::from(match standing {
            Standing::Alive => 0,
            Standing::BuildBroken => 1,
            Standing::Blocked => 2,
            Standing::PartialAlive => 3,
        }),
        Err(error) => {
            eprintln!("bcinr-release refused: {error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<Standing, String> {
    let cli = parse_cli()?;
    let repository_root = cli
        .repository_root
        .canonicalize()
        .map_err(|error| format!("failed to resolve repository root: {error}"))?;
    let profile_path = resolve_under_root(&repository_root, &cli.profile);
    let profile = load_profile(&profile_path)?;

    let output_directory = cli
        .output_override
        .as_deref()
        .unwrap_or(&profile.output_directory);
    validate_output_directory(output_directory)?;
    let output_path = repository_root.join(output_directory);
    let logs_path = output_path.join("logs");

    let started_unix_ms = unix_time_ms();
    let started = Instant::now();
    let provenance_before = provenance::capture(&repository_root);
    let mut issues = preflight_issues(
        &profile.repository,
        cli.expected_head.as_deref(),
        &provenance_before,
    );
    let preflight_blocked = issues.iter().any(|issue| issue.blocking);

    fs::create_dir_all(&logs_path)
        .map_err(|error| format!("failed to create evidence directory: {error}"))?;

    let rails = profile
        .rails
        .iter()
        .map(|rail| execute_rail(&repository_root, &logs_path, rail))
        .collect::<Vec<_>>();
    let artifacts = profile
        .artifacts
        .iter()
        .map(|artifact| inspect_artifact(&repository_root, artifact))
        .collect::<Vec<_>>();
    let byte_identity = profile
        .byte_identity
        .iter()
        .map(|identity| inspect_identity(&repository_root, identity))
        .collect::<Vec<_>>();

    let provenance_after = provenance::capture(&repository_root);
    if provenance_after.dirty {
        issues.push(AdmissionIssue {
            code: "TREE_DIRTY_AFTER_VERIFICATION".to_owned(),
            message: format!(
                "verification changed tracked or unignored repository state: {:?}",
                provenance_after.status
            ),
            blocking: true,
        });
    }
    if provenance_before.head_sha != provenance_after.head_sha {
        issues.push(AdmissionIssue {
            code: "HEAD_CHANGED_DURING_VERIFICATION".to_owned(),
            message: format!(
                "HEAD changed from {:?} to {:?}",
                provenance_before.head_sha, provenance_after.head_sha
            ),
            blocking: true,
        });
    }

    let required_rail_failure = rails.iter().any(|rail| rail.required && !rail.passed);
    let optional_rail_failure = rails.iter().any(|rail| !rail.required && !rail.passed);
    let required_artifact_failure = artifacts
        .iter()
        .any(|artifact| artifact.required && !artifact.present);
    let optional_artifact_failure = artifacts
        .iter()
        .any(|artifact| !artifact.required && !artifact.present);
    let required_identity_failure = byte_identity
        .iter()
        .any(|identity| identity.required && !identity.identical);
    let optional_identity_failure = byte_identity
        .iter()
        .any(|identity| !identity.required && !identity.identical);
    let postflight_blocked = issues.iter().any(|issue| issue.blocking);

    let standing = if preflight_blocked || postflight_blocked {
        Standing::Blocked
    } else if required_rail_failure {
        Standing::BuildBroken
    } else if required_artifact_failure || required_identity_failure {
        Standing::Blocked
    } else if optional_rail_failure || optional_artifact_failure || optional_identity_failure {
        Standing::PartialAlive
    } else {
        Standing::Alive
    };

    let receipt = ReleaseReceipt {
        schema_version: 1,
        release: profile.release,
        repository: profile.repository,
        profile_path: profile_path.display().to_string(),
        expected_head_sha: cli.expected_head,
        started_unix_ms,
        finished_unix_ms: unix_time_ms(),
        duration_ms: started.elapsed().as_millis(),
        provenance_before,
        provenance_after,
        rails,
        artifacts,
        byte_identity,
        issues,
        standing,
    };

    let receipt_path = output_path.join("receipt.json");
    let receipt_bytes = serde_json::to_vec_pretty(&receipt)
        .map_err(|error| format!("failed to serialize release receipt: {error}"))?;
    fs::write(&receipt_path, &receipt_bytes)
        .map_err(|error| format!("failed to write {}: {error}", receipt_path.display()))?;
    let digest = hash_file(&receipt_path)
        .map_err(|error| format!("failed to hash {}: {error}", receipt_path.display()))?;
    let digest_path = output_path.join("receipt.blake3");
    fs::write(&digest_path, format!("{digest}  receipt.json\n"))
        .map_err(|error| format!("failed to write {}: {error}", digest_path.display()))?;

    println!("release={}", receipt.release);
    println!("standing={:?}", receipt.standing);
    println!("receipt={}", receipt_path.display());
    println!("receipt_blake3={digest}");

    Ok(standing)
}

fn parse_cli() -> Result<Cli, String> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("verify") => {}
        _ => {
            return Err(
                "usage: bcinr-release verify --profile <path> [--repository-root <path>] [--expected-head <sha>] [--output <target/path>]"
                    .to_owned(),
            )
        }
    }

    let mut profile = None;
    let mut repository_root = PathBuf::from(".");
    let mut expected_head = None;
    let mut output_override = None;

    while let Some(argument) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| format!("missing value for {argument}"))?;
        match argument.as_str() {
            "--profile" => profile = Some(PathBuf::from(value)),
            "--repository-root" => repository_root = PathBuf::from(value),
            "--expected-head" => expected_head = Some(value),
            "--output" => output_override = Some(value),
            _ => return Err(format!("unknown argument {argument}")),
        }
    }

    Ok(Cli {
        profile: profile.ok_or_else(|| "--profile is required".to_owned())?,
        repository_root,
        expected_head,
        output_override,
    })
}

fn resolve_under_root(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn preflight_issues(
    expected_repository: &str,
    expected_head: Option<&str>,
    provenance: &crate::model::Provenance,
) -> Vec<AdmissionIssue> {
    let mut issues = Vec::new();
    match &provenance.head_sha {
        Some(actual) if expected_head.is_some_and(|expected| expected != actual) => {
            issues.push(AdmissionIssue {
                code: "HEAD_MISMATCH".to_owned(),
                message: format!("expected HEAD {:?}, observed {actual}", expected_head),
                blocking: true,
            });
        }
        None => issues.push(AdmissionIssue {
            code: "HEAD_UNKNOWN".to_owned(),
            message: "unable to resolve exact git HEAD".to_owned(),
            blocking: true,
        }),
        _ => {}
    }

    match &provenance.remote_url {
        Some(remote) if remote.contains(expected_repository) => {}
        Some(remote) => issues.push(AdmissionIssue {
            code: "REPOSITORY_MISMATCH".to_owned(),
            message: format!("expected repository {expected_repository}, observed remote {remote}"),
            blocking: true,
        }),
        None => issues.push(AdmissionIssue {
            code: "REMOTE_UNKNOWN".to_owned(),
            message: "unable to resolve origin remote".to_owned(),
            blocking: true,
        }),
    }

    if provenance.dirty {
        issues.push(AdmissionIssue {
            code: "TREE_DIRTY_BEFORE_VERIFICATION".to_owned(),
            message: format!("repository is dirty before verification: {:?}", provenance.status),
            blocking: true,
        });
    }
    issues
}

fn inspect_artifact(
    repository_root: &Path,
    spec: &crate::model::ArtifactSpec,
) -> ArtifactReceipt {
    let path = repository_root.join(&spec.path);
    match fs::metadata(&path) {
        Ok(metadata) if metadata.is_file() => match hash_file(&path) {
            Ok(digest) => ArtifactReceipt {
                id: spec.id.clone(),
                path: spec.path.clone(),
                required: spec.required,
                present: true,
                size_bytes: Some(metadata.len()),
                blake3: Some(digest),
                error: None,
            },
            Err(error) => ArtifactReceipt {
                id: spec.id.clone(),
                path: spec.path.clone(),
                required: spec.required,
                present: false,
                size_bytes: Some(metadata.len()),
                blake3: None,
                error: Some(error.to_string()),
            },
        },
        Ok(_) => ArtifactReceipt {
            id: spec.id.clone(),
            path: spec.path.clone(),
            required: spec.required,
            present: false,
            size_bytes: None,
            blake3: None,
            error: Some("artifact is not a regular file".to_owned()),
        },
        Err(error) => ArtifactReceipt {
            id: spec.id.clone(),
            path: spec.path.clone(),
            required: spec.required,
            present: false,
            size_bytes: None,
            blake3: None,
            error: Some(error.to_string()),
        },
    }
}

fn inspect_identity(
    repository_root: &Path,
    spec: &crate::model::ByteIdentitySpec,
) -> ByteIdentityReceipt {
    let left = repository_root.join(&spec.left);
    let right = repository_root.join(&spec.right);
    let left_metadata = fs::metadata(&left).ok();
    let right_metadata = fs::metadata(&right).ok();
    let left_hash = hash_file(&left).ok();
    let right_hash = hash_file(&right).ok();

    match compare_files(&left, &right) {
        Ok(identical) => ByteIdentityReceipt {
            id: spec.id.clone(),
            left: spec.left.clone(),
            right: spec.right.clone(),
            required: spec.required,
            identical,
            left_size_bytes: left_metadata.as_ref().map(fs::Metadata::len),
            right_size_bytes: right_metadata.as_ref().map(fs::Metadata::len),
            left_blake3: left_hash,
            right_blake3: right_hash,
            error: None,
        },
        Err(error) => ByteIdentityReceipt {
            id: spec.id.clone(),
            left: spec.left.clone(),
            right: spec.right.clone(),
            required: spec.required,
            identical: false,
            left_size_bytes: left_metadata.as_ref().map(fs::Metadata::len),
            right_size_bytes: right_metadata.as_ref().map(fs::Metadata::len),
            left_blake3: left_hash,
            right_blake3: right_hash,
            error: Some(error.to_string()),
        },
    }
}
