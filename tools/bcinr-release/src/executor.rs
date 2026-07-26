use crate::model::{RailReceipt, RailSpec};
use std::fs::{self, File};
use std::io::{self, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub fn unix_time_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

pub fn execute_rail(repository_root: &Path, logs_directory: &Path, spec: &RailSpec) -> RailReceipt {
    let stdout_path = logs_directory.join(format!("{}.stdout.log", spec.id));
    let stderr_path = logs_directory.join(format!("{}.stderr.log", spec.id));
    let started_unix_ms = unix_time_ms();
    let started = Instant::now();

    let command_vector = std::iter::once(spec.program.clone())
        .chain(spec.args.iter().cloned())
        .collect::<Vec<_>>();

    let stdout_file = match File::create(&stdout_path) {
        Ok(file) => file,
        Err(error) => {
            return failed_receipt(
                spec,
                command_vector,
                started_unix_ms,
                started.elapsed().as_millis(),
                &stdout_path,
                &stderr_path,
                format!("failed to create stdout log: {error}"),
            )
        }
    };
    let stderr_file = match File::create(&stderr_path) {
        Ok(file) => file,
        Err(error) => {
            return failed_receipt(
                spec,
                command_vector,
                started_unix_ms,
                started.elapsed().as_millis(),
                &stdout_path,
                &stderr_path,
                format!("failed to create stderr log: {error}"),
            )
        }
    };

    let program = resolve_program(repository_root, &spec.program);
    let mut command = Command::new(program);
    command
        .args(&spec.args)
        .current_dir(repository_root.join(&spec.working_directory))
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file));
    for (name, value) in &spec.environment {
        command.env(name, value);
    }

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
                format!("failed to spawn command: {error}"),
            )
        }
    };

    let timeout = Duration::from_secs(spec.timeout_seconds);
    let mut timed_out = false;
    let mut execution_error = None;
    let status: Option<ExitStatus> = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) if started.elapsed() < timeout => {
                thread::sleep(Duration::from_millis(100));
            }
            Ok(None) => {
                timed_out = true;
                if let Err(error) = child.kill() {
                    execution_error = Some(format!("timed out and failed to kill command: {error}"));
                }
                match child.wait() {
                    Ok(status) => break Some(status),
                    Err(error) => {
                        execution_error = Some(format!("failed to wait after timeout: {error}"));
                        break None;
                    }
                }
            }
            Err(error) => {
                execution_error = Some(format!("failed while waiting for command: {error}"));
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
        }
    };

    let stdout_blake3 = hash_file(&stdout_path).ok();
    let stderr_blake3 = hash_file(&stderr_path).ok();
    let passed = status.as_ref().is_some_and(ExitStatus::success)
        && !timed_out
        && execution_error.is_none();

    RailReceipt {
        id: spec.id.clone(),
        phase: spec.phase.clone(),
        command: command_vector,
        working_directory: spec.working_directory.clone(),
        required: spec.required,
        started_unix_ms,
        duration_ms: started.elapsed().as_millis(),
        exit_code: status.and_then(|value| value.code()),
        timed_out,
        spawn_error: execution_error,
        stdout_log: stdout_path.display().to_string(),
        stderr_log: stderr_path.display().to_string(),
        stdout_blake3,
        stderr_blake3,
        passed,
    }
}

fn failed_receipt(
    spec: &RailSpec,
    command: Vec<String>,
    started_unix_ms: u128,
    duration_ms: u128,
    stdout_path: &Path,
    stderr_path: &Path,
    error: String,
) -> RailReceipt {
    RailReceipt {
        id: spec.id.clone(),
        phase: spec.phase.clone(),
        command,
        working_directory: spec.working_directory.clone(),
        required: spec.required,
        started_unix_ms,
        duration_ms,
        exit_code: None,
        timed_out: false,
        spawn_error: Some(error),
        stdout_log: stdout_path.display().to_string(),
        stderr_log: stderr_path.display().to_string(),
        stdout_blake3: hash_file(stdout_path).ok(),
        stderr_blake3: hash_file(stderr_path).ok(),
        passed: false,
    }
}

fn resolve_program(repository_root: &Path, program: &str) -> PathBuf {
    if program.contains('/') || program.contains('\\') {
        repository_root.join(program)
    } else {
        PathBuf::from(program)
    }
}

pub fn hash_file(path: &Path) -> io::Result<String> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut hasher = blake3::Hasher::new();
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
