/**
 * Route handler — returns real project metadata as JSON.
 * Used by external consumers and the UI's client-side refresh.
 */

import { NextResponse } from "next/server";
import { getProjectMeta, getTestStats, getCoreModules } from "@/lib/project";

export const dynamic = "force-dynamic";

export async function GET() {
  const [meta, testStats, modules] = await Promise.all([
    getProjectMeta(),
    getTestStats(),
    getCoreModules(),
  ]);

  const totalFns = modules.reduce((s, m) => s + m.publicFunctions.length, 0);

  return NextResponse.json({
    version: meta.version,
    algorithmCount: meta.algorithmCount,
    coreModuleCount: meta.coreModuleCount,
    msrv: meta.msrv,
    totalCoreFunctions: totalFns,
    testStats,
    source: {
      algorithms: "crates/bcinr-logic/src/algorithms/",
      coreModules: "crates/bcinr-logic/src/",
      version: "bcinr/Cargo.toml",
    },
  });
}
