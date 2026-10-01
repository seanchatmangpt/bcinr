#!/usr/bin/env python3
"""Non-authoritative bounded transform for portable-consequence/1.

The authoritative BCINR core is not modified. This replaceable witness maps a
fixed three-bit admitted input to a fixed decision code.
"""
from __future__ import annotations
from typing import NamedTuple
ALLOW=0; REFUSE_AUTHORITY_REQUIRED=1; REFUSE_AUTHORITY_SCOPE=2; REFUSE_RECEIPT_REQUIRED=3
class Frame(NamedTuple):
    authority_present: bool
    authority_matches: bool
    receipt_capable: bool

def transform(frame:Frame)->int:
    if not frame.authority_present: return REFUSE_AUTHORITY_REQUIRED
    if not frame.authority_matches: return REFUSE_AUTHORITY_SCOPE
    if not frame.receipt_capable: return REFUSE_RECEIPT_REQUIRED
    return ALLOW
