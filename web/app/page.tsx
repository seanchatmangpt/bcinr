/**
 * Home page — RSC. Renders real project stats and a live algorithm count.
 * Data comes from reading the actual bcinr source tree at request time.
 */

import { Suspense } from "react";
import { getProjectMeta, getTestStats } from "@/lib/project";
import { AlgorithmCount } from "@/components/AlgorithmCount";
import { TestStatsPanel } from "@/components/TestStatsPanel";


async function ProjectStats() {
  const meta = await getProjectMeta();
  return (
    <div className="stats-grid">
      <div className="stat-card">
        <span className="value">{meta.algorithmCount}</span>
        <span className="label">Algorithm Modules</span>
      </div>
      <div className="stat-card">
        <span className="value">{meta.coreModuleCount}</span>
        <span className="label">Core Modules</span>
      </div>
      <div className="stat-card">
        <span className="value">{meta.version}</span>
        <span className="label">Version</span>
      </div>
      <div className="stat-card">
        <span className="value">{meta.msrv}</span>
        <span className="label">MSRV</span>
      </div>
    </div>
  );
}

async function LiveTestStats() {
  const stats = await getTestStats();
  return <TestStatsPanel stats={stats} />;
}

export default function HomePage() {
  return (
    <>
      <h1>bcinr</h1>
      <p>
        BranchlessCInRust — a research-grade systems library providing a principled calculus
        for branchless algorithmics. Every primitive is O(1) constant-time, panic-free,
        and formally verified.
      </p>
      <div className="source-notice">
        All numbers rendered here are read live from{" "}
        <code>crates/bcinr-logic/src/</code> — none are hardcoded.
      </div>

      <h2>Project Metrics</h2>
      <Suspense
        fallback={
          <div className="stats-grid">
            {[...Array(4)].map((_, i) => (
              <div key={i} className="stat-card">
                <span className="value" style={{ opacity: 0.2 }}>—</span>
                <span className="label">loading…</span>
              </div>
            ))}
          </div>
        }
      >
        <ProjectStats />
      </Suspense>

      <h2>Test Suite</h2>
      <Suspense fallback={<div className="loading">Loading test stats…</div>}>
        <LiveTestStats />
      </Suspense>

      <h2>Algorithms</h2>
      <Suspense fallback={<div className="loading">Counting algorithms…</div>}>
        <AlgorithmCount />
      </Suspense>

      <p style={{ marginTop: 24 }}>
        <a href="/algorithms" style={{ color: "var(--accent)" }}>
          Browse all algorithms →
        </a>
      </p>
    </>
  );
}
