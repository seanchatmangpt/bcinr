use crate::fs_safety::{create_new_evidence_file, resolve_existing_directory};
use crate::model::{LogReceipt, RailOutcome, RailReceipt, RailSpec};
use crate::provenance::repository_state_digest;
use std::env;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{self, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const POLL_INTERVAL: Duration = Duration::from_millis(25);

struct ResolvedExecutable {
    invocation_path: PathBuf,
    target_path: PathBuf,
    blake3: String,
}

struct LogCapture {
    bytes_observed: u64,
    bytes_retained: u64,
    truncated: bool,
    blake3: String,
    error: Option<String>,
}

pub fn unix_time_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

pub fn execute_rail(
    repository_root: &Path,
    logs_directory: &Path,
    spec: &RailSpec,
    inherited_environment: &[String],
    max_log_bytes: u64,
    source_date_epoch: Option<u64>,
) -> RailReceipt {
    let stdout_path = logs_directory.join(format!("{}.stdout.log", spec.id));
    let stderr_path = logs_directory.join(format!("{}.stderr.log", spec.id));
    let started_unix_ms = unix_time_ms();
    let started = Instant::now();
    let command_vector = std::iter::once(spec.program.clone())
        .chain(spec.args.iter().cloned())
        .collect::<Vec<_>>();

    let working_directory =
        match resolve_existing_directory(repository_root, Path::new(&spec.working_directory)) {
            Ok(path) => path,
            Err(error) => {
                return failed_receipt(
                    spec,
                    command_vector,
                    started_unix_ms,
                    started.elapsed().as_millis(),
                    &stdout_path,
                    &stderr_path,
                    RailOutcome::SpawnRefused,
                    format!("working directory refused: {error}"),
                );
            }
        };

    let resolved = match resolve_program(repository_root, &spec.program, inherited_environment) {
        Ok(value) => value,
        Err(error) => {
            return failed_receipt(
                spec,
                command_vector,
                started_unix_ms,
                started.elapsed().as_millis(),
                &stdout_path,
                &stderr_path,
                RailOutcome::SpawnRefused,
                error,
            );
        }
    };

    let repository_state_before = match repository_state_digest(repository_root) {
        Ok(value) => value,
        Err(error) => {
            return failed_receipt(
                spec,
                command_vector,
                started_unix_ms,
                started.elapsed().as_millis(),
                &stdout_path,
                &stderr_path,
                RailOutcome::EvidenceFailure,
                format!("failed to capture repository state before rail: {error}"),
            );
        }
    };

    let mut command = Command::new(&resolved.invocation_path);
    command
        .args(&spec.args)
        .current_dir(&working_directory)
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut inherited_names = Vec::new();
    for name in inherited_environment {
        if let Some(value) = env::var_os(name) {
            command.env(name, value);
            inherited_names.push(name.clone());
        }
    }
    for (name, value) in &spec.environment {
        command.env(name, value);
    }
    command
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .env("TZ", "UTC")
        .env("CARGO_INCREMENTAL", "0")
        .env("GIT_CONFIG_NOSYSTEM", "1");
    if let Some(epoch) = source_date_epoch {
        command.env("SOURCE_DATE_EPOCH", epoch.to_string());
    }

    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(&mut command, 0);
    #[cfg(target_os = "linux")]
    std::os::linux::process::CommandExt::create_pidfd(&mut command, true);

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            return failed_receipt(
                spec,
                command_vector,
                started_unix_ms,
                started.elapsed().as_millis(),
                &stdout_path,
                &stderr_path,
                RailOutcome::SpawnRefused,
                format!("failed to spawn command: {error}"),
            );
        }
    };

    let stdout = match child.stdout.take() {
        Some(value) => value,
        None => {
            let _ = terminate_process_tree(&mut child);
            let _ = child.wait();
            return failed_receipt(
                spec,
                command_vector,
                started_unix_ms,
                started.elapsed().as_millis(),
                &stdout_path,
                &stderr_path,
                RailOutcome::EvidenceFailure,
                "spawned command did not expose stdout".to_owned(),
            );
        }
    };
    let stderr = match child.stderr.take() {
        Some(value) => value,
        None => {
            let _ = terminate_process_tree(&mut child);
            let _ = child.wait();
            return failed_receipt(
                spec,
                command_vector,
                started_unix_ms,
                started.elapsed().as_millis(),
                &stdout_path,
                &stderr_path,
                RailOutcome::EvidenceFailure,
                "spawned command did not expose stderr".to_owned(),
            );
        }
    };

    let stdout_handle = spawn_log_pump(stdout, stdout_path.clone(), max_log_bytes);
    let stderr_handle = spawn_log_pump(stderr, stderr_path.clone(), max_log_bytes);
    let timeout = Duration::from_secs(spec.timeout_seconds);
    let mut timed_out = false;
    let mut execution_error = None;
    let status = wait_with_timeout(
        &mut child,
        started,
        timeout,
        &mut timed_out,
        &mut execution_error,
    );

    let stdout_receipt = join_log_pump(stdout_handle, &stdout_path);
    let stderr_receipt = join_log_pump(stderr_handle, &stderr_path);
    let executable_blake3_after =
        hash_file_with_context(&resolved.target_path, "bcinr.release.executable.v2").ok();
    let repository_state_after = repository_state_digest(repository_root).ok();
    let executable_changed = executable_blake3_after
        .as_deref()
        .is_none_or(|after| after != resolved.blake3);
    let repository_mutated = repository_state_after
        .as_deref()
        .is_none_or(|after| after != repository_state_before);
    let log_failed = stdout_receipt.error.is_some() || stderr_receipt.error.is_some();

    let outcome = if timed_out {
        RailOutcome::TimedOut
    } else if execution_error.is_some() || log_failed {
        RailOutcome::EvidenceFailure
    } else if executable_changed {
        RailOutcome::ExecutableChanged
    } else if repository_mutated {
        RailOutcome::RepositoryMutated
    } else if status.as_ref().is_some_and(ExitStatus::success) {
        RailOutcome::Passed
    } else {
        RailOutcome::ExitNonzero
    };

    RailReceipt {
        id: spec.id.clone(),
        phase: spec.phase.clone(),
        command: command_vector,
        working_directory: spec.working_directory.clone(),
        required: spec.required,
        inherited_environment: inherited_names,
        explicit_environment: spec.environment.keys().cloned().collect(),
        executable_path: Some(resolved.invocation_path.display().to_string()),
        executable_target: Some(resolved.target_path.display().to_string()),
        executable_blake3_before: Some(resolved.blake3),
        executable_blake3_after,
        repository_state_before: Some(repository_state_before),
        repository_state_after,
        started_unix_ms,
        duration_ms: started.elapsed().as_millis(),
        exit_code: status.as_ref().and_then(ExitStatus::code),
        signal: exit_signal(status.as_ref()),
        timed_out,
        error: execution_error,
        stdout: stdout_receipt,
        stderr: stderr_receipt,
        outcome,
        passed: outcome == RailOutcome::Passed,
    }
}

