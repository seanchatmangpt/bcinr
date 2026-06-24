#!/usr/bin/env python3
"""Emit artifacts/results.ttl — the RESULTS graph for the receipt-cited Crown
proof table.

This is the bridge between the empirical benchmark artifacts (the per-cell
receipts in artifacts/benchmark.receipt.json + the elo_curve) and ggen's
author-time graph. `ggen sync` imports the emitted results.ttl alongside
chess.ttl, runs queries/proof_table.rq over it, and renders
artifacts/proof_table.md.

The graph is SELF-CONTAINED: it carries the full opponent roster + budget axis
(lifted from ontology/benchmark.ttl) plus one cf:ResultCell per cell that was
actually measured (lifted from the run receipt). Cells that were declared but
not measured render as "(not measured)"; opponents declared cf:installed false
render as "not-installed". Nothing is faked — only what the receipt proves.

Run from crates/chess-factory/:  python3 tools/emit_results_ttl.py
"""
import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
RECEIPT = os.path.join(ROOT, "artifacts", "benchmark.receipt.json")
CURVE = os.path.join(ROOT, "artifacts", "elo_curve.json")
BENCH_TTL = os.path.join(ROOT, "ontology", "benchmark.ttl")
OUT = os.path.join(ROOT, "artifacts", "results.ttl")

CF = "https://bcinr.dev/chess-factory/ontology#"


def esc(s: str) -> str:
    return s.replace("\\", "\\\\").replace('"', '\\"')


def parse_roster(path):
    """Lift the opponent roster + budget axis from benchmark.ttl (no rdflib dep)."""
    txt = open(path).read()
    opponents = {}
    budgets = {}
    # Split into `cf:foo a cf:Class ; ... .` blocks.
    blocks = re.split(r"(?=^cf:\w+\s+a\s+cf:)", txt, flags=re.MULTILINE)
    for b in blocks:
        m = re.match(r"cf:(\w+)\s+a\s+cf:(\w+)\s*;", b)
        if not m:
            continue
        local, cls = m.group(1), m.group(2)

        def field(name, default=None):
            mm = re.search(r"cf:%s\s+([^;\.]+)" % name, b)
            return mm.group(1).strip() if mm else default

        def sval(name, default=None):
            v = field(name, default)
            if v is None:
                return default
            v = v.strip()
            if v.startswith('"'):
                return v[1 : v.rindex('"')]
            return v

        if cls == "Opponent":
            opponents[sval("name", local)] = {
                "id": int(sval("id")),
                "name": sval("name", local),
                "kind": sval("engineKind", "uci"),
                "tier": sval("tier", "reference"),
                "installed": sval("installed", "false") == "true",
            }
        elif cls == "BudgetTier":
            budgets[sval("name", local)] = {
                "id": int(sval("id")),
                "name": sval("name", local),
                "micros": int(sval("micros")),
            }
    return opponents, budgets


