# Build Broker Protocol
- **NEVER** run heavy commands (`cargo build`, `cargo test`, `npm run build`, `tsc`) directly.
- You **MUST** first invoke the `bcinr_request_build_slot` command/tool.
- Once acquired, run your command via `run_command`.
- Immediately upon completion, invoke `bcinr_release_build_slot`.
