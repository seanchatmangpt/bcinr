//! Object-centric event log (OCEL-style) types.
//!
//! Default build is allocation-free: an [`OcelEvent`] is `Copy` and links to a bounded set
//! of objects via [`ObjectRefs`]. An `alloc`-gated [`OcelLog`] (a growable event stream) and
//! JSON export are added on top of these core types.
//!
//! # OCEL model
//!
//! In OCEL-2.0, an event is an activity execution at a point in time that is associated with
//! one or more typed objects. [`OcelEvent`] encodes exactly that: an `event_code` (activity
//! type), an `activity` (pattern id), a `timestamp` (logical tick), a bounded set of
//! `(object_type_code, object_id)` pairs ([`ObjectRefs`]), and the admission `status` code.
//!
//! # Invariants
//!
//! * `ObjectRefs::len() <= ObjectRefs::CAP` at all times.
//! * `write_to` is deterministic and architecture-independent (big-endian encoding).
//! * An [`OcelEvent`] with no object links is valid but may indicate a laundering pattern —
//!   see [`ObjectRefs::is_empty`].

use core::fmt;

/// A bounded set of `(object_type_code, object_id)` references for one event.
///
/// Fixed capacity keeps [`OcelEvent`] `Copy` and allocation-free in the default `no_std`
/// build.  Pushing beyond [`Self::CAP`] silently saturates; the excess links are dropped.
///
/// # Invariants
///
/// `self.len() <= Self::CAP` at all times. There is no way to violate this invariant through
/// the public API.
///
/// # Examples
///
/// ```
/// use wasm4games::evidence::ocel::ObjectRefs;
/// let mut refs = ObjectRefs::new();
/// refs.push(1, 42);
/// refs.push(2, 99);
/// assert_eq!(refs.len(), 2);
/// assert_eq!(refs.as_slice(), &[(1, 42), (2, 99)]);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ObjectRefs {
    items: [(u16, u64); Self::CAP],
    len: u8,
}

impl ObjectRefs {
    /// Maximum number of object references a single event may carry.
    pub const CAP: usize = 4;

    /// An empty set of object references.
    ///
    /// Identical to [`Default::default`]; provided as `const fn` for use in `const` contexts.
    #[inline]
    #[must_use = "returns an empty ObjectRefs; discard only if you immediately overwrite it"]
    pub const fn new() -> Self {
        Self {
            items: [(0, 0); Self::CAP],
            len: 0,
        }
    }

    /// Push a `(type_code, id)` reference.
    ///
    /// Silently saturates at [`Self::CAP`]; excess pushes are dropped without error.
    #[inline]
    pub fn push(&mut self, type_code: u16, id: u64) {
        let idx = self.len as usize;
        if idx < Self::CAP {
            self.items[idx] = (type_code, id);
            self.len += 1;
        }
    }

    /// The live `(type_code, object_id)` references as a slice.
    #[inline]
    #[must_use = "returns a slice of the live object references; ignoring it is likely a bug"]
    pub fn as_slice(&self) -> &[(u16, u64)] {
        &self.items[..self.len as usize]
    }

    /// Number of live references.
    #[inline]
    #[must_use = "returns the count of live object references"]
    pub fn len(&self) -> usize {
        self.len as usize
    }

    /// Whether there are no references.
    ///
    /// An event with no object links is valid but may indicate an OCEL-laundering pattern
    /// (an event that cannot be traced back to any real object).
    #[inline]
    #[must_use = "returns true if there are no object references; check this before trusting the event"]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl Default for ObjectRefs {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ObjectRefs {
    /// Formats the object references as a bracketed list: `[(type_code, id), ...]`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[")?;
        for (i, &(tc, id)) in self.as_slice().iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            write!(f, "({tc}, {id})")?;
        }
        f.write_str("]")
    }
}

