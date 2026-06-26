//! M1: Project lifecycle state detection.
//!
//! Scans a workspace root and extracts lifecycle facts from files on disk.
//! Produces a `ProjectLifecycle` value that all other modules consume.

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LifecycleStage {
    IntentCaptured,
    PrdExists,
    PrdAdmitted,
    ArdExists,
    ArdAdmitted,
    WorkUnitsGenerated,
    ImplementationComplete,
    TestsPassed,
    DocsProjected,
    ReleaseReady,
    Published,
}

impl LifecycleStage {
    pub fn predicate_name(&self) -> &'static str {
        match self {
            Self::IntentCaptured => "intent_captured",
            Self::PrdExists => "prd_exists",
            Self::PrdAdmitted => "prd_admitted",
            Self::ArdExists => "ard_exists",
            Self::ArdAdmitted => "ard_admitted",
            Self::WorkUnitsGenerated => "work_units_generated",
            Self::ImplementationComplete => "implementation_complete",
            Self::TestsPassed => "tests_passed",
            Self::DocsProjected => "docs_projected",
            Self::ReleaseReady => "release_ready",
            Self::Published => "published",
        }
    }

    /// All stages in lifecycle order.
    pub fn all() -> &'static [LifecycleStage] {
        &[
            LifecycleStage::IntentCaptured,
            LifecycleStage::PrdExists,
            LifecycleStage::PrdAdmitted,
            LifecycleStage::ArdExists,
            LifecycleStage::ArdAdmitted,
            LifecycleStage::WorkUnitsGenerated,
            LifecycleStage::ImplementationComplete,
            LifecycleStage::TestsPassed,
            LifecycleStage::DocsProjected,
            LifecycleStage::ReleaseReady,
            LifecycleStage::Published,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleEvidence {
    pub stage: LifecycleStage,
    pub source_path: Option<PathBuf>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectLifecycle {
    pub project_name: String,
    pub root: PathBuf,
    pub true_stages: Vec<LifecycleStage>,
    pub evidence: Vec<LifecycleEvidence>,
    pub missing: Vec<LifecycleStage>,
}

impl ProjectLifecycle {
    pub fn has(&self, stage: &LifecycleStage) -> bool {
        self.true_stages.contains(stage)
    }

    /// The earliest missing stage — the lifecycle blocker.
    pub fn next_missing(&self) -> Option<&LifecycleStage> {
        LifecycleStage::all().iter().find(|s| !self.true_stages.contains(s))
    }

    /// PDDL8 init atoms: all true stages as `stage(project)` facts.
    pub fn pddl8_init_atoms(&self) -> Vec<(String, String)> {
        self.true_stages
            .iter()
            .map(|s| (s.predicate_name().to_string(), self.project_name.clone()))
            .collect()
    }
}

/// Scan the workspace root and derive lifecycle state.
pub fn scan(root: &Path) -> ProjectLifecycle {
    let project_name = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string();

    let mut true_stages = Vec::new();
    let mut evidence = Vec::new();

    // intent_captured: any CLAUDE.md, README, or intent*.md
    if exists_any(root, &["CLAUDE.md", "README.md", "intent.md", "INTENT.md"]) {
        true_stages.push(LifecycleStage::IntentCaptured);
        evidence.push(LifecycleEvidence {
            stage: LifecycleStage::IntentCaptured,
            source_path: find_first(root, &["CLAUDE.md", "README.md", "intent.md"]),
            note: "intent file present".into(),
        });
    }

    // prd_exists: docs/prd.md or PRD.md or docs/PRD.md
    let prd_path = find_first(root, &["docs/prd.md", "docs/PRD.md", "PRD.md", "prd.md"]);
    if prd_path.is_some() {
        true_stages.push(LifecycleStage::PrdExists);
        evidence.push(LifecycleEvidence {
            stage: LifecycleStage::PrdExists,
            source_path: prd_path.clone(),
            note: "PRD file present".into(),
        });

        // prd_admitted: PRD contains "## Status: ADMITTED" or "admitted: true"
        if let Some(ref p) = prd_path {
            if file_contains(p, "ADMITTED") {
                true_stages.push(LifecycleStage::PrdAdmitted);
                evidence.push(LifecycleEvidence {
                    stage: LifecycleStage::PrdAdmitted,
                    source_path: Some(p.clone()),
                    note: "PRD contains ADMITTED marker".into(),
                });
            }
        }
    }

    // ard_exists: docs/ard.md or ARD.md
    let ard_path = find_first(root, &["docs/ard.md", "docs/ARD.md", "ARD.md", "ard.md"]);
    if ard_path.is_some() {
        true_stages.push(LifecycleStage::ArdExists);
        evidence.push(LifecycleEvidence {
            stage: LifecycleStage::ArdExists,
            source_path: ard_path.clone(),
            note: "ARD file present".into(),
        });

        if let Some(ref p) = ard_path {
            if file_contains(p, "ADMITTED") {
                true_stages.push(LifecycleStage::ArdAdmitted);
                evidence.push(LifecycleEvidence {
                    stage: LifecycleStage::ArdAdmitted,
                    source_path: Some(p.clone()),
                    note: "ARD contains ADMITTED marker".into(),
                });
            }
        }
    }

    // work_units_generated: docs/work-units.md or .bcinr/work-units.json
    if find_first(root, &["docs/work-units.md", ".bcinr/work-units.json"]).is_some() {
        true_stages.push(LifecycleStage::WorkUnitsGenerated);
        evidence.push(LifecycleEvidence {
            stage: LifecycleStage::WorkUnitsGenerated,
            source_path: find_first(root, &["docs/work-units.md", ".bcinr/work-units.json"]),
            note: "work units file present".into(),
        });
    }

    // implementation_complete: src/ or crates/ with at least one .rs file
    if has_source_files(root) {
        true_stages.push(LifecycleStage::ImplementationComplete);
        evidence.push(LifecycleEvidence {
            stage: LifecycleStage::ImplementationComplete,
            source_path: None,
            note: "source files present under src/ or crates/".into(),
        });
    }

    // tests_passed: .bcinr/test-report.json or test_results.json with passed=true
    if let Some(p) = find_first(root, &[".bcinr/test-report.json", "test_results.json"]) {
        if file_contains(&p, "\"passed\": true") || file_contains(&p, "\"status\": \"passed\"") {
            true_stages.push(LifecycleStage::TestsPassed);
            evidence.push(LifecycleEvidence {
                stage: LifecycleStage::TestsPassed,
                source_path: Some(p),
                note: "test report shows passed".into(),
            });
        }
    }

    // docs_projected: docs/ has more than just prd/ard
    if has_projected_docs(root) {
        true_stages.push(LifecycleStage::DocsProjected);
        evidence.push(LifecycleEvidence {
            stage: LifecycleStage::DocsProjected,
            source_path: None,
            note: "projected docs present".into(),
        });
    }

    // release_ready: .bcinr/release.json or Cargo.toml with publish=true
    if find_first(root, &[".bcinr/release.json", "docs/publish.md"]).is_some() {
        true_stages.push(LifecycleStage::ReleaseReady);
        evidence.push(LifecycleEvidence {
            stage: LifecycleStage::ReleaseReady,
            source_path: find_first(root, &[".bcinr/release.json"]),
            note: "release artifact present".into(),
        });
    }

    // published: .bcinr/receipts/latest.json with goal_reached=true
    if let Some(p) = find_first(root, &[".bcinr/receipts/latest.json", "receipts/latest.json"]) {
        if file_contains(&p, "\"goal_reached\": true") {
            true_stages.push(LifecycleStage::Published);
            evidence.push(LifecycleEvidence {
                stage: LifecycleStage::Published,
                source_path: Some(p),
                note: "receipt with goal_reached=true present".into(),
            });
        }
    }

    let missing: Vec<LifecycleStage> = LifecycleStage::all()
        .iter()
        .filter(|s| !true_stages.contains(s))
        .cloned()
        .collect();

    ProjectLifecycle { project_name, root: root.to_path_buf(), true_stages, evidence, missing }
}

fn exists_any(root: &Path, names: &[&str]) -> bool {
    names.iter().any(|n| root.join(n).exists())
}

fn find_first(root: &Path, names: &[&str]) -> Option<PathBuf> {
    names.iter().find_map(|n| {
        let p = root.join(n);
        if p.exists() { Some(p) } else { None }
    })
}

fn file_contains(path: &Path, needle: &str) -> bool {
    std::fs::read_to_string(path)
        .map(|s| s.contains(needle))
        .unwrap_or(false)
}

fn has_source_files(root: &Path) -> bool {
    for dir in &["src", "crates", "lib"] {
        let p = root.join(dir);
        if p.is_dir() {
            if walkdir::WalkDir::new(&p)
                .max_depth(4)
                .into_iter()
                .filter_map(|e| e.ok())
                .any(|e| e.path().extension().and_then(|x| x.to_str()) == Some("rs"))
            {
                return true;
            }
        }
    }
    false
}

fn has_projected_docs(root: &Path) -> bool {
    let docs = root.join("docs");
    if !docs.is_dir() { return false; }
    let non_prd_ard: Vec<_> = walkdir::WalkDir::new(&docs)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_lowercase();
            e.path().extension().and_then(|x| x.to_str()) == Some("md")
                && name != "prd.md"
                && name != "ard.md"
        })
        .collect();
    non_prd_ard.len() >= 1
}
