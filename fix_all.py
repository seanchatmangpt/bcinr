import os
import re

impls = {
    "succinct_bit_vector_rank": {
        "args": ["val: u64", "aux: u64"],
        "body": "let mask = (1u64.wrapping_shl((aux & 63) as u32).wrapping_sub(1)) | ((aux >= 64) as u64).wrapping_neg(); (val & mask).count_ones() as u64",
        "ref": "if aux >= 64 { val.count_ones() as u64 } else { (val & ((1 << aux) - 1)).count_ones() as u64 }"
    },
    "succinct_bit_vector_select": {
        "args": ["val: u64", "aux: u64"],
        "body": "let mut r = 0; let mut v = val; let mut s = 0; s = ((v.count_ones() as u64) <= aux) as u64 * 64; r |= s; v &= !((1u64.wrapping_shl(s as u32)).wrapping_sub(1)); r",
        "ref": "r" # will fix later, maybe just a simple one
    }
}
