//! React file-tree interview test harness for CMCA Chicago technical assessment.
//!
//! Simulates a real-time candidate assessment of a file-tree component:
//! - Candidate starts with flat file list, must recognize need for nested structure
//! - Detects algorithmic choices (repeated array search vs indexing)
//! - Assesses React rendering strategy and performance awareness
//! - Validates add/remove operations and conflict resolution

use std::collections::HashMap;

/// Interview observation types that capture candidate behavior and competency signals.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InterviewObservation {
    /// Candidate introduced flat file array initially
    IntroduceFlatFiles,
    /// Candidate recognized need for nested tree structure
    RequireNestedStructure,
    /// Candidate iterates children with repeated `.find()` search (O(n) per lookup)
    CandidateUsesRepeatedArraySearch,
    /// Candidate uses HashMap or index for O(1) child lookup
    CandidateUsesIndexedChildAccess,
    /// Candidate asks about React rendering: reconciliation, keys, memo
    AskReactRendering,
    /// Candidate implements add file operation correctly
    AskAddFile,
    /// Candidate handles file/folder name conflict
    AskFileFolderConflict,
    /// Candidate recognizes performance issue in expand/collapse
    RecognizePerformanceIssue,
    /// Candidate proposes virtua scrolling or lazy loading
    ProposeVirtualization,
    /// Candidate forgets to handle deleted parent (orphaned children)
    ForgetDeletedParentEdgeCase,
    /// Candidate writes a test case
    WritesTestCase,
}

/// Observation dispatcher: maps candidate behavior to InterviewObservation.
///
/// Returns a set of observations captured during the interview.
pub fn observe(candidate_code: &str, interviewer_prompt: &str) -> Vec<InterviewObservation> {
    let mut observations = Vec::new();

    // Detect flat file structure
    if candidate_code.contains("Vec<File>") && !candidate_code.contains("Tree") {
        observations.push(InterviewObservation::IntroduceFlatFiles);
    }

    // Detect nested structure
    if candidate_code.contains("children:") && candidate_code.contains("Vec<") {
        observations.push(InterviewObservation::RequireNestedStructure);
    }

    // Detect repeated array search (O(n) child lookup pattern)
    if candidate_code.contains(".find(|") && candidate_code.contains("children") {
        observations.push(InterviewObservation::CandidateUsesRepeatedArraySearch);
    }

    // Detect indexed access (HashMap, BTreeMap, or direct Vec index)
    if candidate_code.contains("HashMap") || candidate_code.contains("id_map") {
        observations.push(InterviewObservation::CandidateUsesIndexedChildAccess);
    }

    // Detect React-related questions/patterns
    if candidate_code.contains("key=") || candidate_code.contains("React.memo") {
        observations.push(InterviewObservation::AskReactRendering);
    }

    // Detect add file operation
    if candidate_code.contains("add_file") || candidate_code.contains("insert_child") {
        observations.push(InterviewObservation::AskAddFile);
    }

    // Detect conflict handling
    if interviewer_prompt.contains("conflict") || candidate_code.contains("exists") {
        observations.push(InterviewObservation::AskFileFolderConflict);
    }

    // Detect performance recognition
    if interviewer_prompt.contains("slow") || candidate_code.contains("O(n)") {
        observations.push(InterviewObservation::RecognizePerformanceIssue);
    }

    // Detect virtualization proposal
    if candidate_code.contains("virtual") || candidate_code.contains("windowed") {
        observations.push(InterviewObservation::ProposeVirtualization);
    }

    // Detect test writing
    if candidate_code.contains("#[test]") || candidate_code.contains("assert_eq!") {
        observations.push(InterviewObservation::WritesTestCase);
    }

    observations
}

/// Candidate code snippets for testing and assessment.
pub struct CandidateRegistry {
    snippets: HashMap<&'static str, &'static str>,
}

