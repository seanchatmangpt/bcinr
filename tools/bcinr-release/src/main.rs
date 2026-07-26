#![cfg_attr(target_os = "linux", feature(linux_pidfd))]
#![cfg_attr(unix, feature(unix_kill_process_group))]
#![forbid(unsafe_code)]

mod config;
mod executor;
mod fs_safety;
mod model;
mod provenance;

use crate::config::{load_profile, validate_output_directory, validate_repo_relative};
use crate::executor::{
    compare_files, execute_rail, hash_bytes, hash_file, hash_file_with_context, skipped_receipt,
    unix_time_ms,
};
use crate::fs_safety::{
    atomic_write, prepare_output_directory, resolve_existing_directory, resolve_existing_file,
    EvidenceLock,
};
use crate::model::{
    AdmissionIssue, ArtifactEntryReceipt, ArtifactReceipt, ByteIdentityReceipt, IssueCode,
    RailOutcome, RailReceipt, ReleaseReceipt, Standing, RECEIPT_SCHEMA_VERSION,
};
use serde::Serialize;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

const MAX_ARTIFACT_TREE_ENTRIES: usize = 4096;

struct Cli {
    profile: PathBuf,
    repository_root: PathBuf,
    expected_head: String,
    output_override: Option<String>,
}

#[derive(Serialize)]
struct EvidenceMaterial<'a> {
    profile_blake3: &'a str,
    verifier_blake3: &'a str,
    expected_head_sha: &'a str,
    provenance_before: &'a crate::model::Provenance,
    provenance_after: &'a crate::model::Provenance,
    rails: &'a [RailReceipt],
    artifacts: &'a [ArtifactReceipt],
    byte_identity: &'a [ByteIdentityReceipt],
    issues: &'a [AdmissionIssue],
    standing: Standing,
}

fn main() -> ExitCode {
    match run() {
        Ok(standing) => ExitCode::from(match standing {
            Standing::Alive => 0,
            Standing::BuildBroken => 10,
            Standing::Blocked => 20,
            Standing::PartialAlive => 30,
            Standing::Unknown => 40,
            Standing::Unsupported => 50,
        }),
        Err(error) => {
            eprintln!("bcinr-release refused: {error}");
            ExitCode::from(70)
        }
    }
}

