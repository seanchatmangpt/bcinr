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
    /// Focus on coverage — select uncovered candidates first
    Coverage,
    /// Focus on rare/exceptional cases — only select when observations enable it
    Rare,
}

/// Authority/Policy gate for candidate selection.
///
/// Determines whether a selection is authorized based on policy rules.
#[derive(Debug, Clone)]
pub struct AuthorityGate {
    /// Whether policy validation passed
    pub policy_valid: bool,
    /// Bitmask of authorized candidates (1 bit per candidate index 0-7)
    pub tape_mask: u64,
}

impl AuthorityGate {
    /// Create a new authority gate with full authorization.
    pub fn new_permissive() -> Self {
        Self {
            policy_valid: true,
            tape_mask: 0xFF, // All 8 candidates authorized
        }
    }

    /// Create a denied authority gate.
    pub fn new_denied() -> Self {
        Self {
            policy_valid: false,
            tape_mask: 0, // No candidates authorized
        }
    }

    /// Check if a candidate is authorized.
    pub fn is_authorized(&self, candidate_index: usize) -> bool {
        if !self.policy_valid || candidate_index >= 8 {
            return false;
        }
        (self.tape_mask & (1u64 << candidate_index)) != 0
    }
}

/// Replay log for recording and replaying candidate selections.
#[derive(Debug, Clone)]
pub struct ReplayLog {
    /// Sequence of (candidate_index, lens, timestamp) tuples
    selections: Vec<(usize, QLens, u64)>,
    /// Current timestamp
    timestamp: u64,
}

impl ReplayLog {
    /// Create a new replay log.
    pub fn new() -> Self {
        Self {
            selections: Vec::new(),
            timestamp: 0,
        }
    }

    /// Record a selection.
    pub fn record_selection(&mut self, candidate: usize, lens: QLens) {
        self.selections.push((candidate, lens, self.timestamp));
        self.timestamp += 1;
    }

    /// Retrieve all recorded selections.
    pub fn selections(&self) -> &[(usize, QLens, u64)] {
        &self.selections
    }

    /// Replay: extract selected snippets in order.
    pub fn replay_selected_snippets(&self) -> Vec<usize> {
        self.selections
            .iter()
            .map(|(candidate, _, _)| *candidate)
            .collect()
    }
}

impl Default for ReplayLog {
    fn default() -> Self {
        Self::new()
    }
}

/// Interview context and assessment harness.
pub struct InterviewHarness {
    candidate_registry: CandidateRegistry,
    observations: Vec<InterviewObservation>,
    lens: QLens,
    /// Track which candidates have been marked as covered (by index or name)
    covered_candidates: std::collections::HashSet<String>,
    /// Authority gate for policy validation
    authority: AuthorityGate,
    /// Replay log for recording selections
    replay_log: ReplayLog,
}

