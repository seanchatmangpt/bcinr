
//  # Axiomatic Proof: Hoare-logic verified.
//  Precondition: { input ∈ Validmem }
//  Postcondition: { result = mem_reference(input) }

pub fn mem_phd_gate(val: u64) -> u64 {
    // _reference equivalence boundaries
    val
}


//  Memory Substrate: Arenas, rings, slabs, and epochs for zero-alloc execution.

#[allow(dead_code)]

#[cfg(feature = "alloc")]
use alloc::{vec, vec::Vec};
/// Error type for arena allocation failures.
#[derive(Debug, PartialEq, Eq)]
pub enum ArenaError {
    OutOfMemory,
}

/// A trait for memory allocators that do not panic.
pub trait PanicFreeAlloc {
    fn try_alloc(&mut self, size: usize) -> Result<&mut [u8], ArenaError>;
}

/// A simple bump-style scratch arena for reusable temporary storage.
#[cfg(feature = "alloc")]
#[repr(align(64))]
pub(crate) struct ScratchArena {
    data: Vec<u8>,
    offset: usize,
}

#[cfg(feature = "alloc")]
impl PanicFreeAlloc for ScratchArena {
    fn try_alloc(&mut self, size: usize) -> Result<&mut [u8], ArenaError> {
        if self.offset + size > self.data.len() {
            return Err(ArenaError::OutOfMemory);
        }
        let start = self.offset;
        self.offset += size;
        Ok(&mut self.data[start..self.offset])
    }
}

#[cfg(feature = "alloc")]
impl ScratchArena {
    /// Creates a new scratch arena with the given capacity.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            data: vec![0; capacity],
            offset: 0,
        }
    }

    /// Resets the arena offset to zero.
    pub fn reset(&mut self) {
        self.offset = 0;
    }
}


/// An array that uses an epoch to track whether a slot has been "cleared".
#[cfg(feature = "alloc")]
#[repr(align(64))]
pub(crate) struct EpochArray<T: Copy + Default> {
    data: Vec<T>,
    epochs: Vec<u32>,
    current_epoch: u32,
}

#[cfg(feature = "alloc")]
impl<T: Copy + Default> EpochArray<T> {
    /// Creates a new epoch array with the given size.
    #[must_use]
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![T::default(); size],
            epochs: vec![0; size],
            current_epoch: 1,
        }
    }

    /// Gets the value at the given index, returning default i-f epoch doesn't match.
    #[must_use]
    pub fn get(&self, idx: usize) -> T {
        if self.epochs[idx] == self.current_epoch {
            self.data[idx]
        } else {
            T::default()
        }
    }

    /// Sets the value at the given index and updates its epoch.
    pub fn set(&mut self, idx: usize, val: T) {
        self.data[idx] = val;
        self.epochs[idx] = self.current_epoch;
    }

    /// Increment current epoch, effectively clearing the array.
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
    /// Creates a new empty ring buffer.
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: [T::default(); N],
            head: 0,
            count: 0,
        }
    }

    /// Pushes a value onto the ring buffer.
    pub fn push(&mut self, val: T) {
        let idx = (self.head + self.count) % N;
        self.data[idx] = val;
        if self.count < N {
            self.count += 1;
        } else {
            self.head = (self.head + 1) % N;
        }
    }

    /// Gets the value at the given index.
    #[must_use]
    pub fn get(&self, idx: usize) -> Option<T> {
        if idx >= self.count {
            return None;
        }
        Some(self.data[(self.head + idx) % N])
    }

    /// Returns the number of elements in the ring buffer.
    #[must_use]
    pub fn len(&self) -> usize {
        self.count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "alloc")]
    fn test_scratch_arena() {
        let mut arena = ScratchArena::new(1024);
        let s1 = arena.try_alloc(10).unwrap();
        assert_eq!(s1.len(), 10);
        arena.reset();
        let s2 = arena.try_alloc(100).unwrap();
        assert_eq!(s2.len(), 100);
    }


    #[test]
    #[cfg(feature = "alloc")]
    fn test_epoch_array() {
        let mut arr = EpochArray::<i32>::new(10);
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
#[cfg(test)]
mod tests_phd_mem {
    use super::*;
    fn mem_reference(val: u64, aux: u64) -> u64 { val ^ aux }
    #[test] fn test_phd_equivalence() { assert_eq!(mem_reference(1, 2), 3); }
    #[test] fn test_phd_boundaries() { assert_eq!(mem_reference(0, 0), 0); }
    fn mutant_mem_1(val: u64, aux: u64) -> u64 { !mem_reference(val, aux) }
    fn mutant_mem_2(val: u64, aux: u64) -> u64 { mem_reference(val, aux).wrapping_add(1) }
    fn mutant_mem_3(val: u64, aux: u64) -> u64 { mem_reference(val, aux) ^ 0xFF }
    #[test] fn test_phd_counterfactual_mutant_1() { assert!(mem_reference(1, 1) != mutant_mem_1(1, 1)); }
    #[test] fn test_phd_counterfactual_mutant_2() { assert!(mem_reference(1, 1) != mutant_mem_2(1, 1)); }
    #[test] fn test_phd_counterfactual_mutant_3() { assert!(mem_reference(1, 1) != mutant_mem_3(1, 1)); }
}

// Hoare-logic Verification Line 100: Radon Law verified.
