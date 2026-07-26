use crate::model::Provenance;
use std::path::Path;
use std::process::Command;

pub fn capture(repository_root: &Path) -> Provenance {
    let mut capture_errors = Vec::new();
    let status_text = capture_output(
        repository_root,
        "git",
        &["status", "--porcelain=v1", "--untracked-files=all"],
        "git status",
        &mut capture_errors,
    )
    .unwrap_or_default();
    let status = nonempty_lines(&status_text);

    let submodule_text = capture_output(
        repository_root,
        "git",
        &["submodule", "status", "--recursive"],
        "git submodule status",
        &mut capture_errors,
    )
    .unwrap_or_default();
    let submodules = nonempty_lines(&submodule_text);

    let head_sha = capture_output(
        repository_root,
        "git",
        &["rev-parse", "--verify", "HEAD^{commit}"],
        "git HEAD",
        &mut capture_errors,
    );
    let object_format = capture_output(
        repository_root,
        "git",
        &["rev-parse", "--show-object-format"],
        "git object format",
        &mut capture_errors,
    );
    let commit_unix_seconds = capture_output(
        repository_root,
        "git",
        &["show", "-s", "--format=%ct", "HEAD"],
        "git commit timestamp",
        &mut capture_errors,
    )
    .and_then(|value| match value.parse::<u64>() {
        Ok(parsed) => Some(parsed),
        Err(error) => {
            capture_errors.push(format!("invalid git commit timestamp {value:?}: {error}"));
            None
        }
    });
    let branch = capture_output(
        repository_root,
        "git",
        &["branch", "--show-current"],
        "git branch",
        &mut capture_errors,
    );
    let remote_url = capture_output(
        repository_root,
        "git",
        &["remote", "get-url", "origin"],
        "git origin",
        &mut capture_errors,
    );
    let normalized_repository = remote_url.as_deref().and_then(normalize_remote_repository);
    if remote_url.is_some() && normalized_repository.is_none() {
        capture_errors.push("origin remote could not be normalized to owner/name".to_owned());
    }

    Provenance {
        head_sha,
        object_format,
        commit_unix_seconds,
        branch,
        remote_url,
        normalized_repository,
        dirty: !status.is_empty(),
        status,
        submodules,
        rustc: capture_output(
            repository_root,
            "rustc",
            &["-Vv"],
            "rustc version",
            &mut capture_errors,
        ),
        cargo: capture_output(
            repository_root,
            "cargo",
            &["-Vv"],
            "cargo version",
            &mut capture_errors,
        ),
        operating_system: std::env::consts::OS.to_owned(),
        architecture: std::env::consts::ARCH.to_owned(),
        capture_errors,
    }
}

pub fn repository_state_digest(repository_root: &Path) -> Result<String, String> {
    let status = command_output(
        repository_root,
        "git",
        &["status", "--porcelain=v1", "--untracked-files=all"],
    )?;
    let submodules = command_output(
        repository_root,
        "git",
        &["submodule", "status", "--recursive"],
    )?;
    let head = command_output(
        repository_root,
        "git",
        &["rev-parse", "--verify", "HEAD^{commit}"],
    )?;

    let mut hasher = blake3::Hasher::new_derive_key("bcinr.release.repository-state.v2");
    update_framed(&mut hasher, head.as_bytes());
    update_framed(&mut hasher, status.as_bytes());
    update_framed(&mut hasher, submodules.as_bytes());
    Ok(hasher.finalize().to_hex().to_string())
}

pub fn normalize_remote_repository(remote: &str) -> Option<String> {
    let trimmed = remote.trim().trim_end_matches('/').trim_end_matches(".git");
    let path = if let Some(value) = trimmed.strip_prefix("git@github.com:") {
        value
    } else if let Some(value) = trimmed.strip_prefix("ssh://git@github.com/") {
        value
    } else if let Some(value) = trimmed.strip_prefix("https://github.com/") {
        value
    } else {
        trimmed.strip_prefix("http://github.com/")?
    };

    let mut parts = path.split('/');
    let owner = parts.next()?;
    let repository = parts.next()?;
    if owner.is_empty() || repository.is_empty() || parts.next().is_some() {
        return None;
    }
    Some(format!(
        "{}/{}",
        owner.to_ascii_lowercase(),
        repository.to_ascii_lowercase()
    ))
}

fn capture_output(
    repository_root: &Path,
    program: &str,
    args: &[&str],
    label: &str,
    errors: &mut Vec<String>,
) -> Option<String> {
    match command_output(repository_root, program, args) {
        Ok(value) if value.is_empty() => None,
        Ok(value) => Some(value),
        Err(error) => {
            errors.push(format!("{label}: {error}"));
            None
        }
    }
}

fn command_output(repository_root: &Path, program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(repository_root)
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .output()
        .map_err(|error| format!("failed to execute {program}: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "{program} exited with {:?}: {}",
            output.status.code(),
            stderr.trim()
        ));
    }
    String::from_utf8(output.stdout)
        .map(|value| value.trim().to_owned())
        .map_err(|error| format!("{program} emitted non-UTF-8 output: {error}"))
}

fn nonempty_lines(value: &str) -> Vec<String> {
    value
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn update_framed(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

#[cfg(test)]
mod tests {
    use super::normalize_remote_repository;

    #[test]
    fn normalizes_supported_github_remotes() {
        assert_eq!(
            normalize_remote_repository("https://github.com/SeanChatmanGPT/bcinr.git"),
            Some("seanchatmangpt/bcinr".to_owned())
        );
        assert_eq!(
            normalize_remote_repository("git@github.com:seanchatmangpt/bcinr.git"),
            Some("seanchatmangpt/bcinr".to_owned())
        );
    }

    #[test]
    fn refuses_adjacent_or_ambiguous_remotes() {
        assert_eq!(
            normalize_remote_repository("https://example.com/seanchatmangpt/bcinr.git"),
            None
        );
        assert_eq!(
            normalize_remote_repository("https://github.com/other/seanchatmangpt/bcinr.git"),
            None
        );
    }
}
