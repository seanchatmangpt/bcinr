import os
import json

out_dir = "/Users/sac/bcinr/.agents/worker_v5_part9/new_rs_files"
dest_dir = "/Users/sac/bcinr/crates/bcinr-logic/src/algorithms"

batch1 = [
    "delta_encode_simd_u32.rs",
    "delta_decode_simd_u32.rs",
    "branchless_stack_spsc.rs",
    "branchless_ring_buffer_mpmc.rs",
    "lockfree_skip_list_step.rs",
    "waitfree_queue_push.rs",
    "hazard_pointer_retire.rs",
    "epoch_based_reclamation_step.rs"
]

payloads = []
for name in batch1:
    src_path = os.path.join(out_dir, name)
    dest_path = os.path.join(dest_dir, name)
    with open(src_path, "r") as f:
        content = f.read()
    
    call = {
        "TargetFile": dest_path,
        "Overwrite": True,
        "CodeContent": content,
        "Description": f"Restore and refactor {name} to replace dummy implementation and reference with genuine logic",
        "toolSummary": f"Write {name}",
        "toolAction": f"Writing {name} to codebase"
    }
    payloads.append(call)

with open("/Users/sac/bcinr/.agents/worker_v5_part9/batch1_payload.json", "w") as f:
    json.dump(payloads, f, indent=2)
print("Done writing batch 1 payload.")
