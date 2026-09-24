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