fn run() -> Result<Standing, String> {
    let cli = parse_cli()?;
    let repository_root = cli
        .repository_root
        .canonicalize()
        .map_err(|error| format!("failed to resolve repository root: {error}"))?;
    if !repository_root.join(".git").exists() {
        return Err(format!(
            "repository root {} does not contain .git",
            repository_root.display()
        ));
    }

    let profile_text = cli
        .profile
        .to_str()
        .ok_or_else(|| "profile path must be UTF-8".to_owned())?;
    validate_repo_relative(profile_text, "profile path")?;
    let profile_path = resolve_existing_file(&repository_root, &cli.profile)
        .map_err(|error| format!("profile path refused: {error}"))?;
    let loaded = load_profile(&profile_path)?;
    let profile = loaded.profile;

    let output_directory = cli
        .output_override
        .as_deref()
        .unwrap_or(&profile.output_directory);
    validate_output_directory(output_directory)?;
    let output_relative = Path::new(output_directory);
    let output_path = prepare_output_directory(&repository_root, output_relative)
        .map_err(|error| format!("failed to prepare evidence directory: {error}"))?;

    let started_unix_ms = unix_time_ms();
    let started = Instant::now();
    let _lock = EvidenceLock::acquire(&output_path, &cli.expected_head, started_unix_ms)
        .map_err(|error| format!("failed to acquire evidence lock: {error}"))?;
    let logs_path = output_path.join("logs");
    fs::create_dir(&logs_path)
        .map_err(|error| format!("failed to create evidence log directory: {error}"))?;
    set_private_directory_permissions(&logs_path)
        .map_err(|error| format!("failed to secure evidence log directory: {error}"))?;

    let verifier_path = env::current_exe()
        .and_then(|path| path.canonicalize())
        .map_err(|error| format!("failed to resolve verifier executable: {error}"))?;
    let verifier_blake3 = hash_file_with_context(&verifier_path, "bcinr.release.verifier.v2")
        .map_err(|error| format!("failed to hash verifier executable: {error}"))?;

    let provenance_before = provenance::capture(&repository_root);
    let mut issues = preflight_issues(&profile.repository, &cli.expected_head, &provenance_before);
    let preflight_blocked = issues.iter().any(|issue| issue.blocking);

    let mut rails = Vec::with_capacity(profile.rails.len());
    if preflight_blocked {
        rails.extend(
            profile
                .rails
                .iter()
                .map(|rail| skipped_receipt(rail, "preflight admission refused")),
        );
    } else {
        let source_date_epoch = provenance_before.commit_unix_seconds;
        let mut stop_reason = None;
        for rail in &profile.rails {
            if let Some(reason) = &stop_reason {
                rails.push(skipped_receipt(rail, reason));
                continue;
            }
            let receipt = execute_rail(
                &repository_root,
                &logs_path,
                rail,
                &profile.inherited_environment,
                profile.max_log_bytes,
                source_date_epoch,
            );
            match receipt.outcome {
                RailOutcome::ExecutableChanged => {
                    issues.push(AdmissionIssue {
                        code: IssueCode::ExecutableChanged,
                        message: format!("rail {} executable changed during execution", rail.id),
                        blocking: true,
                    });
                    stop_reason =
                        Some("execution authority changed during verification".to_owned());
                }
                RailOutcome::RepositoryMutated => {
                    issues.push(AdmissionIssue {
                        code: IssueCode::RepositoryMutated,
                        message: format!("rail {} mutated repository state", rail.id),
                        blocking: true,
                    });
                    stop_reason = Some("repository mutation stopped subsequent rails".to_owned());
                }
                _ => {}
            }
            rails.push(receipt);
        }
    }

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
    append_postflight_issues(&provenance_before, &provenance_after, &mut issues);

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
    let blocked = issues.iter().any(|issue| issue.blocking);

    let standing = if blocked {
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

    let evidence_material = EvidenceMaterial {
        profile_blake3: &loaded.blake3,
        verifier_blake3: &verifier_blake3,
        expected_head_sha: &cli.expected_head,
        provenance_before: &provenance_before,
        provenance_after: &provenance_after,
        rails: &rails,
        artifacts: &artifacts,
        byte_identity: &byte_identity,
        issues: &issues,
        standing,
    };
    let evidence_bytes = serde_json::to_vec(&evidence_material)
        .map_err(|error| format!("failed to serialize evidence material: {error}"))?;
    let evidence_root = hash_bytes(&evidence_bytes, "bcinr.release.evidence-root.v2");

    let receipt = ReleaseReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION,
        release: profile.release,
        repository: profile.repository,
        profile_path: cli.profile.display().to_string(),
        profile_blake3: loaded.blake3,
        verifier_path: verifier_path.display().to_string(),
        verifier_blake3,
        expected_head_sha: cli.expected_head,
        output_directory: output_directory.to_owned(),
        started_unix_ms,
        finished_unix_ms: unix_time_ms(),
        duration_ms: started.elapsed().as_millis(),
        provenance_before,
        provenance_after,
        rails,
        artifacts,
        byte_identity,
        issues,
        evidence_root,
        standing,
    };
    write_receipt(&output_path, &receipt)?;

    println!("release={}", receipt.release);
    println!("standing={:?}", receipt.standing);
    println!("receipt={}", output_path.join("receipt.json").display());
    println!("evidence_root={}", receipt.evidence_root);

    Ok(standing)
}

