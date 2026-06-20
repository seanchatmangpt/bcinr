//! Tamper-evident rolling receipt chain over emitted events.
//!
//! Reuses [`bcinr_logic`]'s FNV-1a substrate receipt. This is a *telemetry* receipt, not a
//! cryptographic signature; it witnesses execution order so replays can be compared.
//!
//! # Model
//!
//! A [`ReceiptChain`] is a mutable accumulator. Each call to [`ReceiptChain::append`] folds
//! one [`OcelEvent`]'s scalar fields into a rolling FNV-1a hash. The chain can be *sealed*
//! at any point via [`ReceiptChain::seal`] or [`ReceiptEnvelope::seal`], producing a
//! deterministic `u64` digest that identifies the exact sequence of events seen so far.
//!
//! # Invariants
//!
//! * The chain is append-only; there is no way to remove or reorder entries.
//! * `ReceiptChain::count()` wraps on `u32::MAX` overflow (extremely unlikely in practice).
//! * [`ReceiptEnvelope`] is `Copy` and `Hash`; it can be stored, compared, or transmitted.
//!
//! # `no_std` compatibility
//!
//! [`ReceiptChain`] wraps `DeterministicSubstrateReceipt` which is `const`-constructible;
//! the whole chain is stack-allocated with zero heap use. [`ReceiptEnvelope`] is a plain
//! `Copy` struct.

use core::fmt;

use crate::evidence::ocel::OcelEvent;
use bcinr_logic::patterns::integrity_receipt::DeterministicSubstrateReceipt;

/// A rolling receipt chain.
///
/// Mirrors `wasm4pm-compat`'s `ReceiptChain` in shape.  Each appended [`OcelEvent`] is
/// folded via FNV-1a mixing of `event_code`, `status`, and `timestamp`. The chain is sealed
/// by calling [`Self::seal`] which returns the current 64-bit rolling hash.
///
/// # Examples
///
/// ```
/// use wasm4games::evidence::receipt::ReceiptChain;
/// use wasm4games::evidence::ocel::OcelEvent;
///
/// let mut chain = ReceiptChain::new();
/// chain.append(&OcelEvent::new(1, 2, 100, 4));
/// chain.append(&OcelEvent::new(3, 4, 101, 4));
/// assert_eq!(chain.count(), 2);
/// let hash = chain.seal();
/// assert_ne!(hash, 0); // extremely unlikely to be zero
/// ```
pub struct ReceiptChain {
    inner: DeterministicSubstrateReceipt,
    count: u32,
}

impl ReceiptChain {
    /// A fresh chain with no events folded.
    ///
    /// Identical to [`Default::default`]; provided as `const fn` for use in `const` contexts.
    #[inline]
    #[must_use = "returns a new ReceiptChain; bind it to a variable"]
    pub const fn new() -> Self {
        Self {
            inner: DeterministicSubstrateReceipt::new(),
            count: 0,
        }
    }

    /// Fold one event into the chain.
    ///
    /// Mixes `event_code`, `status`, and `timestamp` into the rolling FNV-1a hash and
    /// increments the event count (wrapping on overflow).
    #[inline]
    pub fn append(&mut self, ev: &OcelEvent) {
        self.inner
            .record(ev.event_code as u64, ev.status as u64, ev.timestamp);
        self.count = self.count.wrapping_add(1);
    }

    /// Number of events folded so far.
    ///
    /// Wraps on `u32::MAX` overflow (> 4 billion events per chain).
    #[inline]
    #[must_use = "returns the event count; ignoring it is likely a bug"]
    pub fn count(&self) -> u32 {
        self.count
    }

    /// Seal the chain to its current rolling hash.
    ///
    /// Returns a 64-bit digest that uniquely identifies the sequence of events folded so
    /// far. Calling `seal` does **not** consume or reset the chain; you can continue
    /// appending after sealing.
    #[inline]
    #[must_use = "returns the 64-bit chain hash; bind or use it"]
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

impl fmt::Debug for ReceiptChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReceiptChain")
            .field("count", &self.count)
            .field("seal", &self.inner.finalize())
            .finish()
    }
}

impl fmt::Display for ReceiptChain {
    /// Formats as `ReceiptChain(count=N, hash=0xHHHHHHHHHHHHHHHH)`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ReceiptChain(count={}, hash={:#018x})",
            self.count,
            self.inner.finalize()
        )
    }
}

/// A sealed, fixed-capacity receipt envelope.
///
/// Mirrors `wasm4pm-compat`'s `ReceiptChainConst<N>` / `ReceiptEnvelope` shape. The const
/// parameter `N` is reserved for future use (e.g., bounded replay buffers); it does not
/// currently affect the size or layout.
///
/// # Examples
///
/// ```
/// use wasm4games::evidence::receipt::{ReceiptChain, ReceiptEnvelope};
/// use wasm4games::evidence::ocel::OcelEvent;
///
/// let mut chain = ReceiptChain::new();
/// chain.append(&OcelEvent::new(1, 2, 10, 4));
/// let envelope: ReceiptEnvelope<4> = ReceiptEnvelope::seal(&chain);
/// assert_eq!(envelope.count, 1);
/// assert_eq!(envelope.chain_hash, chain.seal());
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ReceiptEnvelope<const N: usize> {
    /// The sealed rolling hash.
    pub chain_hash: u64,
    /// Number of events folded into the chain.
    pub count: u32,
}