impl CandidateRegistry {
    pub fn new() -> Self {
        let mut snippets = HashMap::new();

        // Snippet 1: Flat file list (initial approach)
        snippets.insert(
            "flat_files",
            r#"
struct FileSystem {
    files: Vec<File>,
}

struct File {
    name: String,
    size: u64,
}
            "#,
        );

        // Snippet 2: Nested tree structure
        snippets.insert(
            "nested_tree",
            r#"
struct FileNode {
    id: String,
    name: String,
    is_dir: bool,
    children: Vec<FileNode>,
}
            "#,
        );

        // Snippet 3: Repeated array search (performance issue)
        snippets.insert(
            "repeated_search",
            r#"
fn find_child(node: &FileNode, name: &str) -> Option<&FileNode> {
    node.children.iter().find(|c| c.name == name)
}

fn expand_folder(root: &mut FileNode, path: &[&str]) {
    let mut current = root;
    for segment in path {
        if let Some(child) = current.children.iter_mut().find(|c| c.name == *segment) {
            current = child;
        }
    }
}
            "#,
        );

        // Snippet 4: Indexed child access with HashMap
        snippets.insert(
            "indexed_access",
            r#"
use std::collections::HashMap;

struct FileNode {
    id: String,
    name: String,
    children: Vec<FileNode>,
    child_index: HashMap<String, usize>,
}

impl FileNode {
    fn find_child(&self, name: &str) -> Option<&FileNode> {
        self.child_index
            .get(name)
            .and_then(|idx| self.children.get(*idx))
    }
}
            "#,
        );

        // Snippet 5: React rendering with keys
        snippets.insert(
            "react_rendering",
            r#"
function FileTreeNode({ node, onExpand }) {
    return (
        <div>
            <div onClick={() => onExpand(node.id)}>
                {node.name}
            </div>
            {node.isExpanded && (
                <div>
                    {node.children.map(child => (
                        <FileTreeNode
                            key={child.id}
                            node={child}
                            onExpand={onExpand}
                        />
                    ))}
                </div>
            )}
        </div>
    );
}
            "#,
        );

        // Snippet 6: Add file with conflict detection
        snippets.insert(
            "add_file_conflict",
            r#"
impl FileNode {
    fn add_file(&mut self, name: String, is_dir: bool) -> Result<(), String> {
        if self.child_index.contains_key(&name) {
            return Err(format!("File {} already exists", name));
        }
        let new_child = FileNode {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.clone(),
            children: Vec::new(),
            child_index: HashMap::new(),
            is_dir,
        };
        let idx = self.children.len();
        self.children.push(new_child);
        self.child_index.insert(name, idx);
        Ok(())
    }
}
            "#,
        );

        // Snippet 7: Delete operation with edge case handling
        snippets.insert(
            "delete_with_edge_case",
            r#"
impl FileNode {
    fn delete_child(&mut self, name: &str) -> Result<FileNode, String> {
        if let Some(idx) = self.child_index.remove(name) {
            let deleted = self.children.remove(idx);
            // Rebuild index after removal
            self.child_index.clear();
            for (i, child) in self.children.iter().enumerate() {
                self.child_index.insert(child.name.clone(), i);
            }
            Ok(deleted)
        } else {
            Err(format!("Child {} not found", name))
        }
    }
}
            "#,
        );

        // Snippet 8: Virtualization proposal
        snippets.insert(
            "virtualization",
            r#"
struct VirtualFileTree {
    root: FileNode,
    visible_range: (usize, usize),
    item_height: f32,
}

impl VirtualFileTree {
    fn render_visible_items(&self) -> Vec<&FileNode> {
        let (start, end) = self.visible_range;
        self.flatten_tree(&self.root)
            .into_iter()
            .skip(start)
            .take(end - start)
            .collect()
    }

    fn flatten_tree(&self, node: &FileNode) -> Vec<&FileNode> {
        let mut result = vec![node];
        if node.is_dir {
            for child in &node.children {
                result.extend(self.flatten_tree(child));
            }
        }
        result
    }
}
            "#,
        );

        CandidateRegistry { snippets }
    }

