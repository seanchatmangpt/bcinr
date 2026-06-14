/**
 * Route handler — returns the real algorithm list as JSON.
 * Edge-compatible: only reads FS, no native modules.
 */

import { NextResponse } from "next/server";
import { getAlgorithmNames, algorithmTier } from "@/lib/project";

export const dynamic = "force-dynamic";

export async function GET() {
  const names = await getAlgorithmNames();
  const algorithms = names.map((name, i) => ({
    name,
    index: i + 1,
    tier: algorithmTier(i),
  }));

  return NextResponse.json({
    count: algorithms.length,
    algorithms,
    source: "crates/bcinr-logic/src/algorithms/",
  });
}