/// An object-centric event: an activity at a logical time, linked to real objects.
///
/// # Fields
///
/// * `event_code` — Event-type code, matches [`crate::ir::EventKind::code`].
/// * `activity` — Pattern id that produced the event.
/// * `timestamp` — Logical tick at which the event occurred.  Use [`From<u64>`] to build a
///   tick wrapper; here the raw `u64` is stored directly.
/// * `objects` — The [`ObjectRefs`] set linking this event to typed objects.
/// * `status` — Admission [`crate::class::status`] code at emission time.
///
/// # Invariants
///
/// The event is `Copy`; all state is inline with no heap pointers.
///
/// # Examples
///
/// ```
/// use wasm4games::evidence::ocel::OcelEvent;
/// let mut ev = OcelEvent::new(0x01, 0x02, 100, 4);
/// ev.objects.push(0xAA, 9999);
/// assert_eq!(ev.objects.len(), 1);
/// assert_eq!(ev.timestamp, 100);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct OcelEvent {
    /// Event-type code (matches [`crate::ir::EventKind::code`]).
    pub event_code: u16,
    /// The pattern id that produced the event.
    pub activity: u16,
    /// Logical tick at which the event occurred.
    pub timestamp: u64,
    /// The objects the event links to.
    pub objects: ObjectRefs,
    /// Admission [`crate::class::status`] code at emission time.
    pub status: u8,
}

impl OcelEvent {
    /// Construct an event with no object links yet.
    ///
    /// Object references can be pushed after construction via [`ObjectRefs::push`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::evidence::ocel::OcelEvent;
    /// let ev = OcelEvent::new(1, 2, 3, 4);
    /// assert!(ev.objects.is_empty());
    /// assert_eq!(ev.event_code, 1);
    /// assert_eq!(ev.activity, 2);
    /// assert_eq!(ev.timestamp, 3);
    /// assert_eq!(ev.status, 4);
    /// ```
    #[inline]
    #[must_use = "returns the new OcelEvent; bind it to a variable"]
    pub const fn new(event_code: u16, activity: u16, timestamp: u64, status: u8) -> Self {
        Self {
            event_code,
            activity,
            timestamp,
            objects: ObjectRefs::new(),
            status,
        }
    }

    /// Fixed-size header of the [`Self::write_to`] encoding, in bytes.
    ///
    /// Layout: `event_code` (2) + `activity` (2) + `timestamp` (8) + `status` (1) +
    /// object count (1) = 14 bytes total.
    pub const HEADER_BYTES: usize = 14;

    /// Bytes used to encode a single object reference: `type_code` (2) + `id` (8) = 10 bytes.
    pub const OBJECT_BYTES: usize = 10;

    /// Exact number of bytes [`Self::write_to`] will write for this event.
    ///
    /// Equal to [`Self::HEADER_BYTES`] + `self.objects.len()` × [`Self::OBJECT_BYTES`].
    #[inline]
    #[must_use = "returns the exact byte count needed for write_to; use it to pre-size your buffer"]
    pub fn encoded_len(&self) -> usize {
        Self::HEADER_BYTES + self.objects.len() * Self::OBJECT_BYTES
    }

    /// Serialize this event into `buf` using a compact, deterministic big-endian encoding,
    /// returning the number of bytes written.
    ///
    /// The encoding is allocation-free and self-describing enough to round-trip the scalar
    /// fields and every object reference. All multi-byte integers are big-endian so the byte
    /// stream is identical across architectures (a prerequisite for cross-host receipts).
    ///
    /// Returns [`EvidenceError::BufferTooSmall`] without writing if `buf` cannot hold the
    /// full record (see [`Self::encoded_len`]).
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::evidence::ocel::OcelEvent;
    /// let ev = OcelEvent::new(0x0102, 0x0304, 0x05, 4);
    /// let mut buf = [0u8; 64];
    /// let n = ev.write_to(&mut buf).unwrap();
    /// assert_eq!(n, ev.encoded_len());
    /// ```
    #[must_use = "returns the number of bytes written; check for EvidenceError::BufferTooSmall"]
    pub fn write_to(&self, buf: &mut [u8]) -> Result<usize, EvidenceError> {
        let need = self.encoded_len();
        if buf.len() < need {
            return Err(EvidenceError::BufferTooSmall);
        }
        let mut at = 0usize;
        // Local helper keeps the cursor logic in one place and avoids intermediate allocs.
        let put_u16 = |buf: &mut [u8], at: &mut usize, v: u16| {
            buf[*at..*at + 2].copy_from_slice(&v.to_be_bytes());
            *at += 2;
        };
        put_u16(buf, &mut at, self.event_code);
        put_u16(buf, &mut at, self.activity);
        buf[at..at + 8].copy_from_slice(&self.timestamp.to_be_bytes());
        at += 8;
        buf[at] = self.status;
        at += 1;
        let refs = self.objects.as_slice();
        buf[at] = refs.len() as u8;
        at += 1;
        for &(type_code, id) in refs {
            buf[at..at + 2].copy_from_slice(&type_code.to_be_bytes());
            at += 2;
            buf[at..at + 8].copy_from_slice(&id.to_be_bytes());
            at += 8;
        }
        Ok(at)
    }
}

