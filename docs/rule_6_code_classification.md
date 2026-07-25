# Rule 6: Authoritative versus Non-Authoritative Code

According to the BCINR Deterministic Substrate Constitution (`AGENTS.md`), every source file and function must be explicitly classified into one of the following four categories. Each classification carries distinct rules and permissions:

## 1. Authoritative Runtime
**Definition:** Code that can affect:
- allocation
- adaptive state
- admission
- certificate verification
- refusal masks
- resource prices
- semantic mass
- standing projections
- persistent state

**Rules & Permissions:** 
- It inherits **every absolute runtime law** (Rule 3).
- Must have exactly `CC = 1` per authoritative function (strictly branchless).
- Must perform zero heap allocations (strictly `no alloc` / `#![no_std]`).
- Must contain no data-dependent branches, data-dependent loop termination, panic paths, unwinding, floating-point operations, dynamic dispatch, indirect calls, runtime parsing, variable graph traversal, or runtime stability discovery.
- Must operate on fixed-width inputs/outputs with bounded memory access and execution work.

## 2. Slow Rail
**Definition:** Code performing tasks such as:
- RDF parsing
- SHACL validation
- certificate derivation
- symbolic mathematics
- eigenvalue search
- code generation
- artifact serialization
- CLI display
- dashboards
- test references
- benchmark orchestration

**Rules & Permissions:**
- The slow rail **may branch and allocate**.
- **Isolation Constraint:** It must **never** be linked into or invoked from the authoritative hot path.

## 3. Test-Only Oracle
**Definition:** An independent mathematical specification.
**Rules & Permissions:**
- Must be fully excluded from production features.
- Used strictly for verification, independent reference semantics, and axiomatic proofs.

## 4. Generated Authoritative Code
**Definition:** Generated source that is executed by the runtime.
**Rules & Permissions:**
- Generated code is **not exempt** from absolute runtime laws.
- It must pass **all authoritative gates** (branchlessness, `CC=1`, zero-allocation, etc.) *after* generation.