fn parse_cli() -> Result<Cli, String> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("verify") => {}
        _ => return Err(usage()),
    }

    let mut profile = None;
    let mut repository_root = None;
    let mut expected_head = None;
    let mut output_override = None;
    let mut seen = BTreeSet::new();

    while let Some(argument) = args.next() {
        if !seen.insert(argument.clone()) {
            return Err(format!("duplicate argument {argument}"));
        }
        let value = args
            .next()
            .ok_or_else(|| format!("missing value for {argument}"))?;
        match argument.as_str() {
            "--profile" => profile = Some(PathBuf::from(value)),
            "--repository-root" => repository_root = Some(PathBuf::from(value)),
            "--expected-head" => expected_head = Some(validate_head_sha(&value)?),
            "--output" => output_override = Some(value),
            _ => return Err(format!("unknown argument {argument}")),
        }
    }

    Ok(Cli {
        profile: profile.ok_or_else(|| "--profile is required".to_owned())?,
        repository_root: repository_root.unwrap_or_else(|| PathBuf::from(".")),
        expected_head: expected_head.ok_or_else(|| "--expected-head is required".to_owned())?,
        output_override,
    })
}

fn validate_head_sha(value: &str) -> Result<String, String> {
    let normalized = value.trim().to_ascii_lowercase();
    if !matches!(normalized.len(), 40 | 64)
        || !normalized.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err("--expected-head must be a full 40- or 64-digit hexadecimal SHA".to_owned());
    }
    Ok(normalized)
}

fn usage() -> String {
    "usage: bcinr-release verify --profile <repository-relative-path> --expected-head <full-sha> [--repository-root <path>] [--output <target/path>]".to_owned()
}

fn preflight_issues(
    expected_repository: &str,
    expected_head: &str,
    provenance: &crate::model::Provenance,
) -> Vec<AdmissionIssue> {
    let mut issues = Vec::new();
    if !provenance.capture_errors.is_empty() {
        issues.push(AdmissionIssue {
            code: IssueCode::ProvenanceIncomplete,
            message: format!("provenance capture failed: {:?}", provenance.capture_errors),
            blocking: true,
        });
    }

    match &provenance.head_sha {
        Some(actual) if actual != expected_head => issues.push(AdmissionIssue {
            code: IssueCode::HeadMismatch,
            message: format!("expected HEAD {expected_head}, observed {actual}"),
            blocking: true,
        }),
        None => issues.push(AdmissionIssue {
            code: IssueCode::HeadUnknown,
            message: "unable to resolve exact git HEAD".to_owned(),
            blocking: true,
        }),
        _ => {}
    }

    match &provenance.normalized_repository {
        Some(actual) if actual == &expected_repository.to_ascii_lowercase() => {}
        Some(actual) => issues.push(AdmissionIssue {
            code: IssueCode::RepositoryMismatch,
            message: format!("expected repository {expected_repository}, observed {actual}"),
            blocking: true,
        }),
        None => issues.push(AdmissionIssue {
            code: IssueCode::RemoteUnknown,
            message: "unable to resolve exact origin repository".to_owned(),
            blocking: true,
        }),
    }

    if provenance.dirty {
        issues.push(AdmissionIssue {
            code: IssueCode::TreeDirtyBeforeVerification,
            message: format!(
                "repository is dirty before verification: {:?}",
                provenance.status
            ),
            blocking: true,
        });
    }
    if provenance
        .submodules
        .iter()
        .any(|line| !line.starts_with(' '))
    {
        issues.push(AdmissionIssue {
            code: IssueCode::ProvenanceIncomplete,
            message: format!("submodule state is not exact: {:?}", provenance.submodules),
            blocking: true,
        });
    }
    if provenance.rustc.is_none()
        || provenance.cargo.is_none()
        || provenance.object_format.is_none()
        || provenance.commit_unix_seconds.is_none()
    {
        issues.push(AdmissionIssue {
            code: IssueCode::ProvenanceIncomplete,
            message: "toolchain or git provenance is incomplete".to_owned(),
            blocking: true,
        });
    }
    issues
}

