# CMCA Challenger Use-Case Validation

This layer converts CMCA's technical properties into buyer-relevant proof. It does not replace the Divan benchmark suite. The benchmark suite answers **how fast the mechanism executes**. These scenarios answer **whether the mechanism changes the buyer's operational risk, decision, and acceptance criteria**.

## Validation contract

Every use case must carry five fields:

1. **Buyer** — the accountable operational and economic audience.
2. **Teach** — a non-obvious risk or cost the buyer is likely underestimating.
3. **Tailor** — the CMCA capability expressed in that buyer's operating language.
4. **Take control** — a concrete acceptance criterion or next decision.
5. **Executable proof** — a deterministic success, refusal, or fallback through public CMCA APIs.

A sales narrative has no standing unless the executable proof passes. A passing technical test has no Challenger standing unless the buyer, reframe, and decision are explicit.

## Scenario portfolio

| Use case | Commercial reframe | CMCA proof | Buyer decision |
|---|---|---|---|
| Cloud inference routing | The primary risk is not routing quality; it is unbounded adaptation without evidence. | Stable telemetry certifies, matching receipts admit, allocation succeeds. | Require certified learning before autonomous routing changes. |
| Fraud operations under distribution shift | A model can remain fast after its assumptions have failed. | Drift refuses certification; selection-only fallback remains available. | Make drift refusal and fallback contractual acceptance criteria. |
| Industrial control rollout | A certificate is only valid for the governed configuration it identifies. | A stale certificate digest is refused before actuation. | Require runtime and certificate identity at deployment. |
| Logistics dispatch stabilization | Optimization can destroy throughput by switching modes faster than the operation settles. | A dwell-time violation is refused before allocation. | Set the stabilization interval before adaptive dispatch is approved. |
| Marketplace pricing control | An optimizer can improve its objective while violating the economics of the business. | An out-of-envelope multiplier is refused before standing is granted. | Make envelope refusal a go-live gate rather than a later audit. |

## Challenger conversation pattern

### 1. Warm up with the operating objective

Start with the buyer's current objective: lower inference cost, reduce fraud loss, increase plant throughput, stabilize dispatch, or protect marketplace margin.

### 2. Reframe the hidden failure

Move the conversation away from generic optimization quality. The central failure is **unreceipted adaptation**: a system changes behavior without current evidence that the configuration, telemetry, envelope, and outcome still agree.

### 3. Quantify the operational consequence

Use the buyer's own units: excess cloud spend, false-positive review load, unsafe control transitions, re-routing churn, margin leakage, or audit exposure.

### 4. Present the new capability

CMCA separates three states that conventional optimization stacks often collapse:

- certified adaptive execution;
- typed refusal;
- safe non-learning fallback.

This is the buyer-visible distinction. The product is not merely a faster allocator. It is an allocator that can prove when adaptation has standing and refuse when it does not.

### 5. Take control with an acceptance test

Do not end with a feature demonstration. Ask the buyer to adopt one executable acceptance criterion from the scenario portfolio. The criterion should run in CI and remain attached to the production configuration.

## Evidence hierarchy

The preferred commercial proof order is:

1. deterministic use-case outcome;
2. typed refusal or safe fallback;
3. end-to-end execution benchmark;
4. subsystem benchmark;
5. primitive kernel benchmark.

This ordering prevents nanosecond measurements from becoming the sales story. Performance supports the commercial claim, but the use-case outcome defines it.

## Implementation

The executable scenarios live in `bcinr-bench/tests/cmca_challenger_use_cases.rs`. They use the same public CMCA surfaces as the benchmark suite and intentionally avoid mocks, private functions, and narrative-only assertions.

Run the layer directly with:

```bash
cargo test -p bcinr-bench --test cmca_challenger_use_cases
```
