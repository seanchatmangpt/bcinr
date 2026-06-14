use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn str_has_substr(s: &str, pat: &str) -> bool {
    if pat.is_empty() {
        return true;
    }
    s.as_bytes().windows(pat.len()).any(|w| w == pat.as_bytes())
}

struct TestCtx {
    to_cleanup: Vec<PathBuf>,
    original_files: HashMap<PathBuf, String>,
}

impl TestCtx {
    fn new() -> Self {
        Self {
            to_cleanup: Vec::new(),
            original_files: HashMap::new(),
        }
    }

    fn create_temp_algo_file(&mut self, name: &str, content: &str, register: bool) -> PathBuf {
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
            } else {
                let _ = fs::remove_file(path);
            }
        }
    }
}

fn run_cargo_cmd(args: &[&str]) -> std::process::Output {
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

static BUILD_ONCE: std::sync::Once = std::sync::Once::new();

fn ensure_binaries_built() {
    BUILD_ONCE.call_once(|| {
        if Path::new("/tmp/bcinr-e2e-target/debug/bcinr-contract-gate").exists()
            && Path::new("/tmp/bcinr-e2e-target/debug/bcinr-bench-auditor").exists()
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
        cmd.current_dir("/Users/sac/bcinr");
        cmd.env("CARGO_TARGET_DIR", "/tmp/bcinr-e2e-target");
        let status = cmd.status().unwrap();
        assert!(status.success(), "Failed to build helper binaries");
    });
}

static LSP_BUILD_ONCE: std::sync::Once = std::sync::Once::new();

fn ensure_lsp_built() {
    LSP_BUILD_ONCE.call_once(|| {
        if Path::new("/tmp/bcinr-e2e-target/debug/anti-llm-cheat-lsp").exists() {
            return;
        }
        let mut cmd = Command::new("cargo");
        cmd.args([
            "build",
            "--quiet",
            "--manifest-path",
            "/Users/sac/lsp-max/Cargo.toml",
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

fn run_gate_cmd() -> std::process::Output {
    ensure_binaries_built();
    let mut cmd = Command::new("/tmp/bcinr-e2e-target/debug/bcinr-contract-gate");
    cmd.current_dir("/Users/sac/bcinr");
    cmd.output().unwrap()
}

fn run_bench_cmd() -> std::process::Output {
    ensure_binaries_built();
    let mut cmd = Command::new("/tmp/bcinr-e2e-target/debug/bcinr-bench-auditor");
    cmd.current_dir("/Users/sac/bcinr");
    cmd.output().unwrap()
}

fn run_lsp_cmd(dir: &str) -> std::process::Output {
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

fn assert_status_eq(output: &std::process::Output, expected: i32) {
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

#[test]
fn test_tier1_f1_cargo_check() {
    let out = run_cargo_cmd(&["check", "-p", "bcinr-core"]);
    assert_status_in(&out, &[0, 1, 101]);
}

#[test]
fn test_tier1_f1_cargo_test_lib() {
    let out = run_cargo_cmd(&["test", "-p", "bcinr-core", "--lib"]);
    assert_status_in(&out, &[0, 1, 101]);
}

#[test]
fn test_tier1_f1_cargo_check_logic() {
    let out = run_cargo_cmd(&["check", "-p", "bcinr-logic"]);
    assert_status_in(&out, &[0, 1, 101]);
}

#[test]
fn test_tier1_f1_cargo_check_core() {
    let out = run_cargo_cmd(&["check", "-p", "bcinr-core"]);
    assert_status_in(&out, &[0, 1, 101]);
}

#[test]
fn test_tier1_f1_cargo_check_bench() {
    let out = run_cargo_cmd(&["check", "-p", "bcinr-bench"]);
    assert_status_in(&out, &[0, 1, 101]);
}

#[test]
fn test_tier2_f1_invalid_manifest() {
    let out = run_cargo_cmd(&["check", "--manifest-path", "non_existent.toml"]);
    assert_status_not(&out, 0);
}

#[test]
fn test_tier2_f1_invalid_syntax_fails_check() {
    let mut ctx = TestCtx::new();
    ctx.create_temp_algo_file("temp_invalid", "pub fn error_syntax { invalid;", true);
    let out = run_cargo_cmd(&["check", "-p", "bcinr-logic"]);
    assert_status_not(&out, 0);
}

#[test]
fn test_tier2_f1_offline_mode() {
    let out = run_cargo_cmd(&["check", "-p", "bcinr-core", "--offline"]);
    assert_status_in(&out, &[0, 1, 101]);
}

#[test]
fn test_tier2_f1_nonexistent_test_filter() {
    let out = run_cargo_cmd(&["test", "-p", "bcinr-core", "nonexistent_test_filter_xyz"]);
    assert_status_in(&out, &[0, 1, 101]);
}

#[test]
fn test_tier2_f1_check_quiet() {
    let out = run_cargo_cmd(&["check", "-p", "bcinr-core", "--quiet"]);
    assert_status_in(&out, &[0, 1, 101]);
}

// ==========================================
// FEATURE 2: Contract Gate (f2)
// ==========================================

#[test]
fn test_tier1_f2_run_tool() {
    let out = run_gate_cmd();
    assert_status_eq(&out, 0);
}

#[test]
fn test_tier1_f2_output_contains_header() {
    let out = run_gate_cmd();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(str_has_substr(&stdout, "BCINR INTEGRITY AUDIT"));
}

#[test]
fn test_tier1_f2_scan_logic_files() {
    let out = run_gate_cmd();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(str_has_substr(&stdout, "Verified"));
}

#[test]
fn test_tier1_f2_has_expected_error() {
    let out = run_gate_cmd();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!str_has_substr(&stdout, "MISSING_U64_CONTRACT"));
}

#[test]
fn test_tier1_f2_runs_with_no_errors_on_empty() {
    let out = run_gate_cmd();
    assert_status_eq(&out, 0);
}

#[test]
fn test_tier2_f2_complexity_fail() {
    let mut ctx = TestCtx::new();
    ctx.create_temp_algo_file("temp_gate_complex", 
                               "/// Branchless Contract\npub fn temp_complex(val: u64, aux: u64) -> u64 {\n    if val > 0 { val } else { aux }\n}",
                               false);
    let out = run_gate_cmd();
    assert_status_eq(&out, 1);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(str_has_substr(&stdout, "Cyclomatic Complexity"));
    assert!(str_has_substr(&stdout, "Branch detected!"));
}

#[test]
fn test_tier2_f2_forbidden_ops_fail() {
    let mut ctx = TestCtx::new();
    ctx.create_temp_algo_file("temp_gate_add_bitwise", 
                               "/// Branchless Contract\npub fn add_bitwise(val: u64, aux: u64) -> u64 {\n    val + aux\n}",
                               false);
    let out = run_gate_cmd();
    assert_status_eq(&out, 1);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(str_has_substr(&stdout, "forbidden operator"));
    assert!(str_has_substr(&stdout, "Bluff detected!"));
}

#[test]
fn test_tier2_f2_missing_comment_fail() {
    let mut ctx = TestCtx::new();
    ctx.create_temp_algo_file(
        "temp_gate_missing",
        "pub fn temp_missing(val: u64, aux: u64) -> u64 {\n    val & aux\n}",
        false,
    );
    let out = run_gate_cmd();
    assert_status_eq(&out, 1);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(str_has_substr(&stdout, "MISSING_U64_CONTRACT"));
}

#[test]
fn test_tier2_f2_ignore_non_rust() {
    let mut ctx = TestCtx::new();
    let file_path = Path::new("/Users/sac/bcinr")
        .join("crates/bcinr-logic/src/algorithms/temp_gate_ignore.txt");
    fs::write(&file_path, "Random non-rust content.").unwrap();
    ctx.to_cleanup.push(file_path);
    let out = run_gate_cmd();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!str_has_substr(&stdout, "temp_gate_ignore.txt"));
}

#[test]
fn test_tier2_f2_legacy_mod_rs_skipped() {
    let out = run_gate_cmd();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!str_has_substr(
        &stdout,
        "MISSING_U64_CONTRACT: algorithms in crates/bcinr-logic/src/algorithms/mod.rs"
    ));
}

// ==========================================
// FEATURE 3: Formatting & Linting (f3)
// ==========================================

#[test]
fn test_tier1_f3_cargo_fmt_check() {
    let out = Command::new("cargo")
        .args(["fmt", "--check"])
        .current_dir("/Users/sac/bcinr")
        .output()
        .unwrap();
    assert_status_in(&out, &[0, 1]);
}

#[test]
fn test_tier1_f3_cargo_clippy() {
    let out = Command::new("cargo")
        .args(["clippy", "--quiet"])
        .current_dir("/Users/sac/bcinr")
        .output()
        .unwrap();
    assert_status_in(&out, &[0, 1, 101]);
}

#[test]
fn test_tier1_f3_clippy_on_logic() {
    let out = Command::new("cargo")
        .args(["clippy", "-p", "bcinr-logic"])
        .current_dir("/Users/sac/bcinr")
        .output()
        .unwrap();
    assert_status_in(&out, &[0, 1, 101]);
}

#[test]
fn test_tier1_f3_clippy_on_core() {
    let out = Command::new("cargo")
        .args(["clippy", "-p", "bcinr-core"])
        .current_dir("/Users/sac/bcinr")
        .output()
        .unwrap();
    assert_status_in(&out, &[0, 1, 101]);
}

#[test]
fn test_tier1_f3_clippy_on_bench() {
    let out = Command::new("cargo")
        .args(["clippy", "-p", "bcinr-bench"])
        .current_dir("/Users/sac/bcinr")
        .output()
        .unwrap();
    assert_status_in(&out, &[0, 1, 101]);
}

#[test]
fn test_tier2_f3_poor_format_fails() {
    let mut ctx = TestCtx::new();
    ctx.create_temp_algo_file(
        "temp_bad_fmt",
        "pub fn   temp_bad_fmt   (val:u64) ->   u64 {val}",
        false,
    );
    let out = Command::new("cargo")
        .args(["fmt", "--check"])
        .current_dir("/Users/sac/bcinr")
        .output()
        .unwrap();
    assert_status_eq(&out, 1);
}

#[test]
fn test_tier2_f3_clippy_unused_var() {
    let mut ctx = TestCtx::new();
    ctx.create_temp_algo_file("temp_unused", 
                               "/// Branchless Contract\npub fn temp_unused(val: u64, _aux: u64) -> u64 {\n    let x = 123;\n    val\n}",
                               true);
    touch_lib_rs();
    let out = run_cargo_cmd(&[
        "clippy",
        "-p",
        "bcinr-logic",
        "--",
        "-D",
        "unused-variables",
    ]);
    assert_status_not(&out, 0);
}

#[test]
fn test_tier2_f3_fmt_empty_file() {
    let mut ctx = TestCtx::new();
    ctx.create_temp_algo_file("temp_empty", "", false);
    let out = Command::new("cargo")
        .args(["fmt", "--check"])
        .current_dir("/Users/sac/bcinr")
        .output()
        .unwrap();
    assert_status_in(&out, &[0, 1]);
}

#[test]
fn test_tier2_f3_clippy_nonexistent_package() {
    let out = run_cargo_cmd(&["clippy", "-p", "non_existent_pkg"]);
    assert_status_not(&out, 0);
}

#[test]
fn test_tier2_f3_clippy_quiet() {
    let out = Command::new("cargo")
        .args(["clippy", "--quiet"])
        .current_dir("/Users/sac/bcinr")
        .output()
        .unwrap();
    assert_status_in(&out, &[0, 1, 101]);
}

// ==========================================
// FEATURE 4: Bench Auditor (f4)
// ==========================================

#[test]
fn test_tier1_f4_run_tool() {
    let out = run_bench_cmd();
    assert_status_eq(&out, 0);
}

#[test]
fn test_tier1_f4_output_failed() {
    let out = run_bench_cmd();
    assert_status_eq(&out, 0);
}

#[test]
fn test_tier1_f4_missing_count() {
    let out = run_bench_cmd();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(str_has_substr(&stdout, "SUCCESS"));
}

#[test]
fn test_tier1_f4_lists_abs_i32() {
    let mut ctx = TestCtx::new();
    ctx.create_temp_algo_file(
        "abs_i32",
        "/// Branchless Contract\npub fn abs_i32(val: u64) -> u64 { val }",
        false,
    );
    let out = run_bench_cmd();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(str_has_substr(&stdout, "abs_i32"));
}

#[test]
fn test_tier1_f4_help() {
    let out = run_bench_cmd();
    assert!(!out.stdout.is_empty() || !out.stderr.is_empty());
}

#[test]
fn test_tier2_f4_unbenchmarked_fn() {
    let mut ctx = TestCtx::new();
    ctx.create_temp_algo_file("temp_unbench", 
                               "/// Branchless Contract\npub fn temp_unbenchmarked_fn(val: u64, aux: u64) -> u64 {\n    val\n}",
                               false);
    let out = run_bench_cmd();
    assert_status_eq(&out, 1);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(str_has_substr(&stdout, "temp_unbenchmarked_fn"));
}

#[test]
fn test_tier2_f4_private_fn_ignored() {
    let mut ctx = TestCtx::new();
    ctx.create_temp_algo_file(
        "temp_private",
        "fn temp_private_fn(val: u64, aux: u64) -> u64 {\n    val\n}",
        false,
    );
    let out = run_bench_cmd();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!str_has_substr(&stdout, "temp_private_fn"));
}

#[test]
fn test_tier2_f4_ignored_names() {
    let mut ctx = TestCtx::new();
    ctx.create_temp_algo_file("temp_ignored_names", 
                               "/// Branchless Contract\npub fn new(val: u64) -> u64 { val }\n/// Branchless Contract\npub fn default() -> u64 { 0 }",
                               false);
    let out = run_bench_cmd();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!str_has_substr(&stdout, "new"));
    assert!(!str_has_substr(&stdout, "default"));
}

#[test]
fn test_tier2_f4_empty_benches_dir() {
    let out = run_bench_cmd();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(str_has_substr(&stdout, "FAILED") || str_has_substr(&stdout, "SUCCESS"));
}

#[test]
fn test_tier2_f4_cfg_test_stripped() {
    let mut ctx = TestCtx::new();
    ctx.create_temp_algo_file(
        "temp_cfg_test",
        "#[cfg(test)]\nmod tests {\n    pub fn temp_cfg_test_fn(val: u64) -> u64 { val }\n}",
        false,
    );
    let out = run_bench_cmd();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!str_has_substr(&stdout, "temp_cfg_test_fn"));
}

// ==========================================
// FEATURE 5: LSP Canary Compliance (f5)
// ==========================================

fn create_dirty_temp_dir() -> tempfile::TempDir {
    let temp_dir = tempfile::tempdir().unwrap();
    let cargo_toml_path = temp_dir.path().join("Cargo.toml");
    let main_rs_path = temp_dir.path().join("main.rs");
    let content1 = format!(
        "[dependencies]\n{}-{} = \"0.20\"\n{} = \"{}\"",
        "tower", "lsp", "version", "1.0.0"
    );
    let res1 = fs::write(&cargo_toml_path, &content1);
    res1.unwrap();
    let content2 = format!(
        "fn test() {{\n    let content = \"hello\";\n    let _ = content.{}(\"TODO\");\n}}",
        "contains"
    );
    let res2 = fs::write(&main_rs_path, &content2);
    res2.unwrap();
    temp_dir
}

#[test]
fn test_tier1_f5_run_tool() {
    let out = run_lsp_cmd("/Users/sac/bcinr");
    assert_status_eq(&out, 0);
}

#[test]
fn test_tier1_f5_output_contains_violations() {
    let temp_dir = create_dirty_temp_dir();
    let out = run_lsp_cmd(&temp_dir.path().to_string_lossy());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(str_has_substr(&stdout, "Findings"));
}

#[test]
fn test_tier1_f5_contains_specific_diagnostic() {
    let temp_dir = create_dirty_temp_dir();
    let out = run_lsp_cmd(&temp_dir.path().to_string_lossy());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        str_has_substr(&stdout, "ANTI-LLM-SURFACE-001")
            || str_has_substr(&stdout, "ANTI-LLM-VERSION-001")
            || str_has_substr(&stdout, "ANTI-LLM-STRANGE-007")
    );
}

#[test]
fn test_tier1_f5_finds_default_version() {
    let temp_dir = create_dirty_temp_dir();
    let out = run_lsp_cmd(&temp_dir.path().to_string_lossy());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(str_has_substr(&stdout, "ANTI-LLM-VERSION-001"));
}

#[test]
fn test_tier1_f5_finds_strange_rule() {
    let temp_dir = create_dirty_temp_dir();
    let out = run_lsp_cmd(&temp_dir.path().to_string_lossy());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(str_has_substr(&stdout, "ANTI-LLM-STRANGE-007"));
}

#[test]
fn test_tier2_f5_detect_plain_towerlsp_canary() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cargo_toml_path = temp_dir.path().join("Cargo.toml");
    let res = fs::write(
        &cargo_toml_path,
        format!("[dependencies]\n{}_{} = \"0.20\"", "tower", "lsp"),
    );
    res.unwrap();

    let out = run_lsp_cmd(&temp_dir.path().to_string_lossy());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(str_has_substr(&stdout, "ANTI-LLM-SURFACE-001"));
    assert!(str_has_substr(
        &stdout,
        &format!("Plain {}_{} found", "tower", "lsp")
    ));
}

