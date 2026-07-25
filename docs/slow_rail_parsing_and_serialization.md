# Slow Rail Operations: Parsing, Serialization, and Handoff

In the BCINR Deterministic Substrate, code is strictly bifurcated between the **Authoritative Runtime** (the hot path) and the **Slow Rail**. According to Rule 6 of the `AGENTS.md` Constitution, operations like RDF parsing, SHACL validation, and artifact serialization belong exclusively to the slow rail. 

## 1. Why are Parsing and String Manipulation Banned from the Hot Path?

The authoritative runtime is engineered as a "hard substrate" for deterministic execution. Subjecting the hot path to string manipulation and variable-graph parsing fundamentally violates several core architectural laws of BCINR:

* **The Radon Law ($CC=1$)**: Parsing strings, traversing variable-length graphs (like RDF triples), or deserializing dynamic JSON/XML inherently requires data-dependent loop termination (`while not EOF`) and control flow branching (`if char == 'x'`, `match token`). The hot path prohibits all data-dependent branches and mandates an absolute $CC=1$.
* **The Zero-Allocation Boundary**: Artifact serialization and string manipulation inherently require dynamic memory (heap allocations, vectors, strings). The authoritative runtime operates strictly under `#![no_std]` with `0` heap allocations, relying instead on fixed bounds and static memory structures. 
* **Fixed Bounded Execution Work**: The substrate guarantees execution in mathematically bounded, deterministic time to prevent timing side channels. Parsing variable-length documents means the execution time scales with the input size, destroying the constant-time guarantee.
* **No Runtime Theorem Discovery (Rule 12)**: Parsing and validating complex semantic structures (like checking SHACL constraints) involves discovering structural relationships and verifying arbitrary graphs. The hot path is permitted only to *verify* fixed witnesses, not to discover them.

## 2. How the Slow Rail Handles These Tasks

Because the slow rail is strictly isolated—meaning it is never linked into or invoked from the authoritative call graph—it is granted the permissions necessary to perform complex semantic processing:
* It **may branch and allocate**.
* It processes variable-length text, iterates over RDF semantic graphs, and performs unbounded computations to enforce SHACL constraints.
* It parses human-readable artifacts, coordinates I/O, serializes states for dashboards, and derives mathematical proofs (like eigenvalue bounds or static domination properties).

The slow rail serves as the rich semantic environment where all messy, variable, and allocative work happens *before* admitting any configuration into the determinism of the substrate.

## 3. The Handoff: Bridging Slow Rail to Hot Path

If the hot path cannot parse or read strings, how does it process the rules derived from an RDF document? It relies on a structural paradigm shift: **Rich semantics upstream, fixed deterministic mechanics downstream.**

The slow rail distills parsed documents into fixed-width, branchless mathematical primitives, which are then passed to the hot path:

### A. Distillation into Fixed-Width Profiles
Instead of sending a graph to the hot path, the slow rail generates fixed-size state structures or Rust source code (e.g., `generated/stability_profile.rs`). Strings and dynamic arrays are compiled down into fixed-point arrays, bounds, weighting vectors, matrices, and contraction margins.

### B. Packed Digests and Certificates
To ensure integrity without parsing, the slow rail computes a cryptographic or deterministic `Digest` binding the entire parsed configuration (e.g., the RDF structure, generated tables, and bounds). The hot path operates exclusively on these packed digests. 

### C. Branchless Verification & Typed Refusals
When the data enters the hot path, it is processed via the **ReceiptSound Law (Rule 11)** and mask-based execution:
1. The hot path accepts fixed-width data and the `Digest`.
2. It verifies the input using branchless arithmetic operations (e.g., comparing digests, fixed multiply-accumulate operations to check bounds).
3. It derives a full-width boolean mask (`0` or `!0`) representing admission (e.g., `valid_digest_mask & valid_bounds_mask`).
4. It performs a fieldwise masked commit using bitwise selection: `select(mask, candidate, current)`.

If the handoff data is invalid (e.g., the digest mismatches, or bounds are exceeded), the hot path does not attempt to parse a reason. It immediately emits a mathematically typed refusal (like `CMCA_CERTIFICATE_DIGEST_MISMATCH` or `CMCA_RUNTIME_ENVELOPE_VIOLATED`) and falls back gracefully via masked assignment, leaving the adaptive state bit-for-bit unchanged.
