#![allow(clippy::cargo_common_metadata)]
//! Library surface for `bcinr-mcp`, exposing only what benches/tests outside
//! the binary crate need (the MCP+ admission-key cache) — the MCP server
//! itself remains a binary (`src/main.rs`), not a library API.
pub mod cache;