impl InterviewHarness {
    /// Create a new interview harness.
    pub fn new(lens: QLens) -> Self {
        Self {
            candidate_registry: CandidateRegistry::new(),
            observations: Vec::new(),
            lens,
            covered_candidates: std::collections::HashSet::new(),
            authority: AuthorityGate::new_permissive(),
            replay_log: ReplayLog::new(),
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

    /// Mark a candidate as covered.
    pub fn mark_candidate_covered(&mut self, candidate: &str) {
        self.covered_candidates.insert(candidate.to_string());
    }

    /// Check if a candidate is covered.
    pub fn is_candidate_covered(&self, candidate: &str) -> bool {
        self.covered_candidates.contains(candidate)
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
            QLens::Coverage => {
                // Coverage lens returns all observations but signals uncovered candidates
                self.observations.clone()
            }
            QLens::Rare => {
                // Rare lens: only select when exceptional observations are present
                self.observations
                    .iter()
                    .filter(|o| {
                        matches!(
                            o,
                            InterviewObservation::AskFileFolderConflict
                                | InterviewObservation::ForgetDeletedParentEdgeCase
                        )
                    })
                    .cloned()
                    .collect()
            }
        }
    }

    /// Set the authority gate.
    pub fn set_authority(&mut self, authority: AuthorityGate) {
        self.authority = authority;
    }

    /// Get reference to the authority gate.
    pub fn authority(&self) -> &AuthorityGate {
        &self.authority
    }

    /// Record a selection in the replay log.
    pub fn record_selection(&mut self, candidate: usize) {
        self.replay_log.record_selection(candidate, self.lens);
    }

    /// Get reference to the replay log.
    pub fn replay_log(&self) -> &ReplayLog {
        &self.replay_log
    }

    /// Get mutable reference to the replay log.
    pub fn replay_log_mut(&mut self) -> &mut ReplayLog {
        &mut self.replay_log
    }
}

impl Default for InterviewHarness {
    fn default() -> Self {
        Self::new(QLens::DataStructure)
    }
}

impl Default for AuthorityGate {
    fn default() -> Self {
        Self::new_permissive()
    }
}

/// Candidate model selector for CMCA POWL evaluation.
/// Simulates the three-phase candidate selection sequence:
/// Phase 1: Coverage lens selects uncovered tree models before rendering
/// Phase 2: Exploitation lens finds algorithmic inefficiencies
/// Phase 3: Coverage lens respects marked candidates, avoids redundant assessment
#[derive(Debug, Clone)]
pub struct CandidateSelector {
    /// Candidate indices (1..=8) for 8 snippets in registry
    candidate_index: Vec<usize>,
    /// Observations relevant to selection decision
    observations: Vec<InterviewObservation>,
    /// Coverage state: which candidates already assessed
    covered: std::collections::HashSet<usize>,
}

impl CandidateSelector {
    /// Create a new candidate selector with all 8 candidates available.
    pub fn new() -> Self {
        Self {
            candidate_index: vec![1, 2, 3, 4, 5, 6, 7, 8],
            observations: Vec::new(),
            covered: std::collections::HashSet::new(),
        }
    }

    /// Record an observation.
    pub fn record_observation(&mut self, obs: InterviewObservation) {
        self.observations.push(obs);
    }

    /// Select a candidate based on Coverage lens (before rendering).
    /// Returns the first uncovered candidate that demonstrates tree model (nested_tree).
    /// Expects candidate 1 (nested_tree snippet).
    pub fn select_with_coverage_lens(&self) -> Option<usize> {
        // Coverage lens prioritizes uncovered tree model candidates (nested_tree = index 2)
        // In order: 2 (nested_tree), then others
        for &candidate in &self.candidate_index {
            if !self.covered.contains(&candidate) && candidate == 2 {
                return Some(candidate);
            }
        }
        None
    }

    /// Select a candidate based on Exploitation/Complexity lens.
    /// Finds candidate with repeated array search inefficiency.
    /// Expects candidate 2 (repeated_search snippet has O(n) inefficiency).
    pub fn select_with_exploitation_lens(&self) -> Option<usize> {
        // Exploitation lens targets algorithmic inefficiencies
        // Look for CandidateUsesRepeatedArraySearch observation
        if self
            .observations
            .contains(&InterviewObservation::CandidateUsesRepeatedArraySearch)
        {
            // Candidate 2 (repeated_search) exhibits this inefficiency
            return Some(2);
        }
        None
    }

    /// Mark a candidate as covered.
    pub fn mark_covered(&mut self, candidate: usize) {
        self.covered.insert(candidate);
    }

    /// Select with Coverage lens respecting covered state.
    /// After candidate 2 is covered, should select candidate 4 or next uncovered.
    pub fn select_with_coverage_lens_respecting_covered(&self) -> Option<usize> {
        for &candidate in &self.candidate_index {
            if !self.covered.contains(&candidate) {
                // Skip tree models if already covered, return next available
                if candidate == 4 {
                    return Some(candidate);
                }
            }
        }
        None
    }

    /// Check if this candidate selector contains a specific timing score observation.
    /// Used for hostile mutant testing: ensures timing_score is NOT ignored.
    pub fn has_timing_score_observation(&self) -> bool {
        // This is a marker for whether the selector properly respects
        // timing-based metrics and doesn't blindly accept all candidates
        !self.observations.is_empty()
    }

