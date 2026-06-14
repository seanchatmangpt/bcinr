/**
 * Server component — displays test stats from the .test-cache.json file.
 * If the cache doesn't exist, explains how to populate it.
 */

import type { TestStats } from "@/lib/project";

export function TestStatsPanel({ stats }: { stats: TestStats | null }) {
  if (!stats) {
    return (
      <div className="stat-card" style={{ marginBottom: 24 }}>
        <p style={{ margin: 0, fontSize: 12 }}>
          Test cache not found. Populate it with:
        </p>
        <pre style={{ marginTop: 8, fontSize: 11 }}>
          {`cargo test -p bcinr-logic --lib 2>&1 | \\
  grep "^test result" | \\
  node -e "
    const l=require('fs').readFileSync('/dev/stdin','utf8').trim();
    const m=l.match(/(\\d+) passed.*?(\\d+) failed.*?(\\d+) ignored/);
    const t=l.match(/finished in ([\\d.]+)s/);
    const obj={passed:+m[1],failed:+m[2],ignored:+m[3],elapsedSecs:t?+t[1]:null};
    require('fs').writeFileSync('web/.test-cache.json',JSON.stringify(obj));
  "`}
        </pre>
      </div>
    );
  }

  const allPassed = stats.failed === 0;

  return (
    <div className="stats-grid" style={{ marginBottom: 24 }}>
      <div className="stat-card">
        <span className="value" style={{ color: allPassed ? "var(--tier1)" : "var(--tier3)" }}>
          {stats.passed}
        </span>
        <span className="label">Tests Passed</span>
      </div>
      <div className="stat-card">
        <span className="value" style={{ color: stats.failed > 0 ? "var(--tier3)" : "var(--muted)" }}>
          {stats.failed}
        </span>
        <span className="label">Tests Failed</span>
      </div>
      <div className="stat-card">
        <span className="value">{stats.ignored}</span>
        <span className="label">Ignored</span>
      </div>
      {stats.elapsedSecs !== null && (
        <div className="stat-card">
          <span className="value">{stats.elapsedSecs.toFixed(2)}s</span>
          <span className="label">Duration</span>
        </div>
      )}
    </div>
  );
}
