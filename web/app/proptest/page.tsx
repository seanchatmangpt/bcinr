/**
 * Proptest regressions page — RSC, reads real .txt seed files from
 * crates/bcinr-logic/proptest-regressions/algorithms/.
 * These files are the proptest failure corpus: shrunk input seeds that
 * are re-run on every test to prevent regressions.
 */

import fs from "fs/promises";
import path from "path";

export const dynamic = "force-dynamic";

interface RegressionEntry {
  algorithm: string;
  seeds: string[];
}

async function getRegressions(): Promise<RegressionEntry[]> {
  const dir = path.resolve(
    process.cwd(),
    "../crates/bcinr-logic/proptest-regressions/algorithms"
  );
  const files = (await fs.readdir(dir)).filter((f) => f.endsWith(".txt")).sort();

  return Promise.all(
    files.map(async (f) => {
      const content = await fs.readFile(path.join(dir, f), "utf8");
      // Extract seed lines (lines starting with "cc ")
      const seeds = content
        .split("\n")
        .filter((l) => l.startsWith("cc "))
        .map((l) => l.replace(/^cc /, "").trim());
      return { algorithm: f.replace(/\.txt$/, ""), seeds };
    })
  );
}

export default async function ProptestPage() {
  const regressions = await getRegressions();
  const totalSeeds = regressions.reduce((s, r) => s + r.seeds.length, 0);
  const withSeeds = regressions.filter((r) => r.seeds.length > 0);

  return (
    <>
      <a href="/" className="back-link">← Home</a>
      <h1>Proptest Regression Corpus</h1>
      <p>
        Failure seeds discovered by proptest and saved for permanent replay.
        Each entry is a shrunk minimal counter-example that previously caused
        an equivalence failure between the branchless implementation and its
        reference.
      </p>

      <div className="source-notice">
        Source: <code>crates/bcinr-logic/proptest-regressions/algorithms/</code> —
        {regressions.length} files, {totalSeeds} saved seeds
      </div>

      <div className="stats-grid" style={{ marginBottom: 32 }}>
        <div className="stat-card">
          <span className="value">{regressions.length}</span>
          <span className="label">Regression Files</span>
        </div>
        <div className="stat-card">
          <span className="value">{totalSeeds}</span>
          <span className="label">Saved Seeds</span>
        </div>
        <div className="stat-card">
          <span className="value">{withSeeds.length}</span>
          <span className="label">Algorithms with Seeds</span>
        </div>
        <div className="stat-card">
          <span className="value">{regressions.length - withSeeds.length}</span>
          <span className="label">Empty (no failures yet)</span>
        </div>
      </div>

      <h2>Algorithms with Seeds</h2>
      {withSeeds.length === 0 && (
        <p>No algorithms have saved failure seeds yet.</p>
      )}
      <div className="module-grid" style={{ marginBottom: 32 }}>
        {withSeeds.map((r) => (
          <div key={r.algorithm} className="module-card">
            <h3>{r.algorithm}</h3>
            <ul className="fn-list">
              {r.seeds.map((seed, i) => (
                <li key={i}>
                  <code style={{ fontSize: 10, wordBreak: "break-all" }}>
                    {seed.slice(0, 72)}{seed.length > 72 ? "…" : ""}
                  </code>
                </li>
              ))}
            </ul>
          </div>
        ))}
      </div>

      <h2>All Regression Files ({regressions.length})</h2>
      <div className="algo-grid">
        {regressions.map((r) => (
          <div
            key={r.algorithm}
            className="algo-card"
            style={{
              borderColor: r.seeds.length > 0 ? "var(--tier2)" : undefined,
            }}
          >
            <div className="name">{r.algorithm}</div>
            <span
              className={`tier ${r.seeds.length > 0 ? "tier-2" : "tier-1"}`}
            >
              {r.seeds.length} seed{r.seeds.length !== 1 ? "s" : ""}
            </span>
          </div>
        ))}
      </div>
    </>
  );
}