def main():
    receipt = json.load(open(RECEIPT))
    curve = json.load(open(CURVE))
    opponents, budgets = parse_roster(BENCH_TTL)

    run_id = receipt["run_id"]
    replay_pointer = receipt["replay_pointer"]
    sanity_seed_hex = receipt.get("sanity_seed_hex", "")
    chain_head = receipt.get("cell_chain_head", "")
    matrix_hash = receipt.get("input_matrix_blake3", "")
    book_hash = receipt.get("opening_book_blake3", "")

    # Map curve points by (opponent, budget) for the rich stats (CI, perf_elo).
    cpts = {}
    for p in curve.get("points", []):
        cpts[(p["opponent"], p["budget"])] = p

    cells = receipt.get("cell_receipts", [])

    L = []
    w = L.append
    w("@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .")
    w("@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .")
    w("@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .")
    w("@prefix cf:   <%s> ." % CF)
    w("")
    w("#################################################################")
    w("# bcinr Chess Factory — RESULTS GRAPH (receipt-cited Crown proof table).")
    w("#")
    w("# DO NOT hand-edit: emitted by tools/emit_results_ttl.py from the empirical")
    w("# benchmark artifacts (artifacts/benchmark.receipt.json + elo_curve.json) and")
    w("# the opponent roster (ontology/benchmark.ttl). `ggen sync` imports this graph")
    w("# alongside chess.ttl and renders artifacts/proof_table.md from it.")
    w("#")
    w("# Every cf:ResultCell carries the blake3 receipt hash that proves it, chained")
    w("# to the run receipt. Unmeasured cells and not-installed engines are NOT faked.")
    w("#################################################################")
    w("")
    w("cf:Crown_Run a cf:ResultRun ;")
    w('    cf:runId           "%s" ;' % esc(run_id))
    w('    cf:replayPointer   "%s" ;' % esc(replay_pointer))
    w('    cf:cellChainHead   "%s" ;' % esc(chain_head))
    w('    cf:inputMatrixHash "%s" ;' % esc(matrix_hash))
    w('    cf:openingBookHash "%s" ;' % esc(book_hash))
    w('    cf:sanitySeedHex   "%s" ;' % esc(sanity_seed_hex))
    w('    rdfs:comment "Top of the receipt chain for the Crown proof table." .')
    w("")
    w("#################################################################")
    w("# Opponents (roster axis). cf:rInstalled / cf:rTier drive rendering.")
    w("#################################################################")
    for o in sorted(opponents.values(), key=lambda x: x["id"]):
        w("cf:R_opp_%s a cf:ResultOpponent ;" % o["name"])
        w("    cf:rOppId    %d ;" % o["id"])
        w('    cf:rOppName  "%s" ;' % esc(o["name"]))
        w('    cf:rTier     "%s" ;' % esc(o["tier"]))
        w("    cf:rInstalled %s ." % ("true" if o["installed"] else "false"))
        w("")
    w("#################################################################")
    w("# Budget tiers (tau axis / columns).")
    w("#################################################################")
    for bdg in sorted(budgets.values(), key=lambda x: x["id"]):
        w("cf:R_bud_%s a cf:ResultBudget ;" % bdg["name"])
        w("    cf:rBudId    %d ;" % bdg["id"])
        w('    cf:rBudName  "%s" ;' % esc(bdg["name"]))
        w("    cf:rBudMicros %d ." % bdg["micros"])
        w("")
    w("#################################################################")
    w("# Measured cells (one cf:ResultCell per cell the receipt proves).")
    w("#################################################################")
    n = 0
    for c in cells:
        opp = c["opponent"]
        bud = c["budget"]
        if opp not in opponents or bud not in budgets:
            continue
        n += 1
        pt = cpts.get((opp, bud), {})
        cid = 1000 + opponents[opp]["id"] * 100 + budgets[bud]["id"]
        score = c.get("score_rate", pt.get("score_rate", 0.0))
        ci_lo = pt.get("ci_lo", score)
        ci_hi = pt.get("ci_hi", score)
        perf_elo = pt.get("perf_elo", 0.0)
        w("cf:R_cell_%s_%s a cf:ResultCell ;" % (opp, bud))
        w("    cf:rCellId      %d ;" % cid)
        w("    cf:rCellOpp     cf:R_opp_%s ;" % opp)
        w("    cf:rCellBud     cf:R_bud_%s ;" % bud)
        w('    cf:rCellOppName "%s" ;' % esc(opp))
        w('    cf:rCellBudName "%s" ;' % esc(bud))
        w("    cf:rScoreRate   %.4f ;" % score)
        w("    cf:rCiLo        %.4f ;" % ci_lo)
        w("    cf:rCiHi        %.4f ;" % ci_hi)
        w("    cf:rPerfElo     %.1f ;" % perf_elo)
        w("    cf:rWins        %d ;" % c.get("wins", 0))
        w("    cf:rDraws       %d ;" % c.get("draws", 0))
        w("    cf:rLosses      %d ;" % c.get("losses", 0))
        w("    cf:rFactoryUs   %d ;" % c.get("factory_us_median", 0))
        w("    cf:rFactoryUsP99 %d ;" % c.get("factory_us_p99", 0))
        w("    cf:rOppUs       %d ;" % c.get("opponent_us_median", 0))
        w("    cf:rOppUsP99    %d ;" % c.get("opponent_us_p99", 0))
        w('    cf:rGoCommand   "%s" ;' % esc(c.get("go_command", "")))
        w('    cf:rReceiptHash "%s" ;' % esc(c.get("cell_blake3", "")))
        w('    cf:rPrevHash    "%s" ;' % esc(c.get("prev_hash", "")))
        w('    cf:rRunId       "%s" ;' % esc(run_id))
        w('    cf:rReplayPtr   "%s" .' % esc(replay_pointer))
        w("")

    open(OUT, "w").write("\n".join(L) + "\n")
    print("wrote %s — %d opponents, %d budgets, %d measured cells" %
          (OUT, len(opponents), len(budgets), n))


if __name__ == "__main__":
    sys.exit(main())
