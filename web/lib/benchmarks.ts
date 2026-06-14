/**
 * Reads real benchmark data from criterion's JSON output in target/criterion/.
 * Falls back to bench source file names if criterion hasn't been run.
 */

import fs from "fs/promises";
import path from "path";

const REPO_ROOT = path.resolve(process.cwd(), "..");

export interface BenchResult {
  name: string;
  meanNs: number;
  stdDevNs: number;
  medianNs: number;
  unit: string;
}

export interface BenchGroup {
  groupName: string;
  results: BenchResult[];
}

/** Parse criterion's estimates.json for a single benchmark. */
async function readCriterionEstimates(
  criterionDir: string,
  benchName: string
): Promise<BenchResult | null> {
  const estimatesPath = path.join(
    criterionDir,
    benchName,
    "new",
    "estimates.json"
  );
  try {
    const raw = await fs.readFile(estimatesPath, "utf8");
    const data = JSON.parse(raw) as {
      mean: { point_estimate: number };
      std_dev: { point_estimate: number };
      median: { point_estimate: number };
    };
    const meanNs = data.mean.point_estimate;
    return {
      name: benchName,
      meanNs,
      stdDevNs: data.std_dev.point_estimate,
      medianNs: data.median.point_estimate,
      unit: meanNs < 1000 ? "ns" : meanNs < 1_000_000 ? "μs" : "ms",
    };
  } catch {
    return null;
  }
}

/** Read all available criterion results from target/criterion/. */
export async function getCriterionResults(): Promise<{
  groups: BenchGroup[];
  totalBenches: number;
  hasCriterionData: boolean;
}> {
  const criterionDir = path.join(REPO_ROOT, "target/criterion");

  let entries: string[] = [];
  try {
    entries = await fs.readdir(criterionDir);
  } catch {
    return { groups: [], totalBenches: 0, hasCriterionData: false };
  }

  // criterion stores one directory per benchmark group + one per function
  // filter out non-benchmark dirs
  const benchNames = entries.filter(
    (e) => !e.startsWith(".") && e !== "report"
  );

  const results: BenchResult[] = (
    await Promise.all(
      benchNames.map((name) => readCriterionEstimates(criterionDir, name))
    )
  ).filter((r): r is BenchResult => r !== null);

  // Group by algorithm family prefix
  const groupMap = new Map<string, BenchResult[]>();
  for (const r of results) {
    // "parallel_bits_deposit_u64_avg" → "parallel_bits_deposit_u64"
    const key = r.name.replace(/_avg$|_min$|_max$/, "");
    if (!groupMap.has(key)) groupMap.set(key, []);
    groupMap.get(key)!.push(r);
  }

  const groups: BenchGroup[] = [...groupMap.entries()].map(([g, rs]) => ({
    groupName: g,
    results: rs,
  }));

  return {
    groups,
    totalBenches: results.length,
    hasCriterionData: results.length > 0,
  };
}

/** Read bench function names from the source files (no execution needed). */
export async function getBenchFunctionNames(): Promise<string[]> {
  const benchDir = path.join(REPO_ROOT, "bcinr-bench/benches");
  const files = (await fs.readdir(benchDir)).filter((f) => f.endsWith(".rs"));
  const names: string[] = [];
  for (const f of files) {
    const src = await fs.readFile(path.join(benchDir, f), "utf8");
    const matches = [...src.matchAll(/bench_function\("([^"]+)"/g)];
    for (const m of matches) names.push(m[1]);
  }
  return [...new Set(names)].sort();
}
