//! LSP backend wiring all modules together.
//!
//! Architecture:
//!   textDocument events → lifecycle scan → projection → planner_client → virtual_docs
//!   diagnostics published per-workspace-root
//!   custom commands via workspace/executeCommand

use std::path::PathBuf;
use std::sync::Arc;
use dashmap::DashMap;
use lsp_max::{Client, LanguageServer};
use lsp_types_max::*;
use tokio::sync::Mutex;

fn uri_to_path(uri: &Uri) -> Option<PathBuf> {
    let s = uri.as_str();
    if let Some(path) = s.strip_prefix("file://") {
        Some(PathBuf::from(path))
    } else {
        None
    }
}

use crate::{
    bounds,
    build_broker,
    code_actions,
    diagnostics as diag_mod,
    lifecycle,
    planner_client,
    projection,
    publish_gate,
    virtual_docs,
};

/// Cached projection state for a workspace root.
///
/// Two-tier: CANDIDATE (always present after any analysis) vs ADMITTED (explicit executeTape only).
struct CachedPlan {
    projection: projection::Pddl8Projection,
    lifecycle: lifecycle::ProjectLifecycle,
    bounds_report: bounds::BoundReport,
    /// Phase 1: candidate plan — always updated on scan/project/plan
    candidate: Option<planner_client::PlanCandidate>,
    /// Phase 2: admitted result — only updated by explicit bcinrPddl.executeTape command
    admission: Option<planner_client::PlanResult>,
    gate: publish_gate::PublishGate,
}

pub struct PddlLspBackend {
    client: Client,
    /// URI string → document text
    documents: DashMap<String, String>,
    /// workspace root → latest plan cache
    plan_cache: Arc<Mutex<Option<CachedPlan>>>,
    /// workspace root detected on initialize
    workspace_root: Arc<Mutex<Option<PathBuf>>>,
    /// build broker state
    broker: Arc<Mutex<build_broker::BuildBrokerState>>,
}

