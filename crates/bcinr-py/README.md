# bcinr-py

Python bindings for the bcinr library via [PyO3](https://pyo3.rs/).

Exposes POWL workflow compilation, admission context routing, UTF-8 codepoint
counting, and branchless mask operations from the bcinr ecosystem.

## Installation

Requires [maturin](https://github.com/PyO3/maturin):

```bash
cargo install maturin
```

Install into the current Python environment (development mode):

```bash
cd crates/bcinr-py
maturin develop
```

Build a distributable wheel:

```bash
cd crates/bcinr-py
maturin build --release
pip install target/wheels/bcinr-*.whl
```

## Usage

```python
import bcinr

# Compile a POWL sequence
result = bcinr.compile_sequence("A,B,C")
print(result)  # {"ok": True, "op_count": 3, "topology": "Sequence"}

# Admission context routing
ctx = bcinr.admit_context(tenant_class=3, urgency_tier=15,
                          resource_load=0, has_sla_token=True)
print(ctx["topology"])  # "Priority"

# UTF-8 codepoint counting (branchless)
n = bcinr.count_codepoints("héllo")
print(n)  # 5

# Branchless mask operations
print(bcinr.mask_and(0xFF, 0x0F))  # 15
print(bcinr.mask_or(0xF0, 0x0F))   # 255
print(bcinr.popcount(0xFF))         # 8
```

## API

| Function | Signature | Description |
|----------|-----------|-------------|
| `compile_sequence` | `(labels: str) -> dict` | Compile comma-separated labels into POWL tape |
| `admit_context` | `(tenant_class, urgency_tier, resource_load, has_sla_token) -> dict` | Route to process topology |
| `count_codepoints` | `(data: str) -> int` | Count Unicode codepoints branchlessly |
| `mask_and` | `(a: int, b: int) -> int` | Bitwise AND |
| `mask_or` | `(a: int, b: int) -> int` | Bitwise OR |
| `mask_xor` | `(a: int, b: int) -> int` | Bitwise XOR |
| `popcount` | `(value: int) -> int` | Population count (set bits) |

## Architecture

- **PyO3 version:** 0.22 with `abi3-py38` (compatible with Python 3.8+)
- **maturin:** Handles build, packaging, and Python linking
- **Dependencies:** `bcinr-logic` (branchless algorithms), `bcinr-powl` (workflow runtime)