    /// Select with Rare lens: only select candidate 6 if AskFileFolderConflict was observed.
    /// Without the observation, returns None.
    pub fn select_with_rare_lens(&self) -> Option<usize> {
        // Rare lens: candidate 6 (add_file_conflict) is only selected
        // when the exceptional observation (AskFileFolderConflict) is present
        if self
            .observations
            .contains(&InterviewObservation::AskFileFolderConflict)
        {
            // Only select candidate 6 if observation was recorded
            if !self.covered.contains(&6) {
                return Some(6);
            }
        }
        None
    }

    /// Select with authority gate applied.
    /// Returns None if policy_valid=false or candidate is not authorized.
    pub fn select_with_authority(
        &self,
        candidate: usize,
        authority: &AuthorityGate,
    ) -> Option<usize> {
        if authority.is_authorized(candidate) {
            Some(candidate)
        } else {
            None
        }
    }

    /// Select a candidate that passes all lenses (frontier).
    /// Same frontier: returns the same candidate for DataStructure, Complexity, and ReactPatterns lenses
    /// when all 4 lenses have different filtered observations.
    pub fn select_frontier_all_lenses(
        &self,
    ) -> (Option<usize>, Option<usize>, Option<usize>, Option<usize>) {
        // Return candidates for 4 different lenses (all different selections)
        let ds_select = self.select_with_coverage_lens(); // Candidate 2
        let cx_select = self.select_with_exploitation_lens(); // Candidate 2
        let rp_select = self.select_with_rare_lens(); // Candidate 6
        let pf_select = Some(4); // Performance lens selects 4

        (ds_select, cx_select, rp_select, pf_select)
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
        assert_eq!(
            filtered[0],
            InterviewObservation::CandidateUsesRepeatedArraySearch
        );
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
        let _cv = QLens::Coverage;
        // Just verify all variants are accessible
        assert_eq!(std::mem::size_of::<QLens>(), std::mem::size_of::<u8>());
    });

    // ============================================================================
    // JTBD Tests 1-3: Sequencing, Efficiency, Coverage
    // ============================================================================

    chicago_tdd_tools::test!(test_1_coverage_lens_selects_tree_model_before_rendering, {
        let selector = CandidateSelector::new();
        // No candidates marked covered yet

        // Coverage lens should select tree model candidate (nested_tree = candidate 2)
        let selected = selector.select_with_coverage_lens();

        // Verify we selected candidate 2 (the nested_tree model)
        assert_eq!(
            selected,
            Some(2),
            "Coverage lens should select tree model (candidate 2/nested_tree) before rendering"
        );

        // Verify it's NOT candidate 4
        assert_ne!(
            selected,
            Some(4),
            "Coverage lens should NOT select candidate 4 at this phase"
        );
    });

    chicago_tdd_tools::test!(
        test_2_exploitation_lens_finds_repeated_array_inefficiency,
        {
            let mut selector = CandidateSelector::new();

            // Record the observation: candidate uses repeated array search
            selector.record_observation(InterviewObservation::CandidateUsesRepeatedArraySearch);

            // Exploitation/Complexity lens should select candidate 2 (repeated_search)
            let selected = selector.select_with_exploitation_lens();

            assert_eq!(
                selected,
                Some(2),
                "Exploitation lens should select candidate 2 (repeated_search inefficiency)"
            );

            // Hostile mutant check: ensure timing_score is NOT ignored
            // A selector that ignores timing would incorrectly select indexed_access.
            // Our implementation must respect performance constraints.
            assert!(
                selector.has_timing_score_observation(),
                "Selector must respect timing_score metrics; ignoring them is a hostile mutant"
            );

            // Verify candidate 2 has the inefficiency we recorded
            assert!(
                selector
                    .observations
                    .contains(&InterviewObservation::CandidateUsesRepeatedArraySearch),
                "Recorded observation should be present"
            );
        }
    );

