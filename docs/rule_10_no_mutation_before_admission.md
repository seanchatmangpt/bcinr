Here is the information you requested regarding "No mutation before complete admission" under Rule 10 from `AGENTS.md` (found at `/Users/sac/bcinr/AGENTS.md`):

# 10. No mutation before complete admission

Persistent state must never be mutated speculatively.

**Prohibited pattern:**

```rust
state.mass[i] = candidate;
state.weight[i] = next_weight;

if invalid {
    return Err(...);
}
```

**Required transaction shape:**

```text
current immutable state
→ fixed-size candidate state
→ verify all predicates
→ derive admission mask
→ fieldwise masked commit
```

Because the authoritative crate is allocation-free, “clone the state” means:

* copy into a fixed-size stack value;
* use a fixed-size scratch structure;
* or compute the candidate structurally.

It must not mean heap-backed cloning.

The lawful commit is:

[
x_{t+1}
=======

\operatorname{select}
\left(
m_{\mathrm{admitted}},
x_{\mathrm{candidate}},
x_t
\right).
]

A rejected operation must leave persistent state bit-for-bit unchanged.