pub fn skipped_receipt(spec: &RailSpec, reason: &str) -> RailReceipt {
    let stdout_path = format!("skipped/{}.stdout.log", spec.id);
    let stderr_path = format!("skipped/{}.stderr.log", spec.id);
    RailReceipt {
        id: spec.id.clone(),
        phase: spec.phase.clone(),
        command: std::iter::once(spec.program.clone())
            .chain(spec.args.iter().cloned())
            .collect(),
        working_directory: spec.working_directory.clone(),
        required: spec.required,
        inherited_environment: Vec::new(),
        explicit_environment: spec.environment.keys().cloned().collect(),
        executable_path: None,
        executable_target: None,
        executable_blake3_before: None,
        executable_blake3_after: None,
        repository_state_before: None,
        repository_state_after: None,
        started_unix_ms: unix_time_ms(),
        duration_ms: 0,
        exit_code: None,
        signal: None,
        timed_out: false,
        error: Some(reason.to_owned()),
        stdout: empty_log_receipt(&stdout_path),
        stderr: empty_log_receipt(&stderr_path),
        outcome: RailOutcome::SkippedAfterRefusal,
        passed: false,
    }
}

fn wait_with_timeout(
    child: &mut Child,
    started: Instant,
    timeout: Duration,
    timed_out: &mut bool,
    execution_error: &mut Option<String>,
) -> Option<ExitStatus> {
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Some(status),
            Ok(None) if started.elapsed() < timeout => thread::sleep(POLL_INTERVAL),
            Ok(None) => {
                *timed_out = true;
                if let Err(error) = terminate_process_tree(child) {
                    *execution_error = Some(format!(
                        "timed out and failed to terminate process group: {error}"
                    ));
                }
                return match child.wait() {
                    Ok(status) => Some(status),
                    Err(error) => {
                        *execution_error = Some(format!("failed to wait after timeout: {error}"));
                        None
                    }
                };
            }
            Err(error) => {
                *execution_error = Some(format!("failed while waiting for command: {error}"));
                let _ = terminate_process_tree(child);
                let _ = child.wait();
                return None;
            }
        }
    }
}

#[cfg(unix)]
fn terminate_process_tree(child: &mut Child) -> io::Result<()> {
    std::os::unix::process::ChildExt::kill_process_group(child)
}

