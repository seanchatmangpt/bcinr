mod backend;
mod bounds;
mod build_broker;
mod code_actions;
mod diagnostics;
mod lifecycle;
mod planner_client;
mod projection;
mod publish_gate;
mod virtual_docs;

use backend::PddlLspBackend;
use lsp_max::{LspService, Server};

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(PddlLspBackend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
