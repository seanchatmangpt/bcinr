/**
 * Server component — reads real algorithm count from FS.
 */

import { getAlgorithmNames } from "@/lib/project";

export async function AlgorithmCount() {
  const names = await getAlgorithmNames();
  const tiers = [
    names.slice(0, 100).length,
    names.slice(100, 200).length,
    names.slice(200).length,
  ];

  return (
    <div style={{ display: "flex", gap: 16, flexWrap: "wrap", marginBottom: 24 }}>
      {[1, 2, 3].map((t) => (
        <div
          key={t}
          className="stat-card"
          style={{ flex: "0 0 auto", minWidth: 140 }}
        >
          <span className="value" style={{ fontSize: 28 }}>{tiers[t - 1]}</span>
          <span className="label">Tier {t} Algorithms</span>
        </div>
      ))}
    </div>
  );
}
