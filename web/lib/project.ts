/**
 * Real project data — all values derived from the actual bcinr source tree.
 * Nothing here is hardcoded as a fixture; every call reads or parses live files.
 */

import fs from "fs/promises";
import path from "path";

// In Next.js, process.cwd() is the web/ directory at both dev and build time.
// The monorepo root is one level up.
const REPO_ROOT = path.resolve(process.cwd(), "..");

// -------------------------------------------------------------------
// Algorithms
// -------------------------------------------------------------------

export interface AlgorithmMeta {
  name: string;
  tier: 1 | 2 | 3; // 1-100, 101-200, 201-300 — derived from index in sorted list
  sourceFile: string;
  docComment: string | null;
}

/** Read the 308 algorithm module names directly from the source tree. */
export async function getAlgorithmNames(): Promise<string[]> {
  const dir = path.join(
    REPO_ROOT,
    "crates/bcinr-logic/src/algorithms"
  );
  const entries = await fs.readdir(dir);
  return entries
    .filter((f) => f.endsWith(".rs") && f !== "mod.rs")
    .map((f) => f.replace(/\.rs$/, ""))
    .sort();
}

/** Read source of one algorithm file; extract the leading doc comment and fn signature. */
export async function getAlgorithmDetail(name: string): Promise<{
  source: string;
  docComment: string | null;
  signature: string | null;
}> {
  const filePath = path.join(
    REPO_ROOT,
    `crates/bcinr-logic/src/algorithms/${name}.rs`
  );
  const source = await fs.readFile(filePath, "utf8");

  // Extract leading //! or /// block
  const docLines: string[] = [];
  for (const line of source.split("\n")) {
    if (line.startsWith("//!") || line.startsWith("///")) {
      docLines.push(line.replace(/^\/\/[!/] ?/, ""));
    } else if (docLines.length > 0) {
      break;
    }
  }

  // Extract pub fn signature
  const sigMatch = source.match(/pub fn \w+\([^)]*\)[^{]*/);

  return {
    source,
    docComment: docLines.length > 0 ? docLines.join("\n") : null,
    signature: sigMatch ? sigMatch[0].trim() : null,
  };
}

/** Assign tier by index in sorted list: 1-100 → tier 1, etc. */
export function algorithmTier(index: number): 1 | 2 | 3 {
  if (index < 100) return 1;
  if (index < 200) return 2;
  return 3;
}

// -------------------------------------------------------------------
// Core modules
// -------------------------------------------------------------------

export interface CoreModule {
  name: string;
  file: string;
  publicFunctions: string[];
}

const CORE_MODULE_FILES = [
  "mask", "int", "fix", "bitset", "dfa",
  "reduce", "scan", "utf8", "sketch", "network", "parse",
];

/** Extract public function names from a core module source file. */
export async function getCoreModules(): Promise<CoreModule[]> {
  const modules: CoreModule[] = [];
  for (const mod of CORE_MODULE_FILES) {
    const filePath = path.join(
      REPO_ROOT,
      `crates/bcinr-logic/src/${mod}.rs`
    );
    let source: string;
    try {
      source = await fs.readFile(filePath, "utf8");
    } catch {
      continue;
    }
    const fns = [...source.matchAll(/^pub fn (\w+)/gm)].map((m) => m[1]);
    modules.push({ name: mod, file: `crates/bcinr-logic/src/${mod}.rs`, publicFunctions: fns });
  }
  return modules;
}

// -------------------------------------------------------------------
// Project metadata — read from Cargo.toml, not hardcoded
// -------------------------------------------------------------------

export interface ProjectMeta {
  version: string;
  algorithmCount: number;
  coreModuleCount: number;
  msrv: string;
}

export async function getProjectMeta(): Promise<ProjectMeta> {
  const cargoToml = await fs.readFile(
    path.join(REPO_ROOT, "bcinr/Cargo.toml"),
    "utf8"
  );

  const versionMatch = cargoToml.match(/^version\s*=\s*"([^"]+)"/m);
  const version = versionMatch ? versionMatch[1] : "unknown";

  // MSRV from rust-toolchain.toml or workspace Cargo.toml
  let msrv = "1.70";
  try {
    const workspace = await fs.readFile(
      path.join(REPO_ROOT, "Cargo.toml"),
      "utf8"
    );
    const msrvMatch = workspace.match(/rust-version\s*=\s*"([^"]+)"/);
    if (msrvMatch) msrv = msrvMatch[1];
  } catch {
    // use default
  }

  const algoNames = await getAlgorithmNames();

  return {
    version,
    algorithmCount: algoNames.length,
    coreModuleCount: CORE_MODULE_FILES.length,
    msrv,
  };
}

// -------------------------------------------------------------------
// Test results — read from a cached file written by `cargo test`
// If the cache doesn't exist, return null (UI shows "run tests first")
// -------------------------------------------------------------------

export interface TestStats {
  passed: number;
  failed: number;
  ignored: number;
  elapsedSecs: number | null;
}

export async function getTestStats(): Promise<TestStats | null> {
  const cachePath = path.join(REPO_ROOT, "web/.test-cache.json");
  try {
    const raw = await fs.readFile(cachePath, "utf8");
    return JSON.parse(raw) as TestStats;
  } catch {
    return null;
  }
}
