# Development plan

Build plan for the KTX encoder and inspector in [requirements.md](requirements.md). The work is cut into waves of small tasks that low-context sub-agents can run without judgment calls. Each task names the files it owns, what to build, and the commands that prove it is done.

Read [AGENTS.md](../AGENTS.md) and [decision-context.md](decision-context.md) first. This plan does not override them.

## How to run this plan

1. Run waves in order. A wave starts only when every task in the previous wave passes its checks and is merged.
2. Tasks inside one wave run in parallel. They own disjoint files, so they cannot conflict.
3. Each sub-agent works in its own git worktree, branched from the integration branch at the start of the wave (`isolation: "worktree"` on the Agent tool). It runs `npm ci` there, does the task, runs `npm run check`, and commits on its branch.
4. The orchestrator (a capable agent or a human) reviews each branch, merges it into the integration branch, and runs `npm run check` again on the merged result. That is the wave gate.
5. A task that cannot finish without editing a file it does not own stops and reports. The orchestrator fixes the contract or reassigns the file. Sub-agents never fix someone else's file.

### Branches and commits

- Development happens on the integration branch `dev`. `main` is the release branch: every push to `main` can publish a release (see Wave 6). `dev` merges into `main` only when the orchestrator decides to ship, the first time after Wave 7.
- The local default branch is currently `master`. Rename it to `dev` and create `main` from it before the first push to `https://github.com/MONOGRID/KTX-GUI`.
- Every commit follows [Conventional Commits](https://www.conventionalcommits.org/). The commit message drives the version bump and the changelog, so it matters. Task commits use `<type>(<scope>): <summary> [<task ID>]`, for example `feat(encoder): option rules engine [T3.3]`.

| Wave | Default type |
| --- | --- |
| 0 | `docs` or `test` |
| 1 | `build` |
| 2–5 | `feat` (`build` for tooling-only tasks) |
| 6 | `ci` |
| 7 | `docs` or `test` |

A change that breaks a previously released behavior or file format adds `!` after the type and a `BREAKING CHANGE:` footer. Before the first release there is nothing to break, so no task in this plan uses it.

### Sub-agent prompt template

```text
You are implementing task <ID> from docs/development-plan.md in C:\Users\danie\Desktop\tauri-test.

Before writing code, read:
- AGENTS.md
- docs/development-plan.md: the "Global rules" section and task <ID>
- every file listed under "Read" in task <ID>

Rules:
- Create or edit ONLY the files listed under "Owns" in task <ID>.
- If you need a change in any other file, stop and report what you need and why. Do not make it.
- Do not add dependencies that are not listed in the task.
- Do not invent CLI flags, option values, or JSON fields. Take them from docs/requirements.md or docs/ktx-cli/.
- Fix every finding from lint, typecheck, clippy, and fallow in the code. Do not disable a rule, add a suppression, raise a threshold, or edit .fallowrc.json to get a green run. The only allowed suppression is the "written ahead of its consumer" marker described in Global rules.
- Finish by running every command under "Done when", then `npm run check`, and paste the output.
- Commit on your branch with a Conventional Commit message: <type>(<scope>): <summary> [<ID>].

Report: files changed, command output, anything you could not do.
```

### Quality gate: `npm run check`

One command runs every check, in the same order locally, in each sub-agent's worktree, at the wave gate, and in CI. T1.1 adds these scripts to `package.json`; until T1.1 merges, run the pieces by hand.

| Script | Runs | Catches |
| --- | --- | --- |
| `check:rust` | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo machete` (all with `--manifest-path src-tauri/Cargo.toml`) | Rust format, lints, dead code (`dead_code` is denied by `-D warnings`), failing tests, unused crates |
| `check:web` | `npm run build` (vue-tsc + vite), `npm run lint`, `npm run lint:style` | Type errors, ESLint, Stylelint |
| `check:fallow` | `fallow dead-code --fail-on-issues`, `fallow dupes --threshold 3`, `fallow health --min-score 85 --min-severity high` | Unused files, exports, types, dependencies, Vue props and emits, stale suppressions; duplicated code above 3% of the codebase; health score below 85 or any high-severity complexity finding |
| `check` | `check:rust`, then `check:web`, then `check:fallow`, stopping at the first failure | Everything above |

On top of `npm run check`, the project's Claude Code hook (`.claude/hooks/fallow-gate.sh`, installed by `fallow setup-hooks`) runs `fallow audit` before every `git commit` and `git push` an agent issues, and blocks it on a failing verdict. The hook needs bash and `jq`; without `jq` it skips with a notice (fails open), so `npm run check` stays the gate of record.

Fallow analyzes TypeScript, JavaScript, Vue, and CSS only. Rust dead code is covered by clippy with `-D warnings`, and unused Rust dependencies by `cargo machete`.

CI additionally checks that `src/bindings/` matches the Rust types (`git diff --exit-code src/bindings` after `cargo test`). Generated bindings are committed.

Thresholds (3% duplication, score 85, severity `high`) are set once in T1.1 and change only through a dated entry in [decision-context.md](decision-context.md).

## Global rules

Every sub-agent follows these. They restate AGENTS.md and add the conventions this plan relies on.

- **Rust does the work.** Anything that encodes, decodes, parses, converts, builds a command line, or touches the filesystem is Rust. Vue renders and calls commands.
- **Command bodies are thin.** A file under `src-tauri/src/commands/` validates input, calls engine functions, and maps errors. Logic lives in engine modules with unit tests.
- **Errors.** Every command returns `Result<T, AppError>` (`src-tauri/src/error.rs`). Never `unwrap()` or `expect()` outside tests and `main`.
- **Shared types.** Every struct or enum that crosses IPC derives `serde::Serialize`/`Deserialize`, `ts_rs::TS`, and has `#[ts(export)]` and `#[serde(rename_all = "camelCase")]`. Generated files land in `src/bindings/`. Never edit `src/bindings/` by hand.
- **CLI strings.** Enum variants that map to a `ktx` value use that exact string as the serde name, and expose `fn cli(&self) -> &'static str` returning the same string. A unit test checks the two agree for every variant (`strum::EnumIter`).
- **Frontend calls.** Vue calls Rust only through `src/api/commands.ts` and listens only through `src/api/events.ts`. No direct `invoke` or `listen` elsewhere.
- **TypeScript.** `strict`, no `any`, no casts, no `@ts-ignore`. Prefer `satisfies`.
- **Styles.** UnoCSS utilities and the shortcuts in `uno.config.ts`. SCSS only for what utilities cannot express, local to the component.
- **Reuse.** VueUse before a hand-written composable.
- **Tests.** Rust engine code has unit tests next to it (`#[cfg(test)] mod tests`). Tests that need the real `ktx` binary are marked `#[ignore]` and run in CI with `cargo test -- --include-ignored` after `npm run fetch-ktx`.
- **Fixtures** live in `src-tauri/tests/fixtures/` and are read-only for every task except T0.3.
- **Code written ahead of its consumer.** Scaffold and Wave 2 tasks create TypeScript exports and Vue components that nothing uses until Wave 5, which fallow reports as unused. Mark each export with a `/** @expected-unused consumed by T5.1 */` JSDoc tag, and each component file nothing imports yet with `// fallow-ignore-file unused-file` plus a `// consumed by T5.4` comment on the line above, in its `<script setup>`. The consuming task deletes the marker. Both forms are tracked by fallow 3.33 (`fallow explain stale-suppression`). Fallow reports a marker that no longer matches a finding as a stale suppression, so a forgotten marker fails `check:fallow`. Wave 7 requires zero markers left (`fallow suppressions` lists them).
- **Duplication.** When `fallow dupes` flags repeated blocks, extract a shared component, composable, or Rust helper. In the option sections (T5.2, T5.3), repeated field markup means `OptionField` should be driven by data, not copied.

## Target layout

Created as stubs in Wave 1, filled in later waves. Each later task owns a subset.

```text
scripts/
  fetch-ktx.mjs              download, verify, extract, stage pinned ktx
ktx-version.json             pinned version, asset names, sha1 per asset
docs/ktx-cli/                captured `ktx <cmd> --help` and sample outputs
src-tauri/
  .cargo/config.toml         TS_RS_EXPORT_DIR = ../src/bindings
  binaries/                  staged ktx-<triple>[.exe] + shared libs (gitignored)
  src/
    lib.rs                   builder, plugins, command registration, state
    menu.rs                  existing menu code, moved out of lib.rs
    error.rs                 AppError
    events.rs                event names and payloads
    state.rs                 AppState (jobs, inspector file, workspace)
    workspace.rs             temp dirs per job
    ktx/
      mod.rs
      options.rs             CreateOptions and every option enum
      rules.rs               OptionReport: enabled/disabled + reasons, warnings
      create_args.rs         CreateOptions -> Vec<String>
      extract_args.rs        ExtractSelection -> Vec<String>, output naming
      sidecar.rs             run ktx, stream stderr, exit codes, cancel
      info_types.rs          types for `ktx info --format json` (info_v1 schema)
      info.rs                run info, parse JSON, KTX1 text fallback
      ktx1.rs                KTX1 header reader
      anim.rs                write KTXanimData into a KTX2 file
      metrics.rs             parse SSIM/PSNR from create stdout
    image/
      mod.rs
      classify.rs            path -> Image/Ktx/Unsupported, frame ordering
      convert.rs             JPEG/WebP/BMP/GIF/TIFF -> PNG, HDR -> EXR
      export.rs              PNG -> JPEG with quality, EXR -> tone-mapped PNG
    commands/
      mod.rs
      app.rs                 ktx_version
      encoder.rs             default_create_options, describe_options, pick_sources, source_thumbnail
      encode.rs              encode, cancel_encode, encoded_preview, save_encoded
      inspector_window.rs    open_inspector, inspector_current_file
      inspect.rs             inspect, inspector_image
      export.rs              export_images
      update.rs              check_update, install_update
    drop.rs                  window drag-drop handler
    updater.rs               launch-time update check
src/
  bindings/                  generated by ts-rs (do not edit)
  api/commands.ts            typed invoke wrapper
  api/events.ts              typed listen wrapper
  shared/components/         Button, Panel, OptionField, ErrorBox, ProgressBar, UpdateBanner
  shared/composables/
  encoder/                   main window (index.html -> src/encoder/main.ts)
  inspector/                 inspector window (inspector.html -> src/inspector/main.ts)
```

## Decisions this plan makes

These are not in the decision log yet. The orchestrator appends them to [decision-context.md](decision-context.md) when the user approves the plan (task T1.1 does it).

| Decision | Why |
| --- | --- |
| Pin KTX-Software **v4.4.2** (latest stable). 5.0.0 is still `rc2` (2026-08-17). | Releases ship pinned stable tools. T0.1 checks whether 4.4.2 has every flag in the spec; gaps go to the user. |
| Rust → TypeScript types through **`ts-rs`**, generated into `src/bindings/`. | AGENTS.md asks for explicit shared payload types. Generation keeps both sides in sync without hand copies. `ts-rs` is stable; `tauri-specta` v2 is still a release candidate. |
| The **option rules engine lives in Rust** (`describe_options`). Vue only shows `enabled` and `reason`. | Option conflicts are tool knowledge, not rendering. Keeping them in Rust next to the args builder means one source of truth, testable with `cargo test`. |
| **Previews travel as PNG bytes** via `tauri::ipc::Response`. Vue turns them into a Blob URL. | No asset-protocol scope, no temp paths in the webview. |
| **Save/open dialogs open from Rust** (`tauri-plugin-dialog` Rust API). | The webview never needs filesystem permissions. |
| **Two Vite entry points**: `index.html` (encoder) and `inspector.html` (inspector). | Each window loads only its own code. The inspector window points at `inspector.html`. |
| Image conversion with the **`image` crate** (features `png`, `jpeg`, `webp`, `bmp`, `gif`, `tiff`, `exr`, `hdr`). Temp dirs with **`tempfile`**. KTX2 reading with **`ktx2`**. | Maintained, pure Rust, cover every conversion the spec needs. |
| Rust unused dependencies checked with **`cargo machete`** in `npm run check`. | Fallow covers only the frontend; clippy covers Rust dead code but not unused crates. |
| Conventional Commits enforced locally with **`husky`** + **`@commitlint/cli`** (`@commitlint/config-conventional`). | semantic-release reads commit messages; a malformed message silently skips a release or a changelog entry. |

## Settled with the user (2026-10-10)

Logged in [decision-context.md](decision-context.md).

| Topic | Answer |
| --- | --- |
| `ktx` version | v4.4.2. If T0.1 finds a spec flag missing, the orchestrator asks the user whether to drop that option for now or move to 5.0. |
| Repository | `https://github.com/MONOGRID/KTX-GUI` |
| Product | name `KTX-GUI`, identifier `com.monogrid.ktx-gui` |
| Updater signing key | The user generates it (`npx tauri signer generate`), stores the private key and password as GitHub secrets, and provides the public key for T6.3. |
| Quality | fallow dead code, duplication, and health run in `npm run check` next to lint and typecheck, at every task and wave gate and in CI. |
| Releases | semantic-release on `main`: version from commit messages, `CHANGELOG.md`, tag, GitHub Release, Tauri artifacts, published automatically. |

## Still open

1. **Public key** for the updater (blocks T6.3 only).
2. **Branch protection on `main`.** If `main` requires pull requests, the release bot cannot push the version commit with the default `GITHUB_TOKEN`. Either allow `github-actions[bot]` to bypass, or create a GitHub App and use its token (T6.2 supports both).

---

## Wave 0 — Spikes

Status: **complete** (2026-10-10). Gaps from T0.1 are listed in [ktx-cli/README.md](ktx-cli/README.md) and need a user decision before Wave 2 option types freeze the codec list. Wave 1 (scaffold) can proceed.

Facts later tasks depend on. Run by **one capable agent or a human**, not a low-context sub-agent: the outcome is not known in advance. Work in a scratch directory; commit only the files listed under Owns.

### T0.1 Capture the pinned CLI surface ✅

- **Owns:** `docs/ktx-cli/*.txt`, `docs/ktx-cli/README.md`
- **Do:**
  1. Download KTX-Software v4.4.2 for the local OS from the GitHub release. Extract `ktx` (Windows: `KTX-Software-4.4.2-Windows-x64.exe` is an NSIS installer; try `7z x`, otherwise install silently with `/S /D=<scratch dir>`).
  2. Save the output of `ktx --version`, `ktx --help`, and `ktx <cmd> --help` for `create`, `encode`, `extract`, `info`, `validate`, `transcode`, `convert`, `deflate` into `docs/ktx-cli/<cmd>.txt`.
  3. In `docs/ktx-cli/README.md`, compare every flag and every value list in requirements.md "Options that must appear in the UI" against `create.txt`. List: present, missing, spelled differently (the spec flags `pg_eotf` and `abobergb` as suspect).
- **Done when:** the README has the comparison table, and any missing flag is reported to the orchestrator, who takes it to the user.

### T0.2 Runtime dependencies and staging layout ✅

- **Owns:** `docs/ktx-cli/staging.md`
- **Do:** For each release target (Windows x64, macOS arm64, macOS x64, Linux x64, Linux arm64):
  1. Find which shared libraries `ktx` loads (`ldd`, `otool -L` plus `otool -l | grep -A2 LC_RPATH`, `dumpbin /dependents`). Expect `ktx.dll` / `libktx.4.dylib` / `libktx.so.4`.
  2. Decide where each library goes so the sidecar finds it after Tauri bundles it: Windows `bundle.resources` next to the exe; macOS `bundle.macOS.files` into `Contents/...` matching the rpath, or patch with `install_name_tool`; Linux `bundle.linux.deb.files` and AppImage, or `patchelf --set-rpath '$ORIGIN'`.
  3. Write the extraction command per asset type (NSIS exe, `.pkg` via `pkgutil --expand-full`, `.tar.bz2`) and the exact source paths inside each archive.
  4. Copy the license file path inside each archive.
- **Done when:** `staging.md` has one table row per target: asset name, extract command, path of `ktx`, paths of libraries, destination in the bundle, rpath fix (if any), license path. Platforms the spike could not test locally are marked "verify in CI".

### T0.3 Fixtures and sample outputs ✅

- **Owns:** `src-tauri/tests/fixtures/**`, `docs/ktx-cli/samples/**`
- **Do:** Using the v4.4.2 `ktx`, create small (≤ 64×64) fixtures and commit them with a `README.md` that lists the command used for each:
  - Sources: `rgba.png`, `rgb.jpg`, `gray.png`, `hdr.exr` (half float RGBA), `frames/frame_001.png` … `frame_004.png`, `cube/` six faces, `odd_size.png` (37×19).
  - KTX2: `basis_lz.ktx2` (with mips), `uastc_zstd.ktx2`, `astc_6x6_srgb.ktx2`, `rgba8_srgb.ktx2`, `cubemap.ktx2`, `array4_mips.ktx2`, `hdr_uastc.ktx2`, `anim.ktx2` (4 layers, KTXanimData written by hand: duration 1, timescale 10, loopCount 0).
  - KTX1: `legacy.ktx` (any valid KTX1, e.g. via `toktx` or a small known sample).
  - Broken: `truncated.ktx2` (first 200 bytes of `basis_lz.ktx2`), `not_ktx.ktx2` (text file).
  - For every `.ktx2`, save `ktx info --format json` to `docs/ktx-cli/samples/<name>.info.json`; for `legacy.ktx` save the text output.
  - Save the stdout of one `ktx create --encode basis-lz --compare-ssim --compare-psnr` run to `docs/ktx-cli/samples/create-metrics.txt`.
  - Save stderr and exit code of one failing create (bad flag) and one invalid input to `docs/ktx-cli/samples/create-errors.txt`.
- **Done when:** fixtures total under 2 MB and every listed file exists.

**Wave 0 gate:** the orchestrator reads `docs/ktx-cli/README.md` and `staging.md`, asks the user about any missing flag, and updates the task text in this plan if a flag list changed. `npm run check` does not exist yet; Wave 0 commits only docs and fixtures.

---

## Wave 1 — Scaffold

Status: **in progress** (2026-10-10). T1.1 scaffold is in place and `npm run check` passes. Remaining for the wave gate: manual `npm run tauri dev` (window title KTX-GUI) and rename local `master` → `dev` when pushing to GitHub.

**One agent, sequential.** This wave creates every file in the target layout with compiling stubs, all dependencies, and all command registrations. After it, later tasks only fill in bodies inside files they own. This is what lets Waves 2–5 run in parallel.

### T1.1 Dependencies, config, module tree, stubs ✅ (`npm run check` green; verify `tauri dev` manually)

- **Owns:** `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/.cargo/config.toml`, `src-tauri/tauri.conf.json`, `src-tauri/capabilities/*.json`, `src-tauri/src/**` (all files, stubs only), `src-tauri/.gitignore`, `package.json`, `package-lock.json`, `vite.config.ts`, `index.html`, `inspector.html`, `src/**` (stubs only), `.gitignore`, `.fallowrc.json`, `.husky/commit-msg`, `commitlint.config.js`, `AGENTS.md` (new sections only), `docs/decision-context.md` (append only)
- **Read:** this plan's "Target layout" and "Decisions this plan makes", `docs/ktx-cli/staging.md`
- **Do:**
  1. Rust dependencies: `tauri-plugin-shell`, `tauri-plugin-dialog`, `tauri-plugin-updater`, `tauri-plugin-process` (relaunch), `ts-rs` (feature `serde-compat`), `strum` (feature `derive`), `thiserror`, `image` (features listed in the decisions table, `default-features = false`), `tempfile`, `ktx2`, `uuid` (feature `v4`), `tokio` (features `sync`, `fs`, `process`). Keep the existing deps.
  2. JS dependencies: `@tauri-apps/plugin-shell` is **not** added (the frontend never runs processes). Add `@vueuse/core`. Dev dependencies: `husky`, `@commitlint/cli`, `@commitlint/config-conventional` (`fallow` is already a dev dependency; keep its `fallow` and `fallow:audit` scripts). Remove `@tauri-apps/plugin-opener` only if nothing uses it. `cargo machete` is a cargo tool, not a crate: document `cargo install cargo-machete` in the report and in T7.3's README.
  3. `.cargo/config.toml`: `[env] TS_RS_EXPORT_DIR = { value = "../src/bindings", relative = true }`.
  4. Rename the app: `package.json` name `ktx-gui`; Cargo package `ktx-gui`, lib `ktx_gui_lib` (update `main.rs`); `tauri.conf.json` `productName: "KTX-GUI"`, `identifier: "com.monogrid.ktx-gui"`, `version: "../package.json"` (Tauri reads the version from `package.json`, so semantic-release bumps one place for the app). Set `package.json` and Cargo version to `0.0.0`; semantic-release owns them from now on. Also in `tauri.conf.json`: `bundle.externalBin: ["binaries/ktx"]`, library entries from `staging.md`, main window title "KTX-GUI", min size 960×640. Do not add the updater config yet (T6.3).
  5. Capabilities: `default.json` for `main`, `inspector.json` for `inspector`. Grant only `core:default`, `core:event:default`, `core:window:default`, `window-state:default`. No `shell:*`, no `fs:*`, no `dialog:*` for the webview.
  6. Move the menu code from `lib.rs` to `menu.rs` unchanged. Remove `greet`.
  7. Create every Rust module in the layout. Each public function has its final signature and body `Err(AppError::NotImplemented("<fn name>"))` or `unimplemented!()` for pure helpers. Register every command from "Command list" below in `generate_handler!`. Register `AppState` with `.manage()`. Register `drop.rs` handler and `updater.rs` setup as no-op calls.
  8. `error.rs`: final `AppError` enum (see "Shared contracts").
  9. `events.rs`: final event name constants (see "Shared contracts"); payload structs may be empty stubs.
  10. Frontend: `index.html` → `src/encoder/main.ts` → `EncoderApp.vue`; `inspector.html` → `src/inspector/main.ts` → `InspectorApp.vue`. Each app renders its title only. Vite `build.rollupOptions.input` lists both HTML files. Delete `src/App.vue`, `src/main.ts`, and unused assets.
  11. `src/api/commands.ts` and `src/api/events.ts` with the generic wrappers described in "Shared contracts", using placeholder types until W2 fills bindings.
  12. Append the decisions table rows to `docs/decision-context.md` as one dated entry.
  13. Quality gate: add the `check:rust`, `check:web`, `check:fallow`, and `check` scripts from "Quality gate" to `package.json`. Chain with `&&` so the scripts run the same in PowerShell, cmd, and bash.
  14. `.fallowrc.json` already exists. Edit it: replace `entry` with `["src/encoder/main.ts", "src/inspector/main.ts"]` (the current `src/main.*` entry is deleted in step 10; check with `fallow list --entry-points`); add `ignorePatterns: ["src/bindings/**"]`; report stale suppressions as errors (look up the rule name in `fallow config-schema`; do not guess). Keep `duplicates.minOccurrences` and everything else as is.
  15. Mark every stub export and component that nothing consumes yet as described in Global rules ("Code written ahead of its consumer").
  16. Commit hooks: `npx husky init`, replace the generated pre-commit hook with a `commit-msg` hook running `npx --no -- commitlint --edit "$1"`; `commitlint.config.js` extends `@commitlint/config-conventional`. Add `"prepare": "husky"` (set by `husky init`).
  17. `AGENTS.md`: in the existing **Fallow** section, add a short paragraph: run `npm run check` before every commit; fallow findings (dead code, duplication, health) are fixed in code like lint errors; no suppressions, threshold changes, or `.fallowrc.json` edits except the "ahead of consumer" marker. Leave the generated task-matrix block untouched. Add a new short **Commits** section: Conventional Commits; the type decides the release (`feat` minor, `fix`/`perf` patch, `!` or `BREAKING CHANGE:` major, others no release).
- **Done when:** `npm run check` passes, `npm run tauri dev` opens the main window titled KTX-GUI, `git ls-files src-tauri/binaries` is empty (binaries are gitignored), and `git commit --allow-empty -m "bad message"` is rejected by the hook.

### Command list (fixed by T1.1)

| Command | Args | Returns | File |
| --- | --- | --- | --- |
| `ktx_version` | — | `KtxVersion` | `commands/app.rs` |
| `default_create_options` | — | `CreateOptions` | `commands/encoder.rs` |
| `describe_options` | `options: CreateOptions`, `sources: Vec<SourceInfo>` | `OptionReport` | `commands/encoder.rs` |
| `pick_sources` | — | `Vec<SourceInfo>` | `commands/encoder.rs` |
| `source_thumbnail` | `path: String` | PNG bytes (`ipc::Response`) | `commands/encoder.rs` |
| `encode` | `request: EncodeRequest` | `EncodeResult` | `commands/encode.rs` |
| `cancel_encode` | `job_id: String` | `()` | `commands/encode.rs` |
| `encoded_preview` | `job_id: String` | PNG bytes | `commands/encode.rs` |
| `save_encoded` | `job_id: String` | `Option<String>` (saved path) | `commands/encode.rs` |
| `open_inspector` | `path: String` | `()` (async) | `commands/inspector_window.rs` |
| `inspector_current_file` | — | `Option<String>` | `commands/inspector_window.rs` |
| `inspect` | `path: String` | `InspectReport` | `commands/inspect.rs` |
| `inspector_image` | `path: String`, `selection: ImageSelection` | PNG bytes | `commands/inspect.rs` |
| `export_images` | `request: ExportRequest` | `ExportResult` | `commands/export.rs` |
| `check_update` | — | `Option<UpdateInfo>` | `commands/update.rs` |
| `install_update` | — | `()` | `commands/update.rs` |

All commands that run `ktx` or touch files are `async`.

---

## Wave 2 — Contracts and foundations

Parallel. Fills in the shared types every later task depends on, plus independent infrastructure.

### Shared contracts

These shapes are fixed. Tasks fill in fields exactly as listed; extra fields need orchestrator approval.

**`AppError`** (`error.rs`), serialized as `{ kind, message, exitCode?, exitMeaning?, stderr? }`:
`NotImplemented(&'static str)`, `Io(String)`, `InvalidInput(String)`, `UnsupportedFile(String)`, `Ktx { exit_code: i32, stderr: String }` (`exitMeaning` from the exit-code table in requirements.md), `Cancelled`, `Image(String)`, `Update(String)`.

**Events** (`events.rs`):

| Name | Payload | Emitted to |
| --- | --- | --- |
| `encoder://sources-added` | `Vec<SourceInfo>` | main |
| `encoder://drop-rejected` | `{ paths: Vec<String>, reason: String }` | main |
| `encode://progress` | `{ jobId, stage: "converting" \| "encoding" \| "writing-anim" \| "previewing" \| "done", stderrLine?: String }` | window that started the job |
| `inspector://file` | `{ path: String }` | inspector |
| `update://status` | `{ state: "available" \| "downloading" \| "installed" \| "error", version?, notes?, downloaded?, total?, message? }` | main |

**`src/api/commands.ts`**: a `Commands` type mapping each command name to `{ args: ...; result: ... }` built from `src/bindings`, and one generic `call<K extends keyof Commands>(name: K, args: Commands[K]["args"]): Promise<Commands[K]["result"]>`. Byte-returning commands map to `ArrayBuffer`. Errors are narrowed to the generated `AppError` type with a type guard, not a cast.

**`src/api/events.ts`**: same idea, `on<K extends keyof Events>(name: K, handler: (payload: Events[K]) => void): Promise<UnlistenFn>`.

### T2.1 Encoding option types

- **Owns:** `src-tauri/src/ktx/options.rs`
- **Read:** requirements.md "Output the tool can produce" and "Options that must appear in the UI", `docs/ktx-cli/create.txt`, `docs/ktx-cli/README.md`
- **Do:** Define `CreateOptions` with nested groups `container`, `filter`, `color`, `basisLz`, `uastcLdr`, `uastcHdr`, `astc`, `encoder` (threads, no-sse, compare-ssim, compare-psnr), `supercompression` (`None | Zstd(u8) | Zlib(u8)`), plus top-level `format: VkFormat` and `encode: Option<EncodeCodec>`. One field per CLI flag in the spec. Flags without a value are `bool`. Optional values are `Option<T>` meaning "not passed, tool default". Enums for every value list (VkFormat incl. all 28 ASTC LDR formats, codecs, mip filters, wrap modes, transfer functions, primaries, texcoord origins, ASTC quality). Swizzle is a validated newtype over 4 chars of `rgba01`. Also define `AnimationOptions { duration: u32, timescale: u32, loop_count: u32 }`. Implement `Default` matching tool defaults (format `R8G8B8A8_SRGB`, no encode). Follow the CLI-string rule from Global rules.
- **Done when:** unit tests cover: default serializes and round-trips through serde; every enum's serde name equals `cli()`; swizzle rejects `rgbx` and `rgb`. `npm run check` passes.

### T2.2 Inspector report types

- **Owns:** `src-tauri/src/ktx/info_types.rs`
- **Read:** https://schema.khronos.org/ktx/info_v1.json, `docs/ktx-cli/samples/*.info.json`, requirements.md "Inspector"
- **Do:** Serde types for the whole info_v1 schema: validation messages, header, level index, index, data format descriptor with samples, every known key/value entry listed in requirements.md, other key/value pairs as a list of `{ key, value }` where the value is text or hex bytes, BasisLZ global data. Use `Option` for anything the schema marks optional. Define `InspectReport { path, fileSize, kind: Ktx2(Ktx2Info) | Ktx1 { text, header: Ktx1Header }, preview: PreviewKind }` and `PreviewKind = Png | TonemappedFromExr | Unavailable { reason }`. Define `ImageSelection { level, face, layer, depthSlice }` (all `u32`).
- **Done when:** a test deserializes every file in `docs/ktx-cli/samples/*.info.json` without error, and `anim.info.json` yields `duration 1, timescale 10, loopCount 0`. `npm run check` passes.

### T2.3 Job, source, and result types; error and event payloads

- **Owns:** `src-tauri/src/error.rs`, `src-tauri/src/events.rs`, `src-tauri/src/state.rs`, `src-tauri/src/ktx/mod.rs` (type re-exports only)
- **Do:** Complete `AppError` and event payloads exactly as in "Shared contracts". Add `SourceInfo { path, name, kind: Ldr | Hdr, width, height, needsConversion: bool }`, `EncodeRequest { jobId, sources: Vec<String>, options: CreateOptions, animation: Option<AnimationOptions> }`, `EncodeResult { jobId, outputBytes, width, height, levels, layers, faces, codec, ssim: Option<f64>, psnr: Option<f64>, warnings: Vec<String>, preview: PreviewKind }`, `ExportRequest { path, selection: ExportSelection (One(ImageSelection) | AllFrames { level } | All), format: Png | Jpeg { quality: u8 } | Exr }`, `ExportResult { files: Vec<String> }`, `KtxVersion { pinned, reported }`, `UpdateInfo { version, notes, date }`, `OptionReport { fields: Vec<FieldState>, warnings: Vec<OptionWarning> }`, `FieldState { key: OptionKey, enabled, reason: Option<String> }`, `OptionKey` (one variant per CLI flag, serde name = flag without `--`), `OptionWarning { key: Option<OptionKey>, level: Info | Warn, message }`. `AppState` holds a `Mutex<HashMap<String, Job>>` (job dir, child handle for cancel, output path) and `Mutex<Option<PathBuf>>` for the inspector file.
- **Done when:** `npm run check` passes and `src/bindings/` contains every type above.

### T2.4 Fetch and stage `ktx`

- **Owns:** `scripts/fetch-ktx.mjs`, `ktx-version.json`, `package.json` (`scripts` entry only), `src-tauri/licenses/KTX-Software-LICENSE.*`
- **Read:** `docs/ktx-cli/staging.md`
- **Do:** Node script with no npm dependencies. Reads `ktx-version.json` (`version`, per-target `{ asset, sha1, ktxPath, libs[] }`). Detects the target triple (`rustc -vV` host, overridable with `--target <triple>`). Downloads the asset from `https://github.com/KhronosGroup/KTX-Software/releases/download/v<version>/<asset>` into `src-tauri/target/ktx-cache/`, checks SHA-1 (fail on mismatch), extracts with the command from `staging.md`, copies `ktx` to `src-tauri/binaries/ktx-<triple>[.exe]`, copies libraries to their staging paths, applies rpath fixes from `staging.md`. Skips work when the staged file already matches. Add `"fetch-ktx": "node scripts/fetch-ktx.mjs"`. Commit the license file from the archive.
- **Done when:** on a clean checkout, `npm run fetch-ktx` then `src-tauri/binaries/ktx-<host triple> --version` prints the pinned version; running it twice does not re-download.

### T2.5 Frontend base components and theme

- **Owns:** `uno.config.ts`, `src/shared/components/*.vue`, `src/shared/styles/*`
- **Read:** requirements.md "Interface"
- **Do:** UnoCSS theme tokens and shortcuts for a compact desktop UI (dark and light via `prefers-color-scheme`): `btn`, `btn-primary`, `panel`, `field-row`, `label`, `hint`, `badge`. Components with typed props and emits: `Button.vue`, `Panel.vue` (collapsible section with title), `ProgressBar.vue` (determinate and indeterminate), `ErrorBox.vue` (shows `AppError`: message, exit code + meaning, stderr in a scrollable monospace block, copy button), `Tooltip.vue`. Also `OptionField.vue`: props `label`, `optionKey`, `state?: FieldState`, default slot for the control; renders the control dimmed and non-interactive when `state.enabled` is false, with `state.reason` as a visible hint, never hidden.
- **Done when:** `npm run check` passes. A temporary demo in `EncoderApp.vue` is **not** allowed (not owned); verify via build and lint only.

### T2.6 Typed API wrappers

- **Owns:** `src/api/commands.ts`, `src/api/events.ts`, `src/api/guards.ts`
- **Depends on:** T2.1–T2.3 types existing. Run this task **last in Wave 2** (after the others merge) or as the first task of Wave 3.
- **Do:** Implement the wrappers from "Shared contracts" against `src/bindings`. Add `isAppError(value: unknown): value is AppError` and a `toAppError(err: unknown): AppError` that wraps non-AppError values as `{ kind: "io", message }`.
- **Done when:** `npm run check` passes; no `as` casts except `as const` (check with `grep -n " as " src/api`).

---

## Wave 3 — Rust engines

Parallel. Pure or near-pure modules with unit tests. No Tauri commands yet.

### T3.1 Sidecar runner

- **Owns:** `src-tauri/src/ktx/sidecar.rs`
- **Do:** `pub async fn run_ktx(app: &AppHandle, args: Vec<String>, on_stderr: impl Fn(String) + Send + 'static, cancel: CancellationHandle) -> Result<KtxOutput, AppError>` using `tauri_plugin_shell::ShellExt::shell().sidecar("ktx")`. Collect stdout and stderr, stream each stderr line to `on_stderr`, return `KtxOutput { code, stdout, stderr }`. Non-zero exit becomes `AppError::Ktx`. Cancellation kills the child and returns `AppError::Cancelled`. Also `pub fn exit_meaning(code: i32) -> &'static str` from the requirements table.
- **Done when:** unit test for `exit_meaning` (0–6 and unknown); an `#[ignore]` test runs `--version` through a test `AppHandle` (`tauri::test::mock_builder`) or, if the shell plugin cannot run under the mock, through `std::process::Command` on the staged binary path. `npm run check` passes.

### T3.2 `ktx create` argument builder

- **Owns:** `src-tauri/src/ktx/create_args.rs`
- **Read:** requirements.md options tables, `docs/ktx-cli/create.txt`
- **Do:** `pub fn build_create_args(opts: &CreateOptions, inputs: &[PathBuf], output: &Path) -> Result<Vec<String>, AppError>`. Emit only flags that are set or differ from "unset". Order: flags, then inputs, then output. Reject combinations the tool rejects (same list as T3.3) with `AppError::InvalidInput`, so a bad request never reaches the binary. Value formatting: floats with `.` decimal separator regardless of locale.
- **Done when:** tests cover: defaults → `["--format","R8G8B8A8_SRGB", in, out]`; every flag in requirements.md appears in at least one test (keep a `const ALL_FLAGS: &[&str]` copied from the spec and assert each is emitted by some test case); zstd + basis-lz rejected; layers with 4 inputs. `npm run check` passes.

### T3.3 Option rules engine

- **Owns:** `src-tauri/src/ktx/rules.rs`
- **Read:** every "not valid with", "only", "ignored unless", "requires", and "mutually exclusive" note in requirements.md, plus the paragraph on sRGB/transfer-function rules and mip generation with non-linear transfer functions
- **Do:** `pub fn describe(opts: &CreateOptions, sources: &[SourceInfo]) -> OptionReport`. Return a `FieldState` for **every** `OptionKey`. Disabled fields carry the reason text from the spec. Warnings cover the color-conversion and non-linear-encode notes. Also `pub fn validate(opts) -> Result<(), Vec<String>>` used by T3.2 for hard errors. Write rules as a flat list of small functions, one per spec note, each with a comment quoting the note.
- **Done when:** one test per spec note (at least 25), plus a test that the report has exactly one entry per `OptionKey` variant. `npm run check` passes.

### T3.4 Input classification and frame ordering

- **Owns:** `src-tauri/src/image/classify.rs`
- **Do:** `pub fn classify(path) -> FileKind` (`Image { ldr|hdr, needs_conversion } | Ktx1 | Ktx2 | Unsupported`), decided by magic bytes first (KTX1 `«KTX 11»`, KTX2 `«KTX 20»`, PNG, JPEG, WebP, BMP, GIF, TIFF, EXR, Radiance HDR), extension second. `pub fn order_frames(paths: Vec<PathBuf>) -> Vec<PathBuf>`: natural sort by the last number in the file stem, then by name.
- **Done when:** tests over the fixtures (each kind), a misnamed file (PNG renamed `.ktx2` → Image), and ordering `frame_10, frame_2, frame_1` → `1, 2, 10`. `npm run check` passes.

### T3.5 Input conversion

- **Owns:** `src-tauri/src/image/convert.rs`
- **Do:** `pub fn prepare_input(src: &Path, workdir: &Path) -> Result<PreparedInput, AppError>`: PNG and EXR pass through unchanged; JPEG/WebP/BMP/GIF/TIFF become PNG (keep alpha, keep 16-bit where the source has it); Radiance HDR becomes half-float EXR. GIF uses the first frame. Also `pub fn read_source_info(path) -> SourceInfo` (dimensions without full decode where `image` allows) and `pub fn thumbnail_png(path, max: u32) -> Vec<u8>`.
- **Done when:** tests convert `rgb.jpg` to a PNG of the same size; EXR passes through byte-identical; thumbnail of `rgba.png` at 32 is ≤ 32 px on the long side. `npm run check` passes.

### T3.6 Output images: JPEG and tone mapping

- **Owns:** `src-tauri/src/image/export.rs`
- **Do:** `pub fn png_to_jpeg(png: &Path, out: &Path, quality: u8) -> Result<()>` (alpha composited over white, quality clamped 1–100). `pub fn exr_to_preview_png(exr: &Path) -> Result<Vec<u8>>`: Reinhard tone map on luminance, then sRGB encode. `pub fn png_bytes(path) -> Result<Vec<u8>>`.
- **Done when:** tests: JPEG output decodes with the same size; quality 10 file is smaller than quality 95; tone-mapped `hdr.exr` decodes as 8-bit RGBA. `npm run check` passes.

### T3.7 `KTXanimData` writer

- **Owns:** `src-tauri/src/ktx/anim.rs`
- **Read:** KTX 2.0 spec sections on key/value data and file layout (https://registry.khronos.org/KTX/specs/2.0/ktxspec.v2.html), requirements.md "Multi-frame"
- **Do:** `pub fn write_anim_data(path: &Path, anim: &AnimationOptions) -> Result<(), AppError>`. Read the whole file, parse the header and index with the `ktx2` crate, rebuild the key/value block with `KTXanimData` added or replaced (12 bytes, three little-endian `u32`), keep keys sorted by byte value as the spec requires, keep every alignment rule (KVD entries 4-byte padded; supercompression global data 8-byte aligned; each mip level aligned per spec), shift `sgdByteOffset` and every level offset by the size delta, write to a temp file in the same dir, then rename over the original. Reject when `layerCount < 2`.
- **Done when:** tests on `array4_mips.ktx2` and `basis_lz.ktx2` (copy to a temp dir first): after writing, the `ktx2` crate reads the file, the key returns the written values, and every level's bytes equal the original level bytes. An `#[ignore]` test runs `ktx validate` on the result and expects exit 0. `npm run check` passes.

### T3.8 `ktx extract` argument builder and naming

- **Owns:** `src-tauri/src/ktx/extract_args.rs`
- **Read:** `docs/ktx-cli/extract.txt`, requirements.md "Encode preview" and "Export"
- **Do:** `pub fn build_extract_args(input, output, selection: &ExtractSpec) -> Vec<String>` where `ExtractSpec` covers `level/face/layer/depth` or `all`, and an optional `transcode` target (`r8 | rg8 | rgb8 | rgba8` and the half-float targets from `extract.txt`). `pub fn transcode_target_for(channels: u8, hdr: bool) -> TranscodeTarget`. `pub fn expected_output_name(base, level, face, layer, depth, dims) -> String` matching `output_level{n}_face{n}_layer{n}_depth{n}.<ext>` with each suffix omitted when that dimension is 1.
- **Done when:** tests for single image, `--all`, transcode target per channel count, and naming with each dimension at 1 and > 1. `npm run check` passes.

### T3.9 Info runner, KTX1 reader, metrics parser

- **Owns:** `src-tauri/src/ktx/info.rs`, `src-tauri/src/ktx/ktx1.rs`, `src-tauri/src/ktx/metrics.rs`
- **Read:** `docs/ktx-cli/samples/*`, KTX1 spec header layout (https://registry.khronos.org/KTX/specs/1.0/ktxspec.v1.html)
- **Do:** `info.rs`: `pub async fn inspect(app, path) -> Result<InspectReport>`: KTX2 → `ktx info --format json`; a non-zero exit with JSON on stdout still returns a report (invalid files show best-effort data); KTX1 → `ktx info` text plus `ktx1::read_header`. Choose `PreviewKind` from the format (HDR/float formats → `TonemappedFromExr`; SINT/UINT → `Unavailable` with reason). `ktx1.rs`: parse the 64-byte header (both endiannesses). `metrics.rs`: parse SSIM and PSNR from the text in `create-metrics.txt`.
- **Done when:** tests: KTX1 header of `legacy.ktx`; metrics from the sample; JSON parsing path from a saved sample (the process call itself is `#[ignore]`). `npm run check` passes.

### T3.10 Workspace

- **Owns:** `src-tauri/src/workspace.rs`
- **Do:** `Workspace::new()` creates one app temp root (`tempfile::Builder` under the OS temp dir, prefix `ktx-encoder-`). `job_dir(job_id)` creates a subdir; `remove_job(job_id)`; the root is removed on drop and on app exit. Also `inspector_cache(path, mtime)` returning a stable subdir for extracted images of one file.
- **Done when:** tests create, reuse, and remove dirs. `npm run check` passes.

---

## Wave 4 — Commands and window wiring

Parallel. Each task fills one command file (registered already in T1.1).

### T4.1 Encoder commands

- **Owns:** `src-tauri/src/commands/encoder.rs`, `src-tauri/src/commands/app.rs`
- **Do:** `default_create_options` → `CreateOptions::default()`. `describe_options` → `rules::describe`. `pick_sources` → Rust multi-file open dialog filtered to supported image extensions, classify, return `SourceInfo` list (KTX files picked here go to `open_inspector` instead). `source_thumbnail` → `convert::thumbnail_png` as `ipc::Response`. `ktx_version` → pinned from `ktx-version.json` (via `include_str!`) and reported from `ktx --version`.
- **Done when:** `npm run check` passes; manual: in `npm run tauri dev`, calling each command from the devtools console via `window.__TAURI__.core.invoke` returns data.

### T4.2 Encode pipeline

- **Owns:** `src-tauri/src/commands/encode.rs`
- **Do:** `encode`: create job dir; emit `converting`; `prepare_input` each source (in order; `order_frames` already applied by the frontend list order); emit `encoding`; `build_create_args` + `run_ktx`, forwarding stderr lines in `encode://progress`; if `animation` is set, emit `writing-anim` and `write_anim_data`; emit `previewing` and extract a preview (transcode target per T3.8, tone-map for HDR); read output metadata via `ktx2`; parse metrics; emit `done`; return `EncodeResult`. Run on `tauri::async_runtime` so the UI never blocks. Store the child handle for `cancel_encode`. `encoded_preview` returns the preview PNG bytes. `save_encoded` opens a save dialog (default name from the first source, `.ktx2`) and copies the output.
- **Done when:** an `#[ignore]` integration test encodes `rgba.png` with defaults and with `basis-lz`, and `frames/*` with animation, asserting exit success and the anim key; `npm run check` passes.

### T4.3 Inspector window and drag-drop

- **Owns:** `src-tauri/src/drop.rs`, `src-tauri/src/commands/inspector_window.rs`
- **Read:** requirements.md "Architecture" (drag-drop and the Windows deadlock note)
- **Do:** `drop.rs`: handle `WindowEvent::DragDrop(DragDropEvent::Drop { paths, .. })` for the `main` window: classify; images → emit `encoder://sources-added` (with `SourceInfo`, frames ordered); KTX files → `open_inspector` for the first one (spawned async); unsupported → `encoder://drop-rejected`. Drops on the inspector window with a KTX file load that file. `open_inspector`: `async`; store the path in `AppState`; reuse window `inspector` if it exists (`set_focus`, emit `inspector://file`); otherwise build it with `WebviewWindowBuilder` at `inspector.html`, title "KTX-GUI Inspector — <file name>", 1100×760. `inspector_current_file` returns the stored path so a freshly loaded window does not miss the first event.
- **Done when:** manual in `npm run tauri dev`: dropping `rgba.png` on main logs the event; dropping `basis_lz.ktx2` opens one inspector window; dropping a second KTX reuses it. `npm run check` passes.

### T4.4 Inspect and image commands

- **Owns:** `src-tauri/src/commands/inspect.rs`
- **Do:** `inspect` → `info::inspect`. `inspector_image`: extract the selected level/face/layer/depth into the workspace inspector cache (cache by selection), transcode Basis/UASTC to an unpacked target, convert KTX1 via `ktx convert` first when extract rejects it, tone-map HDR, return PNG bytes.
- **Done when:** `#[ignore]` tests on `basis_lz.ktx2` level 0 and last level, `cubemap.ktx2` face 3, `array4_mips.ktx2` layer 2 level 1, `hdr_uastc.ktx2`. `npm run check` passes.

### T4.5 Export command

- **Owns:** `src-tauri/src/commands/export.rs`
- **Do:** `export_images`: single image → save dialog with PNG/JPEG/EXR filter; multi-image → folder dialog. Extract to the workspace, then copy PNG/EXR or convert to JPEG with `png_to_jpeg`. Names follow `expected_output_name`. Refuse PNG/JPEG for formats the tool will not write as PNG (EXR only) with a clear `InvalidInput` message. Return the written paths.
- **Done when:** `#[ignore]` test exports all frames of `anim.ktx2` as JPEG into a temp dir (bypass the dialog with an internal `export_to(dir, ...)` function the command calls). `npm run check` passes.

### T4.6 Updater runtime

- **Owns:** `src-tauri/src/updater.rs`, `src-tauri/src/commands/update.rs`
- **Do:** On setup, spawn a task that calls `app.updater()?.check()` (skip silently in debug builds and when no endpoint is configured). On update found, store it in state and emit `update://status` `available` with version and notes. `install_update` downloads with progress events (`downloading`), installs, emits `installed`, then `app.restart()`. Errors emit `error` and never crash the app.
- **Done when:** `npm run check` passes; the debug build starts without errors when the updater config is absent.

---

## Wave 5 — Frontend

Parallel. Vue only renders and calls `src/api`. Each task owns its own folder or files.

### T5.1 Encoder state and shell

- **Owns:** `src/encoder/EncoderApp.vue`, `src/encoder/store.ts`, `src/encoder/components/DropOverlay.vue`, `src/encoder/components/SourceList.vue`
- **Do:** `store.ts`: a single `reactive` store (no Pinia) with `sources: SourceInfo[]`, `options: CreateOptions` (loaded from `default_create_options`), `report: OptionReport` (refreshed through `describe_options` on every options/sources change, debounced 80 ms with `useDebounceFn`), `job` state, and `animation: AnimationOptions | null`. Helper `fieldState(key: OptionKey)`. `EncoderApp.vue`: three-column layout (sources | options | preview), header with app name and pinned `ktx` version. `DropOverlay.vue`: highlight on `getCurrentWebview().onDragDropEvent` `enter`/`over`, reset on `leave`/drop; it does **not** read the dropped paths (Rust does). `SourceList.vue`: thumbnails via `source_thumbnail`, reorder (up/down buttons), remove, "Add files…" (`pick_sources`), listens to `encoder://sources-added` and `encoder://drop-rejected` (inline notice).
- **Done when:** `npm run check` passes; manual: drop images, see thumbnails, reorder, remove.

### T5.2 Option sections A: container, filters, color

- **Owns:** `src/encoder/components/options/ContainerSection.vue`, `FilterSection.vue`, `ColorSection.vue`
- **Read:** requirements.md option tables for these groups
- **Do:** One `OptionField` per option in the matching tables, bound to `store.options`, using `fieldState(key)`. Enum selects list every value from the generated enum type (build the list once with a `satisfies Record<Enum, string>` label map so a missing value is a type error). Warnings from `store.report.warnings` whose `key` is in this section render under the control.
- **Done when:** `npm run check` passes; count of `OptionField` uses equals the option count in those three tables (state the numbers in the report).

### T5.3 Option sections B: format, codecs, encoder flags

- **Owns:** `src/encoder/components/options/FormatSection.vue`, `BasisLzSection.vue`, `UastcLdrSection.vue`, `UastcHdrSection.vue`, `AstcSection.vue`, `EncoderFlagsSection.vue`, `SupercompressionSection.vue`
- **Do:** Same rules as T5.2. Format picker groups uncompressed, ASTC (block size + sRGB/UNORM), and codec (`--encode`). Codec sections are always visible; the inactive ones are disabled with the reason from the report.
- **Done when:** `npm run check` passes; count check as in T5.2.

### T5.4 Animation, encode bar, preview

- **Owns:** `src/encoder/components/AnimationSection.vue`, `EncodeBar.vue`, `PreviewPane.vue`, `src/encoder/components/options/OptionsPanel.vue`
- **Do:** `OptionsPanel.vue` stacks all sections from T5.2/T5.3 plus `AnimationSection` in collapsible `Panel`s (imports only; if a section file is missing at merge time, the orchestrator merges T5.2/T5.3 first). `AnimationSection`: enable toggle (only when ≥ 2 sources), duration, timescale, loop count, computed frame length and sequence length. `EncodeBar`: Encode button (disabled while running or with zero sources), Cancel, stage text from `encode://progress`, indeterminate `ProgressBar`, last stderr lines, `ErrorBox` on failure, Save button after success. `PreviewPane`: source thumbnail vs encoded preview side by side, "Preview, not stored data" label when `preview` is `TonemappedFromExr`, stats (bytes, dimensions, codec, levels, layers, SSIM, PSNR). Fixed-size boxes so nothing shifts when results arrive.
- **Done when:** `npm run check` passes; manual: encode `rgba.png` to basis-lz, see preview and stats, save the file.

### T5.5 Inspector report panels

- **Owns:** `src/inspector/InspectorApp.vue`, `src/inspector/store.ts`, `src/inspector/components/report/*.vue`
- **Read:** requirements.md "Inspector" field list
- **Do:** `store.ts`: current path (from `inspector_current_file` on mount and `inspector://file`), `report: InspectReport`, `selection: ImageSelection`. `InspectorApp.vue`: left column report, right column image viewer area (slot filled by T5.6 components, imported by name). Panels: `StatusPanel`, `HeaderPanel`, `LevelIndexTable`, `IndexPanel`, `DfdPanel` (samples as a table), `KnownMetadataPanel`, `KeyValueTable`, `BasisGlobalDataPanel`, `Ktx1TextPanel`. Every field in the requirements list is rendered; missing optional values show "—".
- **Done when:** `npm run check` passes; manual: open each fixture, every panel renders, `truncated.ktx2` shows the validation errors.

### T5.6 Mip browser, selectors, frame player

- **Owns:** `src/inspector/components/viewer/ImageViewer.vue`, `MipStrip.vue`, `SubresourceSelectors.vue`, `FramePlayer.vue`
- **Do:** `ImageViewer`: fetches `inspector_image` for the selection, shows it at real pixel size and fitted (toggle), pixelated scaling, checkerboard behind alpha, Blob URLs revoked on change. `MipStrip`: one entry per level, size `max(1, base >> i)`. `SubresourceSelectors`: face (6 labeled +X…−Z), layer, depth slice, shown only when the dimension is > 1. `FramePlayer`: shown when `KTXanimData` is present; step, play/pause using `useIntervalFn` at `duration / timescale`, loop per `loopCount`, shows frame count, frame length, sequence length. Preload the next frame to keep playback smooth.
- **Done when:** `npm run check` passes; manual: browse mips of `basis_lz.ktx2`, faces of `cubemap.ktx2`, play `anim.ktx2`.

### T5.7 Export UI

- **Owns:** `src/inspector/components/ExportPanel.vue`
- **Do:** Scope (current image / all frames at current level / everything), format (PNG / JPEG / EXR, PNG/JPEG disabled with reason when `PreviewKind` says the tool will not write PNG), JPEG quality slider (default 90), Export button calling `export_images`, result list of written files, `ErrorBox` on failure.
- **Done when:** `npm run check` passes; manual: export all frames of `anim.ktx2` as JPEG.

### T5.8 Update banner

- **Owns:** `src/shared/components/UpdateBanner.vue`, and one import line plus `<UpdateBanner />` in `src/encoder/EncoderApp.vue` **after T5.1 merges** (orchestrator applies this line)
- **Do:** Listens to `update://status`. Shows version and notes when available, "Install and restart" button (`install_update`), progress bar while downloading, error text.
- **Done when:** `npm run check` passes.

---

## Wave 6 — CI and releases

Can start any time after Wave 2 (needs T2.4 and the `check` scripts from T1.1). Parallel. These tasks touch only CI, release config, and the updater keys in `tauri.conf.json`, so they never collide with Waves 3–5.

### Release flow

Every push to `main` runs `release.yml`:

```text
push to main
  └─ job release (ubuntu)
       semantic-release reads commits since the last tag
       no feat/fix/perf/breaking commit → stop, nothing is released
       otherwise:
         next version (feat → minor, fix/perf → patch, ! or BREAKING CHANGE → major)
         CHANGELOG.md updated, package.json + Cargo.toml + Cargo.lock bumped
         commit "chore(release): X.Y.Z [skip ci]" pushed to main by github-actions[bot]
         tag vX.Y.Z
         draft GitHub Release vX.Y.Z with the generated notes
  └─ job build (matrix of 5 targets, needs release)
       checkout tag vX.Y.Z, fetch-ktx for the target, tauri-action
       uploads installers, updater bundles, .sig files, latest.json to the draft
  └─ job publish (needs build)
       undraft the release and mark it latest
       → the in-app updater sees it through releases/latest/download/latest.json
       semantic-release has already commented "released in vX.Y.Z" on the issues and PRs it closed
```

The release stays a draft until all five builds succeed, so installed apps never see a release with missing platforms. If a build fails, re-run the failed jobs; the tag and changelog commit are already in place.

A `beta` branch publishes prereleases (`X.Y.Z-beta.N`) through the same flow to test the pipeline before shipping. The updater ignores prereleases because it reads `releases/latest`.

The first release from `main` is `1.0.0` (semantic-release's default for a repository with no tags).

This replaces the manual "publish the draft" step in requirements.md. The change is logged in decision-context.md (2026-10-10).

### T6.1 CI workflow

- **Owns:** `.github/workflows/ci.yml`
- **Do:** On push to any branch except `main` and `beta` (the release workflow covers those) and on pull requests. Matrix `windows-latest`, `macos-latest`, `ubuntu-22.04`. Steps: checkout (`fetch-depth: 0`), Linux deps (`libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`), Node LTS with npm cache, Rust stable with `rustfmt` and `clippy`, `Swatinem/rust-cache` on `src-tauri`, `cargo install cargo-machete --locked` (cached), `npm ci`, `npm run fetch-ktx`, `npm run check`, `cargo test --manifest-path src-tauri/Cargo.toml -- --include-ignored`, `git diff --exit-code src/bindings`. On pull requests only, in the Ubuntu job: `npx commitlint --from <PR base sha> --to <PR head sha>`.
- **Done when:** the YAML passes `actionlint` (`docker run --rm -v "$PWD:/repo" -w /repo rhysd/actionlint:latest`, or the `actionlint` binary) and a push to `dev` runs green.

### T6.2 Release workflow

- **Owns:** `.github/workflows/release.yml`
- **Read:** requirements.md "GitHub Actions and auto-update", "Release flow" above
- **Do:**
  - Trigger: `push` to `main` and `beta`; `workflow_dispatch`. `concurrency` group per ref, no cancel in progress.
  - `permissions: contents: write, issues: write, pull-requests: write`.
  - Job `release` (ubuntu-22.04): checkout with `fetch-depth: 0` and `persist-credentials: false`; Node LTS; Rust stable (the version script runs `cargo update`); `npm ci`; `npm run check` (never release a broken commit); `npx semantic-release` with `GITHUB_TOKEN`. If the repo secret `RELEASE_APP_ID` exists, first mint a token with `actions/create-github-app-token` and pass that instead (needed only when `main` is protected). Job outputs come from the `$GITHUB_OUTPUT` lines written by T6.4's config: `published`, `version`, `tag`. Then resolve the draft release id with `gh api repos/<repo>/releases --jq '.[] | select(.tag_name == "<tag>") | .id'` (the list endpoint includes drafts) and output it as `release_id`.
  - Job `build` (`needs: release`, runs only when `published == 'true'`): matrix from the spec: `macos-latest` + `aarch64-apple-darwin`, `macos-latest` + `x86_64-apple-darwin`, `ubuntu-22.04` + `x86_64-unknown-linux-gnu`, `ubuntu-22.04-arm` + `aarch64-unknown-linux-gnu`, `windows-latest` + `x86_64-pc-windows-msvc`. Checkout `ref: <tag>`; Linux deps; Node; Rust with the matrix target installed; rust-cache; `npm ci`; `npm run fetch-ktx -- --target <triple>`; `tauri-apps/tauri-action@v0` with `releaseId: <release_id>`, `includeUpdaterJson: true`, `args: --target <triple>`; env `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` from secrets, and `APPLE_SIGNING_IDENTITY: "-"` on macOS (ad-hoc).
  - Job `publish` (`needs: [release, build]`): `gh release edit <tag> --draft=false`, plus `--latest` on `main` and `--prerelease` on `beta`.
- **Done when:** actionlint passes; a push of a `feat:` commit to `beta` produces a prerelease with five platform artifacts, `.sig` files, `latest.json`, and notes; a `docs:`-only push produces no release.

### T6.3 Updater and bundle config

- **Owns:** `src-tauri/tauri.conf.json` (`bundle.createUpdaterArtifacts`, `plugins.updater`, `bundle.macOS.signingIdentity` only), `src-tauri/capabilities/default.json` (add `updater:default`, `process:allow-restart` only)
- **Needs:** the updater public key from the user.
- **Do:** `bundle.createUpdaterArtifacts: true`; `plugins.updater.pubkey` = the provided public key; `plugins.updater.endpoints: ["https://github.com/MONOGRID/KTX-GUI/releases/latest/download/latest.json"]`; `plugins.updater.windows.installMode: "passive"`; `bundle.macOS.signingIdentity: "-"`.
- **Done when:** `npm run tauri build` succeeds locally with `TAURI_SIGNING_PRIVATE_KEY` set and produces a `.sig` next to the installer; `npm run check` passes.

### T6.4 semantic-release configuration

- **Owns:** `.releaserc.json`, `scripts/set-version.mjs`, `CHANGELOG.md` (create with a title only), `package.json` (`devDependencies` and a `"release": "semantic-release"` script only), `package-lock.json`
- **Do:**
  - Dev dependencies: `semantic-release`, `@semantic-release/changelog`, `@semantic-release/git`, `@semantic-release/exec`, `conventional-changelog-conventionalcommits`. (`commit-analyzer`, `release-notes-generator`, `npm`, and `github` ship with semantic-release.)
  - `.releaserc.json`: `branches: ["main", { "name": "beta", "prerelease": true }]`, `tagFormat: "v${version}"`. Plugins in this order:
    1. `@semantic-release/commit-analyzer` with preset `conventionalcommits`.
    2. `@semantic-release/release-notes-generator` with preset `conventionalcommits` (sections: Features, Bug Fixes, Performance, Breaking Changes; hide `chore`, `ci`, `docs`, `test`, `build`, `refactor`, `style`).
    3. `@semantic-release/changelog` → `CHANGELOG.md`.
    4. `@semantic-release/npm` with `npmPublish: false` (bumps `package.json` and `package-lock.json`; Tauri reads its version from there).
    5. `@semantic-release/exec`: `prepareCmd: "node scripts/set-version.mjs ${nextRelease.version}"`; `successCmd` appends `published=true`, `version=${nextRelease.version}`, and `tag=${nextRelease.gitTag}` to `$GITHUB_OUTPUT` when that variable is set.
    6. `@semantic-release/git` with assets `CHANGELOG.md`, `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock` and message `chore(release): ${nextRelease.version} [skip ci]` followed by a blank line and `${nextRelease.notes}`.
    7. `@semantic-release/github` with `draftRelease: true`, success comments and released labels on (the bot comments on released issues and PRs), no assets (tauri-action uploads them).
  - `scripts/set-version.mjs`: Node, no dependencies. Replaces the `version = "..."` line inside `[package]` of `src-tauri/Cargo.toml` and runs `cargo update --workspace --offline --manifest-path src-tauri/Cargo.toml` so `Cargo.lock` follows without touching other crates. Exits non-zero if the `[package]` version line is not found.
  - Releases happen only in CI. Document `npx semantic-release --dry-run --no-ci` (needs a `GITHUB_TOKEN` with repo access) for previewing the next version locally.
- **Done when:** `node scripts/set-version.mjs 1.2.3` changes only the package version in `Cargo.toml` and `Cargo.lock` (check with `git diff`, then restore both files); `npx semantic-release --dry-run --no-ci` on a release branch prints the computed next version; `npm run check` passes.

---

## Wave 7 — Integration and acceptance

Sequential. A capable agent or a human.

### T7.1 Spec coverage audit

- **Owns:** `docs/acceptance.md`
- **Do:** One row per numbered product requirement and per option in the requirements tables: where it is implemented (Rust file, Vue component), how it was verified (test name or manual step), status. Any gap becomes a follow-up task appended to this plan.
- **Done when:** no row says "missing" without a follow-up task; `fallow suppressions` lists no "consumed by" marker; `npm run check` passes with no baseline.

### T7.2 Smoke test on every OS

- **Owns:** `docs/acceptance.md` (smoke section)
- **Do:** Install the draft release artifacts on Windows x64, macOS arm64, and Linux x64 (and the others when hardware exists). On each: app starts with no KTX tools installed; encode PNG, JPEG, and EXR; encode a 4-frame animation; open each fixture in the inspector; browse mips; play the animation; export PNG and JPEG; install the previous release, merge a `fix:` commit to `main`, let the pipeline publish, and confirm the installed app offers and installs the update.
- **Done when:** every step is recorded as pass or fail with notes.

### T7.3 Docs

- **Owns:** `README.md`, `docs/decision-context.md` (append only)
- **Do:** Replace the template README: what the app does; prerequisites (Node, Rust, `cargo install cargo-machete`); how to run it (`npm ci`, `npm run fetch-ktx`, `npm run tauri dev`); `npm run check`; the commit convention and how it drives releases (link `CHANGELOG.md`); how to bump the `ktx` version (a `feat:` or `fix:` commit editing `ktx-version.json`). Append decision entries for anything decided during implementation that is not yet logged.
- **Done when:** a fresh clone following the README reaches a running app.

---

## Wave summary

Every task ends with `npm run check` in its worktree; every wave ends with `npm run check` on the merged `dev` branch.

| Wave | Tasks | Parallel | Who |
| --- | --- | --- | --- |
| 0 Spikes | T0.1–T0.3 | T0.1 first, then T0.2 and T0.3 | Capable agent or human |
| 1 Scaffold | T1.1 | No | One agent |
| 2 Contracts | T2.1–T2.5, then T2.6 | Yes | Sub-agents |
| 3 Engines | T3.1–T3.10 | Yes | Sub-agents |
| 4 Commands | T4.1–T4.6 | Yes | Sub-agents |
| 5 Frontend | T5.1–T5.7, then T5.8 | Yes (T5.4 merges after T5.2/T5.3) | Sub-agents |
| 6 CI/release | T6.1–T6.4 | Yes, from Wave 3 on | Sub-agents; T6.3 needs the public key |
| 7 Acceptance | T7.1–T7.3 | No | Capable agent or human |
