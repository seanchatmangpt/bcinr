# Oracles

Independent implementations bcinr is differentially tested against.

An oracle here is not a dependency and not a test fixture. It is a second
implementation of the same specification, written by someone else, whose
agreement with ours is evidence and whose disagreement is a finding. The value
comes entirely from the independence: `bcinr-powl` and the POWL reference share
no code, so when they return the same verdict on the same net that is a real
check, and when they differ one of them is wrong.

## Licence boundary

**Nothing under `vendor/` is committed.** The references carry licences that are
incompatible with this repository's MIT OR Apache-2.0 — the POWL reference is
AGPL-3.0 — so they are fetched on demand into `oracles/vendor/`, which is
gitignored. Running a program under a copyleft licence and comparing its output
is ordinary use; copying its source into a permissively licensed tree is not.

Run `./oracles/fetch.sh` to populate `vendor/`. That script is GENERATED from
`ontology/taxonomies.ttl` -- the pinned commit, URL and licence of each oracle
are admitted facts, and `ontology/shapes.ttl` refuses an oracle that does not
state its licence or is not pinned. The pin was briefly written in both the
script and this file, which is the drift the graph exists to prevent,
reintroduced by the harness built to detect it.

## Corpus sharing

Both sides read the *same* case file (`powl/cases.json`). Two hand-maintained
lists of inputs drift, and a differential over drifted inputs silently stops
comparing anything — which is the failure mode this directory exists to catch,
so it must not be reintroduced by the harness itself.

## Current oracles

Declared in `ontology/taxonomies.ttl` as `bc:ReferenceImplementation`; run
`./fetch.sh` to see the current set with its pins. As of writing: the POWL 2.0
reference (Python, AGPL-3.0, wired) and VAL (C++, BSD-3-Clause, read but not
wired -- see `pddl/README.md` for what it would check).

## Running

    ./fetch.sh          # populate vendor/ at the pinned commits
    ./differential.py   # compare verdicts; exit 1 on any disagreement

A disagreement is a finding in one of the two implementations. It is not a
tolerance to widen, and the harness has no threshold to relax.
