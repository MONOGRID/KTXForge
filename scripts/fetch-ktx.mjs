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
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { pipeline } from "node:stream/promises";
import { Readable } from "node:stream";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dirname, "..");
const versionPath = join(root, "ktx-version.json");
const cacheDir = join(root, "src-tauri", "target", "ktx-cache");
const binariesDir = join(root, "src-tauri", "binaries");
const tauriConfPath = join(root, "src-tauri", "tauri.conf.json");

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

/**
 * Sidecars listed in `bundle.externalBin`. `ktx2ktx2` (legacy KTX1 → KTX2) ships
 * in the same release directory as `ktx`, so its archive path follows `ktxPath`.
 */
const SIDECARS = ["ktx", "ktx2ktx2"];

function stagedToolPath(tool, triple, isWindows) {
  return join(binariesDir, isWindows ? `${tool}-${triple}.exe` : `${tool}-${triple}`);
}

function archiveToolPath(ktxPath, tool) {
  return ktxPath.replace(/ktx(\.exe)?$/, `${tool}$1`);
}

function extractArchive(extract, assetPath, outDir) {
  if (extract === "pkg") {
    // 7z only unwraps the xar. Each component Payload stays gzip+cpio,
    // so usr/local/bin/ktx never appears as a file.
    rmSync(outDir, { recursive: true, force: true });
    run("pkgutil", ["--expand-full", assetPath, outDir]);
    return;
  }
  mkdirSync(outDir, { recursive: true });
  if (extract === "tarbz2") {
    run("tar", ["-xjf", assetPath, "-C", outDir]);
    return;
  }
  if (extract === "nsis7z") {
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
      } else if (ent.isFile() || ent.isSymbolicLink()) {
        // soname links (libktx.so.4 → libktx.so.4.4.2) are not regular files
        onFile(full, ent.name);
      }
    }
  }
}

function findInTree(rootDir, relativeHint) {
  const parts = relativeHint.split(/[/\\]/).filter(Boolean);
  const direct = join(rootDir, ...parts);
  if (existsSync(direct)) {
    return direct;
  }
  // Tarballs and expanded pkgs add a prefix (KTX-Software-…/ or Payload/).
  const suffix = parts.join("/");
  let found = null;
  walkFiles(rootDir, (full) => {
    if (found) {
      return;
    }
    const rel = relative(rootDir, full).split(/[/\\]/).join("/");
    if (rel === suffix || rel.endsWith(`/${suffix}`)) {
      found = full;
    }
  });
  if (!found) {
    throw new Error(`could not find ${relativeHint} under ${rootDir}`);
  }
  return found;
}

function reportedVersion(bin) {
  const ver = spawnSync(bin, ["--version"], { encoding: "utf8" });
  return { ok: ver.status === 0, text: `${ver.stdout || ""}${ver.stderr || ""}`.trim() };
}

function libName(libRel) {
  return libRel.split(/[/\\]/).pop();
}

function alreadyStaged(outPaths, libs, version) {
  if (!libs.every((libRel) => existsSync(join(binariesDir, libName(libRel))))) {
    return false;
  }
  return outPaths.every((bin) => {
    if (!existsSync(bin)) {
      return false;
    }
    const { ok, text } = reportedVersion(bin);
    return ok && text.includes(`v${version}`);
  });
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
    copyExecutable(libSrc, join(binariesDir, libName(libRel)), isWindows);
  }
}

/**
 * Where the installed app puts the shared library, relative to the sidecar.
 * Linux: `bundle.resources` lands in `/usr/lib/<productName>/` (deb, rpm,
 * AppImage) while sidecars sit in `/usr/bin/`; `$ORIGIN` covers `binaries/`
 * and `target/<profile>/`. macOS: `bundle.macOS.frameworks` lands in
 * `Contents/Frameworks` (and `target/Frameworks` for dev); the release binary
 * already has `@executable_path`. Windows loads `ktx.dll` from the exe
 * directory, where `bundle.resources` puts it.
 */
