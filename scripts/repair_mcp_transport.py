#!/usr/bin/env python3
"""Install a strict newline-framed stdio pre-parser for bcinr-mcp."""

from pathlib import Path


def replace_once(path: Path, old: str, new: str) -> None:
    source = path.read_text()
    count = source.count(old)
    if count != 1:
        raise RuntimeError(f"{path}: expected one replacement, found {count}")
    path.write_text(source.replace(old, new, 1))


manifest = Path("crates/bcinr-mcp/Cargo.toml")
replace_once(
    manifest,
    'rmcp = { version = "2.0.0", features = ["server", "transport-io", "macros"] }',
    'rmcp = { version = "2.0.0", features = ["server", "transport-io", "transport-async-rw", "macros"] }',
)

main = Path("crates/bcinr-mcp/src/main.rs")
replace_once(
    main,
    '''use rmcp::{
    handler::server::wrapper::Parameters, schemars, tool, tool_router, transport::stdio, ServiceExt,
};''',
    '''use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router, ServiceExt};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};''',
)

entry = '''// ─── Entry point ─────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {'''
strict_transport = '''// ─── Entry point ─────────────────────────────────────────────────────────────

/// Build a strict newline-delimited stdio ingress.
///
/// rmcp 2.2 intentionally ignores syntactically unparsable input. BCINR's
/// protocol contract is stricter: malformed JSON must receive JSON-RPC -32700.
/// Valid JSON frames are forwarded unchanged to rmcp for normal MCP handling.
fn strict_stdio() -> (tokio::io::DuplexStream, tokio::io::Stdout) {
    let (rmcp_read, mut ingress_write) = tokio::io::duplex(64 * 1024);

    tokio::spawn(async move {
        let mut lines = BufReader::new(tokio::io::stdin()).lines();
        let mut error_output = tokio::io::stdout();

        loop {
            let line = match lines.next_line().await {
                Ok(Some(line)) => line,
                Ok(None) => break,
                Err(error) => {
                    tracing::error!("strict stdio ingress failed: {error}");
                    break;
                }
            };

            if serde_json::from_str::<serde_json::Value>(&line).is_err() {
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32700,
                        "message": "Parse error"
                    },
                    "id": serde_json::Value::Null
                });
                let mut encoded = response.to_string();
                encoded.push('\n');
                if error_output.write_all(encoded.as_bytes()).await.is_err()
                    || error_output.flush().await.is_err()
                {
                    break;
                }
                continue;
            }

            if ingress_write.write_all(line.as_bytes()).await.is_err()
                || ingress_write.write_all(b"\n").await.is_err()
                || ingress_write.flush().await.is_err()
            {
                break;
            }
        }
    });

    (rmcp_read, tokio::io::stdout())
}

#[tokio::main]
async fn main() {'''
replace_once(main, entry, strict_transport)
replace_once(main, "server.serve(stdio()).await", "server.serve(strict_stdio()).await")