#[cfg(not(unix))]
fn terminate_process_tree(child: &mut Child) -> io::Result<()> {
    child.kill()
}

fn spawn_log_pump<R>(mut reader: R, path: PathBuf, max_log_bytes: u64) -> JoinHandle<LogCapture>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut writer = match create_new_evidence_file(&path) {
            Ok(file) => Some(file),
            Err(error) => {
                return drain_without_file(
                    &mut reader,
                    format!("failed to create {}: {error}", path.display()),
                );
            }
        };
        let mut hasher = blake3::Hasher::new_derive_key("bcinr.release.log.v2");
        let mut buffer = [0_u8; 64 * 1024];
        let mut bytes_observed = 0_u64;
        let mut bytes_retained = 0_u64;
        let mut error = None;

        loop {
            let read = match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(value) => value,
                Err(read_error) => {
                    error = Some(format!("failed to read process output: {read_error}"));
                    break;
                }
            };
            let chunk = &buffer[..read];
            hasher.update(chunk);
            bytes_observed = bytes_observed.saturating_add(read as u64);

            let remaining = max_log_bytes.saturating_sub(bytes_retained) as usize;
            let retain = remaining.min(read);
            if retain > 0 {
                if let Some(file) = writer.as_mut() {
                    if let Err(write_error) = file.write_all(&chunk[..retain]) {
                        error = Some(format!("failed to retain process output: {write_error}"));
                        writer = None;
                    } else {
                        bytes_retained = bytes_retained.saturating_add(retain as u64);
                    }
                }
            }
        }

        if let Some(file) = writer.as_mut() {
            if let Err(sync_error) = file.flush().and_then(|()| file.sync_all()) {
                error = Some(format!("failed to sync process output: {sync_error}"));
            }
        }

        LogCapture {
            bytes_observed,
            bytes_retained,
            truncated: bytes_observed > bytes_retained,
            blake3: hasher.finalize().to_hex().to_string(),
            error,
        }
    })
}

fn drain_without_file<R>(reader: &mut R, initial_error: String) -> LogCapture
where
    R: Read,
{
    let mut hasher = blake3::Hasher::new_derive_key("bcinr.release.log.v2");
    let mut buffer = [0_u8; 64 * 1024];
    let mut bytes_observed = 0_u64;
    let mut error = initial_error;
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => {
                hasher.update(&buffer[..read]);
                bytes_observed = bytes_observed.saturating_add(read as u64);
            }
            Err(read_error) => {
                error.push_str(&format!("; failed to drain process output: {read_error}"));
                break;
            }
        }
    }
    LogCapture {
        bytes_observed,
        bytes_retained: 0,
        truncated: bytes_observed > 0,
        blake3: hasher.finalize().to_hex().to_string(),
        error: Some(error),
    }
}

fn join_log_pump(handle: JoinHandle<LogCapture>, path: &Path) -> LogReceipt {
    match handle.join() {
        Ok(capture) => LogReceipt {
            path: path.display().to_string(),
            bytes_observed: capture.bytes_observed,
            bytes_retained: capture.bytes_retained,
            truncated: capture.truncated,
            blake3: Some(capture.blake3),
            error: capture.error,
        },
        Err(_) => LogReceipt {
            path: path.display().to_string(),
            bytes_observed: 0,
            bytes_retained: 0,
            truncated: false,
            blake3: None,
            error: Some("log-capture thread panicked".to_owned()),
        },
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "failure receipt construction preserves explicit bounded evidence fields"
)]
fn failed_receipt(
    spec: &RailSpec,
    command: Vec<String>,
    started_unix_ms: u128,
    duration_ms: u128,
    stdout_path: &Path,
    stderr_path: &Path,
    outcome: RailOutcome,
    error: String,
) -> RailReceipt {
    RailReceipt {
        id: spec.id.clone(),
        phase: spec.phase.clone(),
        command,
        working_directory: spec.working_directory.clone(),
        required: spec.required,
        inherited_environment: Vec::new(),
        explicit_environment: spec.environment.keys().cloned().collect(),
        executable_path: None,
        executable_target: None,
        executable_blake3_before: None,
        executable_blake3_after: None,
        repository_state_before: None,
        repository_state_after: None,
        started_unix_ms,
        duration_ms,
        exit_code: None,
        signal: None,
        timed_out: outcome == RailOutcome::TimedOut,
        error: Some(error),
        stdout: empty_log_receipt(&stdout_path.display().to_string()),
        stderr: empty_log_receipt(&stderr_path.display().to_string()),
        outcome,
        passed: false,
    }
}

