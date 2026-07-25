# Rule 21: The Prohibition on Hand-Editing Generated Code

In the BCINR Deterministic Substrate, **Rule 21 (Generated-code law)** mandates strict immutability, determinism, and reproducibility for any code produced by generation tools. The foundational architectural principle of BCINR is *"Rich semantics upstream. Fixed deterministic mechanics downstream."* Generated code serves as the crucial bridge between these upstream semantic models and the downstream fixed execution mechanics, meaning it must rigorously adhere to all absolute runtime laws.

## The Required Byte-Identical Verification Process

To guarantee that the generator output is perfectly reproducible and free of non-deterministic leaks (such as timestamps, local paths, or environment variations), Rule 21 mandates a strict generation verification loop:

1. **Clean generation**: Execute the generation process from a pristine, clean tree.
2. **Digest output**: Cryptographically hash (digest) the resulting output to record its exact byte-state.
3. **Regenerate**: Re-run the generator from scratch.
4. **Verify byte-identical output**: Compare the regenerated artifact against the prior digest to guarantee they match bit-for-bit.

This `clean generation -> digest output -> regenerate -> verify byte-identical` process mathematically proves that identical semantic inputs predictably yield the exact same fixed instruction shape. Generated authoritative code is never exempt from constitutional gates: it must continuously pass the `bcinr-cheat-scanner`, the `CC=1` Radon Law (no branches), object-code disassembly inspection, and explicitly bind to certified source graph digests.

## Why Hand-Editing Invalidates Project Standing

Rule 21 explicitly dictates: **"Hand-editing generated output is prohibited. Generated files with unexplained drift invalidate standing."** 

Manually altering a generated file is an absolute failure that forces the Substrate Integrity Score (SIS) to 0 and immediately triggers the `MaturityScrutiny` quarantine protocol. This invalidation occurs because hand-editing fundamentally undermines the substrate:

1. **Severing the Cryptographic Chain of Proof**: The mathematical contract relies on the generator being the verified author. Hand-editing bypasses the `@hoare_oracle` and structural scanners, injecting unverified mutations into the runtime and breaking the cryptographic linkage between the source graph and the final digest.
2. **Architectural Cheating**: Logic changes *must* exclusively occur upstream in the semantic models or the generator itself. Directly mutating the downstream unrolled output is an illegal attempt to bypass strict implementation gates, substituting mathematical correctness for apparent progress.
3. **Unexplained Drift**: A hand-edited file introduces unexplained structural drift. Because of the rigorous reproducibility process, the next time the regeneration loop runs, the byte-identical verification will immediately fail. 
4. **Volatility & Destruction of Truth**: Manual edits are volatile. They will be silently overwritten and destroyed upon the next clean generation loop, leading to untraceable regressions and divergent states across the continuous integration matrix.
5. **Violation of Strict Write Ownership**: The constitution establishes that generated files may only be modified by the admitted generator. A human or agent editing the file violates exclusive write ownership rules.

In BCINR, there is no exception for "trivial" or "quick" fixes. If the generated output is flawed, the upstream generator must be audited, repaired, and re-verified.
