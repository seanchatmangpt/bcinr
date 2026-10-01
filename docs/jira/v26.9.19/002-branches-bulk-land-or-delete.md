# bcinr: triage 44 PR-less unmerged remote branches

- Standing: OPEN
- Created: 2026-09-19 (v26.9.19 gh survey wave)
- Source: origin branches not merged into `main` with no open PR
- Evidence: `git branch -r --no-merged origin/main`: agent/close-doc-wip-v26.9.1 agent/close-v26-7-26-release-gaps agent/cmca-chicago-jtbd-rebased-v26.7.25 agent/cmca-divan-execution-benchmarks agent/finish-bcinr-lsp agent/finish-v26-7-28-powl2-multifractal agent/finish-v26-7-28-powl2-multifractal-rebased agent/powl-chicago-verifier-base agent/repair-post-integration-verification-ci agent/v26.7.28-production-admission agent/v26.7.28-temporal-swarm brand/forward-deployment-os-2026-08 claude/busy-hypatia-6pm6sw claude/cool-gauss-36yn2a claude/upbeat-cori-19p23x feat/dfcm-federated-capabilities-v26.9.1 feat/wasm4games-dx-pass ggen-embedded-workflow-pack-demo release/26.9.15 worktree-agent-a01b65eb8db9a1fc5 worktree-agent-a1236d07a8bbc9df1 worktree-agent-a15141a52731a5ce7 worktree-agent-a4205f3cab90034ca worktree-agent-a456b476e8fcc9be7 worktree-agent-a54ecc88ccca0c882 worktree-agent-a5bdcb1c3092aca1b worktree-agent-a647b4c21e646e8fa worktree-agent-a7756da7a17f8220e worktree-agent-aa3c392170d4a4202 worktree-agent-aa3d7bb5637043a93 worktree-agent-aa4ca0295c58ac0c7 worktree-agent-aac6916671c3c0c61 worktree-agent-aaefa73e275e75ff9 worktree-agent-ab0353a2665eeba79 worktree-agent-abd50f3805189c899 worktree-agent-ac289efcb919ccf97 worktree-agent-ac349c9c34dfe9f99 worktree-agent-ac806fdc8b762da63 worktree-agent-acfa4bab518e367e8 worktree-agent-ad265efd96198863e worktree-agent-ad274c05de8c33768 worktree-agent-ade29b9b0dfda012e worktree-agent-ade851230d89529e0 worktree-agent-af79100b80d58f103

## Work to complete
- Triage each branch: land (open a PR) or delete (`git push origin --delete <branch>`). Work in batches; record decisions in History.

## Acceptance
- `git branch -r --no-merged origin/main` is empty after `git fetch --prune`.

## History
- 2026-09-19 | OPEN | survey found 44 PR-less branches | full list above | triage pending
