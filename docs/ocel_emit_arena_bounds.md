# OcelEmitArena Architecture and Bounds Management

The `OcelEmitArena` architecture provides a deterministic, zero-allocation mechanism for bump-allocating causal frames off the hot path. Here is how the `ReceiptWorker` and `OcelEmitArena` manage `OcelCausalFrame` objects without hitting dynamic allocation or out-of-bounds errors.

## 1. Off-Hot-Path Event Deferral via MPMC Ring
The authoritative hot path (`petri_tick`) strictly adheres to the $CC=1$ Radon Law and zero-allocation bounds. It never directly allocates frames or performs BLAKE3 hashing. Instead, it pushes tiny `EventWorkItem` objects to a `LockFreeMpmcRing<EventWorkItem, 64>` in ~10ns.

A background worker (or separate fiber) runs `ReceiptWorker::drain()` to securely extract these events off the hot path, accumulating the execution trace before finalizing it.

## 2. Pre-Allocated, Zero-Initialized Arena
To avoid dynamic allocation (e.g., `malloc` or `Vec::push`) during operation, `OcelEmitArena` pre-allocates all necessary memory upfront during initialization (`OcelEmitArena::new()`):
- **Fixed Capacity**: The arena holds exactly 4096 `OcelCausalFrame` objects.
- **Heap Allocation**: Because 4096 frames of 128 bytes each equals 524,288 bytes (which could blow the stack), it uses `std::alloc::alloc_zeroed` to safely initialize a `Box<[OcelCausalFrame; 4096]>` on the heap.
- **Cache-Aligned Layout**: `OcelCausalFrame` is `#[repr(C, align(64))]` and strictly zero-initialized, ensuring bit-for-bit reproducible padding.

## 3. Strict Bump-Allocation and Bounds Enforcement
Once initialized, the arena manages frame allocation deterministically:
- **Sequential Emitting**: The `emit` method uses a simple integer index (`head`) to bump-allocate the next available slot in the pre-allocated array.
- **No Freeing or Resizing**: Frames are allocated sequentially and never freed. There is no dynamic resizing or reallocation mechanism.
- **Out-of-Bounds Prevention**: To prevent buffer overflows, the `emit` function enforces a strict runtime assertion: `assert!(self.head < ARENA_CAPACITY)`. The system relies on the design contract that callers are responsible for bounding the frame counts to fit within the 4096 capacity at manufacture time. If a workflow exceeds this bound, the arena predictably panics (failing fast) rather than dynamically reallocating or corrupting memory.

## 4. Deterministic Hashing Integration
After a frame is bump-allocated and populated (with `instruction_id`, `fired_mask`, `denial`, and `obj_refs`), it is deterministically compacted into a 99-byte buffer. The `ReceiptWorker` then integrates it into a rolling BLAKE3 cryptographic hash chain, completing the receipt generation safely off the hot path.
