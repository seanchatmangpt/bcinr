/**
 * Algorithm detail page — RSC, reads actual source file for this algorithm.
 * Shows: real doc comment, real fn signature, real source lines.
 * generateStaticParams pre-renders all 308 at build time.
 */

import { getAlgorithmNames, getAlgorithmDetail, algorithmTier } from "@/lib/project";
import { notFound } from "next/navigation";

export async function generateStaticParams() {
  const names = await getAlgorithmNames();
  return names.map((name) => ({ name }));
}

export default async function AlgorithmPage({
  params,
}: {
  params: Promise<{ name: string }>;
}) {
  const { name } = await params;
  const names = await getAlgorithmNames();
  const idx = names.indexOf(name);
  if (idx === -1) notFound();

  const tier = algorithmTier(idx);
  const detail = await getAlgorithmDetail(name);

  // Truncate source to first 60 lines for display; link shows the real file path
  const sourceLines = detail.source.split("\n").slice(0, 60).join("\n");
  const totalLines = detail.source.split("\n").length;

  return (
    <>
      <a href="/algorithms" className="back-link">← All Algorithms</a>

      <h1>{name}</h1>

      <div style={{ display: "flex", gap: 8, marginBottom: 16, flexWrap: "wrap" }}>
        <span className={`tier tier-${tier}`} style={{ padding: "3px 10px", borderRadius: 4, fontSize: 12 }}>
          Tier {tier}
        </span>
        <span className="badge">#{idx + 1} of {names.length}</span>
        <span className="badge">fn({name})</span>
      </div>

      <div className="source-notice">
        Source: <code>crates/bcinr-logic/src/algorithms/{name}.rs</code> ({totalLines} lines)
      </div>

      {detail.docComment && (
        <>
          <h2>Documentation</h2>
          <pre style={{ marginBottom: 24, whiteSpace: "pre-wrap" }}>{detail.docComment}</pre>
        </>
      )}

      {detail.signature && (
        <>
          <h2>Signature</h2>
          <div className="sig">{detail.signature}</div>
        </>
      )}

      <h2>Source (first 60 lines)</h2>
      <pre>{sourceLines}</pre>
      {totalLines > 60 && (
        <p style={{ fontSize: 11, marginTop: 8 }}>
          … {totalLines - 60} more lines in{" "}
          <code>crates/bcinr-logic/src/algorithms/{name}.rs</code>
        </p>
      )}
    </>
  );
}
