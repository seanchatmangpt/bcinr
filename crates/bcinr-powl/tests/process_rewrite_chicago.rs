//! Chicago TDD coverage for public POWL process rewrite behavior.

use bcinr_powl::powl2::{validate_powl2, Powl2Model};
use bcinr_powl::process_rewrite::{
    apply_process_patch, diff_validated_processes, eliminate_redundant_silent_nodes,
    find_activities, map_activity_labels, prepare_process_patch, reduce_transitive_process_edges,
    slice_process_activities, ProcessRewriteError, ProcessRewriteLaw,
};
use bcinr_powl::process_toolkit::{
    activity, partial_order, process_digest, sequence, silent, ProcessNodeRef,
};

chicago_tdd_tools::test!(powl2_optimistic_patch_replaces_only_the_observed_target, {
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
    assert_eq!(witness.law, ProcessRewriteLaw::ExactNodeReplacement);
    assert_eq!(witness.before, process_digest(&process));
    assert_eq!(witness.after, process_digest(&rewritten));
    assert_ne!(witness.before, witness.after);
});

chicago_tdd_tools::test!(powl2_stale_patch_is_refused_without_mutating_new_process, {
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

chicago_tdd_tools::test!(powl2_activity_mapping_changes_vocabulary_not_geometry, {
    let process = sequence(vec![activity("validate"), activity("dispatch")]).unwrap();
    let (mapped, witness) =
        map_activity_labels(&process, |label| format!("command::{label}")).unwrap();

    assert_eq!(
        find_activities(&mapped, "command::validate").unwrap().len(),
        1
    );
    assert_eq!(
        find_activities(&mapped, "command::dispatch").unwrap().len(),
        1
    );
    assert_eq!(witness.law, ProcessRewriteLaw::ActivityRelabel);
    assert_ne!(witness.before, witness.after);
});

chicago_tdd_tools::test!(powl2_transitive_reduction_is_recursive_and_witnessed, {
    let nested = partial_order(
        vec![activity("a"), activity("b"), activity("c")],
        vec![(0, 1), (1, 2), (0, 2)],
    )
    .unwrap();
    let original = sequence(vec![nested, activity("done")]).unwrap();
    let (rewritten, witness) = reduce_transitive_process_edges(&original).unwrap();

    let Powl2Model::Sequence(children) = &rewritten else {
        panic!("sequence geometry should remain explicit");
    };
    let Powl2Model::PartialOrder { edges, .. } = &children[0] else {
        panic!("nested partial order should remain explicit");
    };
    assert_eq!(edges.as_slice(), &[(0, 1), (1, 2)]);
    assert_eq!(witness.law, ProcessRewriteLaw::TransitiveReduction);
    assert_eq!(witness.before, process_digest(&original));
    assert_eq!(witness.after, process_digest(&rewritten));
    validate_powl2(&rewritten).unwrap();
});

chicago_tdd_tools::test!(
    powl2_silent_elimination_preserves_partial_order_reachability,
    {
        let original = partial_order(
            vec![activity("a"), silent(), activity("b")],
            vec![(0, 1), (1, 2)],
        )
        .unwrap();
        let (rewritten, witness) = eliminate_redundant_silent_nodes(&original).unwrap();

        assert_eq!(
            rewritten,
            Powl2Model::PartialOrder {
                children: vec![activity("a"), activity("b")],
                edges: vec![(0, 1)],
            }
        );
        assert_eq!(witness.law, ProcessRewriteLaw::SilentNodeElimination);
        validate_powl2(&rewritten).unwrap();
    }
);

chicago_tdd_tools::test!(
    powl2_activity_slice_carries_exact_selection_and_diff_evidence,
    {
        let original = sequence(vec![activity("a"), activity("drop"), activity("c")]).unwrap();
        let (sliced, witness) =
            slice_process_activities(&original, |label| label != "drop").unwrap();

        assert_eq!(
            sliced,
            sequence(vec![activity("a"), activity("c")]).unwrap()
        );
        assert_eq!(witness.rewrite.law, ProcessRewriteLaw::ActivitySlice);
        assert_eq!(witness.retained_activities.len(), 2);
        assert_eq!(witness.removed_activities.len(), 1);
        assert_eq!(witness.removed_activities[0].stable_id(), "n_1");

        let diff = diff_validated_processes(&original, &sliced).unwrap();
        assert_eq!(diff.before, witness.rewrite.before);
        assert_eq!(diff.after, witness.rewrite.after);
        assert_eq!(diff.removed_activities, vec!["drop".to_string()]);
    }
);

chicago_tdd_tools::test!(powl2_activity_slice_preserves_choice_vertices, {
    let original = Powl2Model::ChoiceGraph {
        children: vec![activity("start"), activity("drop"), activity("end")],
        // Def 3.6: sentinels are 3/4, outside the 3-element child set.
        edges: vec![(3, 0), (0, 1), (1, 2), (2, 4)],
        start: 3,
        end: 4,
    };
    let (sliced, _) = slice_process_activities(&original, |label| label != "drop").unwrap();

    assert_eq!(
        sliced,
        Powl2Model::ChoiceGraph {
            children: vec![activity("start"), silent(), activity("end")],
            edges: vec![(3, 0), (0, 1), (1, 2), (2, 4)],
            start: 3,
            end: 4,
        }
    );
    validate_powl2(&sliced).unwrap();
});
