# Slow Rail Architecture in BCINR

In the `bcinr` (BranchlessCInRust) deterministic substrate, **Rule 6** of the Constitution mandates a strict separation between bounded, deterministic execution and unbounded, dynamic logic. This creates two distinct physical and logical domains: the **Authoritative Runtime** (Hot Path) and the **Slow Rail** (Non-authoritative boundary).

## 1. The Authoritative Hot Path vs. The Slow Rail
- **The Authoritative Hot Path**: Bound by the **Radon Law ($CC=1$)** and **Zero-Allocation Boundary**. It must contain strictly zero heap allocations (`#![no_std]`), zero data-dependent branches, and no panic paths. Logic is expressed entirely as bitwise polynomials.
- **The Slow Rail**: Tasks on the Slow Rail are explicitly permitted to use the standard library (`std`), allocate memory on the heap, and branch. Its primary role is to handle complex, non-deterministic tasks offline or asynchronously and derive mathematical witnesses or stability certificates that the Hot Path will later verify using fixed bit-parallel arithmetic.

## 2. Structural Isolation of the Slow Rail

The Slow Rail is strictly prohibited from being linked into or invoked from the Authoritative Hot Path to prevent transitive "infection" of branches and memory allocations. This is physically and mechanically enforced through:

- **Workspace and Crate Isolation**: Authoritative crates (like `bcinr-cmca`) are compiled strictly as `#![no_std]` without `alloc` features. Slow Rail tasks (like code generators, RDF parsers, CLI) are placed in external binaries or distinct tool crates (e.g., `mfw-codegen`). Slow rail crates are *never* declared as `[dependencies]` in the authoritative crates.
- **Object-Code Audits**: To ensure Slow Rail code hasn't breached the boundary, `bcinr` runs strict `audit-object-code` steps over the final release artifacts (`.rlib`/`.a`). System disassemblers (`objdump`) mechanistically verify the complete absence of allocator calls, panic handlers, indirect calls, floating-point math, and runtime loop backedges in the Authoritative Runtime.
- **Hardware-Level Branch Verification**: The build matrix verifies that the release object code encounters effectively zero dynamic branch mispredictions (using `perf stat -e instructions,branch-misses`), structurally proving that Slow Rail logic remains completely isolated from the Hot Path instructions.

## 3. Slow Rail Processing (RDF Parsing & SHACL Validation)

Tasks like **RDF parsing** and **SHACL validation** inherently demand dynamic memory allocation for variable-sized structures, data-dependent loops, and variable graph traversal—all of which violate the Hot Path's $CC=1$ deterministic constraints.

Therefore, the Slow Rail takes on the heavy lifting of resolving Semantic Web data:
1. It parses raw strings and traverses arbitrary dependency cycles Ahead-of-Time (AOT).
2. It validates the SHACL constraints and derives the complex facts.
3. Instead of discovering these graph topologies at runtime, the Slow Rail *flattens* the multidimensional relationships into dense bitmasks and internally bound byte arenas.

## 4. Artifact Serialization & Zero-Copy Ingestion

Traditional runtime deserializers (like `serde_json` or Protocol Buffers) are completely banned from the Authoritative Runtime because they require branching and heap allocation.

To ingest the complex structures derived by RDF/SHACL processing, the Slow Rail implements a purely hardware-aligned serialization strategy:
- **Fixed-Width C-ABI Payloads**: The Slow Rail acts as a producer and serializes flattened topological arrays into payloads that perfectly mirror memory-mapped structs. These use `#[repr(C, align(64))]` to strictly enforce predictable memory layouts aligned perfectly with CPU cache lines.
- **Zero-Cost Pointer Casting**: The Hot Path completely bypasses runtime parsing. It maps the AOT-generated artifact (`mmap` or `include_bytes!`) and performs a zero-cost pointer cast directly to the statically typed slice (`&[u8]` to `&[Powl64Op]`).
- **Cryptographic Binding**: The Hot Path uses the `bytemuck` crate to safely verify struct layouts and checks the payload against a deterministic BLAKE3 digest (`Gamma_CMCA` artifact boundary). This prevents speculative mutation or recovery logic while providing $O(1)$ access, zero allocations, and zero branching.
