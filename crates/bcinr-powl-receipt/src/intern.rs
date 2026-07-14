//! intern — zero-allocation string interning tables for activity labels and object type names.
//!
//! Both tables use a flat byte arena with a u16 offset table. All operations are
//! branchless-friendly and `no_std`-compatible (no heap allocation).

/// Interning table for activity label strings.
///
/// Stores up to 256 distinct labels in a 4096-byte flat arena.
/// Duplicate labels return the same index without inserting a new entry.
pub struct ActivityTable {
    data: [u8; 4096],
    offsets: [u16; 256],
    len: u8,
}

impl ActivityTable {
    /// Create an empty table.
    pub const fn new() -> Self {
        Self {
            data: [0u8; 4096],
            offsets: [0u16; 256],
            len: 0,
        }
    }

    /// Intern `label`, returning its index.
    ///
    /// If the label already exists its existing index is returned.
    /// If the table is full (256 labels) or the arena is exhausted this panics —
    /// callers are responsible for bounding input at manufacture time.
    pub fn intern(&mut self, label: &str) -> u16 {
        let bytes = label.as_bytes();
        // Linear scan for existing entry.
        for i in 0..self.len as usize {
            let stored = self.get_raw(i as u16);
            if stored == bytes {
                return i as u16;
            }
        }
        // Insert new entry.
        let idx = self.len as usize;
        assert!(
            idx < 256,
            "ActivityTable: too many distinct labels (max 256)"
        );
        // Find write position: for idx 0 write at 0; otherwise after previous entry.
        let write_pos: usize = if idx == 0 {
            0
        } else {
            let prev_off = self.offsets[idx - 1] as usize;
            // previous entry length is stored as first byte
            let prev_len = self.data[prev_off] as usize;
            prev_off + 1 + prev_len
        };
        let needed = 1 + bytes.len();
        assert!(write_pos + needed <= 4096, "ActivityTable: arena exhausted");
        self.offsets[idx] = write_pos as u16;
        self.data[write_pos] = bytes.len() as u8;
        self.data[write_pos + 1..write_pos + 1 + bytes.len()].copy_from_slice(bytes);
        self.len += 1;
        idx as u16
    }

    /// Retrieve the label at `idx`.
    ///
    /// # Panics
    ///
    /// Panics if `idx >= self.len`.
    pub fn get(&self, idx: u16) -> &str {
        let raw = self.get_raw(idx);
        // SAFETY: bytes were inserted from a valid &str, so they are valid UTF-8.
        core::str::from_utf8(raw).expect("ActivityTable: corrupted UTF-8")
    }

    fn get_raw(&self, idx: u16) -> &[u8] {
        assert!(
            (idx as usize) < self.len as usize,
            "ActivityTable: index out of bounds"
        );
        let off = self.offsets[idx as usize] as usize;
        let len = self.data[off] as usize;
        &self.data[off + 1..off + 1 + len]
    }
}

impl Default for ActivityTable {
    fn default() -> Self {
        Self::new()
    }
}

// ── ObjTypeTable ─────────────────────────────────────────────────────────────

/// Interning table for object type name strings.
///
/// Identical layout to [`ActivityTable`] — separate type so callers cannot
/// accidentally pass an object type index where an activity index is expected.
pub struct ObjTypeTable {
    data: [u8; 4096],
    offsets: [u16; 256],
    len: u8,
}

impl ObjTypeTable {
    /// Create an empty table.
    pub const fn new() -> Self {
        Self {
            data: [0u8; 4096],
            offsets: [0u16; 256],
            len: 0,
        }
    }

    /// Intern `name`, returning its index.
    pub fn intern(&mut self, name: &str) -> u16 {
        let bytes = name.as_bytes();
        for i in 0..self.len as usize {
            let stored = self.get_raw(i as u16);
            if stored == bytes {
                return i as u16;
            }
        }
        let idx = self.len as usize;
        assert!(idx < 256, "ObjTypeTable: too many distinct types (max 256)");
        let write_pos: usize = if idx == 0 {
            0
        } else {
            let prev_off = self.offsets[idx - 1] as usize;
            let prev_len = self.data[prev_off] as usize;
            prev_off + 1 + prev_len
        };
        let needed = 1 + bytes.len();
        assert!(write_pos + needed <= 4096, "ObjTypeTable: arena exhausted");
        self.offsets[idx] = write_pos as u16;
        self.data[write_pos] = bytes.len() as u8;
        self.data[write_pos + 1..write_pos + 1 + bytes.len()].copy_from_slice(bytes);
        self.len += 1;
        idx as u16
    }

    /// Retrieve the type name at `idx`.
    pub fn get(&self, idx: u16) -> &str {
        let raw = self.get_raw(idx);
        core::str::from_utf8(raw).expect("ObjTypeTable: corrupted UTF-8")
    }

    fn get_raw(&self, idx: u16) -> &[u8] {
        assert!(
            (idx as usize) < self.len as usize,
            "ObjTypeTable: index out of bounds"
        );
        let off = self.offsets[idx as usize] as usize;
        let len = self.data[off] as usize;
        &self.data[off + 1..off + 1 + len]
    }
}

impl Default for ObjTypeTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activity_table_intern_and_get() {
        let mut t = ActivityTable::new();
        let a = t.intern("place_order");
        let b = t.intern("ship");
        let c = t.intern("place_order"); // duplicate
        assert_eq!(a, c, "duplicate must return same index");
        assert_ne!(a, b);
        assert_eq!(t.get(a), "place_order");
        assert_eq!(t.get(b), "ship");
    }

    #[test]
    fn obj_type_table_intern_and_get() {
        let mut t = ObjTypeTable::new();
        let order = t.intern("order");
        let item = t.intern("item");
        assert_eq!(t.get(order), "order");
        assert_eq!(t.get(item), "item");
        assert_eq!(t.intern("order"), order, "duplicate must be idempotent");
    }

    #[test]
    fn activity_table_many_labels() {
        let mut t = ActivityTable::new();
        for i in 0..50u8 {
            let label = alloc_label(i);
            let idx = t.intern(&label);
            assert_eq!(idx as u8, i);
        }
        for i in 0..50u8 {
            let label = alloc_label(i);
            assert_eq!(t.get(i as u16), label.as_str());
        }
    }

    fn alloc_label(i: u8) -> std::string::String {
        std::format!("activity_{i}")
    }
}
