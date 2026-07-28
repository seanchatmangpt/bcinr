#!/usr/bin/env python3
"""Run the POWL reference implementation over the shared corpus.

Emits one JSON record per case on stdout: {name, verdict, detail}. `verdict` is
"converted" or "refused" -- the one judgement both implementations make in the
same vocabulary, and therefore the one that can be compared without translating
between two different model representations.

The reference is AGPL-3.0 and is NOT part of this repository; run
`oracles/fetch.sh` first. If it is absent this script exits non-zero rather than
reporting an empty comparison as a pass.
"""
import json, os, sys

HERE = os.path.dirname(os.path.abspath(__file__))
VENDOR = os.path.join(HERE, "..", "vendor", "powl")

if not os.path.isdir(VENDOR):
    sys.exit("oracle absent: run oracles/fetch.sh (the reference is AGPL-3.0 and is not vendored)")
sys.path.insert(0, VENDOR)

try:
    from pm4py.objects.petri_net.obj import PetriNet
    from pm4py.objects.petri_net.utils import petri_utils
    from powl.conversion.to_powl.from_pn.converter import convert_workflow_net_to_powl
except ImportError as e:
    sys.exit(f"oracle unrunnable: {e} (pip install -r {VENDOR}/requirements.txt)")


def build(case):
    net = PetriNet(case["name"])
    places = {p: PetriNet.Place(p) for p in case["places"]}
    trans = {t: PetriNet.Transition(t, label) for t, label in case["transitions"]}
    for x in places.values():
        net.places.add(x)
    for x in trans.values():
        net.transitions.add(x)
    for p, t in case["pt"]:
        petri_utils.add_arc_from_to(places[p], trans[t], net)
    for t, p in case["tp"]:
        petri_utils.add_arc_from_to(trans[t], places[p], net)
    return net


def main():
    with open(os.path.join(HERE, "cases.json")) as f:
        corpus = json.load(f)["cases"]
    for case in corpus:
        try:
            model = convert_workflow_net_to_powl(build(case))
            record = {"name": case["name"], "verdict": "converted",
                      "detail": " ".join(str(model).split())[:120]}
        except Exception as e:
            record = {"name": case["name"], "verdict": "refused",
                      "detail": f"{type(e).__name__}: {e}"[:120]}
        print(json.dumps(record))


if __name__ == "__main__":
    main()
