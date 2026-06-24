//! ocel — Object-Centric Event Log (OCEL 2.0) support for bcinr-powl.
//!
//! Events are pushed into a `LockFreeMpmcRing` during execution (one CAS, ~10 ns).
//! Serialisation (`events_to_jsonl`) and conformance checking (`OcelCase`) run
//! on a consumer that drains the ring asynchronously — never on the scheduler
//! hot path.
//!
//! # Thread model
//!
//! `OcelRing` wraps the ring directly. The hot-path caller pushes `OcelEvent`
//! items via `push_event`; a background consumer calls `drain_into` periodically
//! to harvest completed events for serialisation or analysis.

#![forbid(unsafe_code)]

use bcinr_logic::patterns::LockFreeMpmcRing;

// ---------------------------------------------------------------------------
// OcelEvent — 24 bytes, Default + Copy for ring compatibility
// ---------------------------------------------------------------------------

/// A single OCEL 2.0 event emitted by the POWL scheduler.
///
/// Kept to 24 bytes so that four events fit in a single cache line (96 bytes
/// with two-word header on typical CPUs).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OcelEvent {
    /// Unique event identifier (monotonic, set by the emitting scheduler).
    pub event_id: u64,
    /// Case (workflow instance) identifier.
    pub case_id: u64,
    /// Logical tick at which the event was emitted.
    pub tick: u32,
    /// Op slot index on the tape (0..63).
    pub op_idx: u8,
    /// Topological order position of this op.
    pub topo: u8,
    /// Reserved for future use.
    pub _pad: [u8; 2],
}

impl Default for OcelEvent {
    fn default() -> Self {
        Self {
            event_id: 0,
            case_id: 0,
            tick: 0,
            op_idx: 0,
            topo: 0,
            _pad: [0u8; 2],
        }
    }
}

// Compile-time size check.
const _OCEL_EVENT_SIZE: () = assert!(
    core::mem::size_of::<OcelEvent>() == 24,
    "OcelEvent must be exactly 24 bytes"
);

// ---------------------------------------------------------------------------
// OcelRing — ring-buffered event emitter
// ---------------------------------------------------------------------------

/// A ring-buffered OCEL event emitter.
///
/// `N` is the ring capacity; must be a power of two (enforced at construction).
/// The hot-path operation is `push_event` (~10 ns, one CAS). The cold-path
/// operation is `drain_into` for background consumption.
pub struct OcelRing<const N: usize> {
    ring: LockFreeMpmcRing<OcelEvent, N>,
}

impl<const N: usize> OcelRing<N> {
    /// Construct a new ring.  Returns `Err` if N is not a power of two.
    pub fn new() -> Result<Self, &'static str> {
        let ring = LockFreeMpmcRing::<OcelEvent, N>::new_checked()?;
        Ok(Self { ring })
    }

    /// Push an event into the ring (hot path, ~10 ns).
    ///
    /// Returns `true` on success, `false` if the ring is full.
    #[inline(always)]
    pub fn push_event(&self, ev: OcelEvent) -> bool {
        self.ring.push_t1(ev) == u32::MAX
    }

    /// Drain available events into `buf` (cold path, background consumer).
    ///
    /// Returns the number of events written.  Stops when the ring is empty
    /// or `buf` is full.
    pub fn drain_into(&self, buf: &mut [OcelEvent]) -> usize {
        let mut count = 0;
        while count < buf.len() {
            match self.ring.pop_t1() {
                (Some(ev), _) => {
                    buf[count] = ev;
                    count += 1;
                }
                _ => break,
            }
        }
        count
    }
}

// ---------------------------------------------------------------------------
// events_to_jsonl — OCEL 2.0 JSON-Lines serialisation (no_std compatible)
// ---------------------------------------------------------------------------

/// Write OCEL 2.0 JSON-Lines for `events` into `out` (a byte buffer).
///
/// Each event is written as one JSON object per line:
/// `{"event_id":N,"case_id":N,"tick":N,"op_idx":N,"topo":N}`
///
/// Returns the number of bytes written, or `0` if the buffer is too small.
/// This function runs on the consumer side — **never called from the scheduler**.
pub fn events_to_jsonl(events: &[OcelEvent], out: &mut [u8]) -> usize {
    let mut pos = 0usize;

    for ev in events {
        // Format: {"event_id":EID,"case_id":CID,"tick":T,"op_idx":O,"topo":TOP}

        // Max length per event: 2 + 10 + 11 + 10 + 11 + 7 + 3 + 9 + 3 + 7 + 3 + 2 = ~80 bytes
        let mut tmp = [0u8; 128];
        let n = write_ocel_json(&mut tmp, ev);
        if pos + n > out.len() {
            break;
        }
        out[pos..pos + n].copy_from_slice(&tmp[..n]);
        pos += n;
    }
    pos
}

/// Write a single OcelEvent as a JSON object line into `buf`.
/// Returns the number of bytes written.
fn write_ocel_json(buf: &mut [u8; 128], ev: &OcelEvent) -> usize {
    let mut pos = 0usize;
    // {"event_id":
    let prefix = b"{\"event_id\":";
    buf[pos..pos + prefix.len()].copy_from_slice(prefix);
    pos += prefix.len();
    pos += write_u64(&mut buf[pos..], ev.event_id);
    // ,"case_id":
    let f2 = b",\"case_id\":";
    buf[pos..pos + f2.len()].copy_from_slice(f2);
    pos += f2.len();
    pos += write_u64(&mut buf[pos..], ev.case_id);
    // ,"tick":
    let f3 = b",\"tick\":";
    buf[pos..pos + f3.len()].copy_from_slice(f3);
    pos += f3.len();
    pos += write_u32(&mut buf[pos..], ev.tick);
    // ,"op_idx":
    let f4 = b",\"op_idx\":";
    buf[pos..pos + f4.len()].copy_from_slice(f4);
    pos += f4.len();
    pos += write_u8_val(&mut buf[pos..], ev.op_idx);
    // ,"topo":
    let f5 = b",\"topo\":";
    buf[pos..pos + f5.len()].copy_from_slice(f5);
    pos += f5.len();
    pos += write_u8_val(&mut buf[pos..], ev.topo);
    // }
    buf[pos] = b'}';
    buf[pos + 1] = b'\n';
    pos += 2;
    pos
}

