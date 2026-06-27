//! Education mode — Sean's career/content/teaching lifecycle as a PDDL8 domain.
//!
//! Covers: interviews, LinkedIn, newsletter, YouTube, Rust lessons.
//! Separate from the main project lifecycle domain.

use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// ---------------------------------------------------------------------------
// Stage enum
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EducationStage {
    CareerPipelineExists,
    InterviewRequestReceived,
    InterviewSlotSelected,
    InterviewConfirmed,
    InterviewPrepComplete,
    LinkedInTopicSelected,
    LinkedInDraftExists,
    LinkedInReviewed,
    LinkedInPublished,
    NewsletterRestarted,
    NewsletterIssueDrafted,
    NewsletterIssueReviewed,
    NewsletterIssuePublished,
    YouTubeTopicSelected,
    YouTubeOutlineExists,
    YouTubeScriptExists,
    YouTubeRecorded,
    YouTubePublished,
    RustLessonSelected,
    RustExampleExists,
    RustExampleTestsPassed,
    RustLessonPublished,
    EducationWeekReceipTED,
    EducationWeekPublished,
}

impl EducationStage {
    pub fn predicate_name(&self) -> &'static str {
        match self {
            Self::CareerPipelineExists => "career_pipeline_exists",
            Self::InterviewRequestReceived => "interview_request_received",
            Self::InterviewSlotSelected => "interview_slot_selected",
            Self::InterviewConfirmed => "interview_confirmed",
            Self::InterviewPrepComplete => "interview_prep_complete",
            Self::LinkedInTopicSelected => "linkedin_topic_selected",
            Self::LinkedInDraftExists => "linkedin_draft_exists",
            Self::LinkedInReviewed => "linkedin_reviewed",
            Self::LinkedInPublished => "linkedin_published",
            Self::NewsletterRestarted => "newsletter_restarted",
            Self::NewsletterIssueDrafted => "newsletter_issue_drafted",
            Self::NewsletterIssueReviewed => "newsletter_issue_reviewed",
            Self::NewsletterIssuePublished => "newsletter_issue_published",
            Self::YouTubeTopicSelected => "youtube_topic_selected",
            Self::YouTubeOutlineExists => "youtube_outline_exists",
            Self::YouTubeScriptExists => "youtube_script_exists",
            Self::YouTubeRecorded => "youtube_recorded",
            Self::YouTubePublished => "youtube_published",
            Self::RustLessonSelected => "rust_lesson_selected",
            Self::RustExampleExists => "rust_example_exists",
            Self::RustExampleTestsPassed => "rust_example_tests_passed",
            Self::RustLessonPublished => "rust_lesson_published",
            Self::EducationWeekReceipTED => "education_week_receipted",
            Self::EducationWeekPublished => "education_week_published",
        }
    }

    pub fn all() -> Vec<EducationStage> {
        vec![
            Self::CareerPipelineExists,
            Self::InterviewRequestReceived,
            Self::InterviewSlotSelected,
            Self::InterviewConfirmed,
            Self::InterviewPrepComplete,
            Self::LinkedInTopicSelected,
            Self::LinkedInDraftExists,
            Self::LinkedInReviewed,
            Self::LinkedInPublished,
            Self::NewsletterRestarted,
            Self::NewsletterIssueDrafted,
            Self::NewsletterIssueReviewed,
            Self::NewsletterIssuePublished,
            Self::YouTubeTopicSelected,
            Self::YouTubeOutlineExists,
            Self::YouTubeScriptExists,
            Self::YouTubeRecorded,
            Self::YouTubePublished,
            Self::RustLessonSelected,
            Self::RustExampleExists,
            Self::RustExampleTestsPassed,
            Self::RustLessonPublished,
            Self::EducationWeekReceipTED,
            Self::EducationWeekPublished,
        ]
    }
}

// ---------------------------------------------------------------------------
// Evidence
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct EducationEvidence {
    pub stage: String,
    pub source: String,
    pub found: bool,
}

// ---------------------------------------------------------------------------
// Workspace
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct EducationWorkspace {
    pub subject: String,
    pub root: PathBuf,
    pub true_stages: Vec<EducationStage>,
    pub missing: Vec<EducationStage>,
    pub evidence: Vec<EducationEvidence>,
}

impl EducationWorkspace {
    pub fn has(&self, stage: &EducationStage) -> bool {
        self.true_stages.contains(stage)
    }

