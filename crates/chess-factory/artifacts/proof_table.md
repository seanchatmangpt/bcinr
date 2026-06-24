# Crown Proof Table — receipt-cited Elo(tau) matrix

Rows = opponents, columns = per-move latency budget tiers. Each measured cell
reports `score% (95% CI) / perf-Elo / factory_us|opp_us` plus the `blake3`
receipt hash that proves it (verifiable against `artifacts/benchmark.receipt.json`)
and the `run_id` that produced it. Engines not installed on this host render
`not-installed` and are NEVER faked; installed-but-unmeasured cells render
`(not measured)`. Display strings (score%, CI, perf-Elo) are pre-formatted in
`artifacts/results.ttl` so this template performs no arithmetic.

| Opponent | tier | t100us | t250us | t500us | t1ms | t10ms |
|----------|------|--------|--------|--------|------|-------|
| stockfish | reference | 46.9% (CI 22.4-71.3%) / 1478E / 3368\|244us / `78d170855e4ed6b974f30f8336c6109c161ee7c9a7bd698d07ac750d61929312` (run `bd7058756e308122d5bc81c054c729c0b6c4a8c069089b2994676a13026cee3f`) | 28.1% (CI 6.1-50.2%) / 1337E / 4233\|302us / `4b1d4977e54868a7eeaa28a2349abd06475f07ee2bf70769882d4e2c7d6f2193` (run `bd7058756e308122d5bc81c054c729c0b6c4a8c069089b2994676a13026cee3f`) | 15.6% (CI 0.0-33.4%) / 1207E / 3238\|284us / `ec1f69314c1a3c2cf7ff6353af784aa72469d681e104ebdf91fc77b6a628eb3c` (run `bd7058756e308122d5bc81c054c729c0b6c4a8c069089b2994676a13026cee3f`) | 6.2% (CI 0.0-18.1%) / 1030E / 8481\|1311us / `826ec55dd9f8292c73559089aa4521ac4319a2a61f27db8bb4fc78371d80db01` (run `bd7058756e308122d5bc81c054c729c0b6c4a8c069089b2994676a13026cee3f`) | 0.0% (CI 0.0-0.0%) / 700E / 10204\|9906us / `674c91a3201e5a5f1c80889e8dd987c8eec77ae86bd05e916d38944faf21e161` (run `bd7058756e308122d5bc81c054c729c0b6c4a8c069089b2994676a13026cee3f`) |
| sanity_random | sanity | 100.0% (CI 100.0-100.0%) / 2300E / 9578\|30us / `8473f87e6d070aa5af7fb266799bd9d08f39b37bd7a163d1b223add15b6d673b` (run `bd7058756e308122d5bc81c054c729c0b6c4a8c069089b2994676a13026cee3f`) | 100.0% (CI 100.0-100.0%) / 2300E / 9571\|29us / `0314f8b0134d05aa1bdcfd4391a64d00197f4276c2838bd97e44a5ddefb97881` (run `bd7058756e308122d5bc81c054c729c0b6c4a8c069089b2994676a13026cee3f`) | 100.0% (CI 100.0-100.0%) / 2300E / 9566\|28us / `8a135fd3b96281d991d2e63e1a7f5ab58f2c694d4f13ecf20c395e914b55d736` (run `bd7058756e308122d5bc81c054c729c0b6c4a8c069089b2994676a13026cee3f`) | 100.0% (CI 100.0-100.0%) / 2300E / 9549\|27us / `0506c0f7dbb6c4ca8bd2331013968bd68d6d6d728261c988e6acbd01391f5f58` (run `bd7058756e308122d5bc81c054c729c0b6c4a8c069089b2994676a13026cee3f`) | 100.0% (CI 100.0-100.0%) / 2300E / 9509\|28us / `a4fd6d93a38cb082c32c3b43557f36fd6921de6f355aa7a963c3faedfc49511a` (run `bd7058756e308122d5bc81c054c729c0b6c4a8c069089b2994676a13026cee3f`) |
| sanity_greedy | sanity | 96.9% (CI 88.3-100.0%) / 2097E / 23309\|40us / `de817a50ea5e71968bf4e9fbe02f79091c7d93568b48ad09a4ea7ef591e144b7` (run `bd7058756e308122d5bc81c054c729c0b6c4a8c069089b2994676a13026cee3f`) | 96.9% (CI 88.3-100.0%) / 2097E / 23780\|35us / `7cc71b16c1b2f25b57b05c31f2d2b64b8834d6ea5d48224712b89190802d8c03` (run `bd7058756e308122d5bc81c054c729c0b6c4a8c069089b2994676a13026cee3f`) | 96.9% (CI 88.3-100.0%) / 2097E / 23680\|40us / `1adb84973656852febb86302e0be32cd0d823cbba44a57f949e2370042934879` (run `bd7058756e308122d5bc81c054c729c0b6c4a8c069089b2994676a13026cee3f`) | 96.9% (CI 88.3-100.0%) / 2097E / 26662\|42us / `d64b6260e66027f2a8ce5c63b693298151fed05cfe75642d05348eed38e499a1` (run `bd7058756e308122d5bc81c054c729c0b6c4a8c069089b2994676a13026cee3f`) | 96.9% (CI 88.3-100.0%) / 2097E / 26221\|44us / `31ae8ff3fb2886cbd9374ce45ec9afdd563f069f7b1f690d1071cd63b5ae4580` (run `bd7058756e308122d5bc81c054c729c0b6c4a8c069089b2994676a13026cee3f`) |
| bcinr_uci | reference | 81.2% (CI 62.1-100.0%) / 1755E / 9338\|1030us / `544ab66afdf4d6fffaf5600ef86be34ec644160538c61686bfc74e070c78bba7` (run `bd7058756e308122d5bc81c054c729c0b6c4a8c069089b2994676a13026cee3f`) | 59.4% (CI 35.3-83.4%) / 1566E / 6852\|1026us / `9ff4f42e68911d684840f00aaa13c4742a421c71a82fc224d85dcebacf7a18f1` (run `bd7058756e308122d5bc81c054c729c0b6c4a8c069089b2994676a13026cee3f`) | 75.0% (CI 53.8-96.2%) / 1691E / 9750\|1033us / `b4662121c2cfffafe929f6cb04675dbc5bde7c5531336efc12e1cab0657397d9` (run `bd7058756e308122d5bc81c054c729c0b6c4a8c069089b2994676a13026cee3f`) | 62.5% (CI 38.8-86.2%) / 1589E / 6852\|1023us / `06732fde6f11b51537e57cab1603666c3de2e9b3f8bf4993fb3c3f7789fd288e` (run `bd7058756e308122d5bc81c054c729c0b6c4a8c069089b2994676a13026cee3f`) | 34.4% (CI 11.1-57.6%) / 1388E / 7973\|10026us / `77e168176f12e0805e987ed524bddaba66f2a2a95250026c1feeafef218d9571` (run `bd7058756e308122d5bc81c054c729c0b6c4a8c069089b2994676a13026cee3f`) |
| bcinr_az | reference | 100.0% (CI 100.0-100.0%) / 2300E / 29915\|1749us / `80894164246d2019083b1dafed30667b3c0aa2dd3fa5ca4b8c9c039e4f30bd48` (run `bd7058756e308122d5bc81c054c729c0b6c4a8c069089b2994676a13026cee3f`) | 100.0% (CI 100.0-100.0%) / 2300E / 29749\|1782us / `4411842340b5092705e7063717a7e1516927ba525c4e38f3681f1d473505a19b` (run `bd7058756e308122d5bc81c054c729c0b6c4a8c069089b2994676a13026cee3f`) | 100.0% (CI 100.0-100.0%) / 2300E / 29789\|1766us / `ca714540429fe16269823768708c5ebc65b2b84302d90fa78b03ff55f630545f` (run `bd7058756e308122d5bc81c054c729c0b6c4a8c069089b2994676a13026cee3f`) | 100.0% (CI 100.0-100.0%) / 2300E / 29725\|1772us / `e8b574810407ce83a49a612b8ee513371fb837b72cebbd49ee21ad8058f893b1` (run `bd7058756e308122d5bc81c054c729c0b6c4a8c069089b2994676a13026cee3f`) | 93.8% (CI 81.9-100.0%) / 1970E / 18539\|14465us / `6d3b4caaf99f259159e5c4efe0ced80be61e6b036b071cb9b9ad07ffc670451c` (run `bd7058756e308122d5bc81c054c729c0b6c4a8c069089b2994676a13026cee3f`) |
| berserk | reference | not-installed | not-installed | not-installed | not-installed | not-installed |
| rubichess | reference | not-installed | not-installed | not-installed | not-installed | not-installed |
| lc0 | reference | not-installed | not-installed | not-installed | not-installed | not-installed |

## Honesty footnote — depth-asymmetric bcinr_uci row

The `bcinr_uci` row is **depth-asymmetric**: the factory searches depth-4 while
`bcinr_uci` is driven at `go movetime 1`, so its headline score is **inflated by
deeper search**, not by a stronger evaluation. An **equal-footing** head-to-head
(both engines at exactly 1 ply) is recorded under `parity_check` in
`artifacts/benchmark.receipt.json`. Read BOTH numbers: the matrix cell is the
unequal/inflated figure; the parity match is the fair one.

## Receipt verification

Every measured cell's receipt hash is the `blake3` of
`prev_hash || opponent || budget || go_command || W/D/L || think_micros (both
sides)`, hash-chained from genesis. The same hashes appear verbatim in the
`cell_receipts[]` array of `artifacts/benchmark.receipt.json`. Not-installed
engines and unmeasured cells carry no receipt and are never assigned one.