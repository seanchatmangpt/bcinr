import re

with open('crates/bcinr-mcp/src/main.rs', 'r') as f:
    main = f.read()

# manufacture_world (LOC 95, depth 5)
old_mw = """    async fn manufacture_world(&self, Parameters(input): Parameters<ManufactureInput>) -> String {
        let domain_ast = match bcinr_pddl::domain_from_pddl(&input.domain_text) {
            Ok(d) => d,
            Err(e) => return format!("Error parsing domain: {e}"),
        };

        let mut out = String::new();
        out.push_str(&format!("(define (problem {})\n", input.problem_name));
        out.push_str(&format!("  (:domain {})\n", domain_ast.name));

        if !input.objects.is_empty() {
            out.push_str("  (:objects\n");
            for (type_name, objs) in &input.objects {
                if type_name == "untyped" {
                    out.push_str(&format!("    {}\n", objs.join(" ")));
                } else {
                    out.push_str(&format!("    {} - {}\n", objs.join(" "), type_name));
                }
            }
            out.push_str("  )\n");
        }

        if !input.init.is_empty() {
            out.push_str("  (:init\n");
            for fact in &input.init {
                let args = if fact.args.is_empty() {
                    String::new()
                } else {
                    format!(" {}", fact.args.join(" "))
                };
                out.push_str(&format!("    ({}{args})\n", fact.predicate));
            }
            out.push_str("  )\n");
        }

        if !input.goal.is_empty() {
            out.push_str("  (:goal (and\n");
            for fact in &input.goal {
                let args = if fact.args.is_empty() {
                    String::new()
                } else {
                    format!(" {}", fact.args.join(" "))
                };
                out.push_str(&format!("    ({}{args})\n", fact.predicate));
            }
            out.push_str("  ))\n");
        }

        out.push_str(")\n");
        out
    }"""
new_mw = """    async fn manufacture_world(&self, Parameters(input): Parameters<ManufactureInput>) -> String {
        let domain_ast = match bcinr_pddl::domain_from_pddl(&input.domain_text) {
            Ok(d) => d,
            Err(e) => return format!("Error parsing domain: {e}"),
        };
        Self::format_manufactured_world(&input, &domain_ast)
    }
}
impl BcinrMcpServer {
    fn format_manufactured_world(input: &ManufactureInput, domain_ast: &bcinr_pddl::Domain) -> String {
        let mut out = String::new();
        out.push_str(&format!("(define (problem {})\n  (:domain {})\n", input.problem_name, domain_ast.name));
        if !input.objects.is_empty() {
            out.push_str("  (:objects\n");
            for (type_name, objs) in &input.objects {
                let joined = objs.join(" ");
                if type_name == "untyped" {
                    out.push_str(&format!("    {}\n", joined));
                } else {
                    out.push_str(&format!("    {} - {}\n", joined, type_name));
                }
            }
            out.push_str("  )\n");
        }
        if !input.init.is_empty() {
            out.push_str("  (:init\n");
            for fact in &input.init {
                let args = if fact.args.is_empty() { String::new() } else { format!(" {}", fact.args.join(" ")) };
                out.push_str(&format!("    ({}{args})\n", fact.predicate));
            }
            out.push_str("  )\n");
        }
        if !input.goal.is_empty() {
            out.push_str("  (:goal (and\n");
            for fact in &input.goal {
                let args = if fact.args.is_empty() { String::new() } else { format!(" {}", fact.args.join(" ")) };
                out.push_str(&format!("    ({}{args})\n", fact.predicate));
            }
            out.push_str("  ))\n");
        }
        out.push_str(")\n");
        out
    }"""
main = main.replace(old_mw, new_mw)

with open('crates/bcinr-mcp/src/main.rs', 'w') as f:
    f.write(main)