impl PddlLspBackend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: DashMap::new(),
            plan_cache: Arc::new(Mutex::new(None)),
            workspace_root: Arc::new(Mutex::new(None)),
            broker: Arc::new(Mutex::new(build_broker::BuildBrokerState::default())),
        }
    }

    async fn root(&self) -> PathBuf {
        self.workspace_root.lock().await
            .clone()
            .unwrap_or_else(|| PathBuf::from("."))
    }

    /// Projection mode: scan → project → plan (candidate) → diagnostics.
    ///
    /// Safe to call on every didOpen/didChange/didSave.
    /// Does NOT execute the tape. Does NOT emit receipts. Does NOT produce OCEL.
    /// Gate stays at PARTIAL at most — never advances to ADMITTED here.
    async fn project_and_cache(&self, trigger_uri: Uri) {
        let root = self.root().await;
        let lc = lifecycle::scan(&root);
        let bounds_report = bounds::check_lifecycle_domain();
        let proj = projection::project(&lc);

        // Phase 1: find candidate plan only
        let candidate = planner_client::plan(&proj).ok();
        let gate = publish_gate::from_lifecycle(&lc);

        {
            let mut cache = self.plan_cache.lock().await;
            // Preserve any existing admission — projection mode never clears it
            let admission = cache.as_mut().and_then(|c| c.admission.take());
            let final_gate = if admission.is_some() {
                cache.as_ref().map(|c| c.gate.clone()).unwrap_or(gate)
            } else {
                gate
            };
            *cache = Some(CachedPlan {
                projection: proj.clone(),
                lifecycle: lc.clone(),
                bounds_report: bounds_report.clone(),
                candidate,
                admission,
                gate: final_gate,
            });
        }

        let mut all_diags = diag_mod::lifecycle_diagnostics(&lc);
        all_diags.extend(diag_mod::bound_diagnostics(&bounds_report));
        self.client.publish_diagnostics(trigger_uri, all_diags, None).await;

        let gate_label = {
            let cache = self.plan_cache.lock().await;
            cache.as_ref().map(|c| c.gate.status_label().to_string()).unwrap_or_default()
        };
        self.client.log_message(
            MessageType::INFO,
            format!(
                "bcinr-pddl-lsp CANDIDATE: project '{}' gate={} next={:?}",
                lc.project_name,
                gate_label,
                lc.next_missing().map(|s| s.predicate_name()),
            ),
        ).await;
    }

    /// Admission mode: execute cached candidate tape → receipt + OCEL → gate ADMITTED.
    ///
    /// ONLY called by explicit bcinrPddl.executeTape command.
    /// Candidate ≠ Admitted. This is the BRCE gate.
    async fn execute_tape_and_admit(&self, trigger_uri: Uri) {
        let root = self.root().await;
        let lc = lifecycle::scan(&root);

        let (candidate, proj) = {
            let cache = self.plan_cache.lock().await;
            match cache.as_ref() {
                Some(c) => (c.candidate.clone(), c.projection.clone()),
                None => {
                    self.client.log_message(
                        MessageType::WARNING,
                        "bcinr-pddl-lsp: no candidate plan cached — run project first",
                    ).await;
                    return;
                }
            }
        };

        let Some(candidate) = candidate else {
            self.client.log_message(
                MessageType::WARNING,
                "bcinr-pddl-lsp: candidate plan is empty — nothing to execute",
            ).await;
            return;
        };

        let case_id = format!("lsp-{}", lc.project_name);
        match planner_client::admit(&candidate, &case_id) {
            Ok(result) => {
                let gate = publish_gate::from_plan_result(&lc, &result);
                let gate_label = gate.status_label().to_string();
                // Persist receipt + OCEL to .bcinr/
                let _ = planner_client::persist_admission(&root, &result);
                {
                    let mut cache = self.plan_cache.lock().await;
                    if let Some(ref mut c) = *cache {
                        c.admission = Some(result);
                        c.gate = gate;
                    }
                }
                self.client.log_message(
                    MessageType::INFO,
                    format!("bcinr-pddl-lsp ADMITTED: gate={gate_label}"),
                ).await;
                let all_diags = diag_mod::lifecycle_diagnostics(&lc);
                self.client.publish_diagnostics(trigger_uri, all_diags, None).await;
            }
            Err(e) => {
                self.client.log_message(
                    MessageType::ERROR,
                    format!("bcinr-pddl-lsp executeTape FAILED: {e}"),
                ).await;
            }
        }
    }

    /// Render a virtual document by URI scheme.
    async fn render_virtual_doc(&self, uri_str: &str) -> Option<String> {
        let root = self.root().await;
        let lc = lifecycle::scan(&root);
        let cache = self.plan_cache.lock().await;
        let broker = self.broker.lock().await;

        Some(match uri_str {
            virtual_docs::URI_LIFECYCLE => virtual_docs::render_lifecycle(&lc),
            virtual_docs::URI_STATUS => {
                let gate = cache.as_ref()
                    .map(|c| c.gate.clone())
                    .unwrap_or_else(|| publish_gate::from_lifecycle(&lc));
                virtual_docs::render_status(&lc, &gate)
            }
            virtual_docs::URI_EVIDENCE => virtual_docs::render_evidence(&lc),
            virtual_docs::URI_NEXT_STEP => {
                let gate = cache.as_ref()
                    .map(|c| c.gate.clone())
                    .unwrap_or_else(|| publish_gate::from_lifecycle(&lc));
                virtual_docs::render_next_step(&lc, &gate)
            }
            virtual_docs::URI_BOUNDS_REPORT => {
                let report = cache.as_ref()
                    .map(|c| c.bounds_report.clone())
                    .unwrap_or_default();
                virtual_docs::render_bounds_report(&report)
            }
            virtual_docs::URI_DOMAIN => {
                cache.as_ref().map(|c| c.projection.domain_text.clone())
                    .unwrap_or_else(projection::emit_domain)
            }
            virtual_docs::URI_PROBLEM => {
                cache.as_ref().map(|c| c.projection.problem_text.clone())
                    .unwrap_or_else(|| projection::emit_problem(&lc))
            }
            virtual_docs::URI_PLAN | virtual_docs::URI_TAPE => {
                if let Some(result) = cache.as_ref().and_then(|c| c.admission.as_ref()) {
                    virtual_docs::render_plan(result)
                } else if let Some(candidate) = cache.as_ref().and_then(|c| c.candidate.as_ref()) {
                    virtual_docs::render_plan_candidate(candidate)
                } else {
                    r#"{"status":"NO_PLAN"}"#.into()
                }
            }
            virtual_docs::URI_LOG => {
                cache.as_ref().and_then(|c| c.admission.as_ref())
                    .map(virtual_docs::render_log)
                    .unwrap_or_else(|| r#"{"status":"CANDIDATE"}"#.into())
            }
            virtual_docs::URI_RECEIPT => {
                cache.as_ref().and_then(|c| c.admission.as_ref())
                    .map(virtual_docs::render_receipt)
                    .unwrap_or_else(|| r#"{"status":"CANDIDATE"}"#.into())
            }
            virtual_docs::URI_OCEL => {
                cache.as_ref().and_then(|c| c.admission.as_ref())
                    .map(virtual_docs::render_ocel)
                    .unwrap_or_else(|| r#"{"status":"CANDIDATE"}"#.into())
            }
            virtual_docs::URI_PUBLISH_GATE => {
                let gate = cache.as_ref()
                    .map(|c| c.gate.clone())
                    .unwrap_or_else(|| publish_gate::from_lifecycle(&lc));
                virtual_docs::render_publish_gate(&gate)
            }
            virtual_docs::URI_BUILD_BROKER => virtual_docs::render_build_broker(&broker),
            virtual_docs::URI_AGENT_ASSIGNMENTS => {
                let gate = cache.as_ref()
                    .map(|c| c.gate.clone())
                    .unwrap_or_else(|| publish_gate::from_lifecycle(&lc));
                virtual_docs::render_agent_assignments(&lc, &gate)
            }
            _ => return None,
        })
    }
}

#[lsp_max::async_trait]
impl LanguageServer for PddlLspBackend {
    async fn initialize(&self, params: InitializeParams) -> lsp_max::jsonrpc::Result<InitializeResult> {
        // Detect workspace root
        if let Some(uri) = params.root_uri {
            if let Some(path) = uri_to_path(&uri) {
                *self.workspace_root.lock().await = Some(path);
            }
        } else if let Some(folders) = params.workspace_folders {
            if let Some(folder) = folders.first() {
                if let Some(path) = uri_to_path(&folder.uri) {
                    *self.workspace_root.lock().await = Some(path);
                }
            }
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec![
                        // Projection mode
                        "bcinrPddl.refreshLifecycle".into(),
                        "bcinrPddl.runPlan".into(),
                        "bcinrPddl.generateProjection".into(),
                        // Admission mode
                        "bcinrPddl.executeTape".into(),
                        // Virtual docs
                        "bcinrPddl.openVirtualDocument".into(),
                        // Lifecycle repair
                        "bcinrPddl.createPrd".into(),
                        "bcinrPddl.createArd".into(),
                        "bcinrPddl.createAdr".into(),
                        "bcinrPddl.generateWorkUnits".into(),
                        "bcinrPddl.deriveArd".into(),
                        // Gate
                        "bcinrPddl.explainPublishGate".into(),
                        "bcinrPddl.verifyReceipt".into(),
                        // Bounds
                        "bcinrPddl.splitNeed9".into(),
                        // Build broker
                        "bcinrPddl.requestBuildSlot".into(),
                        "bcinrPddl.releaseBuildSlot".into(),
                        "bcinrPddl.wrapHeavyCommand".into(),
                        // OCEL
                        "bcinrPddl.emitOcelSnapshot".into(),
                    ],
                    work_done_progress_options: Default::default(),
                }),
                ..ServerCapabilities::default()
            },
            server_info: Some(ServerInfo {
                name: "bcinr-pddl-lsp".into(),
                version: Some("26.6.26".into()),
            }),
            offset_encoding: None,
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "bcinr-pddl-lsp CANDIDATE — lifecycle scanner OPEN")
            .await;
    }

    async fn shutdown(&self) -> lsp_max::jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri_str = params.text_document.uri.to_string();
        self.documents.insert(uri_str, params.text_document.text);
        self.project_and_cache(params.text_document.uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.into_iter().next() {
            let uri_str = params.text_document.uri.to_string();
            self.documents.insert(uri_str, change.text);
        }
        self.project_and_cache(params.text_document.uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.project_and_cache(params.text_document.uri).await;
    }

    async fn code_action(
        &self,
        params: CodeActionParams,
    ) -> lsp_max::jsonrpc::Result<Option<CodeActionResponse>> {
        let codes: Vec<String> = params
            .context
            .diagnostics
            .iter()
            .filter_map(|d| match &d.code {
                Some(NumberOrString::String(s)) => Some(s.clone()),
                _ => None,
            })
            .collect();

        let actions = code_actions::for_diagnostics(&codes, &params.range);
        Ok(Some(actions))
    }

    async fn hover(&self, params: HoverParams) -> lsp_max::jsonrpc::Result<Option<Hover>> {
        let uri_str = params.text_document_position_params.text_document.uri.to_string();

        // If hovering on a bcinr-pddl:// virtual document, render it
        if uri_str.starts_with("bcinr-pddl://") {
            if let Some(content) = self.render_virtual_doc(&uri_str).await {
                return Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!("```json\n{content}\n```"),
                    }),
                    range: None,
                }));
            }
        }

        Ok(None)
    }

    async fn execute_command(
        &self,
        params: ExecuteCommandParams,
    ) -> lsp_max::jsonrpc::Result<Option<serde_json::Value>> {
        match params.command.as_str() {
            // Projection mode: safe, always runs on command
            "bcinrPddl.refreshLifecycle" | "bcinrPddl.runPlan" | "bcinrPddl.generateProjection" => {
                let root = self.root().await;
                let uri_str = format!("file://{}", root.to_string_lossy());
                let uri: Uri = uri_str.parse().unwrap_or_else(|_| {
                    "file:///workspace".parse().unwrap()
                });
                self.project_and_cache(uri).await;
                Ok(Some(serde_json::json!({"status": "CANDIDATE"})))
            }
            // Admission mode: explicit only — executes tape, emits receipt + OCEL
            "bcinrPddl.executeTape" => {
                let root = self.root().await;
                let uri_str = format!("file://{}", root.to_string_lossy());
                let uri: Uri = uri_str.parse().unwrap_or_else(|_| {
                    "file:///workspace".parse().unwrap()
                });
                self.execute_tape_and_admit(uri).await;
                let gate_label = {
                    let cache = self.plan_cache.lock().await;
                    cache.as_ref().map(|c| c.gate.status_label().to_string()).unwrap_or_default()
                };
                Ok(Some(serde_json::json!({"status": gate_label})))
            }
            "bcinrPddl.openVirtualDocument" => {
                let uri_str = params.arguments
                    .first()
                    .and_then(|v| v.as_str())
                    .unwrap_or(virtual_docs::URI_STATUS);
                let content = self.render_virtual_doc(uri_str).await
                    .unwrap_or_else(|| r#"{"error":"unknown virtual document"}"#.into());
                Ok(Some(serde_json::json!({ "content": content })))
            }
            "bcinrPddl.explainPublishGate" => {
                let content = self.render_virtual_doc(virtual_docs::URI_PUBLISH_GATE).await
                    .unwrap_or_default();
                Ok(Some(serde_json::json!({ "content": content })))
            }
            "bcinrPddl.splitNeed9" => {
                Ok(Some(serde_json::json!({
                    "status": "CANDIDATE",
                    "message": "Need9: decompose work package into ≤8 tasks. Edit docs/work-units.md."
                })))
            }
            "bcinrPddl.createPrd" => {
                Ok(Some(serde_json::json!({
                    "status": "CANDIDATE",
                    "template": "# PRD\n\n## Status: CANDIDATE\n\n## Intent\n\n## Goals\n\n## Non-Goals\n"
                })))
            }
            "bcinrPddl.createArd" | "bcinrPddl.deriveArd" => {
                Ok(Some(serde_json::json!({
                    "status": "CANDIDATE",
                    "template": "# ARD\n\n## Status: CANDIDATE\n\n## Architecture Thesis\n\n## Modules\n"
                })))
            }
            "bcinrPddl.createAdr" => {
                Ok(Some(serde_json::json!({
                    "status": "CANDIDATE",
                    "template": "# ADR-001: Title\n\n## Status: CANDIDATE\n\n## Context\n\n## Decision\n\n## Consequences\n",
                    "path": "docs/adr/001-title.md"
                })))
            }
            "bcinrPddl.generateWorkUnits" => {
                Ok(Some(serde_json::json!({
                    "status": "CANDIDATE",
                    "template": "# Work Units\n\n## Unit 1: Name (≤8 tasks)\n\n- [ ] Task 1\n- [ ] Task 2\n",
                    "path": "docs/work-units.md"
                })))
            }
            "bcinrPddl.verifyReceipt" => {
                let root = self.root().await;
                let receipt_path = root.join(".bcinr/receipts/latest.json");
                if receipt_path.exists() {
                    match std::fs::read_to_string(&receipt_path) {
                        Ok(content) => {
                            let goal_reached = content.contains("\"goal_reached\": true");
                            Ok(Some(serde_json::json!({
                                "status": if goal_reached { "ADMITTED" } else { "REFUSED" },
                                "receipt_path": receipt_path.to_string_lossy(),
                                "goal_reached": goal_reached,
                            })))
                        }
                        Err(e) => Ok(Some(serde_json::json!({
                            "status": "RECEIPT_INTEGRITY_ERROR",
                            "error": e.to_string(),
                        }))),
                    }
                } else {
                    Ok(Some(serde_json::json!({"status": "CANDIDATE", "error": "no receipt found"})))
                }
            }
            "bcinrPddl.requestBuildSlot" => {
                let cmd = params.arguments.first().and_then(|v| v.as_str()).unwrap_or("build");
                let mut broker = self.broker.lock().await;
                match broker.request_slot(cmd) {
                    Ok(()) => Ok(Some(serde_json::json!({"status": "AVAILABLE", "command": cmd}))),
                    Err(e) => Ok(Some(serde_json::json!({"status": "BUILD_SLOT_DENIED", "reason": e.reason}))),
                }
            }
            "bcinrPddl.releaseBuildSlot" => {
                let mut broker = self.broker.lock().await;
                broker.release_slot();
                Ok(Some(serde_json::json!({"status": "RELEASED"})))
            }
            "bcinrPddl.wrapHeavyCommand" => {
                let cmd = params.arguments.first().and_then(|v| v.as_str()).unwrap_or("");
                if build_broker::is_heavy_command(cmd) {
                    Ok(Some(serde_json::json!({
                        "status": "CANDIDATE",
                        "wrapped": format!("bcinrPddl.requestBuildSlot && {cmd} && bcinrPddl.releaseBuildSlot"),
                        "note": "Route heavy commands through the build broker to advance lifecycle."
                    })))
                } else {
                    Ok(Some(serde_json::json!({"status": "OK", "note": "Not a heavy command — no broker needed"})))
                }
            }
            "bcinrPddl.emitOcelSnapshot" => {
                let content = self.render_virtual_doc(virtual_docs::URI_OCEL).await
                    .unwrap_or_else(|| r#"{"status":"CANDIDATE"}"#.into());
                Ok(Some(serde_json::json!({"status": "CANDIDATE", "ocel": content})))
            }
            _ => Ok(None),
        }
    }
}
