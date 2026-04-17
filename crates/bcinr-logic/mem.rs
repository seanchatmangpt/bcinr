//! Memory Substrate: Arenas, rings, slabs, and epochs for zero-alloc execution.

/// A simple bump-style scratch arena for reusable temporary storage.
pub(crate) struct ScratchArena {
    data: Vec<u8>,
    offset: usize,
}

impl ScratchArena {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: vec![0; capacity],
            offset: 0,
        }
    }

    pub fn alloc(&mut self, size: usize) -> Option<&mut [u8]> {
        if self.offset + size > self.data.len() {
            return None;
        }
        let start = self.offset;
        self.offset += size;
        Some(&mut self.data[start..self.offset])
    }

    pub fn reset(&mut self) {
        self.offset = 0;
    }
}

/// An array that uses an epoch to track whether a slot has been "cleared".
pub(crate) struct EpochArray<T: Copy + Default> {
    data: Vec<T>,
    epochs: Vec<u32>,
    current_epoch: u32,
}

impl<T: Copy + Default> EpochArray<T> {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![T::default(); size],
            epochs: vec![0; size],
            current_epoch: 1,
        }
    }

    pub fn get(&self, idx: usize) -> T {
        if self.epochs[idx] == self.current_epoch {
            self.data[idx]
        } else {
            T::default()
        }
    }

    pub fn set(&mut self, idx: usize, val: T) {
        self.data[idx] = val;
        self.epochs[idx] = self.current_epoch;
    }

    pub fn clear_all(&mut self) {
        self.current_epoch = self.current_epoch.wrapping_add(1);
        if self.current_epoch == 0 {
            // Rare: reset all epochs on wrap
            self.epochs.fill(0);
            self.current_epoch = 1;
        }
    }
}

/// A fixed-capacity ring buffer.
pub(crate) struct RingBuffer<T: Copy, const N: usize> {
    data: [T; N],
    head: usize,
    count: usize,
}

impl<T: Copy + Default, const N: usize> RingBuffer<T, N> {
    pub fn new() -> Self {
        Self {
            data: [T::default(); N],
            head: 0,
            count: 0,
        }
    }

    pub fn push(&mut self, val: T) {
        let idx = (self.head + self.count) % N;
        self.data[idx] = val;
        if self.count < N {
            self.count += 1;
        } else {
            self.head = (self.head + 1) % N;
        }
    }

    pub fn get(&self, idx: usize) -> Option<T> {
        if idx >= self.count {
            return None;
        }
        Some(self.data[(self.head + idx) % N])
    }

    pub fn len(&self) -> usize {
        self.count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scratch_arena() {
        let mut arena = ScratchArena::new(1024);
        let s1 = arena.alloc(10).unwrap();
        assert_eq!(s1.len(), 10);
        arena.reset();
        let s2 = arena.alloc(100).unwrap();
        assert_eq!(s2.len(), 100);
    }

    #[test]
    fn test_epoch_array() {
        let mut arr = EpochArray::new(10);
        assert_eq!(arr.get(5), 0);
        arr.set(5, 42);
        assert_eq!(arr.get(5), 42);
        arr.clear_all();
        assert_eq!(arr.get(5), 0);
        arr.set(5, 99);
        assert_eq!(arr.get(5), 99);
    }

    #[test]
    fn test_ring_buffer() {
        let mut ring = RingBuffer::<u32, 3>::new();
        ring.push(1);
        ring.push(2);
        ring.push(3);
        assert_eq!(ring.get(0), Some(1));
        ring.push(4);
        assert_eq!(ring.get(0), Some(2));
        assert_eq!(ring.get(2), Some(4));
    }
}
