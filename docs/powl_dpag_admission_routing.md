# Branchless Mathematical Admission Process (`admit.rs`)

The `admit.rs` module implements an O(1) branchless Dynamic Policy Admission Graph (DPAG). It evaluates a packed 64-bit `AdmissionContext` and maps it to a `ProcessTopology` (routing lane) using purely mathematical and bitwise operations. This adheres strictly to the BCINR deterministic mandate (`CC=1`, zero branches, zero allocations).

## 1. Admission Context Encoding

Incoming workflow parameters are bit-packed into a single `u64` word:

| Bits  | Field             | Range | Meaning                                         |
|-------|-------------------|-------|-------------------------------------------------|
| 0..3  | `tenant_class`    | 0..3  | 0=free, 1=standard, 2=enterprise, 3=sovereign   |
| 4..7  | `urgency_tier`    | 0..15 | Higher values indicate more urgency             |
| 8..11 | `resource_load`   | 0..15 | Higher values indicate higher saturation        |
| 12    | `has_sla_token`   | 0/1   | Presence of a Service Level Agreement token     |
| 15    | `is_compensating` | 0/1   | Flag for compensating transactions              |

## 2. Process Topologies (Lanes)

Workflows are routed into one of four execution lanes, sorted by descending priority:
1. **Priority (0)**: Highest priority — enterprise/sovereign tenants with SLA tokens and sufficient urgency.
2. **Standard (1)**: Normal execution lane.
3. **Background (2)**: Best-effort, low-urgency lane.
4. **Quarantine (3)**: Isolated lane for saturated or untrusted contexts.

## 3. Dynamic Admission Parameters

The routing thresholds are globally atomic and dynamically adjusted by an autonomic MAPE-K loop:
* `load_saturation_threshold`: Load level above which everything routes to Quarantine.
* `urgency_priority_threshold`: Minimum urgency for the Priority lane.
* `tenant_class_priority_min`: Minimum tenant class for the Priority lane.
* `tenant_class_standard_min`: Minimum tenant class for the Standard lane.
* `sla_required`: Whether an SLA token is strictly required for the Priority lane.

## 4. Branchless Mathematical Primitives

Instead of `if/else` control flow, the algorithm relies on polynomial bitwise selections.

**`ge_mask(x, y)`**: 
Evaluates $x \ge y$ mathematically. Returns `!0` (all 1s) if true, or `0` if false.
$$ \text{diff} = (y - x - 1) $$
$$ \text{mask} = \text{diff} \gg 63 $$

**`select(mask, active, fallback)`**:
Branchless multiplexer acting as a ternary operator.
$$ \text{result} = (\text{mask} \land \text{active}) \lor (\neg\text{mask} \land \text{fallback}) $$

## 5. Sequential Sign-Mask Multiplexing (The Pipeline)

The admission pipeline operates as a sequence of independent evaluations that are collapsed using sequential multiplexing. 

1. **Quarantine Evaluation**
   $$ q_{mask} = \text{ge\_mask}(\text{load}, \text{load\_saturation\_threshold}) $$
2. **Priority Evaluation**
   $$ tc_{ok} = \text{ge\_mask}(\text{tenant}, \text{tenant\_class\_priority\_min}) $$
   $$ urg_{ok} = \text{ge\_mask}(\text{urgency}, \text{urgency\_priority\_threshold}) $$
   $$ sla_{ok} = \neg(\text{sla\_required\_mask}) \lor \text{has\_sla\_mask} $$
   $$ p_{mask} = tc_{ok} \land urg_{ok} \land sla_{ok} $$
3. **Standard Evaluation**
   $$ s_{mask} = \text{ge\_mask}(\text{tenant}, \text{tenant\_class\_standard\_min}) $$

**Topology Assignment:**
The discriminant is calculated by simulating an `if/else` chain in reverse (from lowest to highest precedence) without branching:
```rust
let v1 = select(s_mask, Standard, Background);
let v2 = select(p_mask, Priority, v1);
let v_final = select(q_mask, Quarantine, v2);
```

The `v_final` integer safely indexes a tiny, fixed-size stack array to yield the final `ProcessTopology` enum value. This ensures 100% deterministic, branchless routing.
