//! Divan benchmarks for the `bcinr-guarded` projections — the crate's first
//! benchmark suite. The crate's entire runtime surface is classification:
//! `powl2_case::classify` (live `Powl2Model` -> admitted case),
//! `runtime_binding::classify_cascaderefusal` (live CMCA refusal ->
//! admitted taxonomy case), and `runtime_binding::classify_refusalreason`
//! (live POWL refusal -> admitted taxonomy case). Each generated `match`
//! has no `_` arm, so these dispatch tables are the drift tripwire itself —
//! their cost is the cost of the guarantee.
//!
//! Convention matches `bcinr-powl/benches/*` and
//! `bcinr-cmca/benches/allocation_bench.rs`: Divan, `harness = false`,
//! `black_box` on inputs AND outputs.
//!
//! Fixtures: one representative `Powl2Model` per constructor (nested,
//! 4-children composites — representative production shapes, not stubs),
//! and the FULL closed variant set of both refusal taxonomies (10
//! `CascadeRefusal` + 10 `RefusalReason`), because the value of a dispatch
//! benchmark over a no-`_`-arm match is per-variant arm coverage. Fixture
//! values are built once (`OnceLock`) and only borrowed per iteration;
//! classification is non-mutating, so per-iteration cost is the dispatch
//! itself. `taxonomy` getters (`name`/`doc`, constant strings) are
//! presentation-only and intentionally not benched.
//!
//! Numbers are receipts, not claims.

use std::sync::OnceLock;

use bcinr_cmca::cascade::{CascadeRefusal, NumericContext};
use bcinr_guarded::powl2_case::classify;
use bcinr_guarded::runtime_binding::{classify_cascaderefusal, classify_refusalreason};
use bcinr_powl::powl2::Powl2Model;
use bcinr_powl::recompose::RecomposeError;
use bcinr_powl::wf_net::NetError;
use bcinr_powl::wf_to_powl::RefusalReason;
use divan::black_box;

fn main() {
    divan::main();
}

// ---------------------------------------------------------------------------
// Powl2Model fixtures — one representative model per constructor
// ---------------------------------------------------------------------------

fn four_activities() -> Vec<Powl2Model> {
    (0..4)
        .map(|i| Powl2Model::Activity(format!("act_{i}")))
        .collect()
}

fn model_activity() -> &'static Powl2Model {
    static M: OnceLock<Powl2Model> = OnceLock::new();
    M.get_or_init(|| Powl2Model::Activity("act".into()))
}

fn model_silent() -> &'static Powl2Model {
    static M: OnceLock<Powl2Model> = OnceLock::new();
    M.get_or_init(|| Powl2Model::Silent)
}

fn model_sequence() -> &'static Powl2Model {
    static M: OnceLock<Powl2Model> = OnceLock::new();
    M.get_or_init(|| Powl2Model::Sequence(four_activities()))
}

fn model_partial_order() -> &'static Powl2Model {
    static M: OnceLock<Powl2Model> = OnceLock::new();
    M.get_or_init(|| Powl2Model::PartialOrder {
        children: four_activities(),
        // Diamond: 0 -> 1 -> 3, 0 -> 2 -> 3 (a strict partial order).
        edges: vec![(0, 1), (0, 2), (1, 3), (2, 3)],
    })
}

fn model_choice_graph() -> &'static Powl2Model {
    static M: OnceLock<Powl2Model> = OnceLock::new();
    M.get_or_init(|| Powl2Model::ChoiceGraph {
        children: four_activities(),
        // Chain over all four children, artificial start/end per Def 3.6.
        edges: vec![(0, 1), (1, 2), (2, 3)],
        start: 0,
        end: 3,
    })
}

fn model_do_redo() -> &'static Powl2Model {
    static M: OnceLock<Powl2Model> = OnceLock::new();
    M.get_or_init(|| Powl2Model::DoRedo {
        body: Box::new(Powl2Model::Activity("body".into())),
        redo: Box::new(Powl2Model::Activity("redo".into())),
        max_redos: 2,
    })
}

#[divan::bench]
fn classify_activity() {
    let out = classify(black_box(model_activity()));
    let _ = black_box(out);
}

#[divan::bench]
fn classify_silent() {
    let out = classify(black_box(model_silent()));
    let _ = black_box(out);
}

#[divan::bench]
fn classify_sequence() {
    let out = classify(black_box(model_sequence()));
    let _ = black_box(out);
}

