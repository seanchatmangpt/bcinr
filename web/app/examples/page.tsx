/**
 * Examples page — RSC, reads real example file names from bcinr/examples/.
 * Each example file was authored to exercise a specific capability cluster.
 */

import fs from "fs/promises";
import path from "path";

export const dynamic = "force-dynamic";

async function getExamples() {
  const dir = path.resolve(process.cwd(), "../bcinr/examples");
  const entries = await fs.readdir(dir);
  const files = entries.filter((f) => f.endsWith(".rs")).sort();

  return Promise.all(
    files.map(async (f) => {
      const src = await fs.readFile(path.join(dir, f), "utf8");
      // Extract the first //! doc comment block
      const docLines: string[] = [];
      for (const line of src.split("\n")) {
        if (line.startsWith("//!")) {
          docLines.push(line.replace(/^\/\/! ?/, ""));
        } else if (docLines.length > 0) {
          break;
        }
      }
      // Extract title from first doc line (after "# ")
      const title = docLines[0]?.replace(/^#\s*/, "") ?? f.replace(/\.rs$/, "");
      return { name: f.replace(/\.rs$/, ""), title, doc: docLines.join("\n") };
    })
  );
}

export default async function ExamplesPage() {
  const examples = await getExamples();

  return (
    <>
      <a href="/" className="back-link">← Home</a>
      <h1>Running Examples</h1>
      <p>
        {examples.length} example files in <code>bcinr/examples/</code> — each
        exercises a real capability cluster with fail-if-fake assertions. Run with:{" "}
        <code>cargo run --example &lt;name&gt; -p bcinr</code>
      </p>
      <div className="source-notice">
        File names and doc comments read live from <code>bcinr/examples/</code>.
      </div>

      <div className="module-grid">
        {examples.map((ex) => (
          <div key={ex.name} className="module-card">
            <h3>{ex.title}</h3>
            <div style={{ fontSize: 10, color: "var(--muted)", marginBottom: 10 }}>
              <code>bcinr/examples/{ex.name}.rs</code>
            </div>
            <p style={{ fontSize: 11, color: "var(--muted)", margin: 0, whiteSpace: "pre-wrap" }}>
              {ex.doc.split("\n").slice(1, 4).join("\n")}
            </p>
          </div>
        ))}
      </div>
    </>
  );
}
