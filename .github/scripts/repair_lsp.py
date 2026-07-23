#!/usr/bin/env python3
"""One-shot deterministic migration from the upstream ANDON stub to BCINR's local engine."""

from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    if old not in text:
        if new in text:
            return text
        raise SystemExit(f"{label}: expected source text not found")
    return text.replace(old, new, 1)


andon_path = Path("crates/bcinr-pddl-lsp/src/andon_bus/mod.rs")
andon = andon_path.read_text()
if "use serde::{Deserialize, Serialize};" not in andon:
    andon = replace_once(
        andon,
        "use lsp_types_max::{Diagnostic, DiagnosticSeverity, MessageType, Position, Range};\n",
        "use lsp_types_max::{Diagnostic, DiagnosticSeverity, MessageType, Position, Range};\nuse serde::{Deserialize, Serialize};\n",
        "andon serde import",
    )
andon = replace_once(
    andon,
    "#[derive(Debug, Clone, PartialEq)]\npub enum AndonSeverity",
    "#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]\npub enum AndonSeverity",
    "andon severity derives",
)
andon = replace_once(
    andon,
    "#[derive(Debug, Clone)]\npub struct AndonEvent",
    "#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct AndonEvent",
    "andon event derives",
)
anchor = "}\n\npub struct AndonAnalysis {"
insertion = '''}

#[derive(Debug, Default)]
pub struct AndonBus {
    events: Vec<AndonEvent>,
}

impl AndonBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, event: AndonEvent) {
        self.events.push(event);
    }

    pub fn events(&self) -> &[AndonEvent] {
        &self.events
    }

    pub fn drain(&mut self) -> Vec<AndonEvent> {
        std::mem::take(&mut self.events)
    }
}

/// Typed custom notification for the BCINR ANDON stream.
pub struct BcinrPddlAndonRaised;

impl lsp_types_max::notification::Notification for BcinrPddlAndonRaised {
    type Params = AndonEvent;
    const METHOD: &'static str = "bcinrPddl/andonRaised";
}

pub struct AndonAnalysis {'''
if "pub struct AndonBus" not in andon:
    andon = replace_once(andon, anchor, insertion, "ANDON bus insertion")
andon_path.write_text(andon)

backend_path = Path("crates/bcinr-pddl-lsp/src/backend.rs")
backend = backend_path.read_text()
old_imports = '''use lsp_max_andon::analysis::AnalysisPipeline;
use lsp_max_andon::andon::{AndonBus, AndonEvent};
use lsp_max_andon::core::InvariantRegistry;
use lsp_max_andon::lsp::{LspMaxAndonRaised, LspPushAdapter};
use lsp_max_andon::patterns::{
    build_brokered_command, build_empty_registry_invariant, build_marker_admission,
    build_need_n_invariant, build_non_empty_check_set, build_receipt_required,
    build_required_artifact_invariant,
};
'''
new_imports = '''use crate::andon_bus::{
    self, AndonAnalysis, AndonBus, AndonEvent, BcinrPddlAndonRaised,
};
'''
backend = replace_once(backend, old_imports, new_imports, "backend imports")
backend = replace_once(
    backend,
    "    pub registry: Arc<StdMutex<InvariantRegistry>>,\n    pub andon_bus: Arc<StdMutex<AndonBus>>,\n",
    "    pub andon_bus: Arc<StdMutex<AndonBus>>,\n",
    "backend fields",
)
registry_setup = '''        let mut registry = InvariantRegistry::new();
        registry.register(build_empty_registry_invariant());
        registry.register(build_required_artifact_invariant("docs/prd.md"));
        registry.register(build_marker_admission("ADMITTED"));
        registry.register(build_need_n_invariant(8));
        registry.register(build_non_empty_check_set());
        registry.register(build_brokered_command());
        registry.register(build_receipt_required());

'''
backend = replace_once(backend, registry_setup, "", "registry setup")
backend = replace_once(
    backend,
    "            registry: Arc::new(StdMutex::new(registry)),\n            andon_bus: Arc::new(StdMutex::new(AndonBus::new())),\n",
    "            andon_bus: Arc::new(StdMutex::new(AndonBus::new())),\n",
    "backend initializer",
)
backend = replace_once(
    backend,
    "cache.as_ref().map(|c| c.gate.clone()).unwrap_or(gate)",
    "cache.as_ref().map(|c| c.gate.clone()).unwrap_or_else(|| gate.clone())",
    "preserved gate",
)
backend = replace_once(
    backend,
    "            } else {\n                gate\n",
    "            } else {\n                gate.clone()\n",
    "candidate gate clone",
)
old_analysis = '''        let andon_events = {
            let registry = self.registry.lock().unwrap();
            AnalysisPipeline::evaluate_registry(&registry)
        };

        for event in andon_events {
            // Andon events are pushed as custom LSP notifications, not as standard diagnostics
            self.push_andon(event).await;
        }
'''
new_analysis = '''        let broker_snapshot = self.broker.lock().await.clone();
        let analysis = AndonAnalysis {
            lifecycle: lc.clone(),
            bounds_report: bounds_report.clone(),
            plan_candidate: None,
            gate: gate.clone(),
            broker: broker_snapshot,
        };

        for event in andon_bus::derive_events(&analysis) {
            all_diags.push(andon_bus::to_lsp_diagnostic(&event));
            self.push_andon(event).await;
        }
'''
backend = replace_once(backend, old_analysis, new_analysis, "backend analysis")
backend = replace_once(
    backend,
    "send_notification::<LspMaxAndonRaised>",
    "send_notification::<BcinrPddlAndonRaised>",
    "notification marker",
)
if "lsp_max_andon" in backend or "AnalysisPipeline" in backend:
    raise SystemExit("stub ANDON references remain in backend")
backend_path.write_text(backend)

manifest_path = Path("crates/bcinr-pddl-lsp/Cargo.toml")
manifest = manifest_path.read_text()
manifest = "\n".join(
    line for line in manifest.splitlines() if not line.startswith("lsp-max-andon = ")
) + "\n"
manifest_path.write_text(manifest)
