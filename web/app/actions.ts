"use server";

/**
 * Server action: write the test-cache.json from pre-run stats.
 * Called from the client; writes to web/.test-cache.json.
 * Does NOT run cargo (would block the request for ~30s).
 * Instead, accepts a stats object POSTed from a CI script.
 */

import fs from "fs/promises";
import path from "path";

const CACHE_PATH = path.resolve(process.cwd(), "../.test-cache.json");

export async function saveTestStats(stats: {
  passed: number;
  failed: number;
  ignored: number;
  elapsedSecs: number | null;
}) {
  // Validate that all fields are present and numeric
  if (
    typeof stats.passed !== "number" ||
    typeof stats.failed !== "number" ||
    typeof stats.ignored !== "number"
  ) {
    throw new Error("Invalid stats payload");
  }
  await fs.writeFile(CACHE_PATH, JSON.stringify(stats, null, 2), "utf8");
}
