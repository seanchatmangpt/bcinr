I have reviewed Rule 28 ("Standing vocabulary") in `/Users/sac/bcinr/AGENTS.md`.

The **`REPORTED_ALIVE`** standing is defined precisely as:
> "An agent reports success, but independent reproduction has not occurred."

### What it means and when it is used
Code is labeled with the `REPORTED_ALIVE` standing when an agent (e.g., the implementation agent) claims that the code successfully passes its checks, builds correctly, or clears certain gates, but this success **has not yet been independently verified**.

This ties into **Rule 27: No self-certification**, which mandates that the implementation agent cannot be the final approver for mathematical correctness, branchlessness, etc. An agent merely stating that the code passes is not sufficient evidence. The `REPORTED_ALIVE` label acts as a temporary, unverified state meaning "success was claimed, but the mandatory independent mechanical artifact (or different role's verification) has not yet confirmed it." Until independent reproduction occurs, it cannot be elevated to stronger standings like `ALIVE` or `BRANCHLESS_ALIVE`.
