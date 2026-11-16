# bcinr migration plan v26.11.16

Standing: UNVERIFIED

## Identity
- Path: /Users/sac/bcinr
- Remote: https://github.com/seanchatmangpt/bcinr.git (upstream origin/bench/rdtsc-tick-tables)
- Branch: bench/rdtsc-tick-tables@1e92b213, last commit 2026-10-11, 892 commits, 562 MB
  (.git alone 528 MB)
- Languages/build: Rust workspace (Cargo.toml), Makefile, LaTeX whitepaper, Python audit scripts
- Nested repos:
  - bcinr/dteam_ref — https://github.com/seanchatmangpt/dteam.git main@5c00d75, 470 commits,
    dirty 19 / untracked 11, 158 unpushed local branches, last 2026-08-23. `du` reports 0B
    (likely a worktree/link; `.gitmodules` missing for path `dev-worktree`).

## Observed state
- Dirty 0 / untracked 0 on the top-level repo
- Ahead 1 of upstream (release bump v26.10.11 commit not pushed); behind 0
- Stash count: 614 (very large); 51 unpushed local branches
- Build dirs: none (no target/); book/ and benchmarks/ tiny
- Submodules: none at top level; dteam_ref reports broken mapping for `dev-worktree`
- Top-level clutter: audit_*.py, *.txt symbol dumps, whitepaper .aux/.log, cmca_rdf_*.md

## What it is
BranchlessCInRust: performance-first Rust library of branchless O(1) primitives with a formal
B-calculus, PDDL planning, POWL workflows and cryptographic receipts; published to crates.io.

## Classification
ACTIVE (census agrees; Oct 2026 release commit, published crates).

## Migration risk
HIGH — 1 unpushed release commit, 51 unpushed branches, 614 stashes, and nested dteam_ref with
19 dirty files plus 158 unpushed branches. All of that lives only in local .git.

## Recommended action
COMMIT-PUSH
1. `cd /Users/sac/bcinr && git push origin bench/rdtsc-tick-tables` (publishes the ahead-1).
2. `git branch -vv | grep -v '\[origin'` to list the 51 local-only branches.
   DECISION: push all (`for b in ...; do git push -u origin $b; done`) or bundle instead.
3. `git stash list | wc -l` (614). DECISION: keep via bundle — stashes do not push.
   `git bundle create /Volumes/ARCHIVE/bcinr-full.bundle --all` plus
   `git stash list > /Volumes/ARCHIVE/bcinr-stashes.txt` for the index.
4. Nested dteam_ref: `cd dteam_ref && git status`; DECISION on 19 modified crate files
   (ccog/insa cog8 runtime); `git add -A && git commit -m "wip: pre-migration"`;
   `git push -u origin <branch>`; bundle its 158 local branches:
   `git bundle create /Volumes/ARCHIVE/dteam_ref-full.bundle --all`.
5. No target/ to clean. Do NOT rsync the 528 MB .git; rely on push + bundle.
6. Re-clone bcinr on new machine; re-clone dteam separately (it is a nested clone, not a
   submodule).

## First commands on the new machine
`git clone https://github.com/seanchatmangpt/bcinr.git && cd bcinr && cargo check --workspace`
(then `make check` if the Makefile target exists)

## See Also
- ../v26.9.19/ , ../v26.9.16/ , ../cmca/
- /Users/sac/docs/jira/v26.11.16/README.md