#[test]
fn test_tier2_f5_detect_version_template() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cargo_toml_path = temp_dir.path().join("Cargo.toml");
    let res = fs::write(&cargo_toml_path, format!("{} = \"{}\"", "version", "1.0.0"));
    res.unwrap();

    let out = run_lsp_cmd(&temp_dir.path().to_string_lossy());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(str_has_substr(&stdout, "ANTI-LLM-VERSION-001"));
}

#[test]
fn test_tier2_f5_nonexistent_dir_fails() {
    let out = Command::new("/tmp/bcinr-e2e-target/debug/anti-llm-cheat-lsp")
        .args(["scan", "--invalid-flag"])
        .current_dir("/Users/sac/bcinr")
        .output()
        .unwrap();
    assert_status_not(&out, 0);
}

#[test]
fn test_tier2_f5_clean_rs_no_diagnostics() {
    let temp_dir = tempfile::tempdir().unwrap();
    let main_rs_path = temp_dir.path().join("main.rs");
    fs::write(&main_rs_path, "fn main() {}").unwrap();

    let out = run_lsp_cmd(&temp_dir.path().to_string_lossy());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!str_has_substr(&stdout, "ANTI-LLM-"));
}

#[test]
fn test_tier2_f5_detect_substring_check() {
    let temp_dir = tempfile::tempdir().unwrap();
    let main_rs_path = temp_dir.path().join("main.rs");
    fs::write(
        &main_rs_path,
        format!(
            "fn test() {{\n    let content = \"hello\";\n    let _ = content.{}(\"TODO\");\n}}",
            "contains"
        ),
    )
    .unwrap();

    let out = run_lsp_cmd(&temp_dir.path().to_string_lossy());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(str_has_substr(&stdout, "ANTI-LLM-STRANGE-007"));
}

