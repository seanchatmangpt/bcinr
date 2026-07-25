# MaturityScrutiny Quarantine Mechanism

Under **Rule 25 (MaturityScrutiny protocol)**, when the Substrate Integrity Score (SIS) drops below 100 (often forced to `0` due to absolute constitutional failures), a strict 9-step remediation process is automatically triggered. The second step of this protocol mandates the **quarantine of affected code**. 

The quarantine is enforced through the following strict mechanisms:

### 1. Physical Isolation
Structurally non-compliant or unverified code is not merely `#cfg`-gated; it is physically moved into explicit directory boundaries (e.g., relocated to a `quarantine/` folder). This physical separation guarantees that the defective code is visually and structurally distinct from the authoritative runtime.

### 2. Package Exclusion
The quarantined code is strictly severed from the published artifact. This is enforced by explicitly excluding the quarantine boundary in the project's configuration (e.g., adding `exclude = ["quarantine/**"]` to `Cargo.toml`). This ensures that the quarantined code never ships in published crate tarballs and fully closes any packaging gaps.

### 3. Build Graph and CI Severance
The affected code is completely detached from the hot path and build execution. 
- CI tasks are re-engineered to operate without invoking the quarantined code.
- Verification must rely on pre-computed artifacts or digests (e.g., running `cargo make verify-generated` to verify committed digests instead of invoking a quarantined generator).
- The quarantined code must remain completely unreachable from the primary structural gates.

### 4. Strict Remediation and Freeze
Once code is quarantined, it is accompanied by an absolute freeze on feature development (Step 1). Developers are strictly prohibited from bypassing the quarantine by moving the feature elsewhere. The code remains isolated until:
- A root-cause report is produced.
- The structural defect is repaired using proper mathematical, branchless constructs.
- All dependent artifacts are fully regenerated.
- The complete verification matrix is rerun and a new standing receipt is formally issued to restore `SIS = 100`.
