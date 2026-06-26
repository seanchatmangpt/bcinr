//! LSP code actions for lifecycle repair and planning.

use lsp_types_max::{CodeAction, CodeActionKind, CodeActionOrCommand, Range};

pub fn create_prd_action() -> CodeActionOrCommand {
    CodeActionOrCommand::CodeAction(CodeAction {
        title: "Create PRD skeleton (docs/prd.md)".into(),
        kind: Some(CodeActionKind::QUICKFIX),
        command: Some(lsp_types_max::Command {
            title: "Create PRD".into(),
            command: "bcinrPddl.createPrd".into(),
            arguments: None,
        }),
        ..CodeAction::default()
    })
}

pub fn create_ard_action() -> CodeActionOrCommand {
    CodeActionOrCommand::CodeAction(CodeAction {
        title: "Derive ARD from admitted PRD (docs/ard.md)".into(),
        kind: Some(CodeActionKind::QUICKFIX),
        command: Some(lsp_types_max::Command {
            title: "Derive ARD".into(),
            command: "bcinrPddl.deriveArd".into(),
            arguments: None,
        }),
        ..CodeAction::default()
    })
}

pub fn run_plan_action() -> CodeActionOrCommand {
    CodeActionOrCommand::CodeAction(CodeAction {
        title: "Generate PDDL8 projection and run bcinr-pddl plan".into(),
        kind: Some(CodeActionKind::REFACTOR),
        command: Some(lsp_types_max::Command {
            title: "Run bcinr-pddl plan".into(),
            command: "bcinrPddl.runPlan".into(),
            arguments: None,
        }),
        ..CodeAction::default()
    })
}

pub fn split_need9_action(context: &str) -> CodeActionOrCommand {
    CodeActionOrCommand::CodeAction(CodeAction {
        title: format!("Split Need9: decompose '{context}' into ≤8 tasks"),
        kind: Some(CodeActionKind::REFACTOR),
        command: Some(lsp_types_max::Command {
            title: "Split Need9 work package".into(),
            command: "bcinrPddl.splitNeed9".into(),
            arguments: Some(vec![serde_json::json!(context)]),
        }),
        ..CodeAction::default()
    })
}

pub fn open_receipt_action() -> CodeActionOrCommand {
    CodeActionOrCommand::CodeAction(CodeAction {
        title: "Open latest execution receipt".into(),
        kind: Some(CodeActionKind::EMPTY),
        command: Some(lsp_types_max::Command {
            title: "Open Receipt".into(),
            command: "bcinrPddl.openVirtualDocument".into(),
            arguments: Some(vec![serde_json::json!("bcinr-pddl://receipt/latest")]),
        }),
        ..CodeAction::default()
    })
}

pub fn open_publish_gate_action() -> CodeActionOrCommand {
    CodeActionOrCommand::CodeAction(CodeAction {
        title: "Explain publish gate blockers".into(),
        kind: Some(CodeActionKind::EMPTY),
        command: Some(lsp_types_max::Command {
            title: "Explain Publish Gate".into(),
            command: "bcinrPddl.explainPublishGate".into(),
            arguments: None,
        }),
        ..CodeAction::default()
    })
}

/// Return all available code actions for a given diagnostic range.
pub fn for_diagnostics(diag_codes: &[String], _range: &Range) -> Vec<CodeActionOrCommand> {
    let mut actions = Vec::new();

    for code in diag_codes {
        match code.as_str() {
            "PRD_MISSING" | "INTENT_MISSING" => actions.push(create_prd_action()),
            "ARD_MISSING" | "PRD_NOT_ADMITTED" => actions.push(create_ard_action()),
            "NO_ADMITTED_PLAN" | "PDDL_PARSE_ERROR" => actions.push(run_plan_action()),
            "WORK_UNIT_NEED9" => actions.push(split_need9_action("work-unit")),
            "PUBLISH_BLOCKED" | "RECEIPT_MISSING" => {
                actions.push(open_receipt_action());
                actions.push(open_publish_gate_action());
            }
            _ => actions.push(run_plan_action()),
        }
    }

    if actions.is_empty() {
        actions.push(run_plan_action());
    }

    actions
}
