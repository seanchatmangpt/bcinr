# bcinr: triage 27 local-only branches

- Standing: OPEN
- Created: 2026-09-19 (v26.9.19 gh survey wave)
- Source: local branches with commits not on `origin/main` and no upstream
- Evidence: `git rev-list --count origin/main..<branch>` > 0 for each: feat/powl-soundness-cli:12 fix/cmca-default-build-and-gates:18 release/26.9.15:29 worktree-agent-a35524c0b2b9abab5:1 worktree-agent-a6b032296ba2ae68a:1 worktree-agent-a6b981ed93db6f2f7:1 worktree-agent-a845a6849d18abdb1:1 worktree-agent-a875087e672a4e9c6:1 worktree-agent-af87fa2b5ef70e76b:1 worktree-agent-af9a0dece9b6fe715:1 worktree-wf_4cd5308c-d0c-11:1 worktree-wf_4ce5f45e-ffa-11:1 worktree-wf_6a91582b-76a-11:1 worktree-wf_8b12adec-fa3-11:1 worktree-wf_9f98ef46-896-11:1 worktree-wf_a428da11-384-11:1 worktree-wf_a832af34-59b-11:1 worktree-wf_cc44edaf-f57-11:1 worktree-wf_edb42221-a80-11:1 worktree-wf_ef15e8d3-5b1-10:2 worktree-wf_ef15e8d3-5b1-6:2 worktree-wf_ef15e8d3-5b1-7:2 worktree-wf_ef15e8d3-5b1-8:2 worktree-wf_f9963199-dc7-1:1 worktree-wf_f9963199-dc7-2:1 worktree-wf_f9963199-dc7-3:1 worktree-wf_f9963199-dc7-4:1

## Work to complete
- Triage each branch: push (`git push -u origin <branch>`) or delete after confirming the commits are obsolete/recoverable. Batch the work; record decisions in History.

## Acceptance
- Every listed branch is pushed or deleted; no local-only branch with unique commits remains.

## History
- 2026-09-19 | OPEN | survey found 27 local-only branches | full list above | triage pending