    pub fn next_missing(&self) -> Option<&EducationStage> {
        self.missing.first()
    }
}

// ---------------------------------------------------------------------------
// Scan helpers
// ---------------------------------------------------------------------------

fn read_file_opt(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
}

fn file_exists(path: &Path) -> bool {
    path.exists()
}

fn dir_has_md(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }
    fs::read_dir(path)
        .map(|rd| rd.flatten().any(|e| e.path().extension().map_or(false, |x| x == "md")))
        .unwrap_or(false)
}

fn dir_has_subdirs(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }
    fs::read_dir(path)
        .map(|rd| rd.flatten().any(|e| e.path().is_dir()))
        .unwrap_or(false)
}

fn any_file_in_dir_contains(path: &Path, needle: &str) -> bool {
    if !path.is_dir() {
        return false;
    }
    WalkDir::new(path)
        .into_iter()
        .flatten()
        .filter(|e| e.path().extension().map_or(false, |x| x == "md"))
        .any(|e| read_file_opt(e.path()).map_or(false, |c| c.contains(needle)))
}

fn any_file_content_longer_than(path: &Path, min_len: usize) -> bool {
    if !path.is_dir() {
        return false;
    }
    WalkDir::new(path)
        .into_iter()
        .flatten()
        .filter(|e| e.path().extension().map_or(false, |x| x == "md"))
        .any(|e| read_file_opt(e.path()).map_or(false, |c| c.len() > min_len))
}

fn walk_find_file(root: &Path, filename: &str) -> bool {
    WalkDir::new(root)
        .into_iter()
        .flatten()
        .any(|e| e.file_name().to_string_lossy() == filename)
}

fn walk_find_json_with(root: &Path, key: &str, value: &str) -> bool {
    WalkDir::new(root)
        .into_iter()
        .flatten()
        .filter(|e| e.path().extension().map_or(false, |x| x == "json"))
        .any(|e| {
            read_file_opt(e.path())
                .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
                .map_or(false, |v| {
                    v.get(key).and_then(|x| x.as_str()).map_or(false, |s| s == value)
                })
        })
}

// ---------------------------------------------------------------------------
// scan()
// ---------------------------------------------------------------------------