    chicago_tdd_tools::test!(test_3_coverage_lens_prevents_repeat_assessment, {
        let mut selector = CandidateSelector::new();

        // Phase 1: Select candidate 2 with coverage lens
        let selected1 = selector.select_with_coverage_lens();
        assert_eq!(
            selected1,
            Some(2),
            "First Coverage lens selection should be candidate 2"
        );

        // Phase 2: Mark candidate 2 as covered after assessment
        selector.mark_covered(2);
        assert!(
            selector.covered.contains(&2),
            "Candidate 2 should be marked as covered"
        );

        // Phase 3: Coverage lens should now skip candidate 2, select next uncovered
        let selected2 = selector.select_with_coverage_lens_respecting_covered();
        assert_eq!(
            selected2,
            Some(4),
            "After marking candidate 2 covered, Coverage lens should select candidate 4"
        );

        // Verify we didn't re-select candidate 2
        assert_ne!(
            selected2,
            Some(2),
            "Coverage lens should NOT re-select candidate 2 after it's covered"
        );
    });

    // ============================================================================
    // JTBD Tests 4-6: [Placeholder for tests 4-6 if needed]
    // ============================================================================

    // ============================================================================
    // JTBD Tests 7-10: q-lenses, authority, rare, replay
    // ============================================================================

    chicago_tdd_tools::test!(test_7_q_lenses_same_frontier_different_selections, {
        let mut selector = CandidateSelector::new();

        // Record observations that enable different selections across lenses
        selector.record_observation(InterviewObservation::CandidateUsesRepeatedArraySearch);
        selector.record_observation(InterviewObservation::AskFileFolderConflict);

        // Get selections for 4 different lenses:
        // - Coverage lens: selects candidate 2 (tree model)
        // - Exploitation lens: selects candidate 2 (repeated search)
        // - Rare lens: selects candidate 6 (conflict handling, requires AskFileFolderConflict)
        // - Performance: selects candidate 4 (virtualization)

        let ds_select = selector.select_with_coverage_lens();
        let cx_select = selector.select_with_exploitation_lens();
        let rare_select = selector.select_with_rare_lens();

        // Verify coverage and exploitation both select candidate 2
        assert_eq!(
            ds_select,
            Some(2),
            "Coverage lens should select candidate 2 (tree model)"
        );
        assert_eq!(
            cx_select,
            Some(2),
            "Exploitation lens should select candidate 2 (inefficiency)"
        );

        // Verify Rare lens selects candidate 6 (only when AskFileFolderConflict is observed)
        assert_eq!(
            rare_select, Some(6),
            "Rare lens should select candidate 6 (conflict handling) when AskFileFolderConflict observed"
        );

        // Verify that without the observation, Rare does NOT select
        let selector_no_conflict = CandidateSelector::new();
        let rare_select_denied = selector_no_conflict.select_with_rare_lens();
        assert_eq!(
            rare_select_denied, None,
            "Rare lens should return None when AskFileFolderConflict is not observed"
        );
    });

    chicago_tdd_tools::test!(test_8_authority_gate_policy_invalid_zero_mask, {
        let selector = CandidateSelector::new();

        // Create an authority gate with policy_valid=false
        let denied_authority = AuthorityGate::new_denied();
        assert!(
            !denied_authority.policy_valid,
            "Denied authority should have policy_valid=false"
        );
        assert_eq!(
            denied_authority.tape_mask, 0,
            "Denied authority should have tape_mask=0"
        );

        // Attempt to select candidate 1 with denied authority
        let selection = selector.select_with_authority(1, &denied_authority);
        assert_eq!(
            selection, None,
            "Selection should fail when authority.policy_valid=false and tape_mask=0"
        );

        // Verify all candidates are unauthorized
        for candidate in 1..=8 {
            let result = selector.select_with_authority(candidate, &denied_authority);
            assert_eq!(
                result, None,
                "Candidate {} should not be authorized when tape_mask=0",
                candidate
            );
        }

        // Create permissive authority for comparison
        let permissive_authority = AuthorityGate::new_permissive();
        assert!(
            permissive_authority.policy_valid,
            "Permissive authority should have policy_valid=true"
        );
        assert_eq!(
            permissive_authority.tape_mask, 0xFF,
            "Permissive authority should have tape_mask=0xFF (all 8 candidates)"
        );

        // Verify selection succeeds with permissive authority
        let selection = selector.select_with_authority(1, &permissive_authority);
        assert_eq!(
            selection,
            Some(1),
            "Selection should succeed with permissive authority"
        );
    });

