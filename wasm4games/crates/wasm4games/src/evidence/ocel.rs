//! Object-centric event log (OCEL-style) types.
//!
//! Default build is allocation-free: an [`OcelEvent`] is `Copy` and links to a bounded set
//! of objects via [`ObjectRefs`]. An `alloc`-gated `OcelLog` (a growable event stream) and
//! JSON export are added on top of these core types.

/// A bounded set of `(object_type_code, object_id)` references for one event.
///
/// Fixed capacity keeps events `Copy` and allocation-free in the default `no_std` build.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ObjectRefs {
    items: [(u16, u64); Self::CAP],
    len: u8,
}

impl ObjectRefs {
    /// Maximum number of object references a single event may carry.
    pub const CAP: usize = 4;

    /// An empty set of object references.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            items: [(0, 0); Self::CAP],
            len: 0,
        }
    }

    /// Push a `(type_code, id)` reference. Silently saturates at [`Self::CAP`].
    #[inline]
    pub fn push(&mut self, type_code: u16, id: u64) {
        let idx = self.len as usize;
        if idx < Self::CAP {
            self.items[idx] = (type_code, id);
            self.len += 1;
        }
    }

    /// The live references.
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[(u16, u64)] {
        &self.items[..self.len as usize]
    }

    /// Number of live references.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.len as usize
    }

    /// Whether there are no references (an OCEL-laundering smell).
    #[inline]
    #[must_use]
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

/// An object-centric event: an activity at a logical time, linked to real objects.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
    #[inline]
    #[must_use]
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
    /// object count (1).
    const HEADER_BYTES: usize = 14;

    /// Bytes used to encode a single object reference: `type_code` (2) + `id` (8).
    const OBJECT_BYTES: usize = 10;

    /// Exact number of bytes [`Self::write_to`] will write for this event.
    #[inline]
    #[must_use]
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
    /// ```
    /// use wasm4games::evidence::ocel::OcelEvent;
    /// let ev = OcelEvent::new(0x0102, 0x0304, 0x05, 4);
    /// let mut buf = [0u8; 64];
    /// let n = ev.write_to(&mut buf).unwrap();
    /// assert_eq!(n, ev.encoded_len());
    /// ```
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

/// An error produced while emitting or serializing evidence.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EvidenceError {
    /// The destination buffer was too small to hold the encoded record.
    BufferTooSmall,
}

/// A growable, object-centric event stream.
///
/// Available only with the `alloc` feature: it owns a heap-backed [`alloc::vec::Vec`] of
/// [`OcelEvent`]s. The default `no_std` build stays allocation-free and uses the bounded
/// [`OcelEvent`] / [`ObjectRefs`] types directly.
#[cfg(feature = "alloc")]
#[derive(Clone, Debug, Default)]
pub struct OcelLog {
    events: alloc::vec::Vec<OcelEvent>,
}

#[cfg(feature = "alloc")]
impl OcelLog {
    /// An empty log.
    #[inline]
    #[must_use]
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
    #[must_use]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Whether the log holds no events.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// The events as a slice.
    #[inline]
    #[must_use]
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
    /// ```
    /// # #[cfg(feature = "alloc")] {
    /// use wasm4games::evidence::ocel::{OcelEvent, OcelLog};
    /// let mut log = OcelLog::new();
    /// log.push(OcelEvent::new(1, 2, 3, 4));
    /// let json = log.to_json();
    /// assert!(json.starts_with('['));
    /// # }
    /// ```
    #[must_use]
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
    fn write_to_reports_exact_len_and_header_only_with_no_objects() {
        let ev = OcelEvent::new(0x0102, 0x0304, 0x0506_0708_090a_0b0c, 4);
        assert_eq!(ev.encoded_len(), OcelEvent::HEADER_BYTES);
        let mut buf = [0u8; 64];
        let n = ev.write_to(&mut buf).unwrap();
        assert_eq!(n, OcelEvent::HEADER_BYTES);
        // Big-endian header layout.
        assert_eq!(&buf[0..2], &0x0102u16.to_be_bytes());
        assert_eq!(&buf[2..4], &0x0304u16.to_be_bytes());
        assert_eq!(&buf[4..12], &0x0506_0708_090a_0b0cu64.to_be_bytes());
        assert_eq!(buf[12], 4); // status
        assert_eq!(buf[13], 0); // object count
    }

    #[test]
    fn write_to_encodes_objects_deterministically() {
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
        assert_eq!(a[..na], b[..nb]); // deterministic
        assert_eq!(a[13], 2); // object count
                              // First object ref begins right after the header.
        assert_eq!(&a[14..16], &0x00aau16.to_be_bytes());
        assert_eq!(&a[16..24], &0x0102_0304_0506_0708u64.to_be_bytes());
    }

    #[test]
    fn write_to_rejects_small_buffer_without_writing() {
        let mut ev = OcelEvent::new(1, 2, 3, 4);
        ev.objects.push(5, 6);
        let mut small = [0u8; 8];
        assert_eq!(ev.write_to(&mut small), Err(EvidenceError::BufferTooSmall));
        // Nothing was written.
        assert_eq!(small, [0u8; 8]);
        // Exact-fit buffer succeeds.
        let mut exact = [0u8; OcelEvent::HEADER_BYTES + OcelEvent::OBJECT_BYTES];
        assert!(ev.write_to(&mut exact).is_ok());
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn ocel_log_collects_events() {
        let mut log = OcelLog::new();
        assert!(log.is_empty());
        log.push(OcelEvent::new(1, 2, 3, 4));
        log.push(OcelEvent::new(5, 6, 7, 8));
        assert_eq!(log.len(), 2);
        assert!(!log.is_empty());
        assert_eq!(log.as_slice().len(), 2);
        assert_eq!(log.iter().count(), 2);
        assert_eq!((&log).into_iter().count(), 2);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn to_json_is_object_centric() {
        let mut a = OcelEvent::new(10, 20, 30, 4);
        a.objects.push(100, 1000);
        a.objects.push(101, 1001);
        let mut log = OcelLog::new();
        log.push(a);
        log.push(OcelEvent::new(11, 21, 31, 7));

        let json = log.to_json();
        assert!(json.starts_with('['));
        assert!(json.ends_with(']'));
        // Scalar fields surface.
        assert!(json.contains("\"event_code\":10"));
        assert!(json.contains("\"activity\":20"));
        assert!(json.contains("\"timestamp\":30"));
        assert!(json.contains("\"status\":4"));
        // Each event carries its own linked objects (object-centric).
        assert!(
            json.contains("\"objects\":[{\"type\":100,\"id\":1000},{\"type\":101,\"id\":1001}]")
        );
        // The second event has an empty object array.
        assert!(json.contains("\"status\":7,\"objects\":[]"));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn empty_log_renders_empty_json_array() {
        let log = OcelLog::new();
        assert_eq!(log.to_json(), "[]");
    }
}
