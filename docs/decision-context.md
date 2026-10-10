# Decision context

Log of decisions for this app. Product behavior stays in [requirements.md](requirements.md).

Read the whole log before choosing or changing an approach. Append a new entry at the bottom when a decision is made. Leave earlier entries in place. If a later entry supersedes an earlier one, say which entry it replaces.

Each entry records the date, what was decided, and why.

## 2026-10-04 — Embed the official `ktx` CLI

**Decided:** Ship the official `ktx` binary from [KTX-Software](https://github.com/KhronosGroup/KTX-Software) inside the app and call it from Rust. Rust builds the argument list, runs the process, converts everyday images the CLI cannot read or write, and writes `KTXanimData` after create. Vue only renders. Do not replace that binary with a crates.io KTX or Basis crate for the first version. [requirements.md](requirements.md) stays as written.

**Why:** The spec asks for every documented `ktx create` option, a preview, an inspector of every reported detail, multi-frame encode and decode with animation metadata, and mip previews. Those already exist as `ktx create`, `ktx extract`, and `ktx info --format json`.

Crates.io was surveyed the same day. Each crate covers a slice. None implements the option set, the official info schema, and extract-to-image together. Binding the missing surface from libktx means reimplementing `ktx create` (color-conversion rules, format constraints, ASTC, HDR codecs, mip filters, warnings). That duplicates the CLI.

Linking libktx in-process stays open for later if process spawn or temp files get in the way. It needs CMake and a C++ build of KTX-Software on every release target, and it adds no encoding options the CLI lacks.

| Crate | Version surveyed | What it covers |
| --- | --- | --- |
| [`ktx2`](https://crates.io/crates/ktx2) 0.5.0 (Apr 2026, ~5.5M downloads) | Pure Rust KTX2 reader: header, data format descriptor, key/value data including `KTXanimData`, mip index, validation. No writer, no pixel decode. |
| [`ktx2-reader`](https://crates.io/crates/ktx2-reader) 0.1.2 (Oct 2020) | Older async KTX2 parser. No meaningful dependents. |
| [`libktx-rs`](https://crates.io/crates/libktx-rs) 0.3.3 (Sep 2021) | Bindings that compile KTX-Software from a git submodule. KTX1 and KTX2 read, write, and transcode. Five years behind current KTX-Software. |
| [`ktx2-rw`](https://crates.io/crates/ktx2-rw) 0.2.4 (Nov 2025) | CMake-builds and statically links KTX-Software v4.4.0. Safe API: create, layers, faces, levels, pixel get/set, Basis compress, transcode, metadata, write `.ktx2`. |
| [`basis-universal`](https://crates.io/crates/basis-universal) 0.3.1 (Nov 2023, ~500k downloads) | C++ bindings. ETC1S and UASTC LDR encode and transcode, mipmaps, cubemaps, arrays, `.basis` or KTX2 output. Vendored upstream predates UASTC HDR and XUASTC. |
| [`basisu`](https://crates.io/crates/basisu) 0.1.0 (Jul 2026) | Pure Rust transcoder only, bit-exact with Basis Universal v2.1.0. No encoder. |
| [`bevy_basisu_saver`](https://crates.io/crates/bevy_basisu_saver) | Bevy asset-pipeline bindings around the C encoder. Marked experimental by its author. |

`ktx2-rw` was the closest in-process option. Its build downloads [KTX-Software v4.4.0](https://github.com/KhronosGroup/KTX-Software/archive/refs/tags/v4.4.0.tar.gz). The first build needs CMake, a C++ compiler, and network. Basis settings on the safe API are ETC1S or UASTC as a boolean, quality, threads, normal-map mode, and one UASTC RDO scalar. Transcode targets there are BC1/3/4/5/7, ETC1/ETC2, ASTC 4×4, PVRTC, and a few uncompressed layouts. ASTC block sizes, UASTC HDR 4×4 and 6×6i, Zstd and Zlib levels, transfer functions, primaries, mip filters, swizzle, and PSNR/SSIM exist on `ktxTexture2_CompressBasisEx` and `ktxTexture2_CompressAstcEx` inside libktx, not in that Rust API.

`basisu` reads `.ktx2` and `.basis` and transcodes ETC1S, UASTC LDR, UASTC HDR, raw ASTC, and XUASTC, including mips, layers, faces, and Zstd. It can preview later. It does not encode.

`basis-universal` 0.3.1 `CompressorParams` covers ETC1S quality, UASTC LDR quality, RDO, mipmaps, color space, and normal maps. Current Basis Universal (v2.5) adds UASTC HDR 4×4, ASTC HDR 6×6, and UASTC HDR 6×6 intermediate. That crate does not track it, and it does not provide the inspector's data format descriptor, validation report, or ARM ASTC encode.

Rust still wraps the CLI where the tool stops, as already specified:

- Convert JPEG, WebP, BMP, GIF, and TIFF to PNG or EXR before `ktx create`. The CLI accepts PNG, EXR, and raw.
- Encode JPEG on export. `ktx extract` writes PNG, EXR, or raw.
- Run `ktx create` off the UI thread and return exit code, stderr, and progress.
- Run `ktx extract` for the encode preview and for each mip, face, and frame in the inspector.
- Parse `ktx info --format json`, which follows the [Khronos info schema](https://schema.khronos.org/ktx/info_v1.json).
- Write the 12-byte `KTXanimData` key (`duration`, `timescale`, `loopCount`) after create. Frames are array layers. `ktx create` has `--layers` and no flag for this key.

**Revisit when:** a maintained Rust binding exposes the full `ktx create`, `ktx info`, and `ktx extract` surface against a current KTX-Software, including ASTC, UASTC HDR, supercompression, and the info schema; or spawn and temp files block a requirement an in-process library would meet; or Khronos stops shipping a `ktx` executable that can be extracted per OS and CPU.

## 2026-10-04 — Frontend coding rules

**Decided:** Agent instructions in [AGENTS.md](../AGENTS.md) now require: Rust for heavy work and Vue only for visualization (already in [requirements.md](requirements.md)); strict TypeScript with no `any` or assertion escapes, preferring `satisfies`, generics, and project utility types; ESLint kept on typescript-eslint `strict` and `strictTypeChecked`; UnoCSS utilities, with component SCSS only when a utility cannot express the rule; VueUse and an existing npm package before a new helper, checked via Context7 or web search first.

**Why:** The product split was already specified. These rules stop the frontend from growing type escapes, one-off CSS, and helpers that VueUse or a small package already provides.

## 2026-10-04 — Remember window bounds, File → Exit

**Decided:** Persist the main window with `tauri-plugin-window-state`, flags size, position, maximized, and visible. The window is created hidden (`visible: false`) and the plugin shows it after restore. The plugin is registered on the Tauri builder, before config windows exist. File → Exit is a native menu item with id `exit` and shortcut Cmd/Ctrl+Q. The click handler calls `app.exit(0)`. On macOS the menu also has the application submenu and Edit, because a menu bar with only File drops copy, paste, hide, and the app menu.

**Why:** Size and position are the request. Maximized is stored with them so a maximized close does not write the monitor corner as the restored frame. Visible is included because the plugin only calls `show` when that flag is set, which is what avoids a flash at the default 800×600. Registering the plugin inside `setup` is too late: Tauri creates config windows before that hook, and this plugin restores on window-ready. Predefined Quit is unsupported on Linux, so Exit is a normal item on every desktop OS. The menu stays in Rust; the frontend does not own window chrome.

## 2026-10-10 — Fallow for TypeScript/Vue cleanup gates

**Decided:** Add [fallow](https://fallow.tools) as a project `devDependency` with `.fallowrc.json` (Vue/Vite entry globs, `duplicates.minOccurrences: 3`), Claude Code PreToolUse gate under `.claude/`, npm scripts `fallow` / `fallow:audit`, and a Fallow section in [AGENTS.md](../AGENTS.md). Warn-default cleanup rules stay advisory; `private-type-leak` stays off. No git pre-commit hook: run via `npx fallow` / the local `devDependency` only; do not require a global install.

**Why:** Static dead-code, duplication, and complexity checks on the Vue/TS surface, without replacing `tsc` or ESLint. Rust stays outside fallow's scope. A git hook that shells out to a bare `fallow` binary breaks when Corepack/nvm has no global activation, and nobody should need `npm install -g` to commit.

## 2026-10-10 — Name, repository, `ktx` pin

**Decided:** The product is `KTX-GUI`, identifier `com.monogrid.ktx-gui`, repository `https://github.com/MONOGRID/KTX-GUI`. The embedded tool is pinned to KTX-Software v4.4.2. The user generates the updater signing key pair and stores the private key in GitHub secrets.

**Why:** v4.4.2 is the latest stable release; 5.0.0 is still a release candidate (rc2, 2026-08-17). If a flag in [requirements.md](requirements.md) is missing from v4.4.2, the user decides between dropping that option for now and moving to 5.0. The identifier must be final before the first public release, because installed apps key their data directory and updates on it. The updater endpoint needs the repository path.

## 2026-10-10 — fallow in the quality gate

**Decided:** `npm run check` is the single quality gate, run before every commit, by every sub-agent task, at every wave merge, and in CI. It runs Rust checks (fmt, clippy with `-D warnings`, tests, `cargo machete`), web checks (vue-tsc build, ESLint, Stylelint), and fallow: `fallow dead-code --fail-on-issues`, `fallow dupes --threshold 3`, `fallow health --min-score 85 --min-severity high`. Fallow findings are fixed in code, like lint errors. The only allowed suppression marks code written ahead of its consumer and names the consuming task; fallow reports it as stale once the code is used. Thresholds change only through a new entry here.

**Why:** The user asked for health, duplication, and dead-code checks in the same routine as lint and typecheck. Running them at every step catches drift while the change is small; parallel sub-agents especially tend to duplicate helpers. Fallow covers TypeScript, Vue, and CSS only, so Rust dead code relies on clippy and unused crates on `cargo machete`.

## 2026-10-10 — Automatic releases with semantic-release

**Decided:** Releases are automatic on push to `main` with semantic-release. Conventional Commits, enforced locally by husky + commitlint and in CI on pull requests, decide the version: `feat` minor, `fix`/`perf` patch, `!` or `BREAKING CHANGE:` major, other types no release. semantic-release writes `CHANGELOG.md`, bumps `package.json` (Tauri reads its version from there) and `Cargo.toml`, commits as `github-actions[bot]`, tags `vX.Y.Z`, and creates a draft GitHub Release with the notes and bot comments on released issues and PRs. The tauri-action matrix uploads artifacts to that draft; a final job publishes it. A `beta` branch publishes prereleases. Development happens on `dev`; merging to `main` ships. This replaces "Publishing the draft is the ship step" in [requirements.md](requirements.md), which is updated to match.

**Why:** The user wants a changelog and releases driven by commit messages and already knows semantic-release. release-please (a bot that opens a release PR to merge) was the alternative; it adds a review step the user did not ask for. The draft stays unpublished until every platform has built, so the updater's `releases/latest` never points at a release with missing artifacts.
