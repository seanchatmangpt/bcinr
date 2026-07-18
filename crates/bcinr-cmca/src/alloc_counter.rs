//! Feature-gated (`alloc-gate`) global-allocator wrapper for asserting that the
//! authoritative `allocator::allocate()` root performs zero heap allocations.
//!
//! Mirrors the pattern in `crates/bcinr-pddl/src/alloc_counter.rs` (`dhat-heap`
//! feature): a lightweight atomic-counter global allocator wrapping `System`,
//! not the `dhat` crate itself. `dhat` produces a JSON heap-profile file meant
//! for its own viewer; this crate instead needs an exact allocation count it
//! can assert `== 0` against inside an ordinary `#[test]`, which a raw counter
//! gives directly.
//!
//! Only compiled under the `alloc-gate` feature; normal builds/tests never
//! link this module and never install a non-default global allocator.

// This module is dev/test-only (behind `alloc-gate`, never enabled by
// default or by any other feature) and is not part of the authoritative
// runtime call graph audited by AGENTS.md Sec 3/7/20 or SAFETY.md's 3
// justified-unsafe-block inventory: `GlobalAlloc` is an upstream `unsafe
// trait` by construction, so implementing it (even as a pure pass-through to
// `System` plus a relaxed atomic counter) requires `unsafe impl` regardless
// of what the impl body does. Scoped `allow` below, not a crate-wide one.
#[cfg(feature = "alloc-gate")]
#[allow(unsafe_code)]
pub mod counting_alloc {
    // Linked directly (not via the crate's own `feature = "std"` gate): this
    // module is only ever compiled into a `std`-linked test binary (an
    // integration test file under `alloc-gate`), so it does not need, and
    // deliberately avoids depending on, the crate's separately-broken
    // `std` feature (`cargo build -p bcinr-cmca --features std` fails
    // independently of this module, in `src/lrc.rs`, at time of writing —
    // not a regression introduced here, and not fixed here per the
    // verifier/implementer separation of duties).
    extern crate std;
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::sync::atomic::{AtomicU64, Ordering};

    static ALLOC_COUNT: AtomicU64 = AtomicU64::new(0);
    static ALLOC_BYTES: AtomicU64 = AtomicU64::new(0);

    /// Global allocator that counts every allocation's count and byte size
    /// before delegating to the system allocator. Install via
    /// `#[global_allocator] static ALLOC: CountingAlloc = CountingAlloc;`
    /// in a test-binary crate root under `#[cfg(feature = "alloc-gate")]`.
    pub struct CountingAlloc;

    // SAFETY: delegates every operation to `System`, which is itself a sound
    // `GlobalAlloc` implementation; the only addition is a relaxed atomic
    // counter update, which cannot violate the allocator contract.
    unsafe impl GlobalAlloc for CountingAlloc {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
            ALLOC_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
            System.alloc(layout)
        }
        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            System.dealloc(ptr, layout)
        }
        unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
            ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
            ALLOC_BYTES.fetch_add(new_size as u64, Ordering::Relaxed);
            System.realloc(ptr, layout, new_size)
        }
    }

    /// Current cumulative (allocation count, bytes allocated) since process start.
    /// Intended usage: snapshot before and after a call under test, subtract.
    pub fn snapshot() -> (u64, u64) {
        (
            ALLOC_COUNT.load(Ordering::Relaxed),
            ALLOC_BYTES.load(Ordering::Relaxed),
        )
    }
}