pub fn scan(root: &Path, subject: &str) -> EducationWorkspace {
    let mut true_stages = Vec::new();
    let mut evidence = Vec::new();

    let mut mark = |stage: EducationStage, source: &str, found: bool| {
        evidence.push(EducationEvidence {
            stage: stage.predicate_name().to_string(),
            source: source.to_string(),
            found,
        });
        if found {
            true_stages.push(stage);
        }
    };

    // --- career ---
    let interviews_path = root.join("career/interviews.json");
    let interviews_exists = file_exists(&interviews_path);
    mark(EducationStage::CareerPipelineExists, "career/interviews.json", interviews_exists);

    let interviews_json: Option<serde_json::Value> = interviews_path
        .exists()
        .then(|| fs::read_to_string(&interviews_path).ok())
        .flatten()
        .and_then(|c| serde_json::from_str(&c).ok());

    let request_received = interviews_json.as_ref().map_or(false, |v| {
        v.get("status")
            .and_then(|s| s.as_str())
            .map_or(false, |s| s == "requested" || s == "received")
    });
    mark(EducationStage::InterviewRequestReceived, "career/interviews.json[status]", request_received);

    let slot_selected = interviews_json.as_ref().map_or(false, |v| {
        v.get("slot_selected").and_then(|b| b.as_bool()).unwrap_or(false)
    });
    mark(EducationStage::InterviewSlotSelected, "career/interviews.json[slot_selected]", slot_selected);

    let confirmed = interviews_json.as_ref().map_or(false, |v| {
        v.get("confirmed").and_then(|b| b.as_bool()).unwrap_or(false)
    });
    mark(EducationStage::InterviewConfirmed, "career/interviews.json[confirmed]", confirmed);

    let prep_path = root.join("career/interview-prep.md");
    let prep_complete = prep_path.exists()
        && read_file_opt(&prep_path).map_or(false, |c| c.contains("ADMITTED"));
    mark(EducationStage::InterviewPrepComplete, "career/interview-prep.md[ADMITTED]", prep_complete);

    // --- linkedin ---
    let posts_dir = root.join("linkedin/posts");
    let linkedin_topic = dir_has_md(&posts_dir);
    mark(EducationStage::LinkedInTopicSelected, "linkedin/posts/*.md", linkedin_topic);

    let linkedin_draft = any_file_content_longer_than(&posts_dir, 50);
    mark(EducationStage::LinkedInDraftExists, "linkedin/posts/*.md[len>50]", linkedin_draft);

    let linkedin_reviewed = any_file_in_dir_contains(&posts_dir, "REVIEWED");
    mark(EducationStage::LinkedInReviewed, "linkedin/posts/*.md[REVIEWED]", linkedin_reviewed);

    let li_post_published = any_file_in_dir_contains(&posts_dir, "STATUS: PUBLISHED");
    let li_receipt = file_exists(&root.join(".bcinr/receipts/linkedin-post-001.json"));
    mark(EducationStage::LinkedInPublished, ".bcinr/receipts/linkedin-post-001.json", li_post_published && li_receipt);

    // --- newsletter ---
    let newsletter_dir = root.join("newsletter");
    let newsletter_restarted = newsletter_dir.is_dir();
    mark(EducationStage::NewsletterRestarted, "newsletter/", newsletter_restarted);

    let issues_dir = root.join("newsletter/issues");
    let newsletter_drafted = dir_has_md(&issues_dir);
    mark(EducationStage::NewsletterIssueDrafted, "newsletter/issues/*.md", newsletter_drafted);

    let newsletter_reviewed = any_file_in_dir_contains(&issues_dir, "REVIEWED");
    mark(EducationStage::NewsletterIssueReviewed, "newsletter/issues/*.md[REVIEWED]", newsletter_reviewed);

    let nl_published = any_file_in_dir_contains(&issues_dir, "STATUS: PUBLISHED");
    let nl_receipt = file_exists(&root.join(".bcinr/receipts/newsletter-001.json"));
    mark(EducationStage::NewsletterIssuePublished, ".bcinr/receipts/newsletter-001.json", nl_published && nl_receipt);

    // --- youtube ---
    let yt_videos_dir = root.join("youtube/videos");
    let yt_topic = dir_has_subdirs(&yt_videos_dir);
    mark(EducationStage::YouTubeTopicSelected, "youtube/videos/*/", yt_topic);

    let yt_root = root.join("youtube");
    let yt_outline = walk_find_file(&yt_root, "outline.md");
    mark(EducationStage::YouTubeOutlineExists, "youtube/**/outline.md", yt_outline);

    let yt_script = walk_find_file(&yt_root, "script.md");
    mark(EducationStage::YouTubeScriptExists, "youtube/**/script.md", yt_script);

    let yt_recorded = walk_find_json_with(&yt_root, "status", "recorded")
        || walk_find_json_with(&yt_root, "status", "published");
    mark(EducationStage::YouTubeRecorded, "youtube/**/recording.json[status=recorded|published]", yt_recorded);

    let yt_pub_json = walk_find_json_with(&yt_root, "status", "published");
    let yt_receipt = file_exists(&root.join(".bcinr/receipts/youtube-001.json"));
    mark(EducationStage::YouTubePublished, ".bcinr/receipts/youtube-001.json", yt_pub_json && yt_receipt);

    // --- rust lessons ---
    let rust_lessons_dir = root.join("lessons/rust");
    let rust_lesson_selected = dir_has_md(&rust_lessons_dir);
    mark(EducationStage::RustLessonSelected, "lessons/rust/*.md", rust_lesson_selected);

    let rust_example = file_exists(&root.join("lessons/rust/examples/src/lib.rs"));
    mark(EducationStage::RustExampleExists, "lessons/rust/examples/src/lib.rs", rust_example);

    let test_report_path = root.join(".bcinr/test-report.json");
    let rust_tests_passed = test_report_path.exists() && {
        fs::read_to_string(&test_report_path)
            .ok()
            .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
            .map_or(false, |v| {
                let by_bool = v.get("passed").and_then(|b| b.as_bool()).unwrap_or(false);
                let by_str = v.get("status").and_then(|s| s.as_str()).map_or(false, |s| s == "passed");
                by_bool || by_str
            })
    };
    mark(EducationStage::RustExampleTestsPassed, ".bcinr/test-report.json[passed=true]", rust_tests_passed);

    let rust_lesson_published = any_file_in_dir_contains(&rust_lessons_dir, "STATUS: PUBLISHED");
    mark(EducationStage::RustLessonPublished, "lessons/rust/*.md[STATUS: PUBLISHED]", rust_lesson_published);

    // --- education week ---
    let ew_receipt_path = root.join(".bcinr/receipts/education-week.json");
    let ew_receipted = ew_receipt_path.exists() && {
        fs::read_to_string(&ew_receipt_path)
            .ok()
            .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
            .map_or(false, |v| {
                v.get("goal_reached").and_then(|b| b.as_bool()).unwrap_or(false)
            })
    };
    mark(EducationStage::EducationWeekReceipTED, ".bcinr/receipts/education-week.json[goal_reached=true]", ew_receipted);

    // published = same as receipted for now
    mark(EducationStage::EducationWeekPublished, ".bcinr/receipts/education-week.json[goal_reached=true]", ew_receipted);

    let all = EducationStage::all();
    let missing: Vec<EducationStage> = all.into_iter().filter(|s| !true_stages.contains(s)).collect();

    EducationWorkspace {
        subject: subject.to_string(),
        root: root.to_path_buf(),
        true_stages,
        missing,
        evidence,
    }
}

