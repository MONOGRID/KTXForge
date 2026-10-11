#!/usr/bin/env node
/**
 * Download, verify, extract, and stage the pinned KTX-Software `ktx` sidecar.
 * No npm dependencies. See docs/ktx-cli/staging.md and ktx-version.json.
 */
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import {
  copyFileSync,
  createWriteStream,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  renameSync,
  rmSync,
  chmodSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { pipeline } from "node:stream/promises";
import { Readable } from "node:stream";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dirname, "..");
const versionPath = join(root, "ktx-version.json");
const cacheDir = join(root, "src-tauri", "target", "ktx-cache");
const binariesDir = join(root, "src-tauri", "binaries");

function parseArgs(argv) {
  const idx = argv.indexOf("--target");
  return { target: idx >= 0 ? argv[idx + 1] : null };
}

function hostTriple() {
  const r = spawnSync("rustc", ["-vV"], { encoding: "utf8" });
  if (r.status !== 0) {
    throw new Error(`rustc -vV failed: ${r.stderr || r.stdout}`);
  }
  const line = (r.stdout || "").split("\n").find((l) => l.startsWith("host:"));
  if (!line) {
    throw new Error("could not parse host triple from rustc -vV");
  }
  return line.slice("host:".length).trim();
}

function sha1File(path) {
  return createHash("sha1").update(readFileSync(path)).digest("hex").toUpperCase();
}

