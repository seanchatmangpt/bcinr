use crate::model::Provenance;
use std::path::Path;
use std::process::Command;

pub fn capture(repository_root: &Path) -> Provenance {
    let status_text = command_output(repository_root, "git", &["status", "--porcelain=v1", "--untracked-files=all"])
        .unwrap_or_default();
    let status = status_text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    Provenance {
        head_sha: command_output(repository_root, "git", &["rev-parse", "HEAD"]),
        branch: command_output(repository_root, "git", &["branch", "--show-current"]),
        remote_url: command_output(repository_root, "git", &["remote", "get-url", "origin"]),
        dirty: !status.is_empty(),
        status,
        rustc: command_output(repository_root, "rustc", &["-Vv"]),
        cargo: command_output(repository_root, "cargo", &["-V"]),
        operating_system: std::env::consts::OS.to_owned(),
        architecture: std::env::consts::ARCH.to_owned(),
    }
}

fn command_output(repository_root: &Path, program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(repository_root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?;
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}