// ---------------------------------------------------------------------------
// PDDL8 domain
// ---------------------------------------------------------------------------

pub fn emit_education_domain() -> String {
    r#"(define (domain education-mode)
  (:requirements :strips)
  (:predicates
    (intent_captured ?s)
    (ocel_present ?s)
    (career_pipeline_exists ?s)
    (interview_request_received ?s)
    (interview_slot_selected ?s)
    (interview_confirmed ?s)
    (interview_prep_complete ?s)
    (linkedin_topic_selected ?s)
    (linkedin_draft_exists ?s)
    (linkedin_reviewed ?s)
    (linkedin_published ?s)
    (newsletter_restarted ?s)
    (newsletter_issue_drafted ?s)
    (newsletter_issue_reviewed ?s)
    (newsletter_issue_published ?s)
    (youtube_topic_selected ?s)
    (youtube_outline_exists ?s)
    (youtube_script_exists ?s)
    (youtube_recorded ?s)
    (youtube_published ?s)
    (rust_lesson_selected ?s)
    (rust_example_exists ?s)
    (rust_example_tests_passed ?s)
    (rust_lesson_published ?s)
    (education_week_receipted ?s)
    (education_week_published ?s)
  )

  (:action capture_career_pipeline
    :parameters (?s)
    :precondition (intent_captured ?s)
    :effect (career_pipeline_exists ?s)
  )

  (:action receive_interview_request
    :parameters (?s)
    :precondition (career_pipeline_exists ?s)
    :effect (interview_request_received ?s)
  )

  (:action select_interview_slot
    :parameters (?s)
    :precondition (interview_request_received ?s)
    :effect (interview_slot_selected ?s)
  )

  (:action confirm_interview
    :parameters (?s)
    :precondition (and (interview_request_received ?s) (interview_slot_selected ?s))
    :effect (interview_confirmed ?s)
  )

  (:action complete_interview_prep
    :parameters (?s)
    :precondition (interview_confirmed ?s)
    :effect (interview_prep_complete ?s)
  )

  (:action select_linkedin_topic
    :parameters (?s)
    :precondition (intent_captured ?s)
    :effect (linkedin_topic_selected ?s)
  )

  (:action draft_linkedin_post
    :parameters (?s)
    :precondition (linkedin_topic_selected ?s)
    :effect (linkedin_draft_exists ?s)
  )

  (:action review_linkedin_post
    :parameters (?s)
    :precondition (linkedin_draft_exists ?s)
    :effect (linkedin_reviewed ?s)
  )

  (:action publish_linkedin_post
    :parameters (?s)
    :precondition (and (linkedin_draft_exists ?s) (linkedin_reviewed ?s))
    :effect (linkedin_published ?s)
  )

  (:action restart_newsletter
    :parameters (?s)
    :precondition (intent_captured ?s)
    :effect (newsletter_restarted ?s)
  )

  (:action draft_newsletter_issue
    :parameters (?s)
    :precondition (newsletter_restarted ?s)
    :effect (newsletter_issue_drafted ?s)
  )

  (:action review_newsletter_issue
    :parameters (?s)
    :precondition (newsletter_issue_drafted ?s)
    :effect (newsletter_issue_reviewed ?s)
  )

  (:action publish_newsletter_issue
    :parameters (?s)
    :precondition (and (newsletter_restarted ?s) (newsletter_issue_drafted ?s) (newsletter_issue_reviewed ?s))
    :effect (newsletter_issue_published ?s)
  )

  (:action select_youtube_topic
    :parameters (?s)
    :precondition (intent_captured ?s)
    :effect (youtube_topic_selected ?s)
  )

  (:action write_youtube_outline
    :parameters (?s)
    :precondition (youtube_topic_selected ?s)
    :effect (youtube_outline_exists ?s)
  )

  (:action write_youtube_script
    :parameters (?s)
    :precondition (youtube_outline_exists ?s)
    :effect (youtube_script_exists ?s)
  )

  (:action record_youtube_video
    :parameters (?s)
    :precondition (youtube_script_exists ?s)
    :effect (youtube_recorded ?s)
  )

  (:action publish_youtube_video
    :parameters (?s)
    :precondition (and (youtube_outline_exists ?s) (youtube_script_exists ?s) (youtube_recorded ?s))
    :effect (youtube_published ?s)
  )

  (:action select_rust_lesson
    :parameters (?s)
    :precondition (intent_captured ?s)
    :effect (rust_lesson_selected ?s)
  )

  (:action create_rust_example
    :parameters (?s)
    :precondition (rust_lesson_selected ?s)
    :effect (rust_example_exists ?s)
  )

  (:action run_rust_example_tests
    :parameters (?s)
    :precondition (rust_example_exists ?s)
    :effect (rust_example_tests_passed ?s)
  )

  (:action publish_rust_lesson
    :parameters (?s)
    :precondition (and (rust_lesson_selected ?s) (rust_example_exists ?s) (rust_example_tests_passed ?s))
    :effect (rust_lesson_published ?s)
  )

  (:action emit_education_receipt
    :parameters (?s)
    :precondition (and
      (interview_confirmed ?s)
      (linkedin_published ?s)
      (newsletter_issue_published ?s)
      (youtube_published ?s)
      (rust_lesson_published ?s)
    )
    :effect (education_week_receipted ?s)
  )

  (:action publish_education_week
    :parameters (?s)
    :precondition (and
      (interview_confirmed ?s)
      (interview_prep_complete ?s)
      (linkedin_published ?s)
      (newsletter_issue_published ?s)
      (youtube_published ?s)
      (rust_lesson_published ?s)
      (education_week_receipted ?s)
      (ocel_present ?s)
    )
    :effect (education_week_published ?s)
  )
)
"#.to_string()
}

