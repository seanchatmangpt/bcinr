# CHATGPT-CLOUD-AGENTS.md — Hosted Agent Execution Protocol for BCINR

This file is an environment-specific addendum to `AGENTS.md` for hosted ChatGPT
sessions, GitHub-connector sessions, ephemeral shells, and CI-observation work.

Read `AGENTS.md` first. This file does not redefine BCINR mathematics, runtime
law, proof obligations, gate jurisdiction, standing vocabulary, or write
ownership. When this file and `AGENTS.md` differ, `AGENTS.md` governs.

## 1. Purpose

Hosted agents frequently see the repository through several boundaries that do
not carry the same evidence:

1. source inspected through a GitHub connector;
2. source changed through a GitHub connector;
3. commands executed in a real checkout;
4. CI observed for a specific remote commit.

Never collapse these boundaries into one claim.

The portable operating law is:

> Orient from current source, classify the real execution boundary, select the
> highest-priority actionable unfinished item, begin lawful work, and keep every
> claim below the evidence actually obtained.

## 2. Operating modes

Classify the session before substantial repository work.

### Local checkout

A real checkout exists in the shell or container. Establish it with commands
such as:

```bash
pwd
git rev-parse --show-toplevel
git status -sb
git branch --show-current
git remote -v
```

Then verify each required tool independently. Only a real checkout can directly
establish uncommitted state, staged state, local diffs, command exit codes,
locally generated artifacts, disassembly output, and local receipt
recomputation.

### GitHub connector only

The repository is accessible through a connected GitHub tool, but no usable
checkout exists. This mode can inspect known files and refs, inspect remote
metadata, create branches, create remote commits, and open pull requests when
the connector exposes those actions.

This mode cannot by itself run Cargo, `just`, scanners, mutation suites,
disassemblers, proof tools, or shell commands. It cannot inspect an uncommitted
working tree or prove that a declared command succeeds.

### Hybrid

A real checkout and the GitHub connector are both available. Keep them aligned:

- resolve the local branch and remote head;
- fetch before comparing;
- confirm a push before describing local work as remote;
- do not reuse stale blob SHAs after a write;
- tie CI observations to the exact pushed commit.

Use local tools for execution, generation, diffs, staging, and object-code
inspection. Use the connector for structured repository, pull-request, and issue
operations.

### CI observation

CI may be the only available execution evidence. CI claims are bounded by the
workflow definition, job steps, logs, artifacts, and exact commit SHA.

Queued is not running. Running is not passed. A green workflow does not prove an
unlisted command. A green older commit does not validate a newer head.

## 3. Evidence classes

Keep these labels distinct in reasoning and reporting:

- **Observed**: source, metadata, logs, or artifacts were read.
- **Executed**: a command actually ran against the claimed checkout or artifact.
- **Changed**: a file, branch, issue, pull request, or artifact was modified.
- **Verified**: an independent check recomputed or exercised the claimed fact.
- **Inferred**: a conclusion was derived from observed evidence.
- **Blocked**: a required boundary could not be reached.

Examples:

- Reading a test proves the test is declared, not that it passes.
- Reading a scanner proves what it intends to inspect, not that it ran.
- Reading disassembly instructions in documentation is not object-code evidence.
- A connector commit proves a remote write, not a local build or gate pass.
- A CI success proves only the commands visible for that exact commit.

## 4. Instruction and source orientation

Before changing a path:

1. read the root `AGENTS.md`;
2. read this file when operating in a hosted or split-boundary environment;
3. check for every more-specific `AGENTS.md` governing the target path;
4. resolve the repository default branch and requested target ref;
5. inspect the current source, manifests, tests, gates, and generated ownership;
6. inspect current planning, ticket, status, or evidence documents that identify
   unfinished work;
7. treat historical notes and remembered repository details as stale until
   re-observed.

Use the precedence and mathematical law declared by `AGENTS.md`. This addendum
only governs how evidence from hosted tools is classified.

## 5. Startup protocol

After orientation, do not stop at summarizing instructions. In the same work
session:

1. state the operating mode actually available;
2. separate observed source, executed commands, changes, inferences, and blocked
   boundaries;
3. identify the highest-priority unfinished item that is actionable in the
   current mode;
4. verify that the item's assumptions still match current source;
5. name the first concrete files, symbols, gates, or evidence artifacts involved;
6. begin useful work without asking the user to restate context already present
   in repository files;
7. remain inside the selected item's actual scope and write ownership;
8. avoid direct edits to generated surfaces;
9. run the narrowest relevant validation available;
10. create a new branch and draft pull request when the environment permits;
11. never merge unless explicitly instructed.

A blocked top-ranked item does not force inactivity. Select the next lawful item
whose dependencies and required boundary are available, while recording why the
higher item remains blocked.

## 6. First substantive response contract

The first substantive response for a repository task must demonstrate
orientation by naming:

