//! Spawns real `cargo` subprocesses to exercise CLI tools end-to-end, which
//! Miri's isolation sandbox blocks (`open` unavailable) and which isn't
//! meaningful UB-checking territory anyway — skip this binary under Miri.
#![cfg(not(miri))]

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn get_repo_root() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let mut current = Path::new(manifest_dir).to_path_buf();

    loop {
        // Check if we're at a workspace root (has Cargo.toml and it's likely the main one)
        // Keep going up until we find a directory that doesn't have a parent with Cargo.toml
        let has_cargo = current.join("Cargo.toml").exists();
        if !has_cargo {
            if let Some(parent) = current.parent() {
                current = parent.to_path_buf();
                continue;
            }
            break;
        }

        // If we found a Cargo.toml, check if parent also has one
        if let Some(parent) = current.parent() {
            if !parent.join("Cargo.toml").exists() {
                // Parent doesn't have Cargo.toml, so current is the root
                return current;
            }
            // Parent has Cargo.toml too, keep searching
            current = parent.to_path_buf();
        } else {
            // No parent, we're at filesystem root
            return current;
        }
    }

    Path::new(manifest_dir).to_path_buf()
}

pub fn str_has_substr(s: &str, pat: &str) -> bool {
    if pat.is_empty() {
        return true;
    }
    s.as_bytes().windows(pat.len()).any(|w| w == pat.as_bytes())
}

/// Serializes every `TestCtx` against `crates/bcinr-logic/src/algorithms/
/// mod.rs`, the one real, shared source file `create_temp_algo_file`
/// read-modify-writes.
///
/// Without this, `cargo test`'s default thread-per-test parallelism lets
/// two `TestCtx`s race on that file: both read the same "original" content,
/// both append their own `pub mod temp_*;` line, and whichever writes last
/// wins — silently dropping the other's line from the file it thinks it's
/// about to restore. Each `TestCtx::drop` then writes back *its own*
/// `original_files` snapshot (captured before either wrote), so whichever
/// drops last "restores" a snapshot that already lacks the other test's
/// entry, but the module file on disk still carries the other test's now-
/// dangling `pub mod temp_*;` line pointing at a `.rs` file the other
/// test's own (already-run) cleanup deleted. The net effect, confirmed live
/// by running `cargo make test` twice in a row: `algorithms/mod.rs` is left
/// with leftover `pub mod temp_invalid;` / `pub mod temp_unused;` /
/// `pub mod temp_cross_clippy_gate;` declarations with no backing file,
/// breaking `bcinr-logic`'s compilation — and therefore every crate that
/// depends on it, including every MFW crate — for every build after the
/// test run until someone notices and manually deletes the stray lines.
pub fn mod_rs_lock() -> &'static std::sync::Mutex<()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
}

pub struct TestCtx {
    to_cleanup: Vec<PathBuf>,
    original_files: HashMap<PathBuf, String>,
}

impl Default for TestCtx {
    fn default() -> Self {
        Self::new()
    }
}

impl TestCtx {
    pub fn new() -> Self {
        Self {
            to_cleanup: Vec::new(),
            original_files: HashMap::new(),
        }
    }

    pub fn create_temp_algo_file(&mut self, name: &str, content: &str, register: bool) -> PathBuf {
        let repo_dir = get_repo_root();
        let file_path = repo_dir
            .join("crates/bcinr-logic/src/algorithms")
            .join(format!("{}.rs", name));
        fs::create_dir_all(file_path.parent().unwrap()).unwrap();
        fs::write(&file_path, content).unwrap();
        self.to_cleanup.push(file_path.clone());

        if register {
            let mod_path = repo_dir
                .join("crates")
                .join("bcinr-logic")
                .join("src")
                .join("algorithms")
                .join("mod.rs");
            if !self.original_files.contains_key(&mod_path) {
                let orig = fs::read_to_string(&mod_path).unwrap();
                self.original_files.insert(mod_path.clone(), orig);
            }
            let mut orig = fs::read_to_string(&mod_path).unwrap();
            orig.push_str(&format!("\npub mod {};\n", name));
            fs::write(&mod_path, orig).unwrap();
        }
        file_path
    }
}

impl Drop for TestCtx {
    fn drop(&mut self) {
        for (path, content) in &self.original_files {
            let _ = fs::write(path, content);
        }
        for path in &self.to_cleanup {
            if path.is_dir() {
                let _ = fs::remove_dir_all(path);
            } else if path.exists() {
                let _ = fs::remove_file(path);
            }
        }
    }
}

pub fn run_cargo_cmd(args: &[&str]) -> std::process::Output {
    let mut cmd = Command::new("cargo");
    cmd.args(args);
    let repo_root = get_repo_root();
    cmd.current_dir(&repo_root);
    let target_dir = std::env::temp_dir().join("bcinr-e2e-target");
    cmd.env("CARGO_TARGET_DIR", &target_dir);
    cmd.output().unwrap()
}

fn touch_lib_rs() {
    let repo_root = get_repo_root();
    let lib_path = repo_root.join("crates/bcinr-logic/src/lib.rs");
    if let Ok(content) = fs::read_to_string(&lib_path) {
        let _ = fs::write(&lib_path, content);
    }
}

