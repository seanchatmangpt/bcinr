//! # Universe64 Contract: DeltaBus (Fan-Out Dispatch)
//! Plane: S — delta dispatch coordination plane.
//! Tier: T1 fan-out dispatch (bounded subscribers).
//! Scope: bounded subscriber table; zero heap.
//! Geometry: each published UDelta is forwarded to all registered subscriber handlers.
//! Delta: consumes UDelta records; does not produce new ones.
//!
//! # Timing contract
//! - **T1 dispatch budget:** ≤ T1_BUDGET_NS per delta (across all subscribers)
//! - **Max heap allocations:** 0
//!
//! # Admissibility
//! Admissible_T1: YES. Fixed-size subscriber table; bounded loop.
//! CC=1: Absolute branchless loop iteration.

use super::scratch::UDelta;

/// Maximum subscribers on the delta bus.
pub const MAX_SUBSCRIBERS: usize = 8;

/// Subscriber channel identifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SubscriberChannel {
    ReadyMask     = 0, // RM
    ProjectionUnit = 1, // PU
    RLKernel      = 2, // RL
    ReceiptChain  = 3, // RC
    BoundaryYard  = 4, // BY
    Custom1       = 5,
    Custom2       = 6,
    Custom3       = 7,
}

/// A subscriber slot: channel id + handler function pointer.
/// Using fn pointer (not closure) to remain zero-allocation.
#[derive(Clone, Copy)]
pub struct Subscriber {
    pub channel: SubscriberChannel,
    pub active: bool,
    pub handler: fn(&UDelta),
}

impl Default for Subscriber {
    fn default() -> Self {
        Self {
            channel: SubscriberChannel::Custom1,
            active: false,
            handler: |_| {},
        }
    }
}

/// Bounded delta bus: publishes UDelta records to registered subscribers.
#[derive(Clone, Copy)]
pub struct DeltaBus {
    subscribers: [Subscriber; MAX_SUBSCRIBERS],
    count: u8,
}

impl DeltaBus {
    pub const fn new() -> Self {
        Self {
            subscribers: [Subscriber {
                channel: SubscriberChannel::Custom1,
                active: false,
                handler: |_| {},
            }; MAX_SUBSCRIBERS],
            count: 0,
        }
    }

    /// Register a subscriber handler. Returns false if bus is full.
    pub fn subscribe(&mut self, channel: SubscriberChannel, handler: fn(&UDelta)) -> bool {
        let admit = (self.count as usize) < MAX_SUBSCRIBERS;
        let slot = (self.count as usize) & (MAX_SUBSCRIBERS - 1);
        self.subscribers[slot] = Subscriber { channel, active: admit, handler };
        self.count = self.count.wrapping_add(admit as u8);
        admit
    }

    /// Deactivate subscriber by channel (branchless masking).
    pub fn unsubscribe(&mut self, channel: SubscriberChannel) {
        for i in 0..self.count as usize {
            let same = (self.subscribers[i].channel == channel) as u8;
            // Branchless: active = active & (NOT same)
            let was = self.subscribers[i].active as u8;
            self.subscribers[i].active = (was & (1 - same)) != 0;
        }
    }

    /// Publish a delta to all active subscribers.
    /// T1-budget: iterates bounded subscriber table.
    #[inline]
    pub fn publish(&self, delta: &UDelta) {
        for i in 0..self.count as usize {
            let s = &self.subscribers[i];
            if s.active {
                (s.handler)(delta);
            }
        }
    }

    /// Publish a batch of deltas.
    pub fn publish_batch(&self, deltas: &[UDelta]) {
        for d in deltas {
            self.publish(d);
        }
    }

    /// Number of active subscribers.
    #[inline(always)]
    pub fn active_count(&self) -> usize {
        let mut n = 0usize;
        for i in 0..self.count as usize {
            n = n.wrapping_add(self.subscribers[i].active as usize);
        }
        n
    }
}

impl Default for DeltaBus {
    fn default() -> Self { Self::new() }
}

/// Per-channel fire counters — isolated atomics prevent inter-test interference.
#[cfg(test)]
mod counters {
    use core::sync::atomic::{AtomicU32, Ordering};
    // One counter per test scenario, keyed by channel ordinal.
    pub static RM:    AtomicU32 = AtomicU32::new(0);
    pub static MULTI: AtomicU32 = AtomicU32::new(0);
    pub static UNSUB: AtomicU32 = AtomicU32::new(0);
    pub static BATCH: AtomicU32 = AtomicU32::new(0);

    pub fn inc_rm(_d: &super::UDelta)    { RM.fetch_add(1, Ordering::Relaxed); }
    pub fn inc_multi(_d: &super::UDelta) { MULTI.fetch_add(1, Ordering::Relaxed); }
    pub fn inc_unsub(_d: &super::UDelta) { UNSUB.fetch_add(1, Ordering::Relaxed); }
    pub fn inc_batch(_d: &super::UDelta) { BATCH.fetch_add(1, Ordering::Relaxed); }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::scratch::UScope;
    use super::counters::*;
    use core::sync::atomic::Ordering;

    fn make_delta() -> UDelta {
        UDelta::new(0, UScope::Cell, 0, 0, 1, !0)
    }

    #[test]
    fn test_subscribe_and_publish() {
        RM.store(0, Ordering::Relaxed);
        let mut bus = DeltaBus::new();
        bus.subscribe(SubscriberChannel::ReadyMask, inc_rm);
        bus.publish(&make_delta());
        assert_eq!(RM.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_publish_multiple_subscribers() {
        MULTI.store(0, Ordering::Relaxed);
        let mut bus = DeltaBus::new();
        bus.subscribe(SubscriberChannel::ReadyMask, inc_multi);
        bus.subscribe(SubscriberChannel::RLKernel, inc_multi);
        bus.publish(&make_delta());
        assert_eq!(MULTI.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_unsubscribe() {
        UNSUB.store(0, Ordering::Relaxed);
        let mut bus = DeltaBus::new();
        bus.subscribe(SubscriberChannel::ReadyMask, inc_unsub);
        bus.subscribe(SubscriberChannel::RLKernel, inc_unsub);
        bus.unsubscribe(SubscriberChannel::ReadyMask);
        assert_eq!(bus.active_count(), 1);
        bus.publish(&make_delta());
        assert_eq!(UNSUB.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_bus_capacity() {
        let mut bus = DeltaBus::new();
        for _ in 0..MAX_SUBSCRIBERS {
            let ok = bus.subscribe(SubscriberChannel::Custom1, |_| {});
            assert!(ok);
        }
        let overflow = bus.subscribe(SubscriberChannel::Custom1, |_| {});
        assert!(!overflow);
    }

    #[test]
    fn test_publish_batch() {
        BATCH.store(0, Ordering::Relaxed);
        let mut bus = DeltaBus::new();
        bus.subscribe(SubscriberChannel::ReceiptChain, inc_batch);
        let deltas = [make_delta(), make_delta(), make_delta()];
        bus.publish_batch(&deltas);
        assert_eq!(BATCH.load(Ordering::Relaxed), 3);
    }
}