impl fmt::Display for OcelEvent {
    /// Formats the event as `OcelEvent(code=N, activity=N, ts=N, status=N, objects=[...])`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "OcelEvent(code={}, activity={}, ts={}, status={}, objects={})",
            self.event_code, self.activity, self.timestamp, self.status, self.objects
        )
    }
}

/// An error produced while emitting or serializing evidence.
///
/// # Variants
///
/// * [`EvidenceError::BufferTooSmall`] — The caller-supplied buffer was shorter than the
///   encoded record. Pre-size the buffer using [`OcelEvent::encoded_len`].
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EvidenceError {
    /// The destination buffer was too small to hold the encoded record.
    BufferTooSmall,
}

impl fmt::Display for EvidenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvidenceError::BufferTooSmall => {
                f.write_str("evidence error: buffer too small for encoded record")
            }
        }
    }
}

/// Convert a raw `u16` event code into an [`OcelEvent`] with zeroed activity, timestamp, and
/// status.
///
/// Useful as a quick constructor when only the event-type code matters.
///
/// # Examples
///
/// ```
/// use wasm4games::evidence::ocel::OcelEvent;
/// let ev = OcelEvent::from(0x0042u16);
/// assert_eq!(ev.event_code, 0x0042);
/// assert_eq!(ev.activity, 0);
/// assert_eq!(ev.timestamp, 0);
/// assert_eq!(ev.status, 0);
/// ```
impl From<u16> for OcelEvent {
    #[inline]
    fn from(event_code: u16) -> Self {
        Self::new(event_code, 0, 0, 0)
    }
}

/// A growable, object-centric event stream.
///
/// Available only with the `alloc` feature: it owns a heap-backed [`alloc::vec::Vec`] of
/// [`OcelEvent`]s. The default `no_std` build stays allocation-free and uses the bounded
/// [`OcelEvent`] / [`ObjectRefs`] types directly.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "alloc")] {
/// use wasm4games::evidence::ocel::{OcelEvent, OcelLog};
/// let mut log = OcelLog::new();
/// log.push(OcelEvent::new(1, 2, 3, 4));
/// assert_eq!(log.len(), 1);
/// # }
/// ```
#[cfg(feature = "alloc")]
#[derive(Clone, Debug, Default)]
pub struct OcelLog {
    events: alloc::vec::Vec<OcelEvent>,
}

#[cfg(feature = "alloc")]
impl OcelLog {
    /// An empty log.
    ///
    /// Identical to [`Default::default`]; provided for discoverability.
    #[inline]
    #[must_use = "returns an empty OcelLog; bind it to a variable"]
    pub fn new() -> Self {
        Self {
            events: alloc::vec::Vec::new(),
        }
    }

    /// Append an event to the stream.
    #[inline]
    pub fn push(&mut self, event: OcelEvent) {
        self.events.push(event);
    }

    /// Number of events in the log.
    #[inline]
    #[must_use = "returns the event count; ignoring it is likely a bug"]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Whether the log holds no events.
    #[inline]
    #[must_use = "returns true if the log is empty; check before iterating"]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// The events as a slice, in emission order.
    #[inline]
    #[must_use = "returns a slice of events; ignoring it discards the reference"]
    pub fn as_slice(&self) -> &[OcelEvent] {
        &self.events
    }

