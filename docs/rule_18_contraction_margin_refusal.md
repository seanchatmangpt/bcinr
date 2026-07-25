Based on the core laws in `AGENTS.md`, the `ContractionMarginInsufficient` typed refusal is intrinsically linked to the stability verification constraints outlined in **Rule 12 (No runtime theorem discovery)**.

According to Rule 12, the `bcinr` substrate strictly prohibits the authoritative runtime ("hot path") from dynamically discovering stability (e.g., via spectral-radius estimation, power iteration, or Lyapunov search). Instead, the non-authoritative "slow rail" computes the stability parameters (like $G$, $d$, and the margin $\delta$) and supplies them as a mathematical witness. The hot path is only allowed to deterministically verify this witness via fixed-point packed-value comparisons, checking constraints such as:
$G_{\mathrm{certified}}d \leq (1-\delta)d$

If this condition fails—meaning the provided witness does not demonstrate a sufficient contraction bound ($\delta$)—the system must reject the operation. 

This is where **Rule 18 (Typed refusals)** applies. Because human-readable text, panics, and silent fallbacks are strictly prohibited in the hot path, any rejected operation must deterministically yield a bounded, typed refusal code. `ContractionMarginInsufficient` is the explicit error enum variant returned when the stability witness provided fails the mathematical verification required by Rule 12. 

In short: **Rule 12 dictates that stability must be statically verified rather than dynamically discovered, and if that verification mathematically fails, Rule 18 dictates the operation must branchlessly return the `ContractionMarginInsufficient` typed refusal.**