fn write_u64(buf: &mut [u8], mut n: u64) -> usize {
    if n == 0 {
        buf[0] = b'0';
        return 1;
    }
    let mut tmp = [0u8; 20];
    let mut len = 0usize;
    while n > 0 {
        tmp[len] = b'0' + (n % 10) as u8;
        n /= 10;
        len += 1;
    }
    for i in 0..len {
        buf[i] = tmp[len - 1 - i];
    }
    len
}

fn write_u32(buf: &mut [u8], n: u32) -> usize {
    write_u64(buf, n as u64)
}

fn write_u8_val(buf: &mut [u8], n: u8) -> usize {
    write_u64(buf, n as u64)
}

// ---------------------------------------------------------------------------
// OcelCase — per-case fired-op accumulator
// ---------------------------------------------------------------------------

/// Accumulates fired op indices for a single workflow case (instance).
///
/// Tracks which ops have fired via a 64-bit bitmask.  Provides conformance
/// checking against an expected trace mask.
#[derive(Clone, Copy, Debug, Default)]
pub struct OcelCase {
    case_id:    u64,
    fired_mask: u64,
    event_count: u32,
}

impl OcelCase {
    /// Create a new case accumulator.
    pub const fn new(case_id: u64) -> Self {
        Self { case_id, fired_mask: 0, event_count: 0 }
    }

    /// Record a fired event into this case.
    ///
    /// Branchless: no conditional on op_idx range (caller must ensure op_idx < 64).
    #[inline(always)]
    pub fn record(&mut self, ev: &OcelEvent) {
        let bit = 1u64 << (ev.op_idx & 63);
        self.fired_mask |= bit;
        self.event_count = self.event_count.saturating_add(1);
    }

    /// Returns a bitmask of all ops that have fired for this case.
    #[inline(always)]
    pub fn fired_mask(&self) -> u64 {
        self.fired_mask
    }

    /// Returns `true` if the fired ops exactly match `expected_trace`.
    ///
    /// Branchless: uses XOR + zero-check.
    #[inline(always)]
    pub fn conforms_to(&self, expected_trace: u64) -> bool {
        (self.fired_mask ^ expected_trace) == 0
    }

    /// The case identifier.
    pub fn case_id(&self) -> u64 {
        self.case_id
    }

    /// Number of events recorded.
    pub fn event_count(&self) -> u32 {
        self.event_count
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ocel_event_size_is_24() {
        assert_eq!(core::mem::size_of::<OcelEvent>(), 24);
    }

    #[test]
    fn ocel_ring_push_pop_roundtrip() {
        let ring = OcelRing::<16>::new().expect("ring construction");
        let ev = OcelEvent {
            event_id: 42,
            case_id: 7,
            tick: 3,
            op_idx: 2,
            topo: 1,
            _pad: [0; 2],
        };
        assert!(ring.push_event(ev), "push must succeed on empty ring");

        let mut buf = [OcelEvent::default(); 4];
        let n = ring.drain_into(&mut buf);
        assert_eq!(n, 1);
        assert_eq!(buf[0], ev);
    }

    #[test]
    fn ocel_ring_full_returns_false() {
        let ring = OcelRing::<4>::new().expect("ring");
        // Fill all 4 slots.
        for i in 0..4u64 {
            let ev = OcelEvent { event_id: i, ..OcelEvent::default() };
            ring.push_event(ev);
        }
        // 5th push should fail (ring full).
        let ev5 = OcelEvent { event_id: 99, ..OcelEvent::default() };
        assert!(!ring.push_event(ev5), "push into full ring must return false");
    }

    #[test]
    fn events_to_jsonl_produces_valid_output() {
        let events = [
            OcelEvent { event_id: 1, case_id: 100, tick: 0, op_idx: 0, topo: 0, _pad: [0; 2] },
            OcelEvent { event_id: 2, case_id: 100, tick: 1, op_idx: 1, topo: 1, _pad: [0; 2] },
        ];
        let mut out = [0u8; 512];
        let n = events_to_jsonl(&events, &mut out);
        assert!(n > 0, "must write some bytes");
        let s = core::str::from_utf8(&out[..n]).expect("valid UTF-8");
        assert!(s.contains("event_id"), "must contain event_id field");
        assert!(s.contains("case_id"), "must contain case_id field");
        // Two lines
        assert_eq!(s.lines().count(), 2);
    }

    #[test]
    fn ocel_case_conforms_to_expected_trace() {
        let mut case = OcelCase::new(1);
        let ev0 = OcelEvent { op_idx: 0, ..OcelEvent::default() };
        let ev1 = OcelEvent { op_idx: 1, ..OcelEvent::default() };
        case.record(&ev0);
        case.record(&ev1);
        assert_eq!(case.fired_mask(), 0b11);
        assert!(case.conforms_to(0b11));
        assert!(!case.conforms_to(0b01));
    }

    #[test]
    fn ocel_case_record_is_idempotent_for_same_op() {
        let mut case = OcelCase::new(5);
        let ev = OcelEvent { op_idx: 3, ..OcelEvent::default() };
        case.record(&ev);
        case.record(&ev);
        assert_eq!(case.fired_mask(), 1 << 3, "double-record must not set extra bits");
    }
}
