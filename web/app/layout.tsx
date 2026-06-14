import type { Metadata } from "next";
import { Suspense } from "react";
import "./globals.css";

export const metadata: Metadata = {
  title: "bcinr — Branchless C in Rust",
  description:
    "Live view of the bcinr algorithmic library: 308 branchless primitives, rendered from real source.",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body>
        <header>
          <nav>
            <a href="/" className="brand">bcinr</a>
            <a href="/algorithms">Algorithms</a>
            <a href="/modules">Core Modules</a>
            <a href="/examples">Examples</a>
            <a href="/changelog">Changelog</a>
          </nav>
        </header>
        <main>
          <Suspense fallback={<div className="loading">Loading…</div>}>
            {children}
          </Suspense>
        </main>
        <footer>
          <p>Data sourced live from <code>crates/bcinr-logic/src/</code> — no fixtures.</p>
        </footer>
      </body>
    </html>
  );
}
