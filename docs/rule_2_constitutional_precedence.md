# Rule 2: Constitutional Precedence

In the `bcinr` project, the governing principle is **"Rich semantics upstream. Fixed deterministic mechanics downstream."** When instructions, guidelines, or preferences conflict, all agents and implementations must strictly adhere to the following 7-step order of precedence (highest to lowest priority):

1. **Mathematical safety and typed refusal**
2. **`AGENTS.md`** (The Constitution)
3. **Repository contract gates**
4. **Crate-local architecture documents**
5. **Issue or task requirements**
6. **Agent preferences**
7. **Implementation convenience**

No agent may weaken a higher-order rule to satisfy a lower-order objective.

## Why Convenience Cannot Override the Constitution

The constitution explicitly mandates that claims such as **“faster,” “simpler,” “idiomatic,”** or **“the compiler will optimize it”** can never override constitutional laws. 

This strict boundary exists because `bcinr` is designed as a civilizational-scale, deterministic computational substrate. The authoritative runtime relies on absolute structural laws (e.g., $CC=1$, zero allocation, no data-dependent branches, fixed instruction shapes). 

Appeals to speed, simplicity, or idiomatic Rust fall under **Step 6 (Agent preferences)** or **Step 7 (Implementation convenience)**. Allowing them to override higher-order rules would compromise the foundational mathematical proofs and the physical predictability of the runtime. In `bcinr`, if an optimization or idiomatic pattern introduces a hidden branch or deviates from the axiomatic reference by even a single bit, it fundamentally violates the mathematical safety (Step 1) of the substrate.
