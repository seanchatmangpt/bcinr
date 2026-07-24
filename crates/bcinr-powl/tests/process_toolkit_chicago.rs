//! Chicago TDD behavior tests for the POWL process-manipulation toolkit.

use bcinr_powl::powl2::{compile_powl2, LowestIndexPolicy, Powl2Model};
use bcinr_powl::process_toolkit::{
    activity, choice_graph, diff_processes, explain_process, normalize_process, partial_order,
    process_digest, process_metrics, process_node, process_nodes, process_to_mermaid, sequence,
    ProcessNodeRef,
};

chicago_tdd_tools::test!(normalization_preserves_compilable_process_behavior, {
    let original = sequence(vec![
        activity("validate"),
        sequence(vec![
            Powl2Model::Silent,
            partial_order(
                vec![activity("reserve"), activity("calculate-shipping")],
                vec![],
            )
            .expect("independent work should form a partial order"),
        ])
        .expect("nested sequence should validate"),
        activity("charge"),
    ])
    .expect("process should validate");

    let normalized = normalize_process(&original).expect("normalization should succeed");
    let mut policy = LowestIndexPolicy;
    let compiled =
        compile_powl2(&normalized, &mut policy).expect("normalized process should compile");
    let metrics = process_metrics(&normalized).expect("metrics should inspect real structure");

    assert_ne!(process_digest(&original), process_digest(&normalized));
    assert_eq!(metrics.activities, 4);
    assert_eq!(metrics.max_parallel_width, 2);
    assert!(!compiled.activity_slots.is_empty());
    assert!(explain_process(&normalized)
        .expect("explanation")
        .contains("max_parallel_width=2"));
});

chicago_tdd_tools::test!(stable_node_paths_support_inspection_and_visualization, {
    let model = choice_graph(
        vec![
            activity("start"),
            activity("manual"),
            activity("automatic"),
            activity("end"),
        ],
        vec![(0, 1), (0, 2), (1, 3), (2, 3)],
        0,
        3,
    )
    .expect("choice graph should validate");

    let nodes = process_nodes(&model).expect("node walk should validate");
    let manual = ProcessNodeRef::root()
        .child(1)
        .expect("bounded child index");
    let selected = process_node(&model, &manual).expect("stable path should resolve");
    let mermaid = process_to_mermaid(&model).expect("visualization should project");

    assert_eq!(nodes.len(), 5);
    assert_eq!(selected, &activity("manual"));
    assert!(mermaid.contains("choice 0->3"));
    assert!(mermaid.contains("activity: manual"));
    assert!(mermaid.contains("choice"));
});

chicago_tdd_tools::test!(process_diff_reports_vocabulary_and_structure_changes, {
    let before = partial_order(
        vec![activity("reserve"), activity("notify")],
        vec![],
    )
    .expect("before process");
    let after = partial_order(
        vec![activity("reserve"), activity("audit"), activity("notify")],
        vec![(0, 2)],
    )
    .expect("after process");

    let diff = diff_processes(&before, &after);

    assert!(diff.structure_changed);
    assert_eq!(diff.added_activities, vec!["audit"]);
    assert!(diff.removed_activities.is_empty());
    assert_ne!(diff.before, diff.after);
});
