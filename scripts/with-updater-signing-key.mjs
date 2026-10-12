#!/usr/bin/env node
/**
 * Load the Tauri updater private key from 1Password into the child process
 * environment only (never writes a key file). Usage:
 *
 *   node scripts/with-updater-signing-key.mjs <command> [args...]
 *   npm run with-signing-key -- npm run tauri -- build
 *
 * Override: KTXFORGE_UPDATER_KEY_OP_REF=op://Vault/Item/field
 */
import { spawnSync } from "node:child_process";

const DEFAULT_REF =
  "op://Tech/KTXForge Tauri updater signing key/notesPlain";

function fail(message) {
  console.error(message);
  process.exit(1);
}

const command = process.argv.slice(2);
if (command.length === 0) {
  fail(
    "Usage: node scripts/with-updater-signing-key.mjs <command> [args...]\n" +
      "Example: npm run with-signing-key -- npm run tauri -- build",
  );
}

const ref = process.env.KTXFORGE_UPDATER_KEY_OP_REF ?? DEFAULT_REF;

const op = spawnSync("op", ["read", ref], {
  encoding: "utf8",
  stdio: ["ignore", "pipe", "pipe"],
});

if (op.error) {
  fail(
    `Could not run \`op\` (${op.error.message}). Install the 1Password CLI and sign in. See docs/updater-signing.md.`,
  );
}

if (op.status !== 0) {
  const detail = (op.stderr ?? op.stdout ?? "").trim();
  fail(
    `\`op read\` failed for ${ref}${detail ? `: ${detail}` : ""}.\n` +
      "Confirm Tech vault access and CLI integration. See docs/updater-signing.md.",
  );
}

const key = op.stdout.trim();
if (key.length === 0) {
  fail(`Empty secret at ${ref}.`);
}

const child = spawnSync(command[0], command.slice(1), {
  stdio: "inherit",
  env: {
    ...process.env,
    TAURI_SIGNING_PRIVATE_KEY: key,
  },
  shell: process.platform === "win32",
});

if (child.error) {
  fail(child.error.message);
}

process.exit(child.status ?? 1);
