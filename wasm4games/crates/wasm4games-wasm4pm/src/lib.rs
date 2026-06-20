//! # wasm4games-wasm4pm — the admission bridge
//!
//! This crate is the **online-only** bridge between the offline, dependency-free
//! [`wasm4games`] evidence types and the upstream **wasm4pm** process miner (via its stable
//! `wasm4pm-compat` surface). It maps candidate evidence emitted by wasm4games kernels onto
//! the canonical wasm4pm-compat types and submits it for admission.
//!
//! ## Doctrine
//!
//! > Engines project worlds; wasm4games operates patterns; **wasm4pm admits evidence.**
//!
//! wasm4games never decides admissibility; it emits candidate evidence. This crate carries
//! that evidence across the boundary to the authority.
//!
//! ## Status: scaffolding pinned against the canonical types
//!
//! The exact upstream API is not vendored here and is unavailable offline, so the mapping
//! functions below are written with **precise signatures** and `// TODO: confirm against
//! wasm4pm-compat <type>` notes against the canonical types:
//!
//! - `OcelLog` — an object-centric event log.
//! - `LinkedOcel` — events with materialized object linkage.
//! - `ReceiptEnvelope` / `ReceiptChain` — sealed, tamper-evident receipts.
//! - `ConformanceResult` — the admission verdict.
//! - `Evidence<T, State, W>` — the typestate evidence wrapper.
//!
//! Because the crate is workspace-**excluded**, it is not part of the offline build; it may
//! not compile until the upstream dependencies are resolved and the `TODO`s are reconciled
//! against the real types. The upstream type paths are written as the canonical
//! `wasm4pm_compat::*` names but kept commented (or behind a feature) so the intent is
//! unambiguous and the diff to "real" code is minimal once the API is confirmed.
//!
//! ## Before use
//!
//! 1. Pin the git revisions in `Cargo.toml` to exact commit hashes.
//! 2. Replace each `/* wasm4pm_compat::Type */` placeholder with the confirmed path.
//! 3. Reconcile every `// TODO: confirm against wasm4pm-compat ...` note.

#![forbid(unsafe_code)]

use wasm4games::evidence::ocel::{OcelEvent, OcelLog};
use wasm4games::evidence::receipt::{ReceiptChain, ReceiptEnvelope};

// Bring the upstream crates into scope. Their exact item paths must be confirmed; until then
// the mapping bodies reference them through the `// TODO` notes rather than concrete imports,
// so this `use` documents the dependency without guessing item names.
//
// use wasm4pm_compat as w4pm;
// use wasm4pm as miner;

/// How an admission attempt resolved.
///
/// This mirrors the shape of `wasm4pm_compat::ConformanceResult` for the common cases. Once
/// the upstream type is confirmed, prefer returning it directly and delete this enum.
///
// TODO: confirm against wasm4pm-compat `ConformanceResult` (variant set + payload fields).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdmissionOutcome {
    /// Evidence was admitted under the active scope.
    Admitted,
    /// Evidence was refused; carries the upstream refusal status code.
    Refused(u8),
    /// The miner could not decide with the supplied evidence.
    Inconclusive,
}

/// Bridge configuration: endpoint / scope handle for the upstream miner.
///
// TODO: confirm against wasm4pm `Miner` / session-handle constructor. The real type likely
// holds a connection, a model handle, and a conformance scope.
#[derive(Clone, Debug, Default)]
pub struct Bridge {
    /// Opaque scope identifier the miner uses to select a reference model.
    pub scope: u64,
}

impl Bridge {
    /// Create a bridge bound to a conformance `scope`.
    #[must_use]
    pub fn new(scope: u64) -> Self {
        Self { scope }
    }

    /// Map an in-memory [`OcelLog`] to the upstream object-centric log and submit it for
    /// admission, returning the verdict.
    ///
    /// This is the primary entry point: it composes [`to_w4pm_log`] with [`admit`].
    ///
    // TODO: confirm against wasm4pm-compat `OcelLog` ingestion + `wasm4pm` admit call.
    pub fn admit_log(&self, log: &OcelLog) -> AdmissionOutcome {
        let w4pm_log = to_w4pm_log(log.as_slice());
        admit(self, w4pm_log)
    }
}

