# Key exposure record (v26.9.25)

Recorded 2026-09-25 from the v26.9.25 topology scan. Base `ebb469f798b9` of `bcinr`.
The private key below is public: it is reachable from refs on the public `origin` remote. It was
already revoked on 2026-09-24 (PR #40, section 'Keys exposed on non-default branches' of
docs/security/signing-key-rotation-v26.9.24.md). This record adds the exposure surface found on
2026-09-25, including the `backup/v26.9.25/*` refs pushed during the single-checkout migration.

The key stays revoked: any receipt or attestation signed with it carries no signing authority
(standing REFUSED, broken_term R_missing_authority). History is not rewritten and no ref is
force-pushed. Revocation is the remedy; the blob stays reachable from the refs listed below.

## Revoked key

| field | value |
|---|---|
| path | `playground/.ggen/keys/signing.key` |
| git blob | `a2de77d9b5229ad453882e530b7002be319df85f` |
| private key file sha256 | `f43b581d76b49b26a62fc9ede242ea41faa127e525806f1f1bbece77bba65cb2` |
| derived Ed25519 public key | `f800fb2bd475aa5c3c84c943ebffcffe257edfeaab5caa39f1e269a6cdb64eb1` |
| first commit on origin | `28078234c432c92cb10b0341062b2d73d84b92ac` (2026-06-23T13:21:17-07:00) |
| commits on origin that add or remove it | 12 |
| origin branches whose history reaches it | 58 (includes `main`: yes) |
| origin branches with the key file at the tip | 20 |

## Replacement

The canonical checkout does not use this key. Its `.ggen/keys` pair is the v26.9.24 rotation
pair: generated locally, never committed (no git object in this repository holds either half),
and the private half derives the public key published in
`docs/security/signing-key-rotation-v26.9.24.md`. Verify receipts signed after 2026-09-24
against that published public key only.

## Exposure surface: origin branches reaching the key

- `agent/chatmangpt-namespace-26.7.29`
- `agent/close-doc-wip-v26.9.1`
- `agent/close-v26-7-26-release-gaps` (key file at tip)
- `agent/cmca-chicago-jtbd-rebased-v26.7.25` (key file at tip)
- `agent/cmca-chicago-jtbd-validation-v26.7.25` (key file at tip)
- `agent/cmca-divan-execution-benchmarks` (key file at tip)
- `agent/finish-bcinr-lsp` (key file at tip)
- `agent/finish-cmca-bcinr`
- `agent/finish-v26-7-28-powl2-multifractal` (key file at tip)
- `agent/finish-v26-7-28-powl2-multifractal-rebased` (key file at tip)
- `agent/generalize-cloud-agent-protocol` (key file at tip)
- `agent/post-pr13-semantic-audit` (key file at tip)
- `agent/powl-chicago-verifier-base` (key file at tip)
- `agent/repair-post-integration-verification-ci` (key file at tip)
- `agent/rust-toolchain-capsule-20260801`
- `agent/semantic-three-way-integration-v26.7.24` (key file at tip)
- `agent/substitutable-transform-contract-v1`
- `agent/v2030-1-1-prd-ard-20260819`
- `agent/v26.7.24-production-pddl-powl-v2` (key file at tip)
- `agent/v26.7.28-production-admission` (key file at tip)
- `agent/v26.7.28-temporal-swarm` (key file at tip)
- `agent/v26.9.1-mfw-cmca-contract-crown`
- `backup/v26.9.25/bcinr_claude_worktrees_agent-a35a85aec63695f13`
- `backup/v26.9.25/bcinr_claude_worktrees_agent-a35a85aec63695f13-dbc9ff7f26`
- `backup/v26.9.25/bcinr_claude_worktrees_agent-a7eb75875dfb2d24c`
- `backup/v26.9.25/bcinr_claude_worktrees_agent-a7eb75875dfb2d24c-4d4d08b36d`
- `backup/v26.9.25/bcinr_claude_worktrees_agent-a83a540ad4d487dda`
- `backup/v26.9.25/bcinr_claude_worktrees_agent-a83a540ad4d487dda-c7ef6aca4f`
- `backup/v26.9.25/bcinr_claude_worktrees_agent-ac65ac5e90a63c54c`
- `backup/v26.9.25/bcinr_claude_worktrees_agent-ac65ac5e90a63c54c-f12b3dba0e`
- `backup/v26.9.25/bcinr_claude_worktrees_wf_7a8d0628-158-1`
- `backup/v26.9.25/bcinr_claude_worktrees_wf_7a8d0628-158-1-dd0235740e`
- `backup/v26.9.25/bcinr_claude_worktrees_wf_7a8d0628-158-3`
- `backup/v26.9.25/bcinr_claude_worktrees_wf_7a8d0628-158-3-c42b2e5ea3`
- `backup/v26.9.25/bcinr_claude_worktrees_wf_7a8d0628-158-4`
- `backup/v26.9.25/bcinr_claude_worktrees_wf_7a8d0628-158-4-26b60e0b75`
- `backup/v26.9.25/bcinr_claude_worktrees_wf_8bf87291-515-1`
- `backup/v26.9.25/bcinr_claude_worktrees_wf_8bf87291-515-1-ff42482909`
- `backup/v26.9.25/bcinr_claude_worktrees_wf_8bf87291-515-2`
- `backup/v26.9.25/bcinr_claude_worktrees_wf_8bf87291-515-2-a439c3bb09`
- `backup/v26.9.25/bcinr_claude_worktrees_wf_8bf87291-515-3`
- `backup/v26.9.25/bcinr_claude_worktrees_wf_8bf87291-515-3-4b2f1aa83a`
- `brand/forward-deployment-os-2026-08`
- `claude/merge-recent-branches-main-hpzna5` (key file at tip)
- `claude/upbeat-cori-19p23x` (key file at tip)
- `feat/dfcm-federated-capabilities-v26.9.1`
- `feat/v26.7.28-powl2-multifractal` (key file at tip)
- `feat/v26.9.19-work-envelope-v2`
- `ggen-embedded-workflow-pack-demo` (key file at tip)
- `integration/all-relevant-20260819`
- `main`
- `prod/ppcx-kernel-boundary-20260911`
- `recovery/cmca-v26.7.17-c2` (key file at tip)
- `refactor/collapse-powl-receipt`
- `release/26.9.15`
- `security/revoke-branch-exposed-keys-v26.9.24`
- `security/rotate-signing-keys-v26.9.24`
- `v26.9.16/rfc-closure`

Of these, 20 are `backup/v26.9.25/*` refs. They preserve migrated work and are kept.