function loaderRpaths(triple) {
  if (triple.includes("linux")) {
    const conf = JSON.parse(readFileSync(tauriConfPath, "utf8"));
    return ["$ORIGIN", `$ORIGIN/../lib/${conf.productName}`];
  }
  if (triple.includes("apple-darwin")) {
    return ["@executable_path", "@executable_path/../Frameworks"];
  }
  return [];
}

function patchelfRpath(bin, rpaths) {
  const want = rpaths.join(":");
  const r = spawnSync("patchelf", ["--print-rpath", bin], { encoding: "utf8" });
  if (r.error) {
    throw new Error("patchelf not found on PATH (needed to set the sidecar rpath)");
  }
  if ((r.stdout || "").trim() === want) {
    return false;
  }
  run("patchelf", ["--set-rpath", want, bin]);
  return true;
}

function machoRpath(bin, rpaths) {
  const loadCommands = run("otool", ["-l", bin]).stdout;
  const missing = rpaths.filter((p) => !loadCommands.includes(` path ${p} `));
  if (missing.length === 0) {
    return false;
  }
  for (const p of missing) {
    run("install_name_tool", ["-add_rpath", p, bin]);
  }
  // install_name_tool invalidates the signature; arm64 refuses unsigned code.
  run("codesign", ["--force", "--sign", "-", bin]);
  return true;
}

/** Point the sidecar at the bundled library. Idempotent; true when it changed `bin`. */
function fixLoaderPath(bin, triple) {
  const rpaths = loaderRpaths(triple);
  if (rpaths.length === 0) {
    return false;
  }
  return triple.includes("linux") ? patchelfRpath(bin, rpaths) : machoRpath(bin, rpaths);
}

function verifyStaged(bin) {
  const { ok, text } = reportedVersion(bin);
  if (!ok) {
    throw new Error(`staged ${bin} failed --version: ${text}`);
  }
  console.log(`staged ${bin}`);
  console.log(text);
}

async function extractAndStage(triple, cfg, version, isWindows) {
  const assetPath = await ensureAssetCached(cfg, version);
  const extractDir = join(cacheDir, `extract-${triple}`);
  rmSync(extractDir, { recursive: true, force: true });
  extractArchive(cfg.extract, assetPath, extractDir);

  stageLibs(extractDir, cfg.libs || [], isWindows);
  for (const tool of SIDECARS) {
    const out = stagedToolPath(tool, triple, isWindows);
    copyExecutable(findInTree(extractDir, archiveToolPath(cfg.ktxPath, tool)), out, isWindows);
    fixLoaderPath(out, triple);
    verifyStaged(out);
  }
}

/** Binaries staged before a loader-path change keep the old rpath. */
function refreshStaged(outPaths, triple) {
  for (const bin of outPaths) {
    if (fixLoaderPath(bin, triple)) {
      verifyStaged(bin);
    }
    console.log(`already staged: ${bin} (${reportedVersion(bin).text})`);
  }
}

async function stageTarget(triple) {
  const manifest = JSON.parse(readFileSync(versionPath, "utf8"));
  const cfg = manifest.targets[triple];
  if (!cfg) {
    throw new Error(`no target entry for ${triple} in ktx-version.json`);
  }

  const isWindows = triple.includes("windows");
  const outPaths = SIDECARS.map((tool) => stagedToolPath(tool, triple, isWindows));
  mkdirSync(binariesDir, { recursive: true });
  mkdirSync(cacheDir, { recursive: true });

  if (alreadyStaged(outPaths, cfg.libs || [], manifest.version)) {
    refreshStaged(outPaths, triple);
    return;
  }

  await extractAndStage(triple, cfg, manifest.version, isWindows);
}

async function main() {
  const { target: targetArg } = parseArgs(process.argv.slice(2));
  await stageTarget(targetArg || hostTriple());
}

main().catch((err) => {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});
