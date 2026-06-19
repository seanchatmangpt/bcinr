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
}
