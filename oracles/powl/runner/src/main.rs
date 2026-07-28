//! Run `bcinr-powl`'s Algorithm 3 over the shared corpus and emit the same
//! record shape the Python oracle does, so the two can be compared line for
//! line.
//!
//! Both sides read `oracles/powl/cases.json`. That is the point: two
//! hand-maintained input lists drift, and a differential over drifted inputs
//! silently stops comparing anything.

use bcinr_powl::wf_net::WfNet;
use bcinr_powl::wf_to_powl::convert;

fn main() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../cases.json");
    let raw = std::fs::read_to_string(path).expect("shared corpus must be readable");
    let corpus: serde_json::Value = serde_json::from_str(&raw).expect("corpus must be valid JSON");

    for case in corpus["cases"].as_array().expect("cases must be an array") {
        let name = case["name"].as_str().expect("case needs a name");

        let places = case["places"]
            .as_array()
            .expect("places")
            .iter()
            .map(|p| p.as_str().expect("place name").to_string());
        let transitions = case["transitions"]
            .as_array()
            .expect("transitions")
            .iter()
            .map(|t| {
                let pair = t.as_array().expect("transition is [name, label]");
                (
                    pair[0].as_str().expect("transition name").to_string(),
                    pair[1].as_str().map(str::to_string),
                )
            });
        let arcs = |key: &str| {
            case[key]
                .as_array()
                .expect("arc list")
                .iter()
                .map(|a| {
                    let pair = a.as_array().expect("arc is [from, to]");
                    (
                        pair[0].as_str().expect("arc source").to_string(),
                        pair[1].as_str().expect("arc target").to_string(),
                    )
                })
                .collect::<Vec<_>>()
        };

        // A net the corpus declares but that fails WfNet::new is reported, not
        // skipped: structural rejection is a verdict too, and the oracle may
        // well accept it.
        let record = match WfNet::new(places, transitions, arcs("pt"), arcs("tp"), "i", "o") {
            Err(e) => format!(r#"{{"name":"{name}","verdict":"refused","detail":"NetError: {e}"}}"#),
            Ok(net) => match convert(&net) {
                Ok(model) => {
                    let detail = format!("{model:?}");
                    let detail: String = detail.chars().take(120).collect();
                    format!(r#"{{"name":"{name}","verdict":"converted","detail":"{}"}}"#,
                            detail.replace('"', "'"))
                }
                Err(refusal) => format!(
                    r#"{{"name":"{name}","verdict":"refused","detail":"{}"}}"#,
                    format!("{:?}", refusal.reason).replace('"', "'")
                ),
            },
        };
        println!("{record}");
    }
}