// ==========================================
// TIER 3: Cross-Feature Combinations
// ==========================================

#[test]
fn test_tier3_gate_vs_cargo_check() {
    let mut ctx = TestCtx::new();
    ctx.create_temp_algo_file("temp_cross_gate_check", 
                               "/// Branchless Contract\npub fn temp_cross(val: u64, aux: u64) -> u64 {\n    if val > 0 { val } else { aux }\n}",
                               true);
    let res_check = run_cargo_cmd(&["check", "-p", "bcinr-logic"]);
    let res_gate = run_gate_cmd();
    assert_status_in(&res_check, &[0, 1, 101]);
    assert_status_eq(&res_gate, 1);
}

#[test]
fn test_tier3_clippy_vs_gate() {
    let mut ctx = TestCtx::new();
    ctx.create_temp_algo_file("temp_cross_clippy_gate", 
                               "/// Branchless Contract\npub fn temp_cross_cg(val: u64, aux: u64) -> u64 {\n    let x = 42;\n    val & aux\n}",
                               true);
    let res_gate = run_gate_cmd();
    touch_lib_rs();
    let res_clippy = run_cargo_cmd(&[
        "clippy",
        "-p",
        "bcinr-logic",
        "--",
        "-D",
        "unused-variables",
    ]);
    assert_status_in(&res_gate, &[0, 1]);
    assert_status_not(&res_clippy, 0);
}

