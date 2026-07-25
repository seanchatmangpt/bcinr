Based on Rule 23 ("Required repository gates") in `AGENTS.md`, here are the detailed requirements:

### 1. What the Final Gate Report Must State
The final report for the executed repository gates must explicitly include the following seven items:
- `command`
- `exit status`
- `files inspected`
- `features inspected`
- `targets inspected`
- `findings`
- `artifact digest`

### 2. Absolute Requirement for Proving Jurisdiction
Before reporting any results, the agent must **prove each task's jurisdiction includes the changed files**. 

The absolute law regarding this is:
> **"A green command with incomplete jurisdiction is not evidence."**

This means that a passing (green) test or scanner result is considered entirely invalid if the command did not actually inspect the relevant changed files, generated output, feature set, or target.
