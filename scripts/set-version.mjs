#!/usr/bin/env node
/**
 * Bump the Cargo package version and refresh Cargo.lock for that crate only.
 * Used by semantic-release via @semantic-release/exec.
 */
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const version = process.argv[2];
if (!version || !/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(version)) {
  console.error("Usage: node scripts/set-version.mjs <semver>");
  process.exit(1);
}

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const cargoTomlPath = join(root, "src-tauri", "Cargo.toml");
const original = readFileSync(cargoTomlPath, "utf8");

// First [package] table only (stop at the next [section], not at EOL — CRLF broke \s*$).
const packageSection =
  /^\[package\][^\n]*\r?\n([\s\S]*?)(?=\r?\n\[)/m;
const match = packageSection.exec(original);
if (!match) {
  console.error("Could not find [package] section in src-tauri/Cargo.toml");
  process.exit(1);
}

const packageBody = match[1];
if (!/^version\s*=\s*"[^"]*"\s*$/m.test(packageBody)) {
  console.error('Could not find version = "..." line inside [package]');
  process.exit(1);
}

const updatedBody = packageBody.replace(
  /^version\s*=\s*"[^"]*"\s*$/m,
  `version = "${version}"`,
);
const updated = `${original.slice(0, match.index)}[package]\n${updatedBody}${original.slice(match.index + match[0].length)}`;
writeFileSync(cargoTomlPath, updated);

const cargo = spawnSync(
  "cargo",
  [
    "update",
    "--workspace",
    "--offline",
    "--manifest-path",
    join(root, "src-tauri", "Cargo.toml"),
  ],
  { stdio: "inherit" },
);

if (cargo.status !== 0) {
  process.exit(cargo.status ?? 1);
}
