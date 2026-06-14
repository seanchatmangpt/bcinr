/**
 * Changelog page — RSC, reads RELEASE_NOTES.md verbatim from the repo root.
 * No transformation: the raw Markdown is displayed as preformatted text.
 */

import fs from "fs/promises";
import path from "path";

export const dynamic = "force-dynamic";

export default async function ChangelogPage() {
  const filePath = path.resolve(process.cwd(), "../RELEASE_NOTES.md");
  const content = await fs.readFile(filePath, "utf8");
  const lines = content.split("\n").length;

  return (
    <>
      <a href="/" className="back-link">← Home</a>
      <h1>Release Notes</h1>
      <div className="source-notice">
        Source: <code>RELEASE_NOTES.md</code> ({lines} lines) — rendered verbatim, no fixtures.
      </div>
      <pre style={{ whiteSpace: "pre-wrap", fontSize: 12, lineHeight: 1.7 }}>
        {content}
      </pre>
    </>
  );
}