impl<const N: usize> ReceiptEnvelope<N> {
    /// Construct a receipt envelope directly from a hash and count.
    ///
    /// Prefer [`Self::seal`] in normal use; this constructor is provided for
    /// deserialization and testing.
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::evidence::receipt::ReceiptEnvelope;
    /// let env: ReceiptEnvelope<4> = ReceiptEnvelope::new(0xDEAD_BEEF_CAFE_BABE, 7);
    /// assert_eq!(env.chain_hash, 0xDEAD_BEEF_CAFE_BABE);
    /// assert_eq!(env.count, 7);
    /// ```
    #[inline]
    #[must_use = "returns the new ReceiptEnvelope; bind it to a variable"]
    pub const fn new(chain_hash: u64, count: u32) -> Self {
        Self { chain_hash, count }
    }

    /// Seal a chain into an envelope of capacity `N`.
    ///
    /// Captures the current hash and count from the chain; the chain itself is not consumed.
    #[inline]
    #[must_use = "returns the sealed ReceiptEnvelope; bind or compare it"]
    pub fn seal(chain: &ReceiptChain) -> Self {
        Self {
            chain_hash: chain.seal(),
            count: chain.count(),
        }
    }
}

impl<const N: usize> fmt::Display for ReceiptEnvelope<N> {
    /// Formats as `ReceiptEnvelope(count=N, hash=0xHHHHHHHHHHHHHHHH)`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ReceiptEnvelope(count={}, hash={:#018x})",
            self.count, self.chain_hash
        )
    }
}

/// Convert a sealed [`ReceiptEnvelope`] to its raw `(chain_hash, count)` tuple.
///
/// Useful for serialization or FFI without losing type information.
impl<const N: usize> From<ReceiptEnvelope<N>> for (u64, u32) {
    #[inline]
    fn from(env: ReceiptEnvelope<N>) -> (u64, u32) {
        (env.chain_hash, env.count)
    }
}

/// Build a [`ReceiptEnvelope`] from a `(chain_hash, count)` tuple.
///
/// Inverse of the `From<ReceiptEnvelope> for (u64, u32)` conversion.
impl<const N: usize> From<(u64, u32)> for ReceiptEnvelope<N> {
    #[inline]
    fn from((chain_hash, count): (u64, u32)) -> Self {
        Self { chain_hash, count }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::ocel::OcelEvent;

    #[test]
    fn receipt_chain_behavior() {
        // fresh chain: zero count, default == new
        let chain = ReceiptChain::new();
        assert_eq!(chain.count(), 0);
        let b = ReceiptChain::default();
        assert_eq!(chain.seal(), b.seal());
        assert_eq!(chain.count(), b.count());

        // append increments count, each new event changes the hash
        let mut chain = ReceiptChain::new();
        chain.append(&OcelEvent::new(1, 2, 100, 4));
        assert_eq!(chain.count(), 1);
        let hash_after_one = chain.seal();
        chain.append(&OcelEvent::new(3, 4, 20, 4));
        assert_eq!(chain.count(), 2);
        assert_ne!(hash_after_one, chain.seal());

        // same events → same hash; order matters
        let ev = OcelEvent::new(42, 7, 999, 0);
        let mut ca = ReceiptChain::new();
        let mut cb = ReceiptChain::new();
        ca.append(&ev);
        cb.append(&ev);
        assert_eq!(ca.seal(), cb.seal());

        let ev1 = OcelEvent::new(1, 0, 1, 0);
        let ev2 = OcelEvent::new(2, 0, 2, 0);
        let mut fwd = ReceiptChain::new();
        fwd.append(&ev1);
        fwd.append(&ev2);
        let mut rev = ReceiptChain::new();
        rev.append(&ev2);
        rev.append(&ev1);
        assert_ne!(fwd.seal(), rev.seal());

        // seal is idempotent and non-consuming
        let mut chain = ReceiptChain::new();
        chain.append(&OcelEvent::new(1, 1, 1, 0));
        assert_eq!(chain.seal(), chain.seal());
        chain.append(&OcelEvent::new(2, 2, 2, 0));
        assert_eq!(chain.count(), 2);
    }

    #[test]
    fn receipt_envelope_behavior() {
        // seal matches chain
        let mut chain = ReceiptChain::new();
        chain.append(&OcelEvent::new(10, 20, 30, 4));
        let env: ReceiptEnvelope<4> = ReceiptEnvelope::seal(&chain);
        assert_eq!(env.chain_hash, chain.seal());
        assert_eq!(env.count, chain.count());

        // new constructor and tuple roundtrip
        let env: ReceiptEnvelope<8> = ReceiptEnvelope::new(0xCAFE_BABE_1234_5678, 42);
        assert_eq!(env.chain_hash, 0xCAFE_BABE_1234_5678);
        assert_eq!(env.count, 42);

        let orig: ReceiptEnvelope<2> = ReceiptEnvelope::new(0xDEAD_BEEF, 3);
        let tuple: (u64, u32) = orig.into();
        let back: ReceiptEnvelope<2> = tuple.into();
        assert_eq!((back.chain_hash, back.count), (orig.chain_hash, orig.count));

        // Copy + Eq
        let a: ReceiptEnvelope<4> = ReceiptEnvelope::new(0x1111, 1);
        let b = a;
        assert_eq!(a, b);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn display_contains_count_and_hash() {
        let mut chain = ReceiptChain::new();
        chain.append(&OcelEvent::new(1, 2, 3, 4));
        let s = alloc::format!("{}", chain);
        assert!(s.contains("count=1") && s.contains("hash="));

        let env: ReceiptEnvelope<4> = ReceiptEnvelope::new(0xABCD, 5);
        let s = alloc::format!("{}", env);
        assert!(s.contains("count=5") && s.contains("hash="));
    }
}