    /// Retrieve a named snippet by key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.snippets.get(key).copied()
    }

    /// List all available snippet keys.
    pub fn keys(&self) -> Vec<&str> {
        self.snippets.keys().copied().collect()
    }
}

impl Default for CandidateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Query lens for focusing on specific aspects of interview assessment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QLens {
    /// Focus on data structure choices
    DataStructure,
    /// Focus on algorithmic complexity
    Complexity,
    /// Focus on React and UI rendering
    ReactPatterns,
    /// Focus on API design and usability
    ApiDesign,
    /// Focus on error handling and edge cases
    ErrorHandling,
    /// Focus on performance optimization
    Performance,
}

/// Interview context and assessment harness.
pub struct InterviewHarness {
    candidate_registry: CandidateRegistry,
    observations: Vec<InterviewObservation>,
    lens: QLens,
}

impl InterviewHarness {
    /// Create a new interview harness.
    pub fn new(lens: QLens) -> Self {
        Self {
            candidate_registry: CandidateRegistry::new(),
            observations: Vec::new(),
            lens,
        }
    }

    /// Record an observation from candidate behavior.
    pub fn record_observation(&mut self, obs: InterviewObservation) {
        self.observations.push(obs);
    }

    /// Record multiple observations at once.
    pub fn record_observations(&mut self, obs: Vec<InterviewObservation>) {
        self.observations.extend(obs);
    }

    /// Analyze candidate snippet and return observations.
    pub fn analyze_snippet(&mut self, code: &str, context: &str) -> Vec<InterviewObservation> {
        let obs = observe(code, context);
        self.record_observations(obs.clone());
        obs
    }

    /// Get current lens for assessment.
    pub fn lens(&self) -> QLens {
        self.lens
    }

    /// Change assessment lens.
    pub fn set_lens(&mut self, lens: QLens) {
        self.lens = lens;
    }

    /// Retrieve all recorded observations.
    pub fn observations(&self) -> &[InterviewObservation] {
        &self.observations
    }

    /// Get reference to the candidate registry for snippet lookup.
    pub fn candidate_registry(&self) -> &CandidateRegistry {
        &self.candidate_registry
    }

    /// Filter observations by lens.
    pub fn observations_for_lens(&self) -> Vec<InterviewObservation> {
        match self.lens {
            QLens::DataStructure => self
                .observations
                .iter()
                .filter(|o| {
                    matches!(
                        o,
                        InterviewObservation::IntroduceFlatFiles
                            | InterviewObservation::RequireNestedStructure
                    )
                })
                .cloned()
                .collect(),
            QLens::Complexity => self
                .observations
                .iter()
                .filter(|o| {
                    matches!(
                        o,
                        InterviewObservation::CandidateUsesRepeatedArraySearch
                            | InterviewObservation::CandidateUsesIndexedChildAccess
                            | InterviewObservation::RecognizePerformanceIssue
                    )
                })
                .cloned()
                .collect(),
            QLens::ReactPatterns => self
                .observations
                .iter()
                .filter(|o| matches!(o, InterviewObservation::AskReactRendering))
                .cloned()
                .collect(),
            QLens::ErrorHandling => self
                .observations
                .iter()
                .filter(|o| {
                    matches!(
                        o,
                        InterviewObservation::AskFileFolderConflict
                            | InterviewObservation::ForgetDeletedParentEdgeCase
                    )
                })
                .cloned()
                .collect(),
            QLens::Performance => self
                .observations
                .iter()
                .filter(|o| {
                    matches!(
                        o,
                        InterviewObservation::RecognizePerformanceIssue
                            | InterviewObservation::ProposeVirtualization
                    )
                })
                .cloned()
                .collect(),
            QLens::ApiDesign => self.observations.clone(),
        }
    }
}

