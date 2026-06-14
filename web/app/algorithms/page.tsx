/**
 * Algorithm browser — RSC, reads 308 algorithm names from source tree.
 * Streaming: names arrive as a single fast async read of the directory listing.
 */

import { getAlgorithmNames, algorithmTier } from "@/lib/project";

export const dynamic = "force-dynamic"; // always read live FS

export default async function AlgorithmsPage() {
  const names = await getAlgorithmNames();

  const tier1 = names.slice(0, 100);
  const tier2 = names.slice(100, 200);
  const tier3 = names.slice(200);

  return (
    <>
      <a href="/" className="back-link">← Home</a>
      <h1>Algorithm Modules</h1>
      <p>
        {names.length} modules read live from{" "}
        <code>crates/bcinr-logic/src/algorithms/</code>. Click any to see its source
        and signature.
      </p>

      <div className="source-notice">
        Tier assignment: sorted alphabetically, 1–100 = Tier 1, 101–200 = Tier 2, 201–
        {names.length} = Tier 3.
      </div>

      <TierSection title="Tier 1 (1–100)" tierClass="tier-1" label="T1" names={tier1} offset={0} />
      <TierSection title="Tier 2 (101–200)" tierClass="tier-2" label="T2" names={tier2} offset={100} />
      <TierSection title="Tier 3 (201–{names.length})" tierClass="tier-3" label="T3" names={tier3} offset={200} />
    </>
  );
}

function TierSection({
  title,
  tierClass,
  label,
  names,
  offset,
}: {
  title: string;
  tierClass: string;
  label: string;
  names: string[];
  offset: number;
}) {
  return (
    <section style={{ marginBottom: 40 }}>
      <h2>{title}</h2>
      <div className="algo-grid">
        {names.map((name, i) => (
          <a key={name} href={`/algorithms/${name}`} className="algo-card">
            <div className="name">{name}</div>
            <span className={`tier ${tierClass}`}>
              #{offset + i + 1} · {label}
            </span>
          </a>
        ))}
      </div>
    </section>
  );
}