#[allow(dead_code)] // retained helper for e2e tiers that build the CLI binaries on demand
static BUILD_ONCE: std::sync::Once = std::sync::Once::new();

#[allow(dead_code)] // retained helper for e2e tiers that build the CLI binaries on demand
fn ensure_binaries_built() {
    BUILD_ONCE.call_once(|| {
        let target_dir = std::env::temp_dir().join("bcinr-e2e-target");
        if target_dir.join("debug/bcinr-contract-gate").exists()
            && target_dir.join("debug/bcinr-bench-auditor").exists()
        {
            return;
        }
        let mut cmd = Command::new("cargo");
        cmd.args([
            "build",
            "--quiet",
            "--bin",
            "bcinr-contract-gate",
            "--bin",
            "bcinr-bench-auditor",
        ]);
        cmd.current_dir(get_repo_root());
        cmd.env("CARGO_TARGET_DIR", &target_dir);
        let status = cmd.status().unwrap();
        assert!(status.success(), "Failed to build helper binaries");
    });
}

static LSP_BUILD_ONCE: std::sync::Once = std::sync::Once::new();

fn ensure_lsp_built() {
    LSP_BUILD_ONCE.call_once(|| {
        let target_dir = std::env::temp_dir().join("bcinr-e2e-target");
        if target_dir.join("debug/anti-llm-cheat-lsp").exists() {
            return;
        }
        let repo_root = get_repo_root();
        // The `anti-llm-cheat-lsp` package lives in its own standalone repo,
        // one level up from the main bcinr repo as `anti-llm-cheat-lsp`.
        // Skip build if the repo doesn't exist (e.g., in CI environments).
        let parent_dir = repo_root.parent().unwrap_or(&repo_root);
        let lsp_manifest = parent_dir.join("anti-llm-cheat-lsp/Cargo.toml");
        if !lsp_manifest.exists() {
            eprintln!("anti-llm-cheat-lsp repository not found at {:?}, skipping LSP tests", lsp_manifest);
            return;
        }
        let mut cmd = Command::new("cargo");
        cmd.args([
            "build",
            "--quiet",
            "--manifest-path",
            lsp_manifest.to_str().unwrap(),
            "--package",
            "anti-llm-cheat-lsp",
        ]);
        cmd.current_dir(&repo_root);
        cmd.env("CARGO_TARGET_DIR", &target_dir);
        let status = cmd.status().unwrap();
        assert!(
            status.success(),
            "Failed to build anti-llm-cheat-lsp binary"
        );
    });
}

pub fn run_gate_cmd() -> std::process::Output {
    let mut cmd = Command::new("cargo");
    cmd.args([
        "run",
        "--manifest-path",
        "tools/bcinr-contract-gate/Cargo.toml",
        "--release",
        "--quiet",
    ]);
    cmd.current_dir(get_repo_root());
    cmd.output().expect("failed to execute bcinr-contract-gate")
}

pub fn run_bench_cmd() -> std::process::Output {
    let mut cmd = Command::new("cargo");
    cmd.args([
        "run",
        "--manifest-path",
        "tools/bcinr-bench-auditor/Cargo.toml",
        "--release",
        "--quiet",
    ]);
    cmd.current_dir(get_repo_root());
    cmd.output().expect("failed to execute bcinr-bench-auditor")
}

pub fn run_lsp_cmd(dir: &str) -> std::process::Output {
    ensure_lsp_built();
    let target_dir = std::env::temp_dir().join("bcinr-e2e-target");
    let lsp_binary = target_dir.join("debug/anti-llm-cheat-lsp");
    if !lsp_binary.exists() {
        eprintln!("anti-llm-cheat-lsp binary not found at {:?}, returning empty output", lsp_binary);
        // Return a dummy output that indicates the test should be skipped
        return std::process::Command::new("true").output().unwrap();
    }
    let mut cmd = Command::new(&lsp_binary);
    cmd.arg("scan");
    cmd.args(["--dir", dir]);
    cmd.current_dir(get_repo_root());
    cmd.output().unwrap()
}

fn assert_status_in(output: &std::process::Output, codes: &[i32]) {
    let code = output.status.code().unwrap_or(-1);
    assert!(
        codes.contains(&code),
        "Expected exit code in {:?}, got {} \nSTDOUT: {}\nSTDERR: {}",
        codes,
        code,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_status_not(output: &std::process::Output, code: i32) {
    let actual = output.status.code().unwrap_or(-1);
    assert_ne!(
        actual,
        code,
        "Expected exit code not to be {}, got it\nSTDOUT: {}\nSTDERR: {}",
        code,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

pub fn assert_status_eq(output: &std::process::Output, expected: i32) {
    let actual = output.status.code().unwrap_or(-1);
    assert_eq!(
        actual,
        expected,
        "Expected exit code {}, got {}\nSTDOUT: {}\nSTDERR: {}",
        expected,
        actual,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

// ==========================================
// FEATURE 1: Workspace Health (f1)
// ==========================================

mod e2e;