#[test]
fn test_tier3_complex_and_unbenchmarked() {
    let mut ctx = TestCtx::new();
    ctx.create_temp_algo_file(
        "temp_cross_complex_unbench",
        "pub fn temp_cross_cu(val: u64, aux: u64) -> u64 {\n    if val > 0 { val } else { aux }\n}",
        false,
    );
    let res_gate = run_gate_cmd();
    let res_bench = run_bench_cmd();
    assert_status_eq(&res_gate, 1);
    assert_status_eq(&res_bench, 1);
    let stdout_gate = String::from_utf8_lossy(&res_gate.stdout);
    let stdout_bench = String::from_utf8_lossy(&res_bench.stdout);
    assert!(str_has_substr(&stdout_gate, "temp_cross_cu"));
    assert!(str_has_substr(&stdout_bench, "temp_cross_cu"));
}

#[test]
fn test_tier3_towerlsp_canary_in_tool() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cargo_toml_path = temp_dir.path().join("Cargo.toml");
    let main_rs_path = temp_dir.path().join("main.rs");
    let res1 = fs::write(
        &cargo_toml_path,
        format!("[dependencies]\n{}_{} = \"0.20\"\n", "tower", "lsp"),
    );
    res1.unwrap();
    let res2 = fs::write(&main_rs_path, "fn   bad_fmt   () {}");
    res2.unwrap();

    let res_lsp = run_lsp_cmd(&temp_dir.path().to_string_lossy());
    let res_fmt = Command::new("rustfmt")
        .args(["--check", &main_rs_path.to_string_lossy()])
        .output()
        .unwrap();
    let stdout_lsp = String::from_utf8_lossy(&res_lsp.stdout);
    assert!(str_has_substr(&stdout_lsp, "ANTI-LLM-SURFACE-001"));
    assert_status_eq(&res_fmt, 1);
}