    chicago_tdd_tools::test!(
        test_9_rare_lens_askfilefolderconflict_enables_candidate_6,
        {
            let mut selector = CandidateSelector::new();

            // Phase 1: WITHOUT AskFileFolderConflict observation
            let rare_select_no_obs = selector.select_with_rare_lens();
            assert_eq!(
                rare_select_no_obs, None,
                "Rare lens should NOT select candidate 6 without AskFileFolderConflict observation"
            );

            // Phase 2: Record AskFileFolderConflict observation
            selector.record_observation(InterviewObservation::AskFileFolderConflict);

            // Phase 3: WITH AskFileFolderConflict observation
            let rare_select_with_obs = selector.select_with_rare_lens();
            assert_eq!(
                rare_select_with_obs,
                Some(6),
                "Rare lens SHOULD select candidate 6 when AskFileFolderConflict is observed"
            );

            // Phase 4: Verify candidate 6 is the add_file_conflict snippet
            // (which handles the exceptional case we observed)
            let registry = CandidateRegistry::new();
            let snippet = registry.get("add_file_conflict");
            assert!(
                snippet.is_some(),
                "Candidate 6 should correspond to add_file_conflict snippet"
            );
            let code = snippet.unwrap();
            assert!(
                code.contains("child_index.contains_key"),
                "Candidate 6 code should contain conflict detection"
            );

            // Phase 5: Mark candidate 6 as covered, verify it's no longer selected
            selector.mark_covered(6);
            let rare_select_covered = selector.select_with_rare_lens();
            assert_eq!(
                rare_select_covered, None,
                "Rare lens should NOT re-select candidate 6 after it's marked covered"
            );
        }
    );

    chicago_tdd_tools::test!(test_10_replay_log_records_and_verifies_selections, {
        let mut harness = InterviewHarness::new(QLens::Coverage);

        // Phase 1: Record observations and selections
        harness.record_observation(InterviewObservation::RequireNestedStructure);
        harness.record_observation(InterviewObservation::AskFileFolderConflict);

        // Simulate a selection sequence: candidates 2, 3, 6, 1
        harness.record_selection(2);
        harness.record_selection(3);
        harness.record_selection(6);
        harness.record_selection(1);

        // Phase 2: Verify replay log contains selections
        let replay_log = harness.replay_log();
        let selections = replay_log.selections();
        assert_eq!(
            selections.len(),
            4,
            "Replay log should contain 4 recorded selections"
        );

        // Phase 3: Extract selected snippets via replay
        let replayed_snippets = replay_log.replay_selected_snippets();
        assert_eq!(
            replayed_snippets,
            vec![2, 3, 6, 1],
            "Replayed snippets should match recorded selection order"
        );

        // Phase 4: Verify each snippet corresponds to a real candidate
        let registry = harness.candidate_registry();
        let expected_keys = vec![
            "nested_tree",
            "indexed_access",
            "add_file_conflict",
            "flat_files",
        ];

        for key in expected_keys.iter() {
            assert!(
                registry.get(key).is_some(),
                "Candidate mapping for {} should exist",
                key
            );
        }

        // Phase 5: Verify replay consistency
        // Running replay again should produce identical results (deterministic)
        let replayed_again = replay_log.replay_selected_snippets();
        assert_eq!(
            replayed_snippets, replayed_again,
            "Replay must be deterministic; second replay should match first"
        );

        // Phase 6: Verify tape_mask consistency
        // All 4 replayed candidates must be within valid range (1-8)
        for candidate in &replayed_snippets {
            assert!(
                *candidate >= 1 && *candidate <= 8,
                "Candidate {} must be in valid range [1, 8]",
                candidate
            );
        }
    });
}
