# CHEAT-006: Scanner Evasion

## Overview
In the BCINR Deterministic Substrate, **CHEAT-006** strictly prohibits any attempt to bypass or subvert the `bcinr-cheat-scanner`. The structural integrity of the branchless substrate relies on rigorous source and object-code auditing. Evading the scanner fundamentally undermines the mathematical contract of the repository, replacing verifiable algorithmic determinism with hidden control flow.

## Prohibited Tactics
The constitution explicitly outlaws the following evasion strategies:
* **Splitting operators across lines**: Breaking syntactic elements to prevent scanner pattern matching.
* **Inserting comments inside tokens**: Using comments as camouflage to break syntax recognition by the scanner.
* **Using macro indirection to hide a pattern**: Obfuscating prohibited branches or instructions by generating them through macros.
* **Moving prohibited code into private helpers**: Attempting to bypass the absolute $CC=1$ law by moving branches out of the public authoritative roots into private functions, under the false assumption they won't be scanned.
* **Moving code into generated output**: Using code generation steps to inject prohibited structures that are not present in the primary source files.
* **Hiding behavior behind traits**: Leveraging dynamic dispatch or complex trait implementations to conceal data-dependent control flow.
* **String construction that produces prohibited source**: Constructing prohibited source code dynamically via string manipulation during code generation to avoid static source checks.

## Why Scanner Evasion is a Constitutional Violation
The BCINR project mandate enforces the **Radon Law ($CC=1$)**, which dictates that logic must be expressed as bitwise polynomials without any `if`, `match`, or data-dependent `loop`. 

1. **Loss of Determinism**: Scanner evasion introduces hidden branches and variable-time operations into the hot path. This creates timing side-channels and destroys the absolute deterministic execution required of the substrate.
2. **Subversion of the Enforcer (`@turing_machine`)**: The `@turing_machine` role relies on the scanner to parse the full syntax tree and inspect macro definitions, generated Rust, and private functions. Bypassing the scanner nullifies the structural audit and merge gatekeeping.
3. **Absolute Failure (SIS = 0)**: Scanner evasion is classified as an absolute failure. Regardless of other metrics, it forces the Substrate Integrity Score (SIS) to `0` and immediately triggers the `MaturityScrutiny` protocol (quarantining affected code and freezing feature development).
4. **Breach of the Verification Matrix**: Every authoritative primitive must have source-level and object-code verification. Evading the scanner violates the requirement that code is "PhD-Verified," replacing cryptographic/mathematical proof with deception. 

Any code attempting these tactics will trigger a `CHEAT[CHEAT-006]` finding and immediately block the merge.