function run(cmd, args) {
  const r = spawnSync(cmd, args, {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  if (r.status !== 0) {
    throw new Error(
      `${cmd} ${args.join(" ")} failed (${r.status}): ${r.stderr || r.stdout}`,
    );
  }
  return r;
}

function find7z() {
  const names = ["7z", "7za", "7z.exe"];
  const found = names.find((name) => {
    const r = spawnSync(name, ["--help"], { encoding: "utf8" });
    return r.status === 0 || `${r.stdout || ""}${r.stderr || ""}`.includes("7-Zip");
  });
  if (!found) {
    throw new Error("7z not found on PATH (needed for Windows NSIS / pkg extract)");
  }
  return found;
}

async function download(url, dest) {
  const res = await fetch(url);
  if (!res.ok || !res.body) {
    throw new Error(`download failed ${res.status}: ${url}`);
  }
  await pipeline(Readable.fromWeb(res.body), createWriteStream(dest));
}

function stagedKtxPath(triple, isWindows) {
  return join(binariesDir, isWindows ? `ktx-${triple}.exe` : `ktx-${triple}`);
}

function extractArchive(extract, assetPath, outDir) {
  mkdirSync(outDir, { recursive: true });
  if (extract === "tarbz2") {
    run("tar", ["-xjf", assetPath, "-C", outDir]);
    return;
  }
  if (extract === "nsis7z" || extract === "pkg") {
    run(find7z(), ["x", assetPath, `-o${outDir}`, "-y"]);
    return;
  }
  throw new Error(`unknown extract mode: ${extract}`);
}

function walkFiles(rootDir, onFile) {
  const stack = [rootDir];
  while (stack.length > 0) {
    const dir = stack.pop();
    let entries;
    try {
      entries = readdirSync(dir, { withFileTypes: true });
    } catch {
      continue;
    }
    for (const ent of entries) {
      const full = join(dir, ent.name);
      if (ent.isDirectory()) {
        stack.push(full);
      } else if (ent.isFile()) {
        onFile(full, ent.name);
      }
    }
  }
}

function findInTree(rootDir, relativeHint) {
  const direct = join(rootDir, relativeHint);
  if (existsSync(direct)) {
    return direct;
  }
  const base = relativeHint.split(/[/\\]/).pop();
  let found = null;
  walkFiles(rootDir, (full, name) => {
    if (!found && name === base) {
      found = full;
    }
  });
  if (!found) {
    throw new Error(`could not find ${relativeHint} under ${rootDir}`);
  }
  return found;
}

function alreadyStaged(outKtx, version) {
  if (!existsSync(outKtx)) {
    return false;
  }
  const ver = spawnSync(outKtx, ["--version"], { encoding: "utf8" });
  const reported = `${ver.stdout || ""}${ver.stderr || ""}`;
  return ver.status === 0 && reported.includes(`v${version}`);
}

async function ensureAssetCached(cfg, version) {
  const assetPath = join(cacheDir, cfg.asset);
  const expected = cfg.sha1.toUpperCase();
  if (existsSync(assetPath) && sha1File(assetPath) === expected) {
    console.log(`cache hit: ${cfg.asset}`);
    return assetPath;
  }
  const url = `https://github.com/KhronosGroup/KTX-Software/releases/download/v${version}/${cfg.asset}`;
  console.log(`downloading ${url}`);
  const tmp = `${assetPath}.partial`;
  await download(url, tmp);
  const got = sha1File(tmp);
  if (got !== expected) {
    rmSync(tmp, { force: true });
    throw new Error(`SHA-1 mismatch for ${cfg.asset}: expected ${expected}, got ${got}`);
  }
  renameSync(tmp, assetPath);
  return assetPath;
}

function copyExecutable(src, dest, isWindows) {
  copyFileSync(src, dest);
  if (!isWindows) {
    chmodSync(dest, 0o755);
  }
}

function stageLibs(extractDir, libs, isWindows) {
  for (const libRel of libs) {
    const libSrc = findInTree(extractDir, libRel);
    const libName = libRel.split(/[/\\]/).pop();
    copyExecutable(libSrc, join(binariesDir, libName), isWindows);
  }
}

function maybePatchelf(outKtx, enabled) {
  if (!enabled) {
    return;
  }
  const r = spawnSync("patchelf", ["--set-rpath", "$ORIGIN", outKtx], {
    encoding: "utf8",
  });
  if (r.status !== 0) {
    console.warn(
      `patchelf skipped or failed (ok if binary already has $ORIGIN): ${r.stderr || r.stdout}`,
    );
  }
}

function verifyStaged(outKtx) {
  const ver = spawnSync(outKtx, ["--version"], { encoding: "utf8" });
  if (ver.status !== 0) {
    throw new Error(`staged ktx failed --version: ${ver.stderr || ver.stdout}`);
  }
  console.log(`staged ${outKtx}`);
  console.log((ver.stdout || ver.stderr || "").trim());
}

function logAlreadyStaged(outKtx) {
  const ver = spawnSync(outKtx, ["--version"], { encoding: "utf8" });
  console.log(
    `already staged: ${outKtx} (${`${ver.stdout || ""}${ver.stderr || ""}`.trim()})`,
  );
}

async function extractAndStage(triple, cfg, version, outKtx, isWindows) {
  const assetPath = await ensureAssetCached(cfg, version);
  const extractDir = join(cacheDir, `extract-${triple}`);
  rmSync(extractDir, { recursive: true, force: true });
  extractArchive(cfg.extract, assetPath, extractDir);

  copyExecutable(findInTree(extractDir, cfg.ktxPath), outKtx, isWindows);
  stageLibs(extractDir, cfg.libs || [], isWindows);
  maybePatchelf(outKtx, Boolean(cfg.patchelf));
  verifyStaged(outKtx);
}

async function stageTarget(triple) {
  const manifest = JSON.parse(readFileSync(versionPath, "utf8"));
  const cfg = manifest.targets[triple];
  if (!cfg) {
    throw new Error(`no target entry for ${triple} in ktx-version.json`);
  }

  const isWindows = triple.includes("windows");
  const outKtx = stagedKtxPath(triple, isWindows);
  mkdirSync(binariesDir, { recursive: true });
  mkdirSync(cacheDir, { recursive: true });

  if (alreadyStaged(outKtx, manifest.version)) {
    logAlreadyStaged(outKtx);
    return;
  }

  await extractAndStage(triple, cfg, manifest.version, outKtx, isWindows);
}

async function main() {
  const { target: targetArg } = parseArgs(process.argv.slice(2));
  await stageTarget(targetArg || hostTriple());
}

main().catch((err) => {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});