#[divan::bench]
fn classify_partial_order() {
    let out = classify(black_box(model_partial_order()));
    let _ = black_box(out);
}

#[divan::bench]
fn classify_choice_graph() {
    let out = classify(black_box(model_choice_graph()));
    let _ = black_box(out);
}

#[divan::bench]
fn classify_do_redo() {
    let out = classify(black_box(model_do_redo()));
    let _ = black_box(out);
}

/// All six constructors back to back: the composite-classification hot path
/// a caller projecting a stream of live models would exercise.
#[divan::bench(counters = [divan::counter::ItemsCount::new(6usize)])]
fn classify_all_six_constructors() {
    let models = [
        model_activity(),
        model_silent(),
        model_sequence(),
        model_partial_order(),
        model_choice_graph(),
        model_do_redo(),
    ];
    for model in black_box(&models) {
        let out = classify(black_box(model));
        let _ = black_box(out);
    }
}

// ---------------------------------------------------------------------------
// CascadeRefusal fixtures — the full closed 10-variant set
// ---------------------------------------------------------------------------

fn cascade_refusals() -> &'static [CascadeRefusal; 10] {
    static R: OnceLock<[CascadeRefusal; 10]> = OnceLock::new();
    R.get_or_init(|| {
        [
            CascadeRefusal::LengthMismatch {
                parents: 8,
                masses: 7,
            },
            CascadeRefusal::ParentOutOfRange {
                node: 3,
                parent: 99,
                len: 8,
            },
            CascadeRefusal::Cyclic { node: 5 },
            CascadeRefusal::NoRoot,
            CascadeRefusal::ExponentOutOfRange {
                lens: 1 << 30,
                max_magnitude: 64,
            },
            CascadeRefusal::DegenerateSiblingSet { parent: Some(0) },
            CascadeRefusal::DegenerateSubtreeLeaves { node: 2 },
            CascadeRefusal::ZeroMassUnderNegativeLens { node: 4, lens: -1 },
            CascadeRefusal::NumericFault {
                operation: NumericContext::ShareDivision,
                node: 1,
                error_code: 7,
            },
            CascadeRefusal::EscortUnderflow {
                node: 6,
                lens: 1,
                mass_bits: 1,
            },
        ]
    })
}

/// Dispatch every closed `CascadeRefusal` variant through the generated
/// no-`_`-arm binding — per-variant arm coverage in one measurement.
#[divan::bench(counters = [divan::counter::ItemsCount::new(10usize)])]
fn classify_all_cascade_refusals() {
    for refusal in black_box(cascade_refusals()) {
        let out = classify_cascaderefusal(black_box(refusal));
        let _ = black_box(out);
    }
}

// ---------------------------------------------------------------------------
// RefusalReason fixtures — the full closed 10-variant set
// ---------------------------------------------------------------------------

fn refusal_reasons() -> &'static [RefusalReason; 10] {
    static R: OnceLock<[RefusalReason; 10]> = OnceLock::new();
    R.get_or_init(|| {
        [
            RefusalReason::IrreducibleFragment { depth: 3 },
            RefusalReason::BudgetExhausted { budget: 1024 },
            RefusalReason::BoundedLanguageAgreementFailed { checked_len: 64 },
            RefusalReason::InternalNetConstruction(NetError::NotWfNet("bench: not wf".into())),
            RefusalReason::VacuousLanguageBound,
            RefusalReason::NotRecomposable(RecomposeError::NetConstruction(NetError::DanglingArc(
                "bench: dangling".into(),
            ))),
            RefusalReason::NotSafe {
                witness_markings: 512,
            },
            RefusalReason::NotSound {
                option_to_complete: false,
                proper_completion: true,
                no_dead_transitions: false,
            },
            RefusalReason::SoundnessUndecided { explored: 10_000 },
            RefusalReason::InternalModelInvalid(bcinr_powl::powl2::Powl2Error::EmptySequence),
        ]
    })
}

/// Dispatch every closed `RefusalReason` variant through the generated
/// no-`_`-arm binding — per-variant arm coverage in one measurement.
#[divan::bench(counters = [divan::counter::ItemsCount::new(10usize)])]
fn classify_all_refusal_reasons() {
    for reason in black_box(refusal_reasons()) {
        let out = classify_refusalreason(black_box(reason));
        let _ = black_box(out);
    }
}
