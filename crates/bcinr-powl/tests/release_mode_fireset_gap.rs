//! Regression test: is `ConcurrencySelector::select_checked`'s
//! FireSet-must-be-a-subset-of-ReadySet-AND-a-member-of-the-concurrency-
//! complex postcondition actually enforced at runtime in *every* build
//! profile, not only in debug builds?
//!
//! `crates/bcinr-powl/src/scheduler.rs`'s `select_checked` used to enforce
//! both halves of that invariant purely via `debug_assert!`. This
//! workspace's `[profile.release]` (top-level `Cargo.toml`) does not set
//! `debug-assertions = true`, so `debug_assertions` is off by Cargo's
//! default in release builds, meaning `debug_assert!` compiled to nothing
//! there — a noncompliant `ConcurrencySelector` (the trait is public and
//! generic, so any implementation reaches the real `scheduler_tick_guarded`
//! entry point) could silently fire a set the guard table forbids in a
//! release build, with zero enforcement.
//!
//! `select_checked` now uses `assert!` instead of `debug_assert!`, so the
//! postcondition is checked unconditionally. This test constructs a
//! deliberately non-compliant `ConcurrencySelector` (`select` just returns
//! the whole `ready` set, ignoring `guards` entirely) and drives it through
//! the real, public `scheduler_tick_guarded` entry point against a guard
//! table that forbids two ops from firing together, then confirms the
//! resulting panic happens under both build profiles:
//!
//!   cargo test -p bcinr-powl --test release_mode_fireset_gap
//!   cargo test -p bcinr-powl --release --test release_mode_fireset_gap

use bcinr_mfw_ir::{Digest, EventSet};
use bcinr_powl::compiler::{compile_powl, PowlAstNode};
use bcinr_powl::scheduler::{scheduler_tick_guarded, ConcurrencySelector, PowlRunState};
use bcinr_powl::tape::v2::{CompiledNonFace, ConcurrencyGuardTable};

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

/// Runs identically (and must panic identically) under both `cargo test`
/// and `cargo test --release` — the postcondition is no longer
/// debug-profile-only.
#[test]
fn noncompliant_selector_is_rejected_in_every_build_profile() {
    let (tape, guards) = ab_tape_and_guards();
    let mut state = PowlRunState::new(&tape);
    let mut selector = AlwaysFireEverything;

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        scheduler_tick_guarded(
            &tape.ops[..tape.len as usize],
            &mut state,
            &mut selector,
            &guards,
        )
    }));

    assert!(
        result.is_err(),
        "expected select_checked's postcondition assert! to panic on a \
         selector that returns a set the guard table does not admit — in \
         this build profile too (release included)"
    );
}

/// Companion positive check: confirm the guard table used above really
/// does forbid `{a, b}` firing together, so the panic above is caused by a
/// genuine violation and not a fixture mistake.
#[test]
fn fixture_guard_table_genuinely_forbids_the_ab_pair() {
    let (_, guards) = ab_tape_and_guards();
    let forbidden = EventSet::empty().with(0).with(1);
    assert!(
        !guards.admits(&forbidden),
        "sanity: the {{a, b}} pair must be forbidden by this fixture's guard table"
    );
}
