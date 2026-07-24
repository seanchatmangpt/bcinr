//! Chicago TDD coverage for public POWL process patching behavior.

use bcinr_powl::process_rewrite::{
    apply_process_patch, find_activities, map_activity_labels, prepare_process_patch,
    ProcessRewriteError,
};
use bcinr_powl::process_toolkit::{activity, process_digest, sequence, ProcessNodeRef};

chicago_tdd_tools::test!(optimistic_patch_replaces_only_the_observed_target, {
    let process = sequence(vec![activity("validate"), activity("dispatch")])
        .expect("process should validate");
    let dispatch = ProcessNodeRef::root()
        .child(1)
        .expect("child reference should fit");
    let patch = prepare_process_patch(&process, dispatch, activity("dispatch-v2"))
        .expect("patch should bind the target root");
    let (rewritten, witness) =
        apply_process_patch(&process, &patch).expect("unchanged target should accept patch");

    assert_eq!(find_activities(&rewritten, "dispatch-v2").unwrap().len(), 1);
    assert_eq!(witness.before, process_digest(&process));
    assert_eq!(witness.after, process_digest(&rewritten));
    assert_ne!(witness.before, witness.after);
});

chicago_tdd_tools::test!(stale_patch_is_refused_without_mutating_new_process, {
    let original = sequence(vec![activity("validate"), activity("dispatch")]).unwrap();
    let target = ProcessNodeRef::root().child(1).unwrap();
    let patch = prepare_process_patch(&original, target, activity("dispatch-v2")).unwrap();
    let current = sequence(vec![activity("validate"), activity("dispatch-hotfix")]).unwrap();
    let current_root = process_digest(&current);

    assert!(matches!(
        apply_process_patch(&current, &patch),
        Err(ProcessRewriteError::StaleTarget { .. })
    ));
    assert_eq!(process_digest(&current), current_root);
});

chicago_tdd_tools::test!(activity_mapping_changes_vocabulary_not_geometry, {
    let process = sequence(vec![activity("validate"), activity("dispatch")]).unwrap();
    let (mapped, witness) =
        map_activity_labels(&process, |label| format!("command::{label}")).unwrap();

    assert_eq!(find_activities(&mapped, "command::validate").unwrap().len(), 1);
    assert_eq!(find_activities(&mapped, "command::dispatch").unwrap().len(), 1);
    assert_ne!(witness.before, witness.after);
});