/// Map a slice of wasm4games [`OcelEvent`]s onto the canonical wasm4pm-compat object-centric
/// log type.
///
/// Each wasm4games event becomes one upstream OCEL event; each `(type_code, id)` in
/// [`wasm4games::evidence::ocel::ObjectRefs`] becomes one materialized object link
/// (`LinkedOcel` edge). The wasm4games `activity` (pattern id) and `status` map onto the
/// upstream activity label and lifecycle state respectively.
///
/// Return type is written as the canonical `wasm4pm_compat::OcelLog`; the placeholder keeps
/// the file parseable until the upstream path is confirmed.
///
// TODO: confirm against wasm4pm-compat `OcelLog` (and `LinkedOcel` for object linkage).
#[must_use]
pub fn to_w4pm_log(events: &[OcelEvent]) -> /* wasm4pm_compat::OcelLog */ W4pmLogStub {
    let mut out = W4pmLogStub::default();
    for ev in events {
        // TODO: confirm against wasm4pm-compat `OcelEvent` constructor / builder. The
        // mapping is field-for-field:
        //   ev.event_code -> upstream event type code
        //   ev.activity   -> upstream activity (pattern id) label
        //   ev.timestamp  -> upstream logical timestamp
        //   ev.status     -> upstream lifecycle/admission state
        //   ev.objects[*] -> upstream LinkedOcel object edges
        out.events.push(W4pmEventStub {
            event_code: ev.event_code,
            activity: ev.activity,
            timestamp: ev.timestamp,
            status: ev.status,
            objects: ev.objects.as_slice().to_vec(),
        });
    }
    out
}

/// Map a wasm4games [`ReceiptEnvelope`] (or live [`ReceiptChain`]) onto the upstream sealed
/// receipt type so the miner can witness execution order alongside the event log.
///
// TODO: confirm against wasm4pm-compat `ReceiptEnvelope` / `ReceiptChain` (hash width + count
// field names).
#[must_use]
pub fn to_w4pm_receipt<const N: usize>(
    envelope: &ReceiptEnvelope<N>,
) -> /* wasm4pm_compat::ReceiptEnvelope */ W4pmReceiptStub {
    W4pmReceiptStub {
        chain_hash: envelope.chain_hash,
        count: envelope.count,
        capacity: N,
    }
}

/// Seal a live [`ReceiptChain`] and map it across the boundary in one step.
///
// TODO: confirm against wasm4pm-compat `ReceiptEnvelope::seal`.
#[must_use]
pub fn seal_w4pm_receipt(chain: &ReceiptChain) -> W4pmReceiptStub {
    W4pmReceiptStub {
        chain_hash: chain.seal(),
        count: chain.count(),
        capacity: 0,
    }
}

/// Submit a mapped log to the upstream miner for admission and return the verdict.
///
/// Real implementation flow:
/// 1. Wrap `log` (and any receipt) in `Evidence<OcelLog, Raw, _>`.
/// 2. Advance the typestate (`Raw -> Parsed -> Admitted`) via the miner using `bridge.scope`.
/// 3. Translate the upstream `ConformanceResult` into [`AdmissionOutcome`].
///
/// Signature is shaped to return the canonical `wasm4pm_compat::ConformanceResult`; the stub
/// return keeps this file parseable offline.
///
// TODO: confirm against wasm4pm `admit`/`conform` entry point and `wasm4pm_compat::Evidence
// <T, State, W>` typestate transitions; then return `wasm4pm_compat::ConformanceResult`.
#[must_use]
pub fn admit(
    _bridge: &Bridge,
    _log: /* wasm4pm_compat::OcelLog */ W4pmLogStub,
) -> /* wasm4pm_compat::ConformanceResult */ AdmissionOutcome {
    // Offline scaffold: the real call hands the evidence to the miner. Until the upstream API
    // is wired, we cannot decide, so we report `Inconclusive`.
    AdmissionOutcome::Inconclusive
}

// ---------------------------------------------------------------------------------------------
// Offline stubs.
//
// These local types stand in for the canonical `wasm4pm_compat` types so this file PARSES
// without the (online-only) upstream crates. They are NOT the real types: when the upstream
// API is confirmed, delete each stub and replace every `/* wasm4pm_compat::... */` placeholder
// with the real path. They are deliberately minimal and dependency-free.
// ---------------------------------------------------------------------------------------------

/// Stand-in for `wasm4pm_compat::OcelLog`. See module docs.
// TODO: replace with `wasm4pm_compat::OcelLog`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct W4pmLogStub {
    /// Mapped events.
    pub events: Vec<W4pmEventStub>,
}

/// Stand-in for `wasm4pm_compat::OcelEvent` (+ its `LinkedOcel` object edges).
// TODO: replace with `wasm4pm_compat::OcelEvent` / `LinkedOcel`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct W4pmEventStub {
    /// Upstream event type code.
    pub event_code: u16,
    /// Upstream activity (pattern id).
    pub activity: u16,
    /// Upstream logical timestamp.
    pub timestamp: u64,
    /// Upstream lifecycle / admission state.
    pub status: u8,
    /// Materialized object links as `(type_code, id)`.
    pub objects: Vec<(u16, u64)>,
}

/// Stand-in for `wasm4pm_compat::ReceiptEnvelope`. See module docs.
// TODO: replace with `wasm4pm_compat::ReceiptEnvelope`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct W4pmReceiptStub {
    /// Sealed rolling hash.
    pub chain_hash: u64,
    /// Number of events folded.
    pub count: u32,
    /// Declared chain capacity (0 when sealed from an unbounded chain).
    pub capacity: usize,
}
