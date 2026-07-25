import re

with open('crates/bcinr-mcp/src/main.rs', 'r') as f:
    main = f.read()

old_b = """    async fn bitset_operations(&self, Parameters(input): Parameters<BitsetInput>) -> String {
        let v = input.value;
        let result = match input.operation.as_str() {
            "popcount" => v.count_ones() as u64,
            "leading_zeros" => v.leading_zeros() as u64,
            "trailing_zeros" => v.trailing_zeros() as u64,
            "msb" => {
                if v == 0 {
                    u64::MAX
                } else {
                    63 - v.leading_zeros() as u64
                }
            }
            "lsb" => {
                if v == 0 {
                    u64::MAX
                } else {
                    v.trailing_zeros() as u64
                }
            }
            _ => {
                return serde_json::json!({ "ok": false, "error": "unknown operation" }).to_string()
            }
        };
        serde_json::json!({
            "ok": true,
            "operation": input.operation,
            "value": v,
            "result": result,
        })
        .to_string()
    }"""
new_b = """    async fn bitset_operations(&self, Parameters(input): Parameters<BitsetInput>) -> String {
        let result = Self::compute_bitset_op(input.value, &input.operation);
        match result {
            Some(res) => serde_json::json!({
                "ok": true,
                "operation": input.operation,
                "value": input.value,
                "result": res,
            }).to_string(),
            None => serde_json::json!({ "ok": false, "error": "unknown operation" }).to_string(),
        }
    }
}
impl BcinrMcpServer {
    fn compute_bitset_op(v: u64, op: &str) -> Option<u64> {
        match op {
            "popcount" => Some(v.count_ones() as u64),
            "leading_zeros" => Some(v.leading_zeros() as u64),
            "trailing_zeros" => Some(v.trailing_zeros() as u64),
            "msb" => Some(if v == 0 { u64::MAX } else { 63 - v.leading_zeros() as u64 }),
            "lsb" => Some(if v == 0 { u64::MAX } else { v.trailing_zeros() as u64 }),
            _ => None,
        }
    }"""
main = main.replace(old_b, new_b)

with open('crates/bcinr-mcp/src/main.rs', 'w') as f:
    f.write(main)

