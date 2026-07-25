Based on my research of `AGENTS.md`, here are the details for the "Authoritative runtime" classification under Rule 6:

### Authoritative Runtime Classification (Rule 6)

Authoritative runtime code encompasses any code that can affect:
* allocation
* adaptive state
* admission
* certificate verification
* refusal masks
* resource prices
* semantic mass
* standing projections
* persistent state

### Absolute Runtime Laws Inherited (Rule 3)
Code classified as "Authoritative runtime" inherits every absolute runtime law. The complete authoritative call graph must satisfy the following constraints:

```text
#![no_std]
no alloc
zero heap allocation
CC = 1 per authoritative function
no data-dependent branches
no data-dependent loop termination
no panic paths
no unwinding
no floating-point operations
no dynamic dispatch
no indirect calls
no runtime parsing
no variable graph traversal
no runtime algorithm search
no runtime stability discovery
fixed-width inputs
fixed-width outputs
fixed bounded memory access
fixed bounded execution work
```

These laws apply transitively, meaning that the full authoritative call graph (including private helpers, macro expansions, generated code, and trait methods) must adhere to these rules without exception. For example, a branchless public function calling a branching private helper or relying on input-dependent jumps is considered a violation.
