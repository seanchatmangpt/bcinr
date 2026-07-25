# CHEAT-001 (Self-canceling operations)

Under Rule 16 (Anti-cheat manifesto) of `AGENTS.md`, **CHEAT-001** refers to operations that do not mathematically contribute to the actual output of a function, but are instead included merely to create "apparent complexity."

### Example
```rust
a.wrapping_add(b) ^ a
```

### Why it is prohibited
In the BCINR framework, every authoritative primitive must fulfill a strict mathematical contract using minimal, deterministic, and branchless mechanics. Operations without a contractual contribution are prohibited because they are seen as "cheating" the complexity or structural audits (e.g., artificially inflating code metrics or evading scanners without doing meaningful work). Every single operation must have a direct, verifiable reason for existing that ties back to the function's mathematical proof and final output.
