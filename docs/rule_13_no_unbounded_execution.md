Based on Rule 13 ("No unbounded execution") in `AGENTS.md`, here are the details regarding prohibited and required iteration constructs:

### Prohibited Iteration Constructs
The following unbounded iteration constructs are explicitly prohibited:
```rust
while value > 0
```
```rust
for item in variable_slice
```
```rust
loop {
    if done {
        break;
    }
}
```
```rust
iterator.take_while(...)
```

### Required Forms of Authoritative Iteration
All authoritative iteration must be:
* compile-time fixed;
* generated;
* macro-unrolled;
* or demonstrated as fully unrolled in release object code.

**Key constraint:** A fixed Rust source loop is not automatically accepted. The final machine code must contain no loop backedge in authoritative symbols.
