Here are the details from Rule 14 "Numeric-law requirements" in `AGENTS.md`:

### Primitives Requiring Special Scrutiny
The following mathematical primitives require special scrutiny:
* reciprocal
* logarithm
* exponential
* fixed-point multiplication
* fixed-point division replacement
* absolute value
* min/max
* clamp
* normalization
* eigenvalue lower bounds
* KL accumulation
* digest comparison

### Epsilon Rule
According to the rule: "No epsilon may be inserted silently." 

Furthermore, every smoothing or clamp constant (like an epsilon) must be:
* named
* derived
* admitted
* included in the influence digest