    /// Iterate over the events in emission order.
    #[inline]
    pub fn iter(&self) -> core::slice::Iter<'_, OcelEvent> {
        self.events.iter()
    }

    /// Render the log as an OCEL-flavored JSON array.
    ///
    /// Object-centric: every event carries its own `objects` array of
    /// `{ "type": <code>, "id": <id> }` links, so the relation between events and objects is
    /// explicit rather than reconstructed. The JSON is hand-rolled (no `serde`) to keep the
    /// crate dependency-free; all values are integers, so no string escaping is required.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "alloc")] {
    /// use wasm4games::evidence::ocel::{OcelEvent, OcelLog};
    /// let mut log = OcelLog::new();
    /// log.push(OcelEvent::new(1, 2, 3, 4));
    /// let json = log.to_json();
    /// assert!(json.starts_with('['));
    /// # }
    /// ```
    #[must_use = "returns an owned JSON string; bind or use it"]
    pub fn to_json(&self) -> alloc::string::String {
        use core::fmt::Write as _;
        let mut s = alloc::string::String::new();
        s.push('[');
        for (i, ev) in self.events.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            // Scalar fields. `write!` to a String is infallible, hence the `let _ =`.
            let _ = write!(
                s,
                "{{\"event_code\":{},\"activity\":{},\"timestamp\":{},\"status\":{},\"objects\":[",
                ev.event_code, ev.activity, ev.timestamp, ev.status
            );
            for (j, &(type_code, id)) in ev.objects.as_slice().iter().enumerate() {
                if j > 0 {
                    s.push(',');
                }
                let _ = write!(s, "{{\"type\":{},\"id\":{}}}", type_code, id);
            }
            s.push_str("]}");
        }
        s.push(']');
        s
    }
}

#[cfg(feature = "alloc")]
impl<'a> IntoIterator for &'a OcelLog {
    type Item = &'a OcelEvent;
    type IntoIter = core::slice::Iter<'a, OcelEvent>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.events.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_refs_behavior() {
        // empty / default
        let r = ObjectRefs::new();
        assert!(r.is_empty());
        assert_eq!(r.len(), 0);
        assert_eq!(r.as_slice(), &[]);
        assert_eq!(ObjectRefs::default(), ObjectRefs::new());

        // push and retrieve
        let mut r = ObjectRefs::new();
        r.push(10, 100);
        r.push(20, 200);
        assert_eq!(r.len(), 2);
        assert!(!r.is_empty());
        assert_eq!(r.as_slice(), &[(10, 100), (20, 200)]);

        // saturates at CAP
        let mut r = ObjectRefs::new();
        for i in 0..ObjectRefs::CAP + 5 {
            r.push(i as u16, i as u64);
        }
        assert_eq!(r.len(), ObjectRefs::CAP);