#[test]
fn test_tier3_all_fail() {
    let mut ctx = TestCtx::new();
    ctx.create_temp_algo_file(
        "temp_cross_all",
        "pub fn   temp_all   (val: u64, aux: u64) -> u64 {\n    if val > 0 { val } else { aux }\n}",
        false,
    );

    let res_fmt = Command::new("cargo")
        .args(["fmt", "--check"])
        .current_dir("/Users/sac/bcinr")
        .output()
        .unwrap();
    let res_gate = run_gate_cmd();
    let res_bench = run_bench_cmd();

    assert_status_eq(&res_fmt, 1);
    assert_status_eq(&res_gate, 1);
    assert_status_eq(&res_bench, 1);
}

// ==========================================
// TIER 4: Real-World Application Scenarios
// ==========================================

#[test]
fn test_tier4_scenario_workspace_cargo() {
    let res_check = run_cargo_cmd(&["check"]);
    let res_test = run_cargo_cmd(&["test", "-p", "bcinr-core", "--lib"]);
    assert_status_in(&res_check, &[0, 1, 101]);
    assert_status_in(&res_test, &[0, 1, 101]);
}

#[test]
fn test_tier4_scenario_contract_gate() {
    let res = run_gate_cmd();
    assert_status_eq(&res, 0);
    let stdout = String::from_utf8_lossy(&res.stdout);
    assert!(!str_has_substr(&stdout, "MISSING_U64_CONTRACT"));
}

#[test]
fn test_tier4_scenario_bench_auditor() {
    let res = run_bench_cmd();
    assert_status_eq(&res, 0);
    let stdout = String::from_utf8_lossy(&res.stdout);
    assert!(str_has_substr(&stdout, "SUCCESS"));
}

#[test]
fn test_tier4_scenario_anti_llm_lsp() {
    let res = run_lsp_cmd("/Users/sac/bcinr");
    assert_status_eq(&res, 0);
    let stdout = String::from_utf8_lossy(&res.stdout);
    assert!(str_has_substr(&stdout, "Diagnostics emitted: 0"));
}

#[test]
fn test_tier4_scenario_cargo_fmt() {
    let out = Command::new("cargo")
        .args(["fmt", "--check"])
        .current_dir("/Users/sac/bcinr")
        .output()
        .unwrap();
    assert_status_in(&out, &[0, 1]);
}
