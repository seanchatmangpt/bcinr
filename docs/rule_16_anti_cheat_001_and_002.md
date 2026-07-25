Here are the details for the Anti-cheat manifesto (Rule 16), specifically CHEAT-001 and CHEAT-002, extracted from `/Users/sac/bcinr/AGENTS.md`:

```markdown
# 16. Anti-cheat manifesto

The following patterns are prohibited throughout production and verification code.

## CHEAT-001 — Self-canceling operations

Examples:

```rust
a.wrapping_add(b) ^ a
```

when the operation is included only to create apparent complexity.

Any operation without a contractual contribution to the output is prohibited.

## CHEAT-002 — Circular oracle

A reference implementation copied from the production implementation.
```
