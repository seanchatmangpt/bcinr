Based on my review of `GEMINI.md`, here is the detailed markdown describing the Substrate Integrity Score (SIS):

### The Substrate Integrity Score (SIS)
In `GEMINI.md`, the Substrate Integrity Score is defined as one of the **Core Architectural Laws** of the `bcinr` project. 

- **"PhD-Verified"**: A file achieves "PhD-Verified" status only when it attains a perfect **100/100 score** on the project's maturity matrix.
- **Maturity Matrix Components**: To reach this perfect score and be considered "PhD-Verified", the file must successfully satisfy three required components:
  1. **Proof**: A formal mathematical or structural proof of the implemented logic.
  2. **Oracle**: An independent axiomatic reference implementation that acts as the source of truth. According to the architecture's "Contract with Teeth," if the implementation deviates from this axiomatic reference by even 1 bit, the verification matrix MUST fail.
  3. **Hostile Tests**: Adversarial tests and mutants designed to aggressively challenge the constraints and assumptions of the implementation.

*(Note: While `GEMINI.md` introduces this high-level maturity matrix definition for SIS, the comprehensive mathematical formula and specific automatic failure states for SIS are further elaborated in Section 24 of the `AGENTS.md` file.)*
