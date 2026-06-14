/**
 * Benchmarks page — RSC.
 * Primary data: criterion JSON from target/criterion/ (real measured ns values).
 * Fallback: bench function names from bcinr-bench/benches/*.rs source files.
 * Never shows placeholder numbers.
 */

import { getCriterionResults, getBenchFunctionNames } from "@/lib/benchmarks";

export const dynamic = "force-dynamic";

function formatTime(ns: number): string {
  if (ns < 1000) return `${ns.toFixed(1)} ns`;
  if (ns < 1_000_000) return `${(ns / 1000).toFixed(1)} μs`;
  return `${(ns / 1_000_000).toFixed(1)} ms`;
}

export default async function BenchmarksPage() {
  const [criterion, benchNames] = await Promise.all([
    getCriterionResults(),
    getBenchFunctionNames(),
  ]);

  return (
    <>
      <a href="/" className="back-link">← Home</a>
      <h1>Benchmarks</h1>

      {criterion.hasCriterionData ? (
        <>
          <p>
            Criterion results from <code>target/criterion/</code> —{" "}
            {criterion.totalBenches} measurements captured.
          </p>
          <div className="source-notice">
            Data source: criterion estimates.json (real measured ns). Run{" "}
            <code>cargo bench</code> to refresh.
          </div>

          <div className="stats-grid" style={{ marginBottom: 32 }}>
            <div className="stat-card">
              <span className="value">{criterion.groups.length}</span>
              <span className="label">Benchmark Functions</span>
            </div>
            <div className="stat-card">
              <span className="value">{criterion.totalBenches}</span>
              <span className="label">Total Measurements</span>
            </div>
          </div>

          <h2>Results</h2>
          <div
            style={{
              display: "grid",
              gap: 2,
              fontFamily: "monospace",
              fontSize: 12,
            }}
          >
            <div
              style={{
                display: "grid",
                gridTemplateColumns: "2fr 1fr 1fr 1fr",
                padding: "6px 12px",
                color: "var(--muted)",
                borderBottom: "1px solid var(--border)",
              }}
            >
              <span>Function</span>
              <span>Mean</span>
              <span>Median</span>
              <span>Std Dev</span>
            </div>
            {criterion.groups.flatMap((g) =>
              g.results.map((r) => (
                <div
                  key={r.name}
                  style={{
                    display: "grid",
                    gridTemplateColumns: "2fr 1fr 1fr 1fr",
                    padding: "5px 12px",
                    borderBottom: "1px solid #111",
                  }}
                >
                  <span style={{ color: "var(--fg)" }}>{r.name}</span>
                  <span style={{ color: "var(--tier1)" }}>
                    {formatTime(r.meanNs)}
                  </span>
                  <span style={{ color: "var(--muted)" }}>
                    {formatTime(r.medianNs)}
                  </span>
                  <span style={{ color: "var(--muted)" }}>
                    ±{formatTime(r.stdDevNs)}
                  </span>
                </div>
              ))
            )}
          </div>
        </>
      ) : (
        <>
          <p>
            No criterion output found in <code>target/criterion/</code>. Benchmarks
            have not been run yet — showing registered benchmark function names from
            source.
          </p>
          <div className="source-notice">
            Run <code>cargo bench</code> from the repo root to populate real timing
            data. This page will then show measured nanosecond values from
            criterion's estimates.json files.
          </div>

          <div className="stats-grid" style={{ marginBottom: 32 }}>
            <div className="stat-card">
              <span className="value">{benchNames.length}</span>
              <span className="label">Registered Bench Functions</span>
            </div>
            <div className="stat-card">
              <span className="value" style={{ fontSize: 16, color: "var(--muted)" }}>
                NOT YET RUN
              </span>
              <span className="label">Timing Data</span>
            </div>
          </div>

          <h2>Registered Benchmark Functions ({benchNames.length})</h2>
          <p style={{ fontSize: 11 }}>
            These names are read from <code>bcinr-bench/benches/*.rs</code> source
            files — real bench registrations, no generated list.
          </p>
          <div className="algo-grid">
            {benchNames.map((name) => (
              <div key={name} className="algo-card">
                <div className="name">{name}</div>
              </div>
            ))}
          </div>
        </>
      )}
    </>
  );
}
