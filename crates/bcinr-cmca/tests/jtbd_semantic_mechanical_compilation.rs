// Named law: bounds explicitly reject mutant divergence
// JTBD 4 (SPECULATIVE, not a confirmed product requirement) — "semantic-to-
// mechanical compilation as a general product": admit meaning (RDF/SHACL/
// generator toolchain) elsewhere, consume only a bounded, verifiable
// artifact here, with zero semantic-toolchain dependency leaking into the
// consuming crate's own dependency graph.
//
// ## What this file validates (Chicago-style: state-based assertions
// against REAL collaborators — no mocks/stubs of CMCA internals)
//
// 1. **Tamper-refusal on ONE real artifact.** Using the REAL
//    `verify_generated_profile` function and `GeneratedManifest`/
//    `GeneratedArtifact` types from `src/artifact.rs`, and the REAL
//    materialized artifact at
//    `generated-artifact/case-studies/{cmca_generation_manifest.json,cmca_generated.rs}`:
//    - a byte-for-byte copy of the real manifest, parsed and verified
//      against the real generated source, is **accepted**
//      (`verify_generated_profile(..) == Ok(())`).
//    - the same manifest with exactly ONE byte of its
//      `generated_payload_digest` hex string flipped is **refused** with
//      the real typed `GeneratedProfileRefusal::PayloadDigestMismatch`
//      variant — not a panic, not a silent accept.
//
// 2. **Absence of semantic-toolchain dependencies in the real dependency
//    graph.** A real `cargo tree -p bcinr-cmca` subprocess is run (via
//    `std::process::Command`, not assumed or described) and its actual
//    stdout is scanned for the crate-name markers of an RDF, Python, or
//    generic-graph-processing toolchain (`oxigraph`, `sophia`, `rio_api`,
//    `pyo3`, `rustpython`, `petgraph`, `networkx`) as well as `mfw` itself.
//    None must appear as a package name in the tree.
//
// ## What this file does NOT validate
//
// - It does **not** validate the "general product" framing for any domain
//   other than CMCA. No PDDL, POWL, pricing, or compliance instantiation of
//   this "admit meaning elsewhere / consume a bounded artifact here"
//   pattern has been built or tested anywhere in this repository. The claim
//   that the pattern generalizes beyond this one CMCA instance is
//   UNVERIFIED by this file.
// - It does not validate every field/check `verify_generated_profile`
//   performs — only the payload-digest tamper path (checks 1–2 of that
//   function's five checks). Schema-version, dimension-mismatch, and
//   floor-conservation refusal paths are already covered by
//   `src/artifact.rs`'s own `#[cfg(test)] mod tests` and are not
//   re-verified here.
// - It does not validate that the `generalization/` artifact pair (only
//   `case-studies/` is used here) round-trips the same way.
// - It does not assert on the *entire* `cargo tree` output — only that a
//   fixed denylist of semantic-toolchain crate names is absent. A
//   dependency not on that denylist could still exist and this test would
//   not catch it.
//
// ## Why `include!` instead of `use bcinr_cmca::artifact::*`
//
// `src/artifact.rs` is declared `#[cfg(test)] pub mod artifact;` at the
// crate root (see `src/lib.rs`), specifically so non-test builds of
// `bcinr-cmca` carry zero additional dependency beyond `bcinr-logic`. That
// `#[cfg(test)]` activates only when the *library crate itself* is compiled
// for its own unit tests (`cargo test -p bcinr-cmca --lib`) — it does
// **not** activate when `bcinr-cmca` is linked as an ordinary dependency
// into an external integration-test binary under `tests/`, which is how
// every file in this directory (including this one) is compiled. This was
// confirmed directly: a probe integration test doing
// `use bcinr_cmca::artifact::GeneratedManifest;` was compiled and failed
// with `error[E0432]: unresolved import ... found an item that was
// configured out`, `src/lib.rs:113: the item is gated here`. That probe was
// then deleted; it is not part of this file's committed test surface.
//
// Given that, this file pulls in the REAL, unmodified `src/artifact.rs`
// source file directly into this test binary's own compilation unit (see
// the `#[path = ...] mod artifact_under_test;` declaration below — full
// rationale for that exact mechanism follows in the next comment block).
// This is not a reimplementation or a mock: it is the literal source file,
// byte-for-byte, compiled a second time in a context where `#[cfg(test)]`
// is satisfied (every `tests/*.rs` binary is compiled with `--cfg test`).
// The tradeoff, stated honestly: the `verify_generated_profile` function
// under test here is a distinct monomorphization from the one exercised by
// `src/artifact.rs`'s own `#[cfg(test)] mod tests`, not literally the same
// compiled symbol — but it is compiled from the identical source, so a
// change to the real verification logic is exercised by this file exactly
// as written, with no duplication of the logic itself.