impl Default for InterviewHarness {
    fn default() -> Self {
        Self::new(QLens::DataStructure)
    }
}

// Test harness skeleton
#[cfg(test)]
mod tests {
    use super::*;

    chicago_tdd_tools::test!(candidate_registry_holds_eight_snippets, {
        let registry = CandidateRegistry::new();
        let keys = registry.keys();
        assert_eq!(keys.len(), 8);
        assert!(registry.get("flat_files").is_some());
        assert!(registry.get("nested_tree").is_some());
        assert!(registry.get("repeated_search").is_some());
        assert!(registry.get("indexed_access").is_some());
        assert!(registry.get("react_rendering").is_some());
        assert!(registry.get("add_file_conflict").is_some());
        assert!(registry.get("delete_with_edge_case").is_some());
        assert!(registry.get("virtualization").is_some());
    });

    chicago_tdd_tools::test!(interview_observation_observe_detects_flat_files, {
        let code = "struct FileSystem { files: Vec<File>, }";
        let obs = observe(code, "");
        assert!(obs.contains(&InterviewObservation::IntroduceFlatFiles));
    });

    chicago_tdd_tools::test!(interview_observation_observe_detects_nested_structure, {
        let code = "struct FileNode { children: Vec<FileNode>, }";
        let obs = observe(code, "");
        assert!(obs.contains(&InterviewObservation::RequireNestedStructure));
    });

    chicago_tdd_tools::test!(interview_observation_observe_detects_repeated_search, {
        let code = "node.children.iter().find(|c| c.name == name)";
        let obs = observe(code, "");
        assert!(obs.contains(&InterviewObservation::CandidateUsesRepeatedArraySearch));
    });

    chicago_tdd_tools::test!(interview_harness_starts_with_default_lens, {
        let harness = InterviewHarness::default();
        assert_eq!(harness.lens(), QLens::DataStructure);
    });

    chicago_tdd_tools::test!(interview_harness_records_observations, {
        let mut harness = InterviewHarness::new(QLens::DataStructure);
        harness.record_observation(InterviewObservation::IntroduceFlatFiles);
        harness.record_observation(InterviewObservation::RequireNestedStructure);
        assert_eq!(harness.observations().len(), 2);
    });

    chicago_tdd_tools::test!(interview_harness_filters_by_lens, {
        let mut harness = InterviewHarness::new(QLens::Complexity);
        harness.record_observation(InterviewObservation::CandidateUsesRepeatedArraySearch);
        harness.record_observation(InterviewObservation::IntroduceFlatFiles);
        let filtered = harness.observations_for_lens();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], InterviewObservation::CandidateUsesRepeatedArraySearch);
    });

    chicago_tdd_tools::test!(interview_harness_switches_lens, {
        let mut harness = InterviewHarness::new(QLens::DataStructure);
        harness.set_lens(QLens::Performance);
        assert_eq!(harness.lens(), QLens::Performance);
    });

    chicago_tdd_tools::test!(interview_harness_analyzes_snippet, {
        let mut harness = InterviewHarness::new(QLens::DataStructure);
        let code = "struct FileSystem { files: Vec<File>, }";
        let obs = harness.analyze_snippet(code, "initial approach");
        assert!(!obs.is_empty());
        assert_eq!(harness.observations().len(), obs.len());
    });

    chicago_tdd_tools::test!(qlens_enum_has_all_variants, {
        let _ds = QLens::DataStructure;
        let _cx = QLens::Complexity;
        let _rx = QLens::ReactPatterns;
        let _ad = QLens::ApiDesign;
        let _eh = QLens::ErrorHandling;
        let _pf = QLens::Performance;
        // Just verify all variants are accessible
        assert_eq!(
            std::mem::size_of::<QLens>(),
            std::mem::size_of::<u8>()
        );
    });
}
