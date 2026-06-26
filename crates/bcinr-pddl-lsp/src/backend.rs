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
    code_actions,
    diagnostics as diag_mod,
    lifecycle,
    planner_client,
    projection,
    publish_gate,
    virtual_docs,
};

/// Cached plan result for a workspace root.
struct CachedPlan {
    projection: projection::Pddl8Projection,
    result: Option<planner_client::PlanResult>,
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
}

impl PddlLspBackend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: DashMap::new(),
            plan_cache: Arc::new(Mutex::new(None)),
            workspace_root: Arc::new(Mutex::new(None)),
        }
    }

    async fn root(&self) -> PathBuf {
        self.workspace_root.lock().await
            .clone()
            .unwrap_or_else(|| PathBuf::from("."))
    }

    /// Full analysis cycle: scan → project → plan → publish gate → diagnostics.
    async fn analyze_and_publish(&self, trigger_uri: Uri) {
        let root = self.root().await;
        let lc = lifecycle::scan(&root);

        // Bound check on lifecycle
        let bound_violations = bounds::check_lifecycle_domain();

        // Projection
        let proj = projection::project(&lc);

        // Attempt planning
        let case_id = format!("lsp-{}", lc.project_name);
        let plan_result = planner_client::plan_and_execute(&proj, &case_id);

        let gate = match &plan_result {
            Ok(r) => publish_gate::from_plan_result(&lc, r),
            Err(_) => publish_gate::from_lifecycle(&lc),
        };

        // Update cache
        {
            let mut cache = self.plan_cache.lock().await;
            *cache = Some(CachedPlan {
                projection: proj.clone(),
                result: plan_result.ok(),
                gate: gate.clone(),
            });
        }

        // Collect all diagnostics
        let mut all_diags = diag_mod::lifecycle_diagnostics(&lc);
        all_diags.extend(diag_mod::bound_diagnostics(&bound_violations));

        // If planner failed and lifecycle is otherwise complete, surface that too
        {
            let cache = self.plan_cache.lock().await;
            if let Some(ref c) = *cache {
                if c.result.is_none() && lc.missing.len() <= 2 {
                    // Only surface planner error when we're close to publish
                    // (avoid noise when lifecycle is far from complete)
                }
            }
        }

        self.client.publish_diagnostics(trigger_uri, all_diags, None).await;

        // Emit semantic notifications via window/showMessage for gate changes
        let gate_msg = format!(
            "bcinr-pddl-lsp: project '{}' publish gate = {}. Next: {:?}",
            lc.project_name,
            gate.status_label(),
            lc.next_missing().map(|s| s.predicate_name())
        );
        self.client.log_message(MessageType::INFO, gate_msg).await;
    }

    /// Render a virtual document by URI scheme.
    async fn render_virtual_doc(&self, uri_str: &str) -> Option<String> {
        let root = self.root().await;
        let lc = lifecycle::scan(&root);
        let cache = self.plan_cache.lock().await;

        Some(match uri_str {
            virtual_docs::URI_LIFECYCLE => virtual_docs::render_lifecycle(&lc),
            virtual_docs::URI_STATUS => {
                let gate = cache.as_ref()
                    .map(|c| c.gate.clone())
                    .unwrap_or_else(|| publish_gate::from_lifecycle(&lc));
                virtual_docs::render_status(&lc, &gate)
            }
            virtual_docs::URI_DOMAIN => {
                cache.as_ref().map(|c| c.projection.domain_text.clone())
                    .unwrap_or_else(|| projection::emit_domain())
            }
            virtual_docs::URI_PROBLEM => {
                cache.as_ref().map(|c| c.projection.problem_text.clone())
                    .unwrap_or_else(|| projection::emit_problem(&lc))
            }
            virtual_docs::URI_PLAN | virtual_docs::URI_TAPE => {
                cache.as_ref().and_then(|c| c.result.as_ref())
                    .map(virtual_docs::render_plan)
                    .unwrap_or_else(|| r#"{"status":"NO_ADMITTED_PLAN"}"#.into())
            }
            virtual_docs::URI_LOG => {
                cache.as_ref().and_then(|c| c.result.as_ref())
                    .map(virtual_docs::render_log)
                    .unwrap_or_else(|| r#"{"status":"CANDIDATE"}"#.into())
            }
            virtual_docs::URI_RECEIPT => {
                cache.as_ref().and_then(|c| c.result.as_ref())
                    .map(virtual_docs::render_receipt)
                    .unwrap_or_else(|| r#"{"status":"CANDIDATE"}"#.into())
            }
            virtual_docs::URI_OCEL => {
                cache.as_ref().and_then(|c| c.result.as_ref())
                    .map(virtual_docs::render_ocel)
                    .unwrap_or_else(|| r#"{"status":"CANDIDATE"}"#.into())
            }
            virtual_docs::URI_PUBLISH_GATE => {
                let gate = cache.as_ref()
                    .map(|c| c.gate.clone())
                    .unwrap_or_else(|| publish_gate::from_lifecycle(&lc));
                virtual_docs::render_publish_gate(&gate)
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
                        "bcinrPddl.generateProjection".into(),
                        "bcinrPddl.runPlan".into(),
                        "bcinrPddl.executeTape".into(),
                        "bcinrPddl.openVirtualDocument".into(),
                        "bcinrPddl.explainPublishGate".into(),
                        "bcinrPddl.splitNeed9".into(),
                        "bcinrPddl.refreshLifecycle".into(),
                        "bcinrPddl.createPrd".into(),
                        "bcinrPddl.deriveArd".into(),
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
        self.analyze_and_publish(params.text_document.uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.into_iter().next() {
            let uri_str = params.text_document.uri.to_string();
            self.documents.insert(uri_str, change.text);
        }
        self.analyze_and_publish(params.text_document.uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.analyze_and_publish(params.text_document.uri).await;
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
            "bcinrPddl.refreshLifecycle" | "bcinrPddl.runPlan" | "bcinrPddl.executeTape"
            | "bcinrPddl.generateProjection" => {
                let root = self.root().await;
                // Trigger re-analysis using workspace root as a synthetic URI
                let uri_str = format!("file://{}", root.to_string_lossy());
                let uri: Uri = uri_str.parse().unwrap_or_else(|_| {
                    "file:///workspace".parse().unwrap()
                });
                self.analyze_and_publish(uri).await;
                Ok(Some(serde_json::json!({"status": "CANDIDATE"})))
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
                    "message": "Need9 split: decompose work package into ≤8 tasks. Edit docs/work-units.md."
                })))
            }
            "bcinrPddl.createPrd" => {
                Ok(Some(serde_json::json!({
                    "status": "CANDIDATE",
                    "template": "# PRD\n\n## Status: CANDIDATE\n\n## Intent\n\n## Goals\n\n## Non-Goals\n"
                })))
            }
            "bcinrPddl.deriveArd" => {
                Ok(Some(serde_json::json!({
                    "status": "CANDIDATE",
                    "template": "# ARD\n\n## Status: CANDIDATE\n\n## Architecture Thesis\n\n## Modules\n"
                })))
            }
            _ => Ok(None),
        }
    }
}
