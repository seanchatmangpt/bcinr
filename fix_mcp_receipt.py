import re

with open('crates/bcinr-mcp/src/main.rs', 'r') as f:
    main = f.read()

old_r = """    async fn receipt_inspect(&self, Parameters(input): Parameters<ReceiptInput>) -> String {
        let data = match serde_json::from_str::<serde_json::Value>(&input.receipt_data) {
            Ok(d) => d,
            Err(e) => {
                return serde_json::json!({
                    "status": "error",
                    "reason": format!("invalid JSON: {}", e)
                })
                .to_string()
            }
        };

        let op_count = data["op_count"].as_u64().unwrap_or(0);
        let makespan = data["makespan"].as_f64().unwrap_or(0.0);
        let admitted = data["admitted"].as_bool().unwrap_or(false);
        let refusal_code = data["refusal_code"].as_u64();
        let chain_valid = data["chain_valid"].as_bool().unwrap_or(true);

        let mut out = format!("Receipt Status: {}\n", if admitted { "ADMITTED" } else { "REJECTED" });
        out.push_str(&format!("Chain Valid: {}\n", chain_valid));
        out.push_str(&format!("Operations: {}\n", op_count));
        out.push_str(&format!("Makespan: {}\n", makespan));

        if !admitted {
            if let Some(code) = refusal_code {
                let reason = match code {
                    1 => "Domain violation: undefined capability required",
                    2 => "Temporal overlap in mutually exclusive operations",
                    3 => "Makespan exceeded hard deadline",
                    4 => "Concurrency limit exceeded (guard table rejection)",
                    5 => "Witness validation failed for external state",
                    _ => "Unknown refusal code",
                };
                out.push_str(&format!("Refusal Reason: {} ({})\n", reason, code));
            } else {
                out.push_str("Refusal Reason: (No code provided)\n");
            }
        }

        // Include trace log if present and not admitted
        if !admitted {
            if let Some(trace) = data["trace"].as_array() {
                out.push_str("\nTrace Log:\n");
                for (i, t) in trace.iter().enumerate() {
                    let ts = t["ts_ns"].as_u64().unwrap_or(0);
                    let op = t["op"].as_str().unwrap_or("unknown");
                    let res = t["res"].as_str().unwrap_or("?");
                    out.push_str(&format!("  [{:04}] ns={} op={} res={}\n", i, ts, op, res));
                }
            }
        }

        out
    }"""
new_r = """    async fn receipt_inspect(&self, Parameters(input): Parameters<ReceiptInput>) -> String {
        Self::receipt_inspect_impl(&input.receipt_data)
    }
}
impl BcinrMcpServer {
    fn receipt_inspect_impl(receipt_data: &str) -> String {
        let data = match serde_json::from_str::<serde_json::Value>(receipt_data) {
            Ok(d) => d,
            Err(e) => return serde_json::json!({ "status": "error", "reason": format!("invalid JSON: {}", e) }).to_string(),
        };

        let op_count = data["op_count"].as_u64().unwrap_or(0);
        let makespan = data["makespan"].as_f64().unwrap_or(0.0);
        let admitted = data["admitted"].as_bool().unwrap_or(false);
        let refusal_code = data["refusal_code"].as_u64();
        let chain_valid = data["chain_valid"].as_bool().unwrap_or(true);

        let mut out = format!("Receipt Status: {}\\n", if admitted { "ADMITTED" } else { "REJECTED" });
        out.push_str(&format!("Chain Valid: {}\\n", chain_valid));
        out.push_str(&format!("Operations: {}\\n", op_count));
        out.push_str(&format!("Makespan: {}\\n", makespan));

        if !admitted {
            if let Some(code) = refusal_code {
                let reason = match code {
                    1 => "Domain violation: undefined capability required",
                    2 => "Temporal overlap in mutually exclusive operations",
                    3 => "Makespan exceeded hard deadline",
                    4 => "Concurrency limit exceeded (guard table rejection)",
                    5 => "Witness validation failed for external state",
                    _ => "Unknown refusal code",
                };
                out.push_str(&format!("Refusal Reason: {} ({})\\n", reason, code));
            } else {
                out.push_str("Refusal Reason: (No code provided)\\n");
            }
            if let Some(trace) = data["trace"].as_array() {
                out.push_str("\\nTrace Log:\\n");
                for (i, t) in trace.iter().enumerate() {
                    let ts = t["ts_ns"].as_u64().unwrap_or(0);
                    let op = t["op"].as_str().unwrap_or("unknown");
                    let res = t["res"].as_str().unwrap_or("?");
                    out.push_str(&format!("  [{:04}] ns={} op={} res={}\\n", i, ts, op, res));
                }
            }
        }
        out
    }"""
main = main.replace(old_r, new_r.replace('\\n', '\n'))

with open('crates/bcinr-mcp/src/main.rs', 'w') as f:
    f.write(main)
