import re

# 1. cache.rs: remove cfg(test)
with open('crates/bcinr-mcp/src/cache.rs', 'r') as f:
    cache = f.read()
cache = cache.replace('#[cfg(test)]', '')
with open('crates/bcinr-mcp/src/cache.rs', 'w') as f:
    f.write(cache)

# main.rs refactoring
with open('crates/bcinr-mcp/src/main.rs', 'r') as f:
    main = f.read()

# 2. de_u64_flex
old_de = """fn de_u64_flex<'de, D: serde::Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
    use serde::de::Error as _;
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum NumOrStr {
        Int(u64),
        Float(f64),
        Str(String),
    }
    match NumOrStr::deserialize(d)? {
        NumOrStr::Int(n) => Ok(n),
        NumOrStr::Float(f) => {
            if f >= 0.0 && f <= u64::MAX as f64 && f.fract() == 0.0 {
                Ok(f as u64)
            } else {
                Err(D::Error::custom(format!("{f} is not a valid u64")))
            }
        }
        NumOrStr::Str(s) => s.parse::<u64>().map_err(D::Error::custom),
    }
}"""
new_de = """#[derive(Deserialize)]
#[serde(untagged)]
enum NumOrStrFlex {
    Int(u64),
    Float(f64),
    Str(String),
}

fn flex_float_to_u64<E: serde::de::Error>(f: f64) -> Result<u64, E> {
    if f >= 0.0 && f <= u64::MAX as f64 && f.fract() == 0.0 {
        Ok(f as u64)
    } else {
        Err(E::custom(format!("{f} is not a valid u64")))
    }
}

fn de_u64_flex<'de, D: serde::Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
    use serde::de::Error as _;
    match NumOrStrFlex::deserialize(d)? {
        NumOrStrFlex::Int(n) => Ok(n),
        NumOrStrFlex::Float(f) => flex_float_to_u64(f),
        NumOrStrFlex::Str(s) => s.parse::<u64>().map_err(D::Error::custom),
    }
}"""
main = main.replace(old_de, new_de)

# 3. pddl_domain_info
old_pddl = """    async fn pddl_domain_info(&self, Parameters(input): Parameters<DomainInput>) -> String {
        // Try PDDL 3.1 parser first (richer info), fall back to STRIPS8
        match bcinr_pddl::domain31_from_pddl(&input.domain_text) {
            Ok(d) => {
                let mut out = format!("Domain: {}\n", d.name);
                if !d.requirements.is_empty() {
                    out.push_str(&format!("Requirements: {}\n", d.requirements.join(", ")));
                }
                out.push_str(&format!("Predicates ({}):\n", d.predicates.len()));
                for (name, params) in &d.predicates {
                    out.push_str(&format!("  {name}/{}\n", params.len()));
                }
                out.push_str(&format!("Actions ({}):\n", d.actions.len()));
                for a in &d.actions {
                    let params: Vec<String> =
                        a.params.iter().map(|(v, t)| format!("{v}: {t}")).collect();
                    out.push_str(&format!("  {}({})\n", a.name, params.join(", ")));
                }
                if !d.durative_actions.is_empty() {
                    out.push_str(&format!(
                        "Durative Actions ({}):\n",
                        d.durative_actions.len()
                    ));
                    for da in &d.durative_actions {
                        out.push_str(&format!("  {}\n", da.name));
                    }
                }
                out
            }
            Err(_) => match bcinr_pddl::domain_from_pddl(&input.domain_text) {
                Ok(d) => {
                    let mut out = format!("Domain: {}\n", d.name);
                    out.push_str(&format!("Predicates ({}):\n", d.predicates.len()));
                    for (pname, arity) in &d.predicates {
                        out.push_str(&format!("  {pname}/{arity}\n"));
                    }
                    out.push_str(&format!("Actions ({}):\n", d.actions.len()));
                    for a in &d.actions {
                        let params = a.params.join(", ");
                        out.push_str(&format!("  {}({})\n", a.name, params));
                    }
                    out
                }
                Err(e) => format!("Error parsing domain: {e}"),
            },
        }
    }"""
new_pddl = """    async fn pddl_domain_info(&self, Parameters(input): Parameters<DomainInput>) -> String {
        match bcinr_pddl::domain31_from_pddl(&input.domain_text) {
            Ok(d) => Self::format_domain31(&d),
            Err(_) => match bcinr_pddl::domain_from_pddl(&input.domain_text) {
                Ok(d) => Self::format_domain_strips(&d),
                Err(e) => format!("Error parsing domain: {e}"),
            },
        }
    }
}
impl BcinrMcpServer {
    fn format_domain31(d: &bcinr_pddl::Domain31) -> String {
        let mut out = format!("Domain: {}\n", d.name);
        if !d.requirements.is_empty() {
            out.push_str(&format!("Requirements: {}\n", d.requirements.join(", ")));
        }
        out.push_str(&format!("Predicates ({}):\n", d.predicates.len()));
        for (name, params) in &d.predicates {
            out.push_str(&format!("  {name}/{}\n", params.len()));
        }
        out.push_str(&format!("Actions ({}):\n", d.actions.len()));
        for a in &d.actions {
            let params: Vec<String> = a.params.iter().map(|(v, t)| format!("{v}: {t}")).collect();
            out.push_str(&format!("  {}({})\n", a.name, params.join(", ")));
        }
        if !d.durative_actions.is_empty() {
            out.push_str(&format!("Durative Actions ({}):\n", d.durative_actions.len()));
            for da in &d.durative_actions {
                out.push_str(&format!("  {}\n", da.name));
            }
        }
        out
    }
    
    fn format_domain_strips(d: &bcinr_pddl::Domain) -> String {
        let mut out = format!("Domain: {}\n", d.name);
        out.push_str(&format!("Predicates ({}):\n", d.predicates.len()));
        for (pname, arity) in &d.predicates {
            out.push_str(&format!("  {pname}/{arity}\n"));
        }
        out.push_str(&format!("Actions ({}):\n", d.actions.len()));
        for a in &d.actions {
            out.push_str(&format!("  {}({})\n", a.name, a.params.join(", ")));
        }
        out
    }"""
main = main.replace(old_pddl, new_pddl)

with open('crates/bcinr-mcp/src/main.rs', 'w') as f:
    f.write(main)