- the applicable instruction hierarchy, including path-specific `AGENTS.md`;
- the real operating mode and execution boundaries;
- the evidence required before the selected item could receive its requested
  BCINR standing;
- the first source and evidence files to inspect or change;
- the useful work already begun in that response.

Do not merely paraphrase instruction documents.

## 7. Selecting actionable work

Determine priority from current repository authority in this order:

1. an explicit current ticket, milestone, release blocker, or ordered backlog;
2. a failing required gate or source-proven broken composition;
3. a missing load-bearing contract, oracle, mutant, scanner, object-code audit,
   or reproducibility artifact;
4. the smallest lawful checkpoint that unblocks a higher-priority consequence.

Before implementation, verify:

- the target source and symbols still exist;
- dependencies and generated inputs are present;
- the file is authoritative, slow-rail, test-only, or generated;
- the requested claim ceiling is possible in the current environment;
- no concurrent branch or remote write has invalidated the observed blob SHA.

Do not promote a documentation priority over a current failing executable gate
unless the repository explicitly orders it that way.

## 8. Connector terminology and read protocol

Use precise language:

- connector fetch is not `cat`;
- connector search is not `grep`;
- connector file replacement is not a local edit;
- connector commit is not a staged working-tree commit;
- remote metadata is not `git status`;
- a file reference is not a local filesystem path.

For a known file, fetch the exact path and ref directly and retain the returned
blob SHA. Re-fetch after every write before describing final content.

Search results may time out or omit matches. A failed or empty connector search
is not proof of absence; confirm candidate paths with direct fetches.

## 9. Connector write protocol

When no checkout exists:

1. create or select a dedicated branch from the current default branch;
2. fetch the target file from that branch;
3. retain its current blob SHA;
4. replace the complete UTF-8 file using that SHA;
5. never update the same path concurrently;
6. re-fetch the committed file;
7. inspect the committed result;
8. compare the branch with its base and audit the changed-file list;
9. open a draft pull request with exact validation boundaries.

A successful connector write proves a remote commit only. It does not prove
formatting, compilation, tests, proofs, mutation kills, scanner coverage, or
object-code properties.

## 10. Validation claims

Run the narrowest command that exercises the changed consequence, followed by
any aggregate gate required by `AGENTS.md` or the ticket.

For every validation claim preserve:

- exact command;
- working directory;
- exact target ref or commit;
- exit status;
- passed, failed, skipped, and conditional cases;
- required toolchain and environment;
- relevant artifact identities.

Do not say tests passed, gates are green, code is branchless, mutants were
killed, or object code is compliant unless that exact evidence was executed or
inspected for the exact commit.

Source-level inspection cannot establish object-code standing. CI cannot
substitute for a different command. A declared test count is not an executed
pass count.

## 11. Completion and reporting

Use the detailed BCINR final report required by `AGENTS.md`. In hosted sessions,
also make the execution boundary explicit:

```text
State:
Repository and ref:
Operating mode:
Source observed:
Commands actually executed:
Remote commits created:
Files changed:
Validation observed:
CI observed:
Commands not executed:
Remaining blockers:
Pull request:
```

Do not claim a stronger standing than the weakest load-bearing evidence permits.
A blocker is an acceptable result. Invented execution evidence is not.

## 12. Reusable repository-work prompt

The following template is intentionally repository-agnostic. Replace the
placeholders with current paths and scope rather than copying stale project
assumptions.

```text
Review the GitHub repository {{repository}}.

Before doing anything substantial, read the root AGENTS.md and, when operating
in a hosted or split-boundary environment, CHATGPT-CLOUD-AGENTS.md. Check for a
more specific AGENTS.md before entering or changing any directory.

Do not rely on remembered repository details. Inspect the current default branch,
the requested target ref, and current source.

Orient from these current repository authorities:
- {{priority_or_status_documents}}
- {{relevant_manifests_and_gate_definitions}}
- {{relevant_current_source_and_tests}}

Then:
1. State the operating mode actually available: local checkout, GitHub connector
   only, hybrid, or CI observation.
2. Distinguish observed source, executed commands, changes made, conclusions
   inferred, verification obtained, and blocked boundaries.
3. Identify the highest-priority unfinished item actionable in that mode.
4. Verify the item's assumptions against current source.
5. Begin implementing the bounded item without asking for context already present
   in repository files.
6. Stay within the item's scope and applicable write ownership. Do not hand-edit
   generated output.
7. Run the narrowest relevant validation available.
8. Commit on a new branch and open a draft pull request when permitted. Do not
   merge.

The first substantive response must name the applicable instruction hierarchy,
real execution boundaries, evidence required for completion, first concrete
files or symbols, and useful work begun in that response.

Do not claim a command passed unless it was actually executed or exact CI logs
for the exact commit show it. Do not describe connector operations as local
shell commands. A blocker is acceptable; invented evidence is not.
```