fn append_postflight_issues(
    before: &crate::model::Provenance,
    after: &crate::model::Provenance,
    issues: &mut Vec<AdmissionIssue>,
) {
    if after.dirty {
        issues.push(AdmissionIssue {
            code: IssueCode::TreeDirtyAfterVerification,
            message: format!(
                "verification changed tracked or unignored repository state: {:?}",
                after.status
            ),
            blocking: true,
        });
    }
    if before.head_sha != after.head_sha {
        issues.push(AdmissionIssue {
            code: IssueCode::HeadChangedDuringVerification,
            message: format!(
                "HEAD changed from {:?} to {:?}",
                before.head_sha, after.head_sha
            ),
            blocking: true,
        });
    }
    if !after.capture_errors.is_empty() {
        issues.push(AdmissionIssue {
            code: IssueCode::ProvenanceIncomplete,
            message: format!("postflight provenance failed: {:?}", after.capture_errors),
            blocking: true,
        });
    }
}

fn inspect_artifact(repository_root: &Path, spec: &crate::model::ArtifactSpec) -> ArtifactReceipt {
    let result = if spec.recursive {
        inspect_artifact_tree(repository_root, spec)
    } else {
        inspect_artifact_file(repository_root, spec)
    };
    match result {
        Ok(entries) => {
            let present = entries.len() >= spec.minimum_files;
            let aggregate_blake3 = (!entries.is_empty()).then(|| aggregate_artifacts(&entries));
            ArtifactReceipt {
                id: spec.id.clone(),
                path: spec.path.clone(),
                required: spec.required,
                recursive: spec.recursive,
                minimum_files: spec.minimum_files,
                present,
                entries,
                aggregate_blake3,
                error: (!present)
                    .then(|| format!("expected at least {} admitted files", spec.minimum_files)),
            }
        }
        Err(error) => ArtifactReceipt {
            id: spec.id.clone(),
            path: spec.path.clone(),
            required: spec.required,
            recursive: spec.recursive,
            minimum_files: spec.minimum_files,
            present: false,
            entries: Vec::new(),
            aggregate_blake3: None,
            error: Some(error),
        },
    }
}

fn inspect_artifact_file(
    repository_root: &Path,
    spec: &crate::model::ArtifactSpec,
) -> Result<Vec<ArtifactEntryReceipt>, String> {
    let path = resolve_existing_file(repository_root, Path::new(&spec.path))
        .map_err(|error| error.to_string())?;
    let metadata = fs::metadata(&path).map_err(|error| error.to_string())?;
    let digest = hash_file(&path).map_err(|error| error.to_string())?;
    Ok(vec![ArtifactEntryReceipt {
        path: spec.path.clone(),
        size_bytes: metadata.len(),
        blake3: digest,
    }])
}

