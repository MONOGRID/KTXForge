import { execSync } from "node:child_process";

// Drop husky's hooksPath so Git LFS can install into .git/hooks.
try {
  execSync("git config --local --unset-all core.hooksPath", { stdio: "ignore" });
} catch {
  // hooksPath was not set
}

execSync("git lfs install --local", { stdio: "inherit" });
