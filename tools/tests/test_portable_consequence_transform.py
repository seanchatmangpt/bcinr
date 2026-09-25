from itertools import product
import importlib.util,sys
from pathlib import Path
P=Path(__file__).resolve().parents[1]/"portable_consequence_transform.py"
spec=importlib.util.spec_from_file_location("pct",P); m=importlib.util.module_from_spec(spec); sys.modules["pct"]=m; spec.loader.exec_module(m)

def oracle(a,matches,r):
    if not a: return 1
    if not matches: return 2
    if not r: return 3
    return 0

def test_exhaustive_three_bit_domain_matches_independent_oracle():
    cases=list(product((False,True),repeat=3)); assert len(cases)==8
    for a,match,r in cases: assert m.transform(m.Frame(a,match,r))==oracle(a,match,r)

def test_transform_has_no_authority_acquisition_state():
    assert set(m.Frame._fields)=={"authority_present","authority_matches","receipt_capable"}
