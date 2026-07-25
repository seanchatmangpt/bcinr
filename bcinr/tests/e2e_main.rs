use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

impl TestCtx {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            to_cleanup: Vec::new(),
            original_files: HashMap::new(),
        }
    }

    pub fn create_temp_algo_file(&mut self, name: &str, content: &str, register: bool) -> PathBuf {
        let repo_dir = "/Users/sac/bcinr";
        let file_path = Path::new(repo_dir)
            .join("crates/bcinr-logic/src/algorithms")
            .join(format!("{}.rs", name));
        fs::create_dir_all(file_path.parent().unwrap()).unwrap();
        fs::write(&file_path, content).unwrap();
        self.to_cleanup.push(file_path.clone());

        if register {
            let mod_path = Path::new(repo_dir)
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
    cmd.current_dir("/Users/sac/bcinr");
    cmd.env("CARGO_TARGET_DIR", "/tmp/bcinr-e2e-target");
    cmd.output().unwrap()
}

fn touch_lib_rs() {
    let lib_path = Path::new("/Users/sac/bcinr/crates/bcinr-logic/src/lib.rs");
    if let Ok(content) = fs::read_to_string(lib_path) {
        let _ = fs::write(lib_path, content);
    }
}

static LSP_BUILD_ONCE: std::sync::Once = std::sync::Once::new();

fn ensure_lsp_built() {
    LSP_BUILD_ONCE.call_once(|| {
        if Path::new("/tmp/bcinr-e2e-target/debug/anti-llm-cheat-lsp").exists() {
            return;
        }
        let mut cmd = Command::new("cargo");
        // The `anti-llm-cheat-lsp` package lives in its own standalone repo,
        // `/Users/sac/anti-llm-cheat-lsp` -- NOT in `/Users/sac/lsp-max`
        // (a different, unrelated workspace with 20+ members, none of them
        // named `anti-llm-cheat-lsp`; pointing here produced "error: package
        // ID specification `anti-llm-cheat-lsp` did not match any packages"
        // for every test that shells out to this binary). Matches
        // Makefile.toml's `lint-anti-llm` task, which already points at the
        // correct repo.
        cmd.args([
            "build",
            "--quiet",
            "--manifest-path",
            "/Users/sac/anti-llm-cheat-lsp/Cargo.toml",
            "--package",
            "anti-llm-cheat-lsp",
        ]);
        cmd.current_dir("/Users/sac/bcinr");
        cmd.env("CARGO_TARGET_DIR", "/tmp/bcinr-e2e-target");
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
    cmd.current_dir("/Users/sac/bcinr");
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
    cmd.current_dir("/Users/sac/bcinr");
    cmd.output().expect("failed to execute bcinr-bench-auditor")
}

pub fn run_lsp_cmd(dir: &str) -> std::process::Output {
    ensure_lsp_built();
    let mut cmd = Command::new("/tmp/bcinr-e2e-target/debug/anti-llm-cheat-lsp");
    cmd.arg("scan");
    cmd.args(["--dir", dir]);
    cmd.current_dir("/Users/sac/bcinr");
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
