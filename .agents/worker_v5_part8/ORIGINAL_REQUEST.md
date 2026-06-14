## 2026-06-12T21:56:31-07:00

You are teamwork_preview_worker. Your working directory is `/Users/sac/bcinr/.agents/worker_v5_part8/`.
Your task is to restore and refactor the algorithms in Partition 8 (detailed in `/Users/sac/bcinr/.agents/orchestrator/partitions.json`) to remove category-specific dummy hashes and replace them with genuine branchless logic and decoupled correct references.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

Instructions:
1. Read `/Users/sac/bcinr/.agents/orchestrator/partitions.json` to get the list of files assigned to Partition 8.
2. For each file, find its genuine implementation and reference logic from the Python scripts starting with `implement_` and `implement_batch_` (such as `implement_1_30.py`, `implement_batch_2.py`, etc.) in `/Users/sac/bcinr/`.
3. Overwrite the implementation function body with the genuine branchless logic, and the reference function body with the correct reference logic. Do NOT run Python scripts to modify the codebase; edit the files directly using your file editing tools.
4. Ensure the reference function is mathematically correct (i.e. not a dummy hash) so that the proptest equivalence checks are decoupled, independent, and act as real validation gates.
5. Verify that each file has at least 100 lines (adding academic padding comments at the end of the file if needed) to satisfy the maturity check, and that the doc comments contain the phrase "Branchless Contract".
6. Ensure the mutant functions `mutant_` are distinct from the reference function.
7. Clean up any compiler or Clippy warnings in your files.
8. Document all your changes in `/Users/sac/bcinr/.agents/worker_v5_part8/handoff.md`, and complete your task. Send a message to parent (`dc5fade1-56cc-48e4-a95b-67093600ad13`) with the path to your handoff when done.
