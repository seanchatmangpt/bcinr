# Signing-key rotation (v26.9.24)

Recorded 2026-09-24 (fleet key scan after the single-repo migration). Base `b76dcb377b29` of `bcinr`.
Every private key listed here was committed to this repository and is therefore compromised: every receipt or
attestation signed with it carries no signing authority (standing REFUSED, broken_term R_missing_authority).
The keys leave the tree (history is not rewritten; no force-push), and each key directory's `.gitignore` now
covers both halves. Every checkout keeps its own pair: ggen generates one on first use, and a tracked public
half without its private half would make that first `ggen sync` refuse [FM-KEY-010/011]. The canonical
checkout's new public key is published below for anyone verifying its future receipts.

| key dir | removed private key sha256 | removed public key sha256 | new public key (canonical checkout) |
|---|---|---|---|
| `.ggen/keys` | `fa834159f5cb00f1ac315b139e5596b497935d0b831ac90cefed2e93390450e6` | `fb14244dea0817abead2f48100bc83d117dc7c0d21bbd8a56263a4e465012a8b` | `f4d4005e92b0c998383ff2cbfb539a3db043103e4342fce4c757b0e0b9196a78` |

## Keys exposed on non-default branches (revoked 2026-09-24)

The v26.9.24 rotation scanned default branches only. A scan of every `origin/*` branch found the
private keys below committed on non-default branches only. Each is compromised and revoked: any receipt
or attestation signed with it carries no signing authority (standing REFUSED, broken_term
R_missing_authority). A disk scan of the canonical checkouts on 2026-09-24 found three of these keys in
use (ggen/packs, ignored files) and replaced them with fresh pairs. Copies in agent worktrees and tool
caches may still hold them. History is not rewritten, so the branches keep the blobs.

| path | private key sha256 | derived public key | branches (count, first) |
|---|---|---|---|
| `playground/.ggen/keys/signing.key` | `f43b581d76b49b26a62fc9ede242ea41faa127e525806f1f1bbece77bba65cb2` | `f800fb2bd475aa5c3c84c943ebffcffe257edfeaab5caa39f1e269a6cdb64eb1` | 20, `agent/close-v26-7-26-release-gaps` |
| `crates/chess-factory/.ggen/keys/signing.key` | `0bcaf55fd8e5a62a21f5470599a17a23730a51a9454614d8eec4f80c0c8fdfee` | `d5d5a819818460f1b6ae15dcf6dbdb07071f87ca74b6a9dae4665ee47d5ac0b3` | 1, `claude/upbeat-cori-19p23x` |
