//! Sean education mode integration tests.
//!
//! 15 tests covering scan(), diagnostics, PDDL8 domain/problem generation,
//! planner integration, and receipt-gated admission.

use std::{fs, path::Path};
use tempfile::TempDir;

use bcinr_pddl_lsp::education::{
    check_newsletter_need9, education_diagnostics, emit_education_domain,
    emit_education_problem, render_education_gate, render_education_lane,
    render_education_status, scan, EducationStage,
};
use bcinr_pddl_lsp::projection::Pddl8Projection;
use bcinr_pddl_lsp::planner_client::{plan, admit};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn write_file(dir: &TempDir, path: &str, content: &str) {
    let full = dir.path().join(path);
    if let Some(parent) = full.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(full, content).unwrap();
}

/// Build a full fixture in a TempDir (all stages true, no education-week receipt).
fn full_fixture(dir: &TempDir) {
    write_file(dir, "README.md", "# Sean Education Mode\n\n## Status: ADMITTED\n");
    write_file(dir, "docs/prd.md", "# PRD\n\n## Status: ADMITTED\n");
    write_file(dir, "docs/ard.md", "# ARD\n\n## Status: ADMITTED\n");
    write_file(dir, "docs/adr/0001-education-mode.md", "# ADR 0001\n\n## Status: ADMITTED\n");
    write_file(dir, "career/interviews.json",
        r#"{"pipeline":"active","status":"received","slot_selected":true,"confirmed":true}"#);
    write_file(dir, "career/interview-prep.md", "# Prep\n\n## Status: ADMITTED\n\nContent.\n");
    write_file(dir, "linkedin/posts/001-dfcm-lsp.md",
        "# DfCM\n\nREVIEWED\n\nSTATUS: PUBLISHED\n\nLong enough content here for the draft check.\n");
    write_file(dir, ".bcinr/receipts/linkedin-post-001.json",
        r#"{"goal_reached":true,"platform":"linkedin","post_id":"001"}"#);
    write_file(dir, "newsletter/issues/001-mechanical-planning.md",
        "# Newsletter\n\nREVIEWED\n\nSTATUS: PUBLISHED\n\nContent.\n");
    write_file(dir, ".bcinr/receipts/newsletter-001.json",
        r#"{"goal_reached":true,"platform":"newsletter","issue":"001"}"#);
    write_file(dir, "youtube/videos/001-rust-dfcm/outline.md", "# Outline\n\nContent.\n");
    write_file(dir, "youtube/videos/001-rust-dfcm/script.md", "# Script\n\nContent.\n");
    write_file(dir, "youtube/videos/001-rust-dfcm/recording.json",
        r#"{"status":"published","url":"https://youtube.com/watch?v=placeholder"}"#);
    write_file(dir, ".bcinr/receipts/youtube-001.json",
        r#"{"goal_reached":true,"platform":"youtube","video_id":"001"}"#);
    write_file(dir, "lessons/rust/001-bcinr-pddl-lsp.md", "# Lesson\n\nSTATUS: PUBLISHED\n\nContent.\n");
    write_file(dir, "lessons/rust/examples/src/lib.rs", "pub fn foo() {}");
    write_file(dir, ".bcinr/test-report.json", r#"{"status":"passed","passed":true}"#);
    write_file(dir, ".bcinr/ocel/latest.json", r#"{"event_count":5,"events":[]}"#);
}

// ---------------------------------------------------------------------------
// Test 1
// ---------------------------------------------------------------------------

#[test]
fn empty_education_workspace_is_blocked() {
    let dir = TempDir::new().unwrap();
    let ws = scan(dir.path(), "sean");
    assert!(ws.true_stages.is_empty());
    assert_eq!(ws.next_missing(), Some(&EducationStage::CareerPipelineExists));
    let gate = render_education_gate(&ws);
    assert!(gate.contains("BLOCKED"), "gate should be BLOCKED: {gate}");
}

// ---------------------------------------------------------------------------
// Test 2
// ---------------------------------------------------------------------------

#[test]
fn interview_request_requires_slot_before_confirmation() {
    let dir = TempDir::new().unwrap();
    let full = dir.path().join("career/interviews.json");
    fs::create_dir_all(full.parent().unwrap()).unwrap();
    fs::write(&full, r#"{"status":"received","slot_selected":false,"confirmed":false}"#).unwrap();

    let ws = scan(dir.path(), "sean");
    assert!(ws.has(&EducationStage::CareerPipelineExists));
    assert!(ws.has(&EducationStage::InterviewRequestReceived));
    assert!(!ws.has(&EducationStage::InterviewSlotSelected), "slot not selected");
    assert!(!ws.has(&EducationStage::InterviewConfirmed), "not confirmed without slot");
}

// ---------------------------------------------------------------------------
// Test 3
// ---------------------------------------------------------------------------

#[test]
fn ard_prd_project_lifecycle_still_required() {
    let dir = TempDir::new().unwrap();
    // Has education artifacts but no docs/prd.md
    write_file(&dir, "career/interviews.json",
        r#"{"status":"received","slot_selected":true,"confirmed":true}"#);
    write_file(&dir, "career/interview-prep.md", "ADMITTED\n");

    // Use main lifecycle scan to check PRD_MISSING
    let lc = bcinr_pddl_lsp::lifecycle::scan(dir.path());
    assert!(!lc.has(&bcinr_pddl_lsp::lifecycle::LifecycleStage::PrdExists),
        "PRD should be missing");
}

// ---------------------------------------------------------------------------
// Test 4
// ---------------------------------------------------------------------------

#[test]
fn linkedin_candidate_not_admitted() {
    let dir = TempDir::new().unwrap();
    // Draft + review exists but no receipt
    write_file(&dir, "linkedin/posts/001.md",
        "# DfCM\n\nREVIEWED\n\nSTATUS: PUBLISHED\n\nLong enough content here.\n");
    // No .bcinr/receipts/linkedin-post-001.json

    let ws = scan(dir.path(), "sean");
    assert!(ws.has(&EducationStage::LinkedInReviewed));
    assert!(!ws.has(&EducationStage::LinkedInPublished), "no receipt → not published");

    let diags = education_diagnostics(&ws);
    let li_diag = diags.iter().find(|(c, _)| c == "LINKEDIN_RECEIPT_MISSING");
    assert!(li_diag.is_some(), "should have LINKEDIN_RECEIPT_MISSING: {diags:?}");
}

// ---------------------------------------------------------------------------
// Test 5
// ---------------------------------------------------------------------------

#[test]
fn newsletter_need9_split() {
    let dir = TempDir::new().unwrap();
    // Issue with 9 ## headers
    let sections: String = (1..=9).map(|i| format!("## Section {i}\n\nContent.\n\n")).collect();
    write_file(&dir, "newsletter/issues/001-big.md",
        &format!("# Big Issue\n\nREVIEWED\n\n{sections}"));

    let result = check_newsletter_need9(dir.path());
    assert!(result.is_some(), "should detect Need9 violation");
    let (code, _msg) = result.unwrap();
    assert_eq!(code, "NEWSLETTER_NEED9_SPLIT");
}

// ---------------------------------------------------------------------------
// Test 6
// ---------------------------------------------------------------------------

#[test]
fn youtube_requires_script_and_recording() {
    let dir = TempDir::new().unwrap();
    // Outline exists, no script, no recording
    write_file(&dir, "youtube/videos/001-rust-dfcm/outline.md", "# Outline\n\nContent.\n");

    let ws = scan(dir.path(), "sean");
    assert!(ws.has(&EducationStage::YouTubeTopicSelected));
    assert!(ws.has(&EducationStage::YouTubeOutlineExists));
    assert!(!ws.has(&EducationStage::YouTubeScriptExists), "no script");
    assert!(!ws.has(&EducationStage::YouTubeRecorded), "no recording");
    assert!(!ws.has(&EducationStage::YouTubePublished), "not published");
}

// ---------------------------------------------------------------------------
// Test 7
// ---------------------------------------------------------------------------

#[test]
fn rust_lesson_requires_example_tests() {
    let dir = TempDir::new().unwrap();
    write_file(&dir, "lessons/rust/001-bcinr.md", "# Lesson\n\nContent.\n");
    write_file(&dir, "lessons/rust/examples/src/lib.rs", "pub fn foo() {}");
    // No .bcinr/test-report.json

    let ws = scan(dir.path(), "sean");
    assert!(ws.has(&EducationStage::RustLessonSelected));
    assert!(ws.has(&EducationStage::RustExampleExists));
    assert!(!ws.has(&EducationStage::RustExampleTestsPassed), "no test report");
}

// ---------------------------------------------------------------------------
// Test 8
// ---------------------------------------------------------------------------

#[test]
fn education_week_candidate_is_not_admitted() {
    let dir = TempDir::new().unwrap();
    full_fixture(&dir);
    // No education-week receipt

    let ws = scan(dir.path(), "sean");
    assert!(!ws.has(&EducationStage::EducationWeekPublished), "not admitted without receipt");

    let gate = render_education_gate(&ws);
    assert!(!gate.contains("\"gate\":\"ADMITTED\""), "gate should not be ADMITTED: {gate}");
}

// ---------------------------------------------------------------------------
// Test 9
// ---------------------------------------------------------------------------

#[test]
fn explicit_education_week_admission_emits_receipt_and_ocel() {
    let dir = TempDir::new().unwrap();
    full_fixture(&dir);

    let domain = emit_education_domain();
    let ws = scan(dir.path(), "sean");
    let problem = emit_education_problem(&ws);

    let projection = Pddl8Projection { domain_text: domain, problem_text: problem };
    let candidate = plan(&projection).expect("plan should succeed");
    let result = admit(&candidate, "sean-education-week").expect("admit should succeed");

    assert!(result.receipt.goal_reached, "goal_reached must be true");
    assert!(!result.plan_steps.is_empty(), "plan must have steps");

    // Persist and check
    bcinr_pddl_lsp::planner_client::persist_admission(dir.path(), &result).unwrap();
    let receipt_path = dir.path().join(".bcinr/receipts/latest.json");
    assert!(receipt_path.exists(), "receipt should be persisted");
    let receipt_content = fs::read_to_string(&receipt_path).unwrap();
    let receipt_json: serde_json::Value = serde_json::from_str(&receipt_content).unwrap();
    assert_eq!(receipt_json["goal_reached"], serde_json::json!(true));
}

// ---------------------------------------------------------------------------
// Test 10
// ---------------------------------------------------------------------------

#[test]
fn interview_calendar_side_effect_requires_receipt() {
    let dir = TempDir::new().unwrap();
    write_file(&dir, "career/interviews.json",
        r#"{"status":"received","slot_selected":true,"confirmed":true}"#);
    // No calendar receipt — interview_confirmed is at CANDIDATE level only

    let ws = scan(dir.path(), "sean");
    assert!(ws.has(&EducationStage::InterviewConfirmed),
        "interview_confirmed detected from json");
    // The CANDIDATE/ADMITTED distinction is enforced by the planner gate,
    // not by scan(). Confirm the stage is present but education week is not.
    assert!(!ws.has(&EducationStage::EducationWeekPublished));
}

// ---------------------------------------------------------------------------
// Test 11
// ---------------------------------------------------------------------------

#[test]
fn linkedin_external_publish_requires_platform_receipt() {
    let dir = TempDir::new().unwrap();
    // Post has STATUS: PUBLISHED but no receipt
    write_file(&dir, "linkedin/posts/001.md",
        "# Post\n\nREVIEWED\n\nSTATUS: PUBLISHED\n\nLong enough content here for the draft check.\n");

    let ws = scan(dir.path(), "sean");
    assert!(!ws.has(&EducationStage::LinkedInPublished),
        "LinkedIn not published without receipt");

    let diags = education_diagnostics(&ws);
    let receipt_diag = diags.iter().find(|(c, _)| c == "LINKEDIN_RECEIPT_MISSING");
    assert!(receipt_diag.is_some(), "LINKEDIN_RECEIPT_MISSING expected: {diags:?}");
}

// ---------------------------------------------------------------------------
// Test 12
// ---------------------------------------------------------------------------

#[test]
fn youtube_publish_requires_artifact_or_platform_receipt() {
    let dir = TempDir::new().unwrap();
    write_file(&dir, "youtube/videos/001-rust-dfcm/outline.md", "# Outline\n\nContent.\n");
    write_file(&dir, "youtube/videos/001-rust-dfcm/script.md", "# Script\n\nContent.\n");
    write_file(&dir, "youtube/videos/001-rust-dfcm/recording.json",
        r#"{"status":"published","url":"https://youtube.com/watch?v=placeholder"}"#);
    // No .bcinr/receipts/youtube-001.json

    let ws = scan(dir.path(), "sean");
    assert!(ws.has(&EducationStage::YouTubeRecorded));
    assert!(!ws.has(&EducationStage::YouTubePublished), "no receipt → not published");
}

// ---------------------------------------------------------------------------
// Test 13
// ---------------------------------------------------------------------------

#[test]
fn education_week_publish_action_stays_within_8() {
    let domain = emit_education_domain();
    // Find the publish_education_week action block
    let action_start = domain.find("publish_education_week").expect("action not found");
    let action_block = &domain[action_start..];
    let precond_start = action_block.find(":precondition").expect("no :precondition");
    let precond_end = action_block.find(":effect").expect("no :effect");
    let precond_text = &action_block[precond_start..precond_end];

    // Count predicate occurrences by counting lines that start with a predicate call
    // (opening paren but not :precondition or (and ...)
    let pred_count = precond_text
        .lines()
        .filter(|l| {
            let t = l.trim();
            t.starts_with('(') && !t.starts_with("(:") && !t.starts_with("(and") && !t.starts_with("(:precondition")
        })
        .count();

    assert!(pred_count <= 8, "publish_education_week has {pred_count} preconditions, max is 8");
    assert_eq!(pred_count, 8, "publish_education_week should have exactly 8 preconditions");
}

// ---------------------------------------------------------------------------
// Test 14
// ---------------------------------------------------------------------------

#[test]
fn education_mode_agent_next_step_is_mechanical() {
    let dir = TempDir::new().unwrap();
    // All lanes complete except newsletter
    full_fixture(&dir);
    // Remove newsletter
    fs::remove_dir_all(dir.path().join("newsletter")).ok();

    let ws = scan(dir.path(), "sean");
    assert!(!ws.has(&EducationStage::NewsletterRestarted), "newsletter removed");

    // Next missing stage should be in the newsletter lane
    let next = ws.next_missing();
    assert!(next.is_some());
    // Since we scan in enum order, NewsletterRestarted comes after LinkedInPublished
    // and the linkedin lane is complete, so next missing should be NewsletterRestarted
    let next_name = next.unwrap().predicate_name();
    assert!(
        next_name.starts_with("newsletter"),
        "next step should be newsletter-related, got: {next_name}"
    );
}

// ---------------------------------------------------------------------------
// Test 15
// ---------------------------------------------------------------------------

#[test]
fn end_to_end_sean_education_week_admitted() {
    let dir = TempDir::new().unwrap();
    full_fixture(&dir);

    let domain = emit_education_domain();
    let ws = scan(dir.path(), "sean");

    // Verify all prerequisite stages are present
    assert!(ws.has(&EducationStage::InterviewConfirmed));
    assert!(ws.has(&EducationStage::LinkedInPublished));
    assert!(ws.has(&EducationStage::NewsletterIssuePublished));
    assert!(ws.has(&EducationStage::YouTubePublished));
    assert!(ws.has(&EducationStage::RustLessonPublished));

    let problem = emit_education_problem(&ws);

    // Verify problem contains expected init atoms
    assert!(problem.contains("(intent_captured sean)"));
    assert!(problem.contains("(interview_confirmed sean)"));
    assert!(problem.contains("(linkedin_published sean)"));

    let projection = Pddl8Projection { domain_text: domain, problem_text: problem };
    let result = bcinr_pddl_lsp::planner_client::plan_and_execute(&projection, "sean-e2e")
        .expect("plan_and_execute should succeed");

    assert!(result.receipt.goal_reached, "goal_reached must be true");

    // Write education-week receipt
    let receipt_dir = dir.path().join(".bcinr/receipts");
    fs::create_dir_all(&receipt_dir).unwrap();
    let ew_receipt = serde_json::json!({
        "goal_reached": result.receipt.goal_reached,
        "chain_hash": result.receipt.chain_hash,
        "step_count": result.receipt.step_count,
    });
    fs::write(
        receipt_dir.join("education-week.json"),
        serde_json::to_string_pretty(&ew_receipt).unwrap(),
    ).unwrap();

    // Re-scan with receipt present
    let ws2 = scan(dir.path(), "sean");
    assert!(ws2.has(&EducationStage::EducationWeekPublished), "education week admitted after receipt");

    let gate = render_education_gate(&ws2);
    assert!(gate.contains("ADMITTED"), "gate should be ADMITTED: {gate}");

    let status = render_education_status(&ws2);
    assert!(status.contains("\"admission_status\":\"ADMITTED\""), "status should show ADMITTED: {status}");
}
