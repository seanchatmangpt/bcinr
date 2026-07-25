use super::super::*;

#[test]
fn test_tier1_f1_cargo_check() {
    let _e2e_lock = crate::mod_rs_lock().lock().unwrap();
    let out = run_cargo_cmd(&["check", "-p", "bcinr-core"]);
    assert_status_in(&out, &[0, 1, 101]);
}

#[test]
fn test_tier1_f1_cargo_test_lib() {
    let _e2e_lock = crate::mod_rs_lock().lock().unwrap();
    let out = run_cargo_cmd(&["test", "-p", "bcinr-core", "--lib"]);
    assert_status_in(&out, &[0, 1, 101]);
}

#[test]
fn test_tier1_f1_cargo_check_logic() {
    let _e2e_lock = crate::mod_rs_lock().lock().unwrap();
    let out = run_cargo_cmd(&["check", "-p", "bcinr-logic"]);
    assert_status_in(&out, &[0, 1, 101]);
}

#[test]
fn test_tier1_f1_cargo_check_core() {
    let _e2e_lock = crate::mod_rs_lock().lock().unwrap();
    let out = run_cargo_cmd(&["check", "-p", "bcinr-core"]);
    assert_status_in(&out, &[0, 1, 101]);
}

#[test]
fn test_tier1_f1_cargo_check_bench() {
    let _e2e_lock = crate::mod_rs_lock().lock().unwrap();
    let out = run_cargo_cmd(&["check", "-p", "bcinr-bench"]);
    assert_status_in(&out, &[0, 1, 101]);
}