fn inspect_artifact_tree(
    repository_root: &Path,
    spec: &crate::model::ArtifactSpec,
) -> Result<Vec<ArtifactEntryReceipt>, String> {
    let root = resolve_existing_directory(repository_root, Path::new(&spec.path))
        .map_err(|error| error.to_string())?;
    let mut pending = vec![root];
    let mut entries = Vec::new();
    let mut visited = 0_usize;

    while let Some(directory) = pending.pop() {
        let mut children = fs::read_dir(&directory)
            .map_err(|error| format!("failed to read {}: {error}", directory.display()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;
        children.sort_by_key(fs::DirEntry::path);

        for child in children {
            visited = visited.saturating_add(1);
            if visited > MAX_ARTIFACT_TREE_ENTRIES {
                return Err(format!(
                    "artifact tree exceeds {MAX_ARTIFACT_TREE_ENTRIES} entries"
                ));
            }
            let path = child.path();
            let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
            if metadata.file_type().is_symlink() {
                return Err(format!("artifact tree contains symlink {}", path.display()));
            }
            if metadata.is_dir() {
                pending.push(path);
            } else if metadata.is_file()
                && spec
                    .suffix
                    .as_deref()
                    .is_none_or(|suffix| path.to_string_lossy().ends_with(suffix))
            {
                let relative = path
                    .strip_prefix(repository_root)
                    .map_err(|error| error.to_string())?
                    .to_string_lossy()
                    .to_string();
                entries.push(ArtifactEntryReceipt {
                    path: relative,
                    size_bytes: metadata.len(),
                    blake3: hash_file(&path).map_err(|error| error.to_string())?,
                });
            }
        }
    }
    entries.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(entries)
}

fn aggregate_artifacts(entries: &[ArtifactEntryReceipt]) -> String {
    let mut hasher = blake3::Hasher::new_derive_key("bcinr.release.artifact-tree.v2");
    for entry in entries {
        update_framed(&mut hasher, entry.path.as_bytes());
        hasher.update(&entry.size_bytes.to_le_bytes());
        update_framed(&mut hasher, entry.blake3.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

fn inspect_identity(
    repository_root: &Path,
    spec: &crate::model::ByteIdentitySpec,
) -> ByteIdentityReceipt {
    let left = resolve_existing_file(repository_root, Path::new(&spec.left));
    let right = resolve_existing_file(repository_root, Path::new(&spec.right));
    match (left, right) {
        (Ok(left), Ok(right)) => {
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
                Err(error) => identity_error(spec, Some(left), Some(right), error.to_string()),
            }
        }
        (left, right) => identity_error(
            spec,
            left.ok(),
            right.ok(),
            "one or both identity paths were refused".to_owned(),
        ),
    }
}

fn identity_error(
    spec: &crate::model::ByteIdentitySpec,
    left: Option<PathBuf>,
    right: Option<PathBuf>,
    error: String,
) -> ByteIdentityReceipt {
    ByteIdentityReceipt {
        id: spec.id.clone(),
        left: spec.left.clone(),
        right: spec.right.clone(),
        required: spec.required,
        identical: false,
        left_size_bytes: left
            .as_ref()
            .and_then(|path| fs::metadata(path).ok())
            .map(|value| value.len()),
        right_size_bytes: right
            .as_ref()
            .and_then(|path| fs::metadata(path).ok())
            .map(|value| value.len()),
        left_blake3: left.as_ref().and_then(|path| hash_file(path).ok()),
        right_blake3: right.as_ref().and_then(|path| hash_file(path).ok()),
        error: Some(error),
    }
}

fn write_receipt(output_path: &Path, receipt: &ReleaseReceipt) -> Result<(), String> {
    let mut receipt_bytes = serde_json::to_vec_pretty(receipt)
        .map_err(|error| format!("failed to serialize release receipt: {error}"))?;
    receipt_bytes.push(b'\n');
    let receipt_digest = hash_bytes(&receipt_bytes, "bcinr.release.receipt.v2");

    atomic_write(&output_path.join("receipt.json"), &receipt_bytes)
        .map_err(|error| format!("failed to atomically write receipt: {error}"))?;
    atomic_write(
        &output_path.join("receipt.blake3"),
        format!("{receipt_digest}  receipt.json\n").as_bytes(),
    )
    .map_err(|error| format!("failed to write receipt digest: {error}"))?;
    atomic_write(
        &output_path.join("evidence.root"),
        format!("{}\n", receipt.evidence_root).as_bytes(),
    )
    .map_err(|error| format!("failed to write evidence root: {error}"))?;
    atomic_write(
        &output_path.join("standing.txt"),
        format!("{:?}\n", receipt.standing).as_bytes(),
    )
    .map_err(|error| format!("failed to write standing: {error}"))?;
    Ok(())
}

fn update_framed(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

fn set_private_directory_permissions(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_head_sha;

    #[test]
    fn admits_only_full_git_hashes() {
        assert!(validate_head_sha("0123456789012345678901234567890123456789").is_ok());
        assert!(validate_head_sha("deadbeef").is_err());
        assert!(validate_head_sha("z123456789012345678901234567890123456789").is_err());
    }
}
