#!/usr/bin/env node
/**
 * Ensure Rust toolchain pieces required by `npm run check` / local Tauri builds.
 * Idempotent. No npm dependencies.
 *
 * Installs (when missing):
 * - rustfmt, clippy (via rustup)
 * - cargo-machete (via cargo install --locked)
 */
import { spawnSync } from "node:child_process";

function run(cmd, args, opts = {}) {
  const r = spawnSync(cmd, args, {
    encoding: "utf8",
    stdio: opts.stdio ?? "inherit",
    ...opts,
  });
  if (r.error) {
    throw r.error;
  }
  if (r.status !== 0) {
    throw new Error(`${cmd} ${args.join(" ")} exited ${String(r.status)}`);
  }
  return r;
}

function hasCmd(name) {
  const r = spawnSync(name, ["--version"], {
    encoding: "utf8",
    stdio: "pipe",
  });
  return r.status === 0;
}

function ensureRustupComponent(name) {
  const listed = spawnSync("rustup", ["component", "list", "--installed"], {
    encoding: "utf8",
    stdio: "pipe",
  });
  if (listed.status !== 0) {
    throw new Error(
      `rustup component list failed: ${listed.stderr || listed.stdout}`,
    );
  }
  const installed = listed.stdout || "";
  if (installed.split("\n").some((line) => line.startsWith(`${name}-`))) {
    return;
  }
  console.log(`installing rustup component ${name}`);
  run("rustup", ["component", "add", name]);
}

function main() {
  if (!hasCmd("rustc") || !hasCmd("cargo")) {
    throw new Error(
      "rustc/cargo not found. Install Rust from https://rustup.rs then re-run.",
    );
  }
  if (!hasCmd("rustup")) {
    throw new Error(
      "rustup not found. Install Rust from https://rustup.rs then re-run.",
    );
  }

  ensureRustupComponent("rustfmt");
  ensureRustupComponent("clippy");

  if (!hasCmd("cargo-machete")) {
    console.log("installing cargo-machete");
    run("cargo", ["install", "cargo-machete", "--locked"]);
  } else {
    console.log("cargo-machete ok");
  }

  console.log("dev tools ready");
}

try {
  main();
} catch (err) {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
}