fn empty_log_receipt(path: &str) -> LogReceipt {
    LogReceipt {
        path: path.to_owned(),
        bytes_observed: 0,
        bytes_retained: 0,
        truncated: false,
        blake3: None,
        error: None,
    }
}

fn resolve_program(
    repository_root: &Path,
    program: &str,
    inherited_environment: &[String],
) -> Result<ResolvedExecutable, String> {
    let invocation_path = if program.contains('/') || program.contains('\\') {
        repository_root.join(program)
    } else {
        if !inherited_environment.iter().any(|name| name == "PATH") {
            return Err("PATH is not admitted for executable resolution".to_owned());
        }
        search_path(program)?
    };
    let metadata = fs::symlink_metadata(&invocation_path).map_err(|error| {
        format!(
            "failed to inspect executable {}: {error}",
            invocation_path.display()
        )
    })?;
    if !(metadata.is_file() || metadata.file_type().is_symlink()) {
        return Err(format!(
            "executable {} is not a file or symlink",
            invocation_path.display()
        ));
    }
    let target_path = invocation_path
        .canonicalize()
        .map_err(|error| format!("failed to canonicalize executable: {error}"))?;
    let target_metadata = fs::metadata(&target_path)
        .map_err(|error| format!("failed to inspect executable target: {error}"))?;
    if !target_metadata.is_file() {
        return Err(format!(
            "executable target {} is not a regular file",
            target_path.display()
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if target_metadata.permissions().mode() & 0o111 == 0 {
            return Err(format!(
                "executable target {} has no execute bit",
                target_path.display()
            ));
        }
    }
    let blake3 = hash_file_with_context(&target_path, "bcinr.release.executable.v2")
        .map_err(|error| format!("failed to hash executable target: {error}"))?;
    Ok(ResolvedExecutable {
        invocation_path,
        target_path,
        blake3,
    })
}

fn search_path(program: &str) -> Result<PathBuf, String> {
    let path = env::var_os("PATH").ok_or_else(|| "PATH is unavailable".to_owned())?;
    let extensions = executable_extensions();
    for directory in env::split_paths(&path) {
        if !directory.is_absolute() {
            continue;
        }
        for extension in &extensions {
            let mut file_name = OsString::from(program);
            file_name.push(extension);
            let candidate = directory.join(file_name);
            if fs::symlink_metadata(&candidate).is_ok() {
                return Ok(candidate);
            }
        }
    }
    Err(format!(
        "executable {program:?} was not found in admitted PATH"
    ))
}

fn executable_extensions() -> Vec<OsString> {
    #[cfg(windows)]
    {
        let value = env::var_os("PATHEXT").unwrap_or_else(|| OsString::from(".EXE;.CMD;.BAT;.COM"));
        let mut extensions = value
            .to_string_lossy()
            .split(';')
            .filter(|part| !part.is_empty())
            .map(OsString::from)
            .collect::<Vec<_>>();
        extensions.insert(0, OsString::new());
        extensions
    }
    #[cfg(not(windows))]
    {
        vec![OsString::new()]
    }
}

#[cfg(unix)]
fn exit_signal(status: Option<&ExitStatus>) -> Option<i32> {
    use std::os::unix::process::ExitStatusExt;
    status.and_then(ExitStatusExt::signal)
}

#[cfg(not(unix))]
fn exit_signal(_status: Option<&ExitStatus>) -> Option<i32> {
    None
}

pub fn hash_file(path: &Path) -> io::Result<String> {
    hash_file_with_context(path, "bcinr.release.artifact.v2")
}

pub fn hash_file_with_context(path: &Path, context: &'static str) -> io::Result<String> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut hasher = blake3::Hasher::new_derive_key(context);
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

pub fn hash_bytes(bytes: &[u8], context: &'static str) -> String {
    let mut hasher = blake3::Hasher::new_derive_key(context);
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

pub fn compare_files(left: &Path, right: &Path) -> io::Result<bool> {
    if fs::metadata(left)?.len() != fs::metadata(right)?.len() {
        return Ok(false);
    }

    let mut left_reader = BufReader::new(File::open(left)?);
    let mut right_reader = BufReader::new(File::open(right)?);
    let mut left_buffer = [0_u8; 64 * 1024];
    let mut right_buffer = [0_u8; 64 * 1024];

    loop {
        let left_read = left_reader.read(&mut left_buffer)?;
        let right_read = right_reader.read(&mut right_buffer)?;
        if left_read != right_read {
            return Ok(false);
        }
        if left_read == 0 {
            return Ok(true);
        }
        if left_buffer[..left_read] != right_buffer[..right_read] {
            return Ok(false);
        }
    }
}
