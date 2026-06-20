//! Tamper-evident rolling receipt chain over emitted events.
//!
//! Reuses [`bcinr_logic`]'s FNV-1a substrate receipt. This is a *telemetry* receipt, not a
//! cryptographic signature; it witnesses execution order so replays can be compared.

use crate::evidence::ocel::OcelEvent;
use bcinr_logic::patterns::integrity_receipt::DeterministicSubstrateReceipt;

/// A rolling receipt chain. Mirrors `wasm4pm-compat`'s `ReceiptChain` in shape.
pub struct ReceiptChain {
    inner: DeterministicSubstrateReceipt,
    count: u32,
}

impl ReceiptChain {
    /// A fresh chain.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            inner: DeterministicSubstrateReceipt::new(),
            count: 0,
        }
    }

    /// Fold one event into the chain.
    #[inline]
    pub fn append(&mut self, ev: &OcelEvent) {
        self.inner
            .record(ev.event_code as u64, ev.status as u64, ev.timestamp);
        self.count = self.count.wrapping_add(1);
    }

    /// Number of events folded so far.
    #[inline]
    #[must_use]
    pub fn count(&self) -> u32 {
        self.count
    }

    /// Seal the chain to its current rolling hash.
    #[inline]
    #[must_use]
    pub fn seal(&self) -> u64 {
        self.inner.finalize()
    }
}

impl Default for ReceiptChain {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// A sealed, fixed-capacity receipt envelope. Mirrors `wasm4pm-compat`'s
/// `ReceiptChainConst<N>` / `ReceiptEnvelope` shape.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ReceiptEnvelope<const N: usize> {
    /// The sealed rolling hash.
    pub chain_hash: u64,
    /// Number of events folded into the chain.
    pub count: u32,
}

impl<const N: usize> ReceiptEnvelope<N> {
    /// Seal a chain into an envelope of capacity `N`.
    #[inline]
    #[must_use]
    pub fn seal(chain: &ReceiptChain) -> Self {
        Self {
            chain_hash: chain.seal(),
            count: chain.count(),
        }
    }
}