// `#[path = ...] mod ...;` (a real file-backed module, not a token-splicing
// `include!`) is used here rather than `include!("../src/artifact.rs")`
// directly: rustc's inner-doc-comment rule (`//!` must be the first thing in
// a module) is enforced against the *macro-expansion* boundary for
// `include!`, so an `include!`-spliced file's own `//!` lines fail to parse
// (E0753) no matter where the `include!` call sits. A `#[path]`-redirected
// `mod` is parsed the same way `mod foo;` normally loads `foo.rs` — as a
// real module file — so `src/artifact.rs`'s own top-of-file `//!` docs parse
// correctly. The compiled bytes are identical to `src/artifact.rs`; only the
// loading mechanism differs from a plain `mod artifact;` declaration
// (necessary because this file lives under `tests/`, one directory below
// `src/`).
#[path = "../src/artifact.rs"]
mod artifact_under_test;
use artifact_under_test::*;

use chicago_tdd_tools::core::governance::{
    Diagnostic, DiagnosticCategory, DiagnosticCode, DiagnosticSink, Severity,
};
use chicago_tdd_tools::observability::ocel::wasm4pm::seal_run;
use chicago_tdd_tools::observability::ocel::OcelCollector;
use std::collections::HashMap;
use std::fs;
use std::process::Command;

const MANIFEST_PATH: &str = "generated-artifact/case-studies/cmca_generation_manifest.json";
const GENERATED_SOURCE_PATH: &str = "generated-artifact/case-studies/cmca_generated.rs";

/// Emit a Diagnostic to `sink` recording one falsifiable JTBD-4 check
/// outcome. State-based: the receipted `OcelCollector` state (its emitted
/// event count, inspectable via `sink`) is the real collaborator being
/// exercised, not a mock of it — mirrors the convention already established
/// in `bcinr-powl/tests/chicago_tdd_integration.rs`.
fn emit_jtbd4_diagnostic(sink: &OcelCollector, message: &str, severity: Severity, elapsed_ns: u64) {
    let d = Diagnostic {
        code: DiagnosticCode::new("CMCA", DiagnosticCategory::Conformance, 4),
        category: DiagnosticCategory::Conformance,
        run_id: "jtbd4-semantic-mechanical-compilation".to_string(),
        agent_id: None,
        location: None,
        message: message.to_string(),
        severity,
        source_module: "jtbd_semantic_mechanical_compilation",
        context: HashMap::new(),
        elapsed_ns,
    };
    let _ = sink.emit(d);
}

/// Loads the REAL materialized case-studies manifest + generated source from
/// disk. Panics (test setup failure, not a refusal-under-test) if either
/// file is missing — this is a precondition, not the property being tested.
fn load_real_case_studies() -> (String, Vec<u8>) {
    let manifest_json =
        fs::read_to_string(MANIFEST_PATH).expect("real case-studies manifest must exist on disk");
    let generated_source = fs::read(GENERATED_SOURCE_PATH)
        .expect("real case-studies generated source must exist on disk");
    (manifest_json, generated_source)
}

/// (a) Part 1: a byte-for-byte copy of the REAL case-studies manifest,
/// verified against the REAL generated source it actually describes, is
/// accepted by the REAL `verify_generated_profile`.
#[test]
fn real_case_studies_artifact_is_accepted() {
    let (manifest_json, generated_source) = load_real_case_studies();
    let manifest: GeneratedManifest = serde_json::from_str(&manifest_json)
        .expect("real manifest must parse as GeneratedManifest");
    let artifact = GeneratedArtifact {
        manifest: &manifest,
        generated_source_bytes: &generated_source,
    };

    let result = verify_generated_profile(&artifact);

    let sink = OcelCollector::new(None);
    emit_jtbd4_diagnostic(
        &sink,
        &format!("real case-studies artifact verify_generated_profile result: {result:?}"),
        if result.is_ok() {
            Severity::Info
        } else {
            Severity::Andon
        },
        1,
    );
    // State-based check on the real collaborator: seal_run reads the
    // collector's real internal event log and produces a receipted Evidence
    // + BLAKE3 digest. A non-empty digest proves the diagnostic above was
    // actually recorded into OcelCollector's real state, not merely called.
    let (_receipted, digest) = seal_run(&sink, "jtbd4-accept-run".to_string())
        .expect("seal_run over the real OcelCollector state must succeed");
    assert!(
        !digest.is_empty(),
        "seal_run must produce a non-empty receipt digest from the collector's real event state"
    );
    assert_eq!(
        result,
        Ok(()),
        "the real, untampered case-studies artifact must be accepted by verify_generated_profile"
    );
}

