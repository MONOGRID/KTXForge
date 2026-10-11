#!/usr/bin/env node
/**
 * Append semantic-release job outputs for GitHub Actions (T6.2 / T6.4).
 * No-op when GITHUB_OUTPUT is unset (local dry-runs).
 */
import "conventional-changelog-conventionalcommits";
import { appendFileSync } from "node:fs";

const out = process.env.GITHUB_OUTPUT;
if (!out) {
  process.exit(0);
}

const version = process.argv[2];
const tag = process.argv[3];
if (!version || !tag) {
  console.error("Usage: node scripts/release-outputs.mjs <version> <tag>");
  process.exit(1);
}

appendFileSync(
  out,
  `published=true\nversion=${version}\ntag=${tag}\n`,
);
