/**
 * Core modules page — RSC, reads public fn names from actual .rs source files.
 */

import { getCoreModules } from "@/lib/project";

export const dynamic = "force-dynamic";

export default async function ModulesPage() {
  const modules = await getCoreModules();
  const totalFns = modules.reduce((s, m) => s + m.publicFunctions.length, 0);

  return (
    <>
      <a href="/" className="back-link">← Home</a>
      <h1>Core Modules</h1>
      <p>
        {modules.length} modules · {totalFns} public functions — read live from{" "}
        <code>crates/bcinr-logic/src/*.rs</code>
      </p>

      <div className="module-grid">
        {modules.map((mod) => (
          <div key={mod.name} className="module-card">
            <h3>{mod.name}</h3>
            <div style={{ fontSize: 10, color: "var(--muted)", marginBottom: 10 }}>
              <code>{mod.file}</code>
            </div>
            <ul className="fn-list">
              {mod.publicFunctions.map((fn) => (
                <li key={fn}><code>{fn}</code></li>
              ))}
            </ul>
          </div>
        ))}
      </div>
    </>
  );
}