/// (a) Part 2: the REAL case-studies manifest with exactly ONE byte of its
/// `generated_payload_digest` hex string flipped is refused, and the
/// refusal is the specific real typed variant
/// `GeneratedProfileRefusal::PayloadDigestMismatch` — proving the
/// "consume a bounded artifact" boundary actually rejects a corrupted
/// artifact rather than silently accepting it.
#[test]
fn tampered_payload_digest_byte_is_refused() {
    let (manifest_json, generated_source) = load_real_case_studies();

    // Locate the real generated_payload_digest hex string in the raw JSON
    // text and flip exactly one hex character to a different, still-valid
    // hex character. This mutates a COPY of the real manifest text; the
    // on-disk artifact is never modified.
    let needle = "\"generated_payload_digest\": \"blake3:";
    let start = manifest_json
        .find(needle)
        .expect("real manifest JSON must contain a generated_payload_digest field")
        + needle.len();
    let mut bytes = manifest_json.into_bytes();
    let target_byte = bytes[start];
    // Flip one hex digit to a different hex digit (never past the digest's
    // own 64-hex-char span, and never to the same character, so this is a
    // guaranteed single-byte corruption of the real digest).
    let flipped = if target_byte == b'0' { b'1' } else { b'0' };
    assert_ne!(
        flipped, target_byte,
        "tamper byte must actually differ from the real digest byte at this position"
    );
    bytes[start] = flipped;
    let tampered_json = String::from_utf8(bytes).expect("mutated manifest must remain valid UTF-8");

    let manifest: GeneratedManifest = serde_json::from_str(&tampered_json).expect(
        "single-hex-byte-flipped manifest must still parse as valid JSON/GeneratedManifest",
    );
    let artifact = GeneratedArtifact {
        manifest: &manifest,
        generated_source_bytes: &generated_source,
    };

    let result = verify_generated_profile(&artifact);

    let sink = OcelCollector::new(None);
    emit_jtbd4_diagnostic(
        &sink,
        &format!("tampered case-studies artifact verify_generated_profile result: {result:?}"),
        Severity::Warning,
        1,
    );
    let (_receipted, digest) = seal_run(&sink, "jtbd4-tamper-run".to_string())
        .expect("seal_run over the real OcelCollector state must succeed");
    assert!(
        !digest.is_empty(),
        "seal_run must produce a non-empty receipt digest from the collector's real event state"
    );
    assert_eq!(
        result,
        Err(GeneratedProfileRefusal::PayloadDigestMismatch),
        "a single tampered hex byte in generated_payload_digest must produce the real typed \
         PayloadDigestMismatch refusal, not acceptance and not a different refusal variant"
    );
}

/// (b) Runs a REAL `cargo tree -p bcinr-cmca` subprocess and asserts the
/// actual stdout contains none of a fixed denylist of RDF/Python/generic-
/// graph-processing/mfw crate-name markers. This is the falsifiable check
/// for "no semantic-toolchain dependency leaks into the consuming crate's
/// own dependency graph."
///
/// NOT covered: this only checks the denylisted names below; it is not an
/// exhaustive audit of every dependency `cargo tree` reports.
#[test]
fn dependency_tree_excludes_semantic_toolchain_crates() {
    let output = Command::new("cargo")
        .args(["tree", "-p", "bcinr-cmca"])
        .output()
        .expect("cargo tree -p bcinr-cmca must be spawnable in this workspace");

    let sink = OcelCollector::new(None);
    emit_jtbd4_diagnostic(
        &sink,
        &format!("cargo tree -p bcinr-cmca exit status: {}", output.status),
        if output.status.success() {
            Severity::Info
        } else {
            Severity::Andon
        },
        1,
    );
    let (_receipted, digest) = seal_run(&sink, "jtbd4-cargo-tree-run".to_string())
        .expect("seal_run over the real OcelCollector state must succeed");
    assert!(
        !digest.is_empty(),
        "seal_run must produce a non-empty receipt digest from the collector's real event state"
    );
    assert!(
        output.status.success(),
        "cargo tree -p bcinr-cmca must exit successfully; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("cargo tree output must be valid UTF-8");

    // Denylist: crate-name markers of an RDF store/parser, a Python
    // embedding, a generic graph-processing library, or the mfw producer
    // toolchain itself. Matched as substrings against the actual tree text
    // (which lists one package per line as `name vX.Y.Z`), so a match here
    // means the named package genuinely appears in the real dependency
    // graph, not an assumption about what "should" be absent.
    const DENYLIST: &[&str] = &[
        "oxigraph",
        "sophia",
        "rio_api",
        "rio_turtle",
        "pyo3",
        "rustpython",
        "petgraph",
        "networkx",
        "mfw",
    ];
    let hits: Vec<&&str> = DENYLIST
        .iter()
        .filter(|needle| stdout.contains(*needle))
        .collect();
    assert!(
        hits.is_empty(),
        "cargo tree -p bcinr-cmca must not contain any semantic-toolchain/mfw crate name; \
         found: {hits:?}\nfull tree output:\n{stdout}"
    );
}