        // Hash is consistent
        use core::hash::{Hash, Hasher};
        struct SimpleHasher(u64);
        impl Hasher for SimpleHasher {
            fn finish(&self) -> u64 {
                self.0
            }
            fn write(&mut self, bytes: &[u8]) {
                for &b in bytes {
                    self.0 = self.0.wrapping_mul(31).wrapping_add(b as u64);
                }
            }
        }
        let mut r = ObjectRefs::new();
        r.push(7, 42);
        let mut h1 = SimpleHasher(0);
        let mut h2 = SimpleHasher(0);
        r.hash(&mut h1);
        r.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn ocel_event_construction_and_encoding() {
        // new / From<u16> / multi-object
        let ev = OcelEvent::new(0x0A, 0x0B, 999, 4);
        assert!(ev.objects.is_empty());
        assert_eq!(
            (ev.event_code, ev.activity, ev.timestamp, ev.status),
            (0x0A, 0x0B, 999, 4)
        );

        let ev = OcelEvent::from(0x1234u16);
        assert_eq!(
            (ev.event_code, ev.activity, ev.timestamp, ev.status),
            (0x1234, 0, 0, 0)
        );
        assert!(ev.objects.is_empty());

        let mut ev = OcelEvent::new(7, 9, 11, 0);
        ev.objects.push(0xAA, 111);
        ev.objects.push(0xBB, 222);
        ev.objects.push(0xCC, 333);
        assert_eq!(
            ev.objects.as_slice(),
            &[(0xAA, 111), (0xBB, 222), (0xCC, 333)]
        );

        // write_to: header-only, big-endian layout
        let ev = OcelEvent::new(0x0102, 0x0304, 0x0506_0708_090a_0b0c, 4);
        assert_eq!(ev.encoded_len(), OcelEvent::HEADER_BYTES);
        let mut buf = [0u8; 64];
        let n = ev.write_to(&mut buf).unwrap();
        assert_eq!(n, OcelEvent::HEADER_BYTES);
        assert_eq!(&buf[0..2], &0x0102u16.to_be_bytes());
        assert_eq!(&buf[2..4], &0x0304u16.to_be_bytes());
        assert_eq!(&buf[4..12], &0x0506_0708_090a_0b0cu64.to_be_bytes());
        assert_eq!((buf[12], buf[13]), (4, 0)); // status, object count

        // write_to: deterministic with objects
        let mut ev = OcelEvent::new(7, 9, 11, 4);
        ev.objects.push(0x00aa, 0x0102_0304_0506_0708);
        ev.objects.push(0x00bb, 0x1112_1314_1516_1718);
        let expected = OcelEvent::HEADER_BYTES + 2 * OcelEvent::OBJECT_BYTES;
        assert_eq!(ev.encoded_len(), expected);
        let mut a = [0u8; 64];
        let mut b = [0u8; 64];
        let na = ev.write_to(&mut a).unwrap();
        let nb = ev.write_to(&mut b).unwrap();
        assert_eq!(na, expected);
        assert_eq!(a[..na], b[..nb]);
        assert_eq!(a[13], 2);
        assert_eq!(&a[14..16], &0x00aau16.to_be_bytes());
        assert_eq!(&a[16..24], &0x0102_0304_0506_0708u64.to_be_bytes());

        // write_to: rejects too-small buffer without writing
        let mut ev = OcelEvent::new(1, 2, 3, 4);
        ev.objects.push(5, 6);
        let mut small = [0u8; 8];
        assert_eq!(ev.write_to(&mut small), Err(EvidenceError::BufferTooSmall));
        assert_eq!(small, [0u8; 8]);
        let mut exact = [0u8; OcelEvent::HEADER_BYTES + OcelEvent::OBJECT_BYTES];
        assert!(ev.write_to(&mut exact).is_ok());

        // EvidenceError Copy + PartialEq
        let e = EvidenceError::BufferTooSmall;
        assert_eq!(e, EvidenceError::BufferTooSmall);
        assert_eq!(e, e); // Copy
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn ocel_log_and_json() {
        // collection basics
        let mut log = OcelLog::new();
        assert!(log.is_empty());
        log.push(OcelEvent::new(1, 2, 3, 4));
        log.push(OcelEvent::new(5, 6, 7, 8));
        assert_eq!(log.len(), 2);
        assert!(!log.is_empty());
        assert_eq!(log.iter().count(), 2);
        assert_eq!((&log).into_iter().count(), 2);

        // empty log renders []
        assert_eq!(OcelLog::new().to_json(), "[]");

        // JSON is object-centric with scalar fields and linked objects
        let mut a = OcelEvent::new(10, 20, 30, 4);
        a.objects.push(100, 1000);
        a.objects.push(101, 1001);
        let mut log = OcelLog::new();
        log.push(a);
        log.push(OcelEvent::new(11, 21, 31, 7));
        let json = log.to_json();
        assert!(json.starts_with('[') && json.ends_with(']'));
        assert!(json.contains("\"event_code\":10"));
        assert!(json.contains("\"activity\":20"));
        assert!(json.contains("\"timestamp\":30"));
        assert!(json.contains("\"status\":4"));
        assert!(
            json.contains("\"objects\":[{\"type\":100,\"id\":1000},{\"type\":101,\"id\":1001}]")
        );
        assert!(json.contains("\"status\":7,\"objects\":[]"));

        // multiple objects per event are all present in JSON
        let mut ev = OcelEvent::new(42, 1, 9999, 0);
        ev.objects.push(0x01, 100);
        ev.objects.push(0x02, 200);
        ev.objects.push(0x03, 300);
        let mut log = OcelLog::new();
        log.push(ev);
        let json = log.to_json();
        assert!(json.contains("\"type\":1,\"id\":100"));
        assert!(json.contains("\"type\":2,\"id\":200"));
        assert!(json.contains("\"type\":3,\"id\":300"));
    }
}