// ---------------------------------------------------------------------------
// PDDL8 problem
// ---------------------------------------------------------------------------

pub fn emit_education_problem(workspace: &EducationWorkspace) -> String {
    let subj = sanitize_subject(&workspace.subject);

    let mut inits = vec![format!("(intent_captured {})", subj)];

    // ocel_present if .bcinr/ocel/latest.json exists
    if file_exists(&workspace.root.join(".bcinr/ocel/latest.json")) {
        inits.push(format!("(ocel_present {})", subj));
    }

    for stage in &workspace.true_stages {
        inits.push(format!("({} {})", stage.predicate_name(), subj));
    }

    let init_str = inits.iter().map(|s| format!("    {}", s)).collect::<Vec<_>>().join("\n");

    format!(
        r#"(define (problem education-{subj})
  (:domain education-mode)
  (:objects {subj})
  (:init
{init_str}
  )
  (:goal (education_week_published {subj}))
)
"#
    )
}

fn sanitize_subject(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

// ---------------------------------------------------------------------------
// Diagnostics
// ---------------------------------------------------------------------------

pub fn education_diagnostics(workspace: &EducationWorkspace) -> Vec<(String, String)> {
    let mut diags = Vec::new();

    for stage in &workspace.missing {
        let (code, msg) = match stage {
            EducationStage::CareerPipelineExists => (
                "INTERVIEW_REQUEST_MISSING",
                "No career pipeline. Create career/interviews.json.",
            ),
            EducationStage::InterviewRequestReceived => (
                "INTERVIEW_REQUEST_MISSING",
                "career/interviews.json exists but status is not 'requested' or 'received'.",
            ),
            EducationStage::InterviewSlotSelected => (
                "INTERVIEW_SLOT_MISSING",
                "Interview request received but no slot_selected=true in career/interviews.json.",
            ),
            EducationStage::InterviewConfirmed => (
                "INTERVIEW_NOT_CONFIRMED",
                "Slot selected but confirmed=true missing from career/interviews.json.",
            ),
            EducationStage::InterviewPrepComplete => (
                "INTERVIEW_PREP_MISSING",
                "career/interview-prep.md missing or does not contain ADMITTED.",
            ),
            EducationStage::LinkedInTopicSelected => (
                "LINKEDIN_TOPIC_MISSING",
                "No .md files found in linkedin/posts/.",
            ),
            EducationStage::LinkedInDraftExists => (
                "LINKEDIN_DRAFT_MISSING",
                "No LinkedIn post with content length > 50 found in linkedin/posts/.",
            ),
            EducationStage::LinkedInReviewed => (
                "LINKEDIN_NOT_REVIEWED",
                "No LinkedIn post contains REVIEWED marker.",
            ),
            EducationStage::LinkedInPublished => {
                // Check if post file has STATUS: PUBLISHED but no receipt
                let posts_dir = workspace.root.join("linkedin/posts");
                let has_published = any_file_in_dir_contains(&posts_dir, "STATUS: PUBLISHED");
                let has_receipt = file_exists(&workspace.root.join(".bcinr/receipts/linkedin-post-001.json"));
                if has_published && !has_receipt {
                    ("LINKEDIN_RECEIPT_MISSING", "LinkedIn post marked STATUS: PUBLISHED but .bcinr/receipts/linkedin-post-001.json missing.")
                } else {
                    ("LINKEDIN_NOT_PUBLISHED", "LinkedIn post not published or receipt missing.")
                }
            }
            EducationStage::NewsletterRestarted => (
                "NEWSLETTER_NOT_RESTARTED",
                "newsletter/ directory does not exist.",
            ),
            EducationStage::NewsletterIssueDrafted => (
                "NEWSLETTER_DRAFT_MISSING",
                "No .md files found in newsletter/issues/.",
            ),
            EducationStage::NewsletterIssueReviewed => (
                "NEWSLETTER_NOT_REVIEWED",
                "No newsletter issue contains REVIEWED marker.",
            ),
            EducationStage::NewsletterIssuePublished => (
                "NEWSLETTER_NOT_PUBLISHED",
                "Newsletter issue not published or receipt missing.",
            ),
            EducationStage::YouTubeTopicSelected => (
                "YOUTUBE_TOPIC_MISSING",
                "No subdirectories found in youtube/videos/.",
            ),
            EducationStage::YouTubeOutlineExists => (
                "YOUTUBE_OUTLINE_MISSING",
                "No outline.md found under youtube/.",
            ),
            EducationStage::YouTubeScriptExists => (
                "YOUTUBE_SCRIPT_MISSING",
                "No script.md found under youtube/.",
            ),
            EducationStage::YouTubeRecorded => (
                "YOUTUBE_RECORDING_MISSING",
                "No recording.json with status=recorded/published found under youtube/.",
            ),
            EducationStage::YouTubePublished => (
                "YOUTUBE_NOT_PUBLISHED",
                "YouTube video not published or receipt missing.",
            ),
            EducationStage::RustLessonSelected => (
                "RUST_LESSON_MISSING",
                "No .md files found in lessons/rust/.",
            ),
            EducationStage::RustExampleExists => (
                "RUST_EXAMPLE_MISSING",
                "lessons/rust/examples/src/lib.rs does not exist.",
            ),
            EducationStage::RustExampleTestsPassed => (
                "RUST_EXAMPLE_TESTS_NOT_PASSED",
                ".bcinr/test-report.json missing or passed/status not true/passed.",
            ),
            EducationStage::RustLessonPublished => (
                "RUST_LESSON_NOT_PUBLISHED",
                "No lesson .md contains STATUS: PUBLISHED.",
            ),
            EducationStage::EducationWeekReceipTED => (
                "EDUCATION_WEEK_BLOCKED",
                ".bcinr/receipts/education-week.json missing or goal_reached not true.",
            ),
            EducationStage::EducationWeekPublished => (
                "EDUCATION_WEEK_CANDIDATE_NOT_ADMITTED",
                "Education week not yet admitted. Run emit_education_receipt action.",
            ),
        };
        diags.push((code.to_string(), msg.to_string()));
    }

    diags
}

/// Check newsletter issues for Need9 boundary (> 8 sections per issue).
pub fn check_newsletter_need9(root: &Path) -> Option<(String, String)> {
    let issues_dir = root.join("newsletter/issues");
    if !issues_dir.is_dir() {
        return None;
    }

    for entry in WalkDir::new(&issues_dir).into_iter().flatten() {
        let path = entry.path();
        if path.extension().map_or(false, |x| x == "md") {
            if let Some(content) = read_file_opt(path) {
                let section_count = content.lines().filter(|l| l.starts_with("## ")).count();
                if section_count > 8 {
                    return Some((
                        "NEWSLETTER_NEED9_SPLIT".to_string(),
                        format!(
                            "{} has {} sections (> 8). Split into two issues to stay within Need9 boundary.",
                            path.display(),
                            section_count
                        ),
                    ));
                }
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Virtual doc renders
// ---------------------------------------------------------------------------

pub fn render_education_status(workspace: &EducationWorkspace) -> String {
    let next = workspace
        .next_missing()
        .map(|s| s.predicate_name().to_string())
        .unwrap_or_else(|| "none".to_string());

    let admitted = workspace.has(&EducationStage::EducationWeekPublished);
    let admission_status = if admitted { "ADMITTED" } else { "NONE" };

    serde_json::json!({
        "mode": "education",
        "subject": workspace.subject,
        "goal": format!("education_week_published({})", workspace.subject),
        "true_stage_count": workspace.true_stages.len(),
        "missing_count": workspace.missing.len(),
        "next_lawful_step": next,
        "candidate_status": "CANDIDATE",
        "admission_status": admission_status,
    })
    .to_string()
}

pub fn render_education_week_plan(workspace: &EducationWorkspace, plan_steps: &[String]) -> String {
    serde_json::json!({
        "mode": "education",
        "subject": workspace.subject,
        "plan_steps": plan_steps,
        "stage_count": workspace.true_stages.len(),
        "missing_count": workspace.missing.len(),
    })
    .to_string()
}

pub fn render_education_lane(workspace: &EducationWorkspace, lane: &str) -> String {
    let lane_stages: Vec<_> = EducationStage::all()
        .into_iter()
        .filter(|s| {
            let name = s.predicate_name();
            match lane {
                "career" => name.starts_with("career") || name.starts_with("interview"),
                "linkedin" => name.starts_with("linkedin"),
                "newsletter" => name.starts_with("newsletter"),
                "youtube" => name.starts_with("youtube"),
                "rust" => name.starts_with("rust"),
                "education" => name.starts_with("education"),
                _ => false,
            }
        })
        .collect();

    let done: Vec<_> = lane_stages
        .iter()
        .filter(|s| workspace.has(s))
        .map(|s| s.predicate_name())
        .collect();
    let missing: Vec<_> = lane_stages
        .iter()
        .filter(|s| !workspace.has(s))
        .map(|s| s.predicate_name())
        .collect();

    serde_json::json!({
        "lane": lane,
        "subject": workspace.subject,
        "done": done,
        "missing": missing,
    })
    .to_string()
}

pub fn render_education_gate(workspace: &EducationWorkspace) -> String {
    if workspace.has(&EducationStage::EducationWeekPublished) {
        serde_json::json!({
            "gate": "ADMITTED",
            "subject": workspace.subject,
        })
        .to_string()
    } else {
        let lanes = ["career", "linkedin", "newsletter", "youtube", "rust", "education"];
        let mut lane_summaries = serde_json::Map::new();
        for lane in &lanes {
            let all_for_lane: Vec<_> = EducationStage::all()
                .into_iter()
                .filter(|s| {
                    let name = s.predicate_name();
                    match *lane {
                        "career" => name.starts_with("career") || name.starts_with("interview"),
                        "linkedin" => name.starts_with("linkedin"),
                        "newsletter" => name.starts_with("newsletter"),
                        "youtube" => name.starts_with("youtube"),
                        "rust" => name.starts_with("rust"),
                        "education" => name.starts_with("education"),
                        _ => false,
                    }
                })
                .collect();
            let missing_count = all_for_lane.iter().filter(|s| !workspace.has(s)).count();
            lane_summaries.insert(
                lane.to_string(),
                serde_json::json!({ "missing": missing_count }),
            );
        }
        serde_json::json!({
            "gate": "BLOCKED",
            "subject": workspace.subject,
            "lanes": lane_summaries,
        })
        .to_string()
    }
}
