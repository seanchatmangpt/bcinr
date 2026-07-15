//! Adversarial verification: is `ConcurrencySelector::select_checked`'s
//! FireSet-must-be-a-subset-of-ReadySet-AND-a-member-of-the-concurrency-
//! complex postcondition actually enforced at runtime, or only in debug
//! builds?
//!
//! `crates/bcinr-powl/src/scheduler.rs`'s `select_checked` enforces both
//! halves of that invariant purely via `debug_assert!` (its own doc comment
//! even advertises this: "costs nothing in release builds"). This crate's
//! workspace `[profile.release]` (top-level `Cargo.toml`) does not set
//! `debug-assertions = true`, so `debug_assertions` is off by Cargo's
//! default in release builds — meaning `debug_assert!` compiles to nothing.
//!
//! This test constructs a deliberately non-compliant `ConcurrencySelector`
//! (`select` just returns the whole `ready` set, ignoring `guards`
//! entirely) and drives it through the real, public
//! `scheduler_tick_guarded` entry point against a guard table that forbids
//! two ops from firing together. The two `#[cfg(...)]`-gated tests below
//! show the actual, empirically-different behavior in the two build modes:
//!
//! - Under `cargo test` (dev profile, `debug_assertions` on): the bad
//!   selector's output is caught and the process panics before anything
//!   fires — the invariant *is* enforced here.
//! - Under `cargo test --release` (release profile, `debug_assertions`
//!   off): the same bad selector's output sails through unchecked, and
//!   `scheduler_tick_guarded` actually fires both forbidden ops together in
//!   the same tick — the invariant is *not* enforced here. `guards.admits`
//!   on the actually-fired set is false, empirically confirmed below.
//!
//! Run both to see the divergence:
//!   cargo test -p bcinr-powl --test release_mode_fireset_gap
//!   cargo test -p bcinr-powl --release --test release_mode_fireset_gap

use bcinr_powl::compiler::{compile_powl, PowlAstNode};
use bcinr_powl::scheduler::{scheduler_tick_guarded, ConcurrencySelector, PowlRunState};
use bcinr_powl::tape::v2::{CompiledNonFace, ConcurrencyGuardTable};
use bcinr_mfw_ir::{Digest, EventSet};

/// A selector that violates its own trait contract on purpose: it ignores
/// `guards` entirely and always hands back the full `ready` set, exactly
/// what a buggy or malicious third-party `ConcurrencySelector`
/// implementation might do (the trait is public and generic —
/// `scheduler_tick_guarded<S: ConcurrencySelector>` accepts any impl).
struct AlwaysFireEverything;

impl ConcurrencySelector for AlwaysFireEverything {
    fn select(&mut self, ready: &EventSet, _guards: &ConcurrencyGuardTable) -> EventSet {
        *ready
    }
}

fn ab_tape_and_guards() -> (bcinr_powl::tape::PowlTape, ConcurrencyGuardTable) {
    let ast = PowlAstNode::PartialOrder {
        children: vec![PowlAstNode::Atom("a"), PowlAstNode::Atom("b")],
        edges: vec![],
    };
    let tape = compile_powl(&ast).unwrap();
    let guards = ConcurrencyGuardTable {
        nonfaces: vec![CompiledNonFace {
            members: EventSet::empty().with(0).with(1),
            witness_digest: Digest::hash(b"a-b-conflict"),
        }],
    };
    (tape, guards)
}

#[cfg(debug_assertions)]
#[test]
fn debug_build_catches_the_noncompliant_selector_via_debug_assert() {
    let (tape, guards) = ab_tape_and_guards();
    let mut state = PowlRunState::new(&tape);
    let mut selector = AlwaysFireEverything;

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        scheduler_tick_guarded(&tape.ops[..tape.len as usize], &mut state, &mut selector, &guards)
    }));

    assert!(
        result.is_err(),
        "expected select_checked's debug_assert! to panic on a selector that \
         returns a set the guard table does not admit"
    );
}

#[cfg(not(debug_assertions))]
#[test]
fn release_build_silently_fires_the_forbidden_pair() {
    let (tape, guards) = ab_tape_and_guards();
    let mut state = PowlRunState::new(&tape);
    let mut selector = AlwaysFireEverything;

    // No panic here: select_checked's postcondition checks compiled to
    // nothing (debug_assertions is off in this build).
    let fired = scheduler_tick_guarded(&tape.ops[..tape.len as usize], &mut state, &mut selector, &guards);

    let fired_set = {
        let mut s = EventSet::empty();
        let mut bits = fired.0;
        while bits != 0 {
            let i = bits.trailing_zeros() as usize;
            bits &= bits - 1;
            s = s.with(i);
        }
        s
    };

    assert_eq!(
        fired.0.count_ones(),
        2,
        "CONFIRMED GAP: in a release build, the noncompliant selector's \
         full-ready-set answer was NOT rejected — both forbidden ops fired \
         in the same tick (fired={:#04b})",
        fired.0
    );
    assert!(
        !guards.admits(&fired_set),
        "the actually-fired set must be exactly the one the guard table \
         forbids, proving select_checked's postcondition was skipped, not \
         merely coincidentally satisfied"
    );
}
