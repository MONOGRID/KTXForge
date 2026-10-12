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

## 2026-10-10 — `TS_RS_EXPORT_DIR` set from `build.rs`

**Decided:** Keep `src-tauri/.cargo/config.toml` as documented, and also set `TS_RS_EXPORT_DIR` to an absolute `../src/bindings` path from `src-tauri/build.rs` via `cargo:rustc-env`. Generated bindings are committed under `src/bindings/`.

**Why:** With a relocated Cargo target directory (CI / sandbox), the relative env from `.cargo/config.toml` alone did not reliably land files in `src/bindings/`. `CARGO_MANIFEST_DIR` is stable.

## 2026-10-10 — Scaffold architecture choices (development plan)

**Decided:** Adopt the plan’s “Decisions this plan makes” table: pin KTX-Software v4.4.2; share Rust→TS types via `ts-rs` into `src/bindings/`; keep the option rules engine in Rust (`describe_options`); send previews as PNG bytes through `tauri::ipc::Response`; open save/open dialogs from the Rust dialog plugin; use two Vite HTML entries (`index.html` encoder, `inspector.html` inspector); convert images with the `image` crate and temp dirs with `tempfile`; read KTX2 with `ktx2`; check unused Rust crates with `cargo machete`; enforce Conventional Commits with husky + commitlint.

**Why:** These choices are required for Wave 1 scaffold (T1.1) so later waves only fill stub bodies. They match AGENTS.md (Rust owns heavy work; explicit shared types) and the settled product/repo decisions already logged.

## 2026-10-10 — Wave 0 spike results (v4.4.2 CLI gaps)

**Decided (interim):** Keep the KTX-Software **v4.4.2** pin for Wave 1 scaffold and document the CLI gaps in [ktx-cli/README.md](ktx-cli/README.md). Do not invent 5.0-only codec names in types until the user chooses: (A) defer missing options / map UI `uastc` to the v4.4.2 value, or (B) move the pin to 5.0. ASTC format strings passed to `ktx` must use the Vulkan `_BLOCK` suffix. KTX1→KTX2 uses legacy `ktx2ktx2` until a `ktx convert` exists. Windows sidecar staging is `ktx.exe` + side-by-side `ktx.dll` (see [ktx-cli/staging.md](ktx-cli/staging.md)).

**Why:** T0.1 compared every UI option in [requirements.md](requirements.md) to `ktx create --help` from the official Windows x64 4.4.2 build. Missing or different: UASTC HDR codecs and flags, `--premultiply-alpha`, `ktx convert`, half-float extract transcode targets; `--encode` is `basis-lz | uastc` only; `pg_eotf` / `abobergb` are real tool spellings. T0.3 fixtures and info samples are in-tree; `hdr_uastc.ktx2` is uncompressed float because encode-to-UASTC on HDR is rejected in 4.4.2.

## 2026-10-10 — Stay on KTX-Software v4.4.2 (defer CLI gaps)

**Decided:** Keep the embedded tool pin at **KTX-Software v4.4.2**. Do **not** upgrade to 5.0 for the missing surface. Wave 2 option types and later CLI wrappers freeze against the captured v4.4.2 help in [ktx-cli/](ktx-cli/). Gaps documented in [ktx-cli/README.md](ktx-cli/README.md) are **deferred** (not exposed in types or UI) until a deliberate pin bump:

- UASTC HDR codecs (`uastc-hdr-4x4`, `uastc-hdr-6x6i`) and all `--uastc-hdr-*` flags
- Spec codec name `uastc-ldr-4x4` (v4.4.2 only has `--encode uastc`; UI/types use that value)
- `--premultiply-alpha`
- Unified `ktx convert` (keep legacy `ktx2ktx2` path when needed)
- Half-float `ktx extract --transcode` targets

ASTC `--format` strings keep the Vulkan `_BLOCK` suffix required by v4.4.2. This supersedes the interim “user must choose A or B” note in the Wave 0 spike entry above; the user chose **pin and defer** (A).

**Why:** The user chose a stable pin over moving to a 5.0 release candidate. Deferring gaps keeps encode args and the option rules engine honest to the binary we ship; inventing 5.0-only names would fail at runtime.

## 2026-10-10 — Wave 3 engine notes

**Decided:**

1. **`run_ktx` vs collect:** `sidecar::run_ktx` maps non-zero exit to `AppError::Ktx`. `sidecar::run_ktx_collect` returns `KtxOutput` for any exit code so `info::inspect` can still parse JSON from invalid files (stdout is not on `AppError::Ktx`).
2. **`CancellationHandle`:** Shared `Arc<AtomicBool>`; cancel kills the shell `CommandChild` and returns `AppError::Cancelled`.
3. **KTXanimData + BasisLZ fixture:** Reject when `layerCount < 2`. Wave 0 `basis_lz.ktx2` is a non-array texture (`layerCount` 0) with SGD, so write success is tested on `array4_mips.ktx2` / `anim.ktx2`; `basis_lz.ktx2` asserts rejection. SGD rewrite code remains for when encode produces layers + BasisLZ.
4. **`hdr.exr` fixture:** Regenerated with alphabetically sorted RGBA channels (`exr` crate). The Wave 0 file failed OpenEXR channel-order validation in both `image` and `exr`, blocking tone-map tests. Preview tone-map loads EXR via the `exr` crate (not `image`'s OpenEXR decoder).
5. **Extract transcode:** v4.4.2 `extract.txt` has only `r8|rg8|rgb8|rgba8` unpacked targets; half-float targets stay deferred per the pin decision.

**Why:** Keep Wave 3 engines honest to the shipped binary and fixtures without inventing 5.0 surface or weakening the anim layer rule.

## 2026-10-10 — Wave 4 command wiring

**Decided:**

1. **`ktx extract` unit dimensions:** `build_extract_args` takes `Dims` and omits `--layer` / `--face` / `--depth` (and `--level` when levels is 1 and level is 0). Explicit `--layer 0` on a non-array texture fails in v4.4.2.
2. **Shared extract helpers:** `ktx/extract_ops.rs` owns ensure-KTX2, extract+preview PNG (transcode / tone-map), and texture meta from `ktx info` or the `ktx2` crate. Encode, inspect, and export call it.
3. **KTX1 → KTX2:** Prefer legacy `ktx2ktx2` beside staged binaries or under `target/ktx-cache` (no `ktx convert` on v4.4.2). If missing, extract may still be tried on the original path.
4. **Process tests without AppHandle:** Encode / inspect / export fixture tests run the staged `ktx-<triple>` via `std::process::Command` when present; they no-op skip when `binaries/` is empty (gitignored). Full `sidecar::run_ktx` + AppHandle inspect remains `#[ignore]`.
5. **Updater plugin:** Registered always; check/install skipped in debug builds and when endpoints/pubkey are absent (errors never crash). Pubkey/endpoints still land in T6.3. Until then `plugins.updater` must be an object (`pubkey: ""`, `endpoints: []`) — a missing/null key panics at plugin init (`invalid type: null, expected struct Config`).
6. **Drag-drop:** Wired on `Builder::on_window_event` (not only setup), async `open_inspector` for Windows.

**Why:** Wave 4 must ship working commands against the Wave 3 engines and the staged sidecar without waiting for T6.3 updater config or a mocked AppHandle.

## 2026-10-11 — shadcn-vue + UnoCSS for UI components

**Decided:** Adopt [shadcn-vue](https://www.shadcn-vue.com) (Reka UI primitives, New York style, `neutral` base) as the component library, styled through existing UnoCSS via [`unocss-preset-shadcn`](https://github.com/unocss-community/unocss-preset-shadcn) (`/v3` for `presetWind3`) and `unocss-preset-animations`. Do **not** run `shadcn-vue init` (it pulls Tailwind); keep an empty `tailwind.config.js` only so the CLI can add components. Dark/light/system mode uses VueUse `useColorMode` (`.dark` on `html`). Path alias `@/*` → `./src/*`. App-specific brand green stays as UnoCSS `brand` tokens so it does not collide with shadcn's `accent` semantic color. UI components live under `src/components/ui/`; add more with `npx shadcn-vue@latest add <name>`.

**Why:** The hand-rolled controls are inconsistent. shadcn-vue gives accessible, copy-in components while AGENTS.md keeps UnoCSS (not Tailwind) as the styling layer. The UnoCSS community preset is the supported bridge.

## 2026-10-11 — Encoder layout: resizable Sources | Preview | Options

**Decided:** Encoder chrome is three horizontal panels — **Sources** (left), **Preview** (center), **Options + Encode bar** (right) — using shadcn-vue `Resizable` (Reka `Splitter`) with `auto-save-id` so sizes persist in `localStorage`. Option rows use a shared two-column grid (`field-row` / `label` UnoCSS shortcuts): labels share one column, truncate with ellipsis when the options panel is narrow, and are real `<label>` elements so a click focuses or toggles the control. Full label text stays on `title` for hover.

**Why:** Preview is the primary visual surface and belongs in the wide center; options are secondary and sit in a narrower, user-resizable rail. Fixed CSS grid columns fought that. Aligned, clickable, truncating labels keep the dense option list readable when the rail is squeezed.

## 2026-10-11 — Encoder controls: Switch, Select, ScrollArea, Input

**Decided:** Encoder option booleans use shadcn-vue `Switch` (not checkboxes). Enumerations use a shared `EnumSelect` over shadcn `Select` (portal popover, not native `<select>`). Number/text fields use shadcn `Input`. Scrollable panes (options rail, sources list, preview body) use shadcn `ScrollArea`. `OptionField` associates labels via `for`/`id` + provide/inject so Switch/Select buttons do not double-toggle inside a wrapping `<label>`. Icons in generated UI components import from `@lucide/vue` (not `lucide-vue-next`).

**Why:** Native selects/scrollbars look unstyled in the desktop webview; switches match a denser settings rail; staying on the already-adopted shadcn-vue + Reka stack keeps one control language.

## 2026-10-11 — Sidecar cancel watcher must not hang on success

**Decided:** `run_ktx_collect` finishes its cancel-watcher task with `abort()` on the success/error path, and only `await`s when the cancel flag was set. The watcher loops until `CancellationHandle` is cancelled, so awaiting it after a normal `ktx` exit blocked forever.

**Why:** Inspector stayed on "Loading…" after `ktx info` completed (~40ms); every successful sidecar call hung on the same await. Encode/export share this path.

## 2026-10-11 — ValidationMessage matches info_v0

**Decided:** `ValidationMessage` is `{ id: u32, type, message, details }` per [info_v0](https://schema.khronos.org/ktx/info_v0.json) (`type` is `fatal` | `error` | `warning`). The previous `level` / string `id` shape was wrong; samples had empty `messages` so it never failed until a file with warnings was opened.

**Why:** Inspector failed to parse `barracuda_base_alpha.ktx2` with `invalid type: integer 6030, expected a string`.

## 2026-10-11 — Command args camelCase; errors via Sonner toast

**Decided:** Tauri v2 IPC renames snake_case command parameters to camelCase for the webview, so frontend `call(...)` args use `jobId` (not `job_id`) for `cancel_encode` / `encoded_preview` / `save_encoded`. Transient and command failures surface through shadcn-vue Sonner (`vue-sonner`) toasts in both encoder and inspector windows via a shared `toastAppError` helper; the inline `ErrorBox` and header error banners are removed.

**Why:** Save and encoded preview failed with `missing required key jobId` while the UI kept snake_case keys. Red inline error boxes are a poor fit for multi-window desktop chrome; toasts keep the panels clean and work the same in the inspector.

## 2026-10-11 — Encoder options: hide unavailable, enable-then-reveal, tooltips

**Decided:** Unavailable encoding options are **hidden**, not dimmed with an inline reason. Whole accordion panels hide when every field inside is disabled per `describe_options`. Panels follow enable-then-reveal (master switch, then linked details). Generate/runtime mipmap live in the **Mipmaps / resample** panel with the filter controls. Each option has a short tooltip from the ktx create docs. This replaces the earlier “every option stays visible and disabled with reason” product rule (requirements + development-plan T5.1 `OptionField` note). Rust remains the enablement source of truth; Vue only renders `FieldState.enabled`.

**Why:** A dense options rail full of greyed-out codec and dependency rows is noise. Hiding inactive controls and nesting details under their enable switch matches how users enable features and keeps the rail scannable.

## 2026-10-11 — Declarative option enablement predicates in Rust

**Decided:** `describe_options` enablement is a declarative gate table in `rules.rs`: each `OptionKey` has zero or more `Gate`s (`Pred` + reason). `Pred` is a small closed set (`Raw`, `ScaleSet`, `EncodeBasisLz`, …) composed with `Any` / `Not`. Multiple gates on one key are AND (each keeps its own reason); `Any` expresses OR (e.g. mipmap filters need generate-mipmap **or** scale **or** width **or** height). Vue does not evaluate predicates: it only shows fields when `FieldState.enabled` is true, treats a missing report entry as disabled, and waits for `store.ready` before rendering the options panel. Container `scale` defaults to `1.0` (`Some(1.0)` in Rust / `1` in the store), so filter controls are available on a fresh form because scale is set.

**Why:** The previous `match` buried AND/OR dependencies in ad-hoc branches; a table makes multi-condition visibility explicit and keeps CLI-faithful OR rules (filters) next to AND rules (UASTC RDO sub-options). Shipping the predicate graph to Vue would duplicate the rules engine and break the Rust-owns-enablement decision. The product default for scale is `1`, not unset.

## 2026-10-11 — No placeholders for known option defaults

**Decided:** Known `ktx create` defaults are real values in `CreateOptions::default` (and the matching store interim object), not empty fields with placeholder / “Default (…)” / “tool default” chrome. Applies especially when dependents key off “value is set”. Current defaults from [ktx-cli/create.txt](../ktx-cli/create.txt): `scale` 1; mipmap filter `lanczos4`, filter scale `1.0`, wrap `clamp`; BasisLZ `qlevel` 128, `clevel` 1, RDO thresholds `1.25`; UASTC quality `1`, RDO-L/D/B/S `1` / `4096` / `10` / `18`; ASTC quality `medium`; `threads` = `available_parallelism` (or 1). Leave unset only when the CLI default is “flag omitted” with no numeric/enum default (`width`/`height`/`layers`/`depth`/`levels`, `max-endpoints`/`max-selectors`, encode none, color assign/convert). UASTC RDO sub-flags are stored at their defaults but emitted only when `--uastc-rdo` is on. Documented in [AGENTS.md](../AGENTS.md).

**Why:** Placeholders looked like defaults but left `null`/`None`, so enablement gates and argv disagreed with what the user saw. Real defaults match the tool and keep dependent controls honest.

## 2026-10-11 — Option tooltips from Rust `describe_options`

**Decided:** Help copy for every `OptionKey` and every enum choice lives in Rust (`ktx/option_help.rs`) and is returned on each `FieldState` from `describe_options` as `help` (option meaning) and `choices: [{ value, help }]` (select-item meaning). Vue renders shadcn-vue `Tooltip` via `HelpTooltip` / `EnumSelect`; it does not own a parallel help catalog. UI-only controls (format group, animation fields, supercompression scheme) keep short local strings. Replaces the TypeScript `optionHelp.ts` map and the hand-rolled CSS tooltip.

**Why:** Same reason as enablement: tool knowledge stays next to the CLI rules engine, one source of truth, testable in Rust. Select items need the same catalog so each codec/filter/primaries value can explain itself.

## 2026-10-11 — Native File / Help menu for all actions

**Decided:** Expand the native menu (still owned in Rust `menu.rs`) to File → Open…, Encode, Cancel Encode, Save…, Clear Sources, Export…, Exit, plus Help → Check for Updates…. Open and Check for Updates run in Rust (dialog / updater). Encode, Cancel, Save, Clear Sources, and Export emit `menu://action` with a typed `MenuAction`; the encoder and inspector frontends share the same handlers as the on-screen buttons. Enablement is synced from Vue via `set_menu_item_enabled` so menu items match button disabled state. Shortcuts: Cmd/Ctrl+O open, Enter encode, `.` cancel, S save, Shift+E export, Q exit. Later extended with View → Zoom In/Out / Actual Size / Fit (see inspector layout entry).

**Why:** Desktop apps expose every primary action in the menu bar with accelerators; the previous menu was Exit-only. Keeping definition and dialogs in Rust preserves the “menu is window chrome” decision while job/export state stays in Vue.

## 2026-10-11 — VS Code–inspired light/dark theme tokens

**Decided:** Override shadcn CSS variables and the app bridge tokens in `theme.css` (loaded after `virtual:uno.css`) to match VS Code Dark+/Light+ surfaces: editor `#1e1e1e` / raised `#252526` / borders `#3e3e42` in dark; white / `#f3f3f3` / `#cecece` in light with stronger ink (`#1e1e1e`, `#616161`). `primary` / `ring` use VS Code blue (`#007acc`) so switches and focus rings are accent-colored rather than grey-on-grey. Switch thumbs stay white; inputs/selects use `bg-background` for inset fields. Brand green remains on `brand` for encode/primary app actions only. Preset stays on `neutral` as the structural base; palette ownership is `theme.css`.

**Why:** Stock shadcn neutral reads as flat grey (especially unchecked switches and muted panels); light felt washed. Desktop users expect VS Code–like contrast between chrome, inputs, and active controls.

## 2026-10-11 — Inspector layout: viewer left, scrollable info, View zoom menu

**Decided:** Inspector uses a horizontal resizable split (`inspector-layout`, same reka-ui pattern as the encoder): left = mip/layer/depth controls + fixed-size image viewport + pinned Export bar; right = path/size header + scrollable report panels. Drop the in-window “KTX-GUI Inspector” title (window title is enough). Mip and layer (and depth) are range sliders with the current selection in the label (e.g. `Mip L0 · 2048×2048`). The viewport does not scroll or resize with the image; the image pans by drag and zooms with Ctrl/Cmd+wheel. Inline View 1:1/Fit buttons move to the native View menu (Zoom In/Out, Actual Size, Fit) via `menu://action` (`zoom-in` / `zoom-out` / `zoom-actual` / `zoom-fit`), handled only in the inspector.

**Why:** The previous two-column grid put metadata on the left and scrolled the whole viewer column, so export and zoom chrome floated away from a usable preview. Matching the encoder’s resizable + pinned-bottom pattern keeps export reachable and lets the report grow without shrinking the canvas.

## 2026-10-11 — Shared ImageViewport; encoder compare wipe

**Decided:** Zoom/pan lives in shared `useZoomPan` + `ImageViewport` (checkerboard, Ctrl/Cmd+wheel, drag pan, View menu via `viewerControls`). Inspector and encoder both use it. Encoder preview is no longer side-by-side: source alone until encode succeeds, then a vertical wipe slider (original left / encoded right) with one shared transform so pan and zoom stay locked. Stats stay under the viewport. `menu://action` zoom is handled in both windows but only when `document.hasFocus()` so the unfocused webview ignores app-wide emits. Replaces the “View zoom handled only in the inspector” part of the inspector-layout entry; encode preview in requirements still means “show encoded vs source,” now as a wipe rather than two columns.

**Why:** Matching inspector navigation in the encoder was requested; a wipe is a clearer quality compare than two independently scaled boxes. Sharing the viewport avoids duplicating transform math.

## 2026-10-11 — Compare wipe is frame-clipped; full-res source preview

**Decided:** The encoder wipe clips the original in *frame* space (overflow strip at `splitPct` of the viewport, inner layer as wide as the frame, same pan/zoom transform as the encoded layer). The divisor and the image cut stay aligned when panning. Preview pane loads `source_preview` (full-resolution PNG / tone-mapped HDR), not `source_thumbnail` (128px list thumb). Both compare layers fill a shared max(natural) canvas via `object-fit: fill`.

**Why:** Clip-path on the transformed image moved the cut with the pan while the handle stayed viewport-centered. Using the list thumbnail made the left side a small inset next to a full encoded preview.

## 2026-10-11 — Instant inspector mip/layer switching via dual cache

**Decided:** Inspector subresource previews use a two-layer cache. Rust short-circuits `inspector_image` on an existing disk PNG under the inspector workdir and reuses `TextureMeta` stashed from `inspect` (no per-slider `ktx info` spawn). The Vue inspector keeps object URLs in an in-memory map keyed by path + selection, dedupes in-flight IPC, keeps the previous image visible while a miss loads, prefetches ±1 neighbors, and sequentially warms the mip chain for the current face/layer/depth. FramePlayer preload shares that map.

**Why:** Moving the mip/layer sliders re-ran `ktx info` and cleared the viewport to “Loading…” on every tick even when the PNG was already on disk; FramePlayer’s preload discarded bytes so it paid twice. Disk + memory cache makes revisited selections instantaneous and first visits feel continuous.

## 2026-10-11 — Inspector scrub: coalesce extracts; fit from header dims

**Decided:** Uncached inspector image fetches run at most one at a time. Interactive slider targets coalesce to the latest selection (superseded requests abort); prefetch/mip-warm sit on a lower-priority queue that interactive loads flush. ImageViewer debounces miss-loads and idle prefetch while scrubbing, and applies cache hits immediately. Fit/zoom uses KTX header × mip as `contentWidth`/`contentHeight` so the viewport never flashes 1:1 while `naturalWidth` is still 0; same-size layer/face changes do not reset fit.

**Why:** Fast layer scrub on large arrays (e.g. 1024² × 200) spawned dozens of parallel `ktx extract` processes and locked the machine. Resetting natural size on every `src` change made `fitScale` fall back to 1 until `@load`, so the image briefly painted full-res.

## 2026-10-11 — Transcode target from DFD model, not sample count

**Decided:** Choose `ktx extract --transcode` from unpacked channel count derived from the DFD color model and channel types (`unpacked_channels_from_dfd`), not from `samples.len()`. ETC1S `RGB` (+ optional `AAA`) maps to 3/4 channels; UASTC `RGBA`/`RGB` likewise. RGBSDA keeps one sample per channel.

**Why:** BasisLZ DFD packs RGB into one sample. Counting samples selected `r8` (solid-red normals) or `rg8` (yellow color, blue dropped). Confirmed against `headphones_norm.ktx2` / `headphones_color_3.ktx2` by comparing CLI extracts.

## 2026-10-11 — Source-aware format defaults; never offer known-fail encode combos

**Decided:**

1. **Hard rule:** The GUI never allows a sources+options combination the app already knows will fail `ktx create`. Rust `describe_options` filters choices and disables fields; when the current options are invalid for the sources it returns an `OptionSuggestion` that Vue must apply; encode validates again and refuses. Documented in [AGENTS.md](../AGENTS.md) and [requirements.md](requirements.md).
2. **HDR sources** (EXR / Radiance): default / suggest `R16G16B16A16_SFLOAT`; only `R16G16B16_SFLOAT` and `R16G16B16A16_SFLOAT` appear; ASTC and `--encode` (BasisLZ / UASTC LDR) are hidden — UASTC HDR is deferred on the v4.4.2 pin.
3. **LDR sources:** keep `R8G8B8A8_SRGB` as the default; hide half-float formats (they mismatch PNG/8-bit inputs).
4. **Mixed LDR+HDR** in one encode is refused (warning + encode blocked). Superseded for product wording by the 2026-10-11 batch outputs entry below: multi-image drop defaults to a **batch of KTX outputs**; packing into one file uses the Inputs role (array layers / cubemap / …) on a selected output.
5. **Tone-map HDR→LDR for compression** is out of scope; stay aligned with what `ktx create` accepts.
6. Split: Rust owns compatibility, filtered choices, and suggestion; Vue applies the suggestion when sources/options change and only renders allowed choices.

**Why:** Dropping an EXR while format stayed on `R8G8B8A8_SRGB` produced a fatal `exr_float` vs 8-bit UNORM from the sidecar. Users should not need to know Vulkan format / input type pairing; the app already classifies `SourceKind` and was ignoring `sources` in `describe_options`.

## 2026-10-11 — Source list changes invalidate encode result

**Decided:** Adding, removing, reordering, or clearing sources resets encoder `job` state to idle (unless a job is currently running). The center preview and Save affordance follow that state, so a stale encoded preview cannot outlive the source list that produced it.

**Why:** After a successful encode, deleting a source cleared the source layer but left `job.status === "done"` and the encoded object URL, so the viewport kept showing the old result until the next encode.

## 2026-10-11 — Hard-block convert-tf / assign-tf vs sRGB format

**Decided:** `*_SRGB` vs transfer-function pairing is a hard rules-engine constraint, not a soft warning. For `*_SRGB` formats, `--convert-tf` choices omit `linear` and `--assign-tf` only lists sRGB aliases; for UNORM/ASTC UNORM formats that have an sRGB twin, `--convert-tf` only offers `linear`. Invalid current values are cleared via `OptionSuggestion` (`convertTf` / `assignTf`); `validate` refuses encode. Soft warnings remain for lossy sRGB→linear and encode color-space advice.

**Why:** The GUI let users pick `--convert-tf linear` with `R8G8B8A8_SRGB`, which `ktx create` fatals on. Same “never offer known-fail combos” rule as source/format pairing.

## 2026-10-11 — Batch KTX outputs + role-typed inputs

**Decided:** Replace the ambiguous single Sources list with:

1. **Top-left:** queue of KTX **outputs** (batch). Add / delete / select; no reorder. Drop → one new output per image.
2. **Bottom-left:** **Inputs** for the selected output, with an **InputRole** selector (`Base2D`, `ArrayLayers`, `CubemapFaces`, `MipChain`, `DepthSlices`). List length is the arity; Container `--layers` / cubemap / depth / (mip-chain) levels are derived, not free-typed. Compound shapes (cube arrays, etc.) deferred.
3. **Center drop** = new outputs (same as top). Panel-targeted drops only; never silently append to layers from chrome.
4. **Shared** `CreateOptions` (+ animation when role is ArrayLayers) for the whole queue. No per-output options, copy/paste, or apply-all. Reject adding an output incompatible with the first (LDR/HDR).
5. **Encode all** with limited parallelism (cap 2), queue modal (`Done i/N`, Cancel queue), Save / Save all.
6. Preview and `describe_options` bind to the selected output’s inputs; revoke object URLs on selection change.

Closes the open “batch vs single-KTX” note from the source-aware format defaults entry.

**Why:** Dropping many albedos was treated as one `ktx create` with 1×1×1×1 expected inputs → fatal. The UI looked like a batch queue but packed every file into one KTX. Splitting outputs vs role-typed inputs makes arity structural and matches artist batch workflows while keeping array/cubemap/mip/depth packing explicit.

## 2026-10-11 — Per-panel drop targets; Inputs on demand; preview drop dialog

**Decided:** Refine the batch-outputs UX:

1. **Drop highlight** is per panel (Outputs / Inputs / Preview), not a full-window green overlay. Zone is committed on the webview `drop` event so routing still works after the highlight clears.
2. **Outputs drop** → one KTX output per image. **Inputs drop** → append to the selected output. **Preview drop** → dialog: separate outputs, or pack into one KTX with a pack role (array layers / cubemap / mip / depth).
3. **Inputs panel** is hidden for single-image outputs. Each Outputs row has **+** to open Inputs and add more images for that output. Role UI drops **Base 2D** (implicit when Inputs is hidden); pack roles remain.
4. Internal `InputRole::Base2D` stays for encode/rules when every output is a single image.

**Why:** Full-window green made targets ambiguous. Base 2D in the role list was noise for the default batch case; packing should be an explicit choice (panel, +, or preview dialog).

## 2026-10-11 — Role-aware Preview: pack sliders, cubemap cross wipe, generated mips

**Decided:** Encoder Preview browses **inputs** by `InputRole` immediately (layer / mip-chain / depth sliders; cubemap as a classic cross with no face slider). After encode, the same frame wipes input vs extract. When the KTX has generated mips (`levels > 1` and role is not `MipChain`), add a **Mip** slider: left stays full-res input(s), right is the extracted mip for resolution-loss compare. Cubemap uses one canvas-composited cross per side under a **single** wipe. `encoded_preview` takes `ImageSelection` and reuses the job preview-cache / `extract_preview_png`. CSS3D depth volume deferred to a follow-up plan.

**Why:** Preview only showed `inputs[0]` vs L0/F0; packing roles were invisible until the inspector. Artists need to verify layers/mips/faces before and after encode without leaving the encoder, and to judge mip filter quality with the same wipe.

## 2026-10-11 — DepthSlices CSS3D volume preview

**Decided:** For `InputRole::DepthSlices`, Preview shows an oblique CSS 3D stack (`perspective` + `transform-style: preserve-3d`) of every source slice plane. The existing **Depth** slider highlights the active plane and drives a separate 2D wipe of that slice vs `encoded_preview` (highlighted-slice wipe, not a duplicated/clipped volume). No WebGL. Slice bitmaps still come from `source_preview` / `encoded_preview`. Generated-mip slider stays off for this role (3D + generate-mipmap is invalid). Cube arrays and raymarched volumes stay out of scope.

**Why:** A single 2D depth slider hid the volume shape. A CSS stack is enough for spatial overview without a second rendering stack; keeping wipe on one slice matches the existing ImageViewport compare and avoids flattening issues from clipping a preserve-3d scene.

## 2026-10-11 — Sticky View zoom mode (Fit / Actual / manual)

**Decided:** `useZoomPan` keeps an explicit sticky mode: **Fit**, **Actual** (1:1), or **manual** (wheel / Zoom In/Out). View menu Fit and Actual set the mode and reset pan; they stay in force across output selection, layer/mip/depth scrubbing, and `ImageViewport` remounts (window-scoped shared state). Content size / `src` changes update pixel dimensions only — they never force Fit. In Actual mode, mip changes shrink the on-screen image (header/mip `contentWidth`/`contentHeight` own the canvas; decode sizes must not `max()` expand it). Manual mode keeps its scale factor. A later Fit/Actual choice or a free zoom replaces the previous rule.

**Why:** Scrubbing layers/mips called `zoomFit()` on every size change, wiping the user's zoom. Fit/Actual from the View menu need to behave like persistent sizing rules, and Actual size is only useful for mip review if smaller levels actually appear smaller.

## 2026-10-11 — Inspector CSS3D depth volume

**Decided:** When an inspected KTX has `pixelDepth > 1` at the selected mip, the inspector viewer shows the shared `DepthVolumeView` stack (same CSS3D oblique planes as encoder DepthSlices Preview) above the existing 2D `ImageViewport` for the highlighted slice. All depth planes for the current level/face/layer load through `inspector_image` via a non-coalescing `ensureInspectorImageUrl` path so scrub coalescing does not drop sibling slices. `DepthVolumeView` lives under `src/shared/components`.

**Why:** Encoder Preview already spatialized 3D packs; opening the same (or any) 3D KTX in the inspector still showed only one 2D slice. Reusing the shared volume view keeps both windows consistent without a second rendering stack.

## 2026-10-11 — Volume orbit + shared cubemap cross

**Decided:** `DepthVolumeView` supports click-drag orbit (`useOrbitDrag`: rotateX / rotateZ with clamped pitch; double-click resets). Cubemap cross compositing (`composeCubemapCross`) lives in `src/shared` and is used by both encoder Preview and inspector when `faceCount > 1`. Inspector face buttons still drive export / selection; the viewport shows the full cross, not a single face.

**Why:** Artists need to tumble the depth stack to read shape, and cubemap layout should match across windows so packing and inspection use one mental model.

## 2026-10-11 — Bound preview RAM: scrub coalesce + LRU + one extract slot

**Decided:**
1. Encoder wipe scrub uses `createScrubGate` (latest-only, single-flight) for source and encoded sides — same coalescing idea as inspector `loadInspectorImageUrl`, so slider ticks do not start a `source_preview` / `encoded_preview` per intermediate frame.
2. Inspector in-memory object-URL map is an LRU soft-capped at 16 entries; active viewport / depth-volume planes are refcount-pinned so eviction cannot revoke displayed blobs. Rust disk PNG cache stays the unbounded fast path.
3. `run_extract` takes a process-wide semaphore of 1 so even a buggy caller cannot spawn parallel UASTC/Zstd `ktx extract` workers.

**Why:** Scrubbing a ~400×1024² array after UASTC+Zstd encode stampeded extracts (right wipe side never caught up) and ballooned RAM to tens of GB. Speed stays on the disk cache + one hot extract; webview RAM stays proportional to pinned/recent frames, not layer count.

## 2026-10-11 — Safe ktx input staging; encode failures via alert + OS notify

**Decided:**
1. `prepare_input` stages basenames that are unsafe for `ktx create` (`,` path lists, leading `@` response files) into the job `inputs/` dir under a hashed `input_<hash>.{png,exr}` name. Converted JPEG/etc. always use that safe name (never the original stem). Safe PNG/EXR still passthrough.
2. Encode failures surface as **one** formatted message via a native dialog alert (`dialog.message`) **and** a native OS notification. No Sonner toast for encode failures, and no inline Encode panel error + stderr duplication. Soft failures elsewhere may still use `toastAppError`.

**Why:** `ktx create` truncates input paths at commas, so files like `…06_08_2026, 18_33_22.jpg` failed with “No such file”. The Encode bar showed `ktx exited with code N` plus a stderr dump while a toast repeated a summary — three surfaces for one failure.

## 2026-10-11 — Product rename to KTXForge

**Decided:** The product is **KTXForge** (not `KTX-GUI`). Package / crate names are `ktxforge` / `ktxforge_lib`; Tauri `productName` is `KTXForge`; identifier is `com.monogrid.ktxforge`; planned repository is `https://github.com/MONOGRID/KTXForge`. Window titles and in-app header use KTXForge / KTXForge Inspector. README describes the app as a KTX GUI that exposes the full encode and inspect surface, with described screenshot placeholders under `docs/screenshots/`. This supersedes the product naming in the 2026-10-04 GitHub / identity entry.

**Why:** A distinct product name for the desktop GUI, separate from the generic “KTX GUI” label used during scaffolding.

## 2026-10-11 — Dual brand masters for app icons

**Decided:** Brand masters live at `assets/icon-simple.png` (anvil + flame + KTX) and `assets/icon-full.png` (adds FORGE wordmark and pixel sparks). `npm run icons` (`scripts/generate-icons.mjs`) resizes them into `src-tauri/icons` and `public/`: sizes **&lt; 128px** from simple, **≥ 128px** from full (ICO layers 16–64 simple / 256 full; ICNS PNG types likewise). Webviews link `public/favicon.ico` + `favicon-32.png`. Regenerating needs `npm install --no-save sharp to-ico @fiahfy/icns` (not permanent deps).

**Why:** `tauri icon` takes one source; the full mark illegibly crushes at taskbar/tray sizes, while the simple mark underuses large dock/installer surfaces.

## 2026-10-11 — Wave 6 CI/release and updater signing key

**Decided:** Ship Wave 6 as `.github/workflows/ci.yml` + `release.yml`, `.releaserc.json`, `scripts/set-version.mjs` / `release-outputs.mjs`, and updater config in `tauri.conf.json` (`createUpdaterArtifacts`, pubkey, `releases/latest/download/latest.json`, Windows `passive`, macOS ad-hoc `-`). Generate the minisign keypair with `tauri signer generate` for this repo; commit only the public key; store the private key as GitHub secret `TAURI_SIGNING_PRIVATE_KEY` (no password). Integration branch is `dev` (renamed from `master`); `main` is created locally for the post–Wave 7 ship merge and is not pushed until then so an empty-repo push does not publish `1.0.0` early.

**Why:** Unblocks CI and the automatic release pipeline on `MONOGRID/KTXForge` while keeping the first public release gated on Wave 7 acceptance.

## 2026-10-11 — Windows menu accelerator fallback via keydown

**Decided:** Keep native muda accelerators on the File/View menu items for display and for macOS/Linux. On Windows only, bind a capture-phase `keydown` listener (`bindWindowsMenuShortcuts`) that maps the same Ctrl chords to menu ids and invokes `dispatch_menu_action`. That command checks the live menu-item enabled flag, then runs the shared `handle_menu_id` path used by `on_menu_event`. Matcher coverage is locked by `npm run test:menu-shortcuts`.

**Why:** WebView2 swallows keyboard input before Win32 `TranslateAcceleratorW`, so menu clicks work but the accelerators next to them do nothing (wry#451 / tauri#6365). A JS fallback is the established workaround; routing through Rust keeps enablement and Open/Exit/updater behavior in one place.

## 2026-10-11 — Expand Darwin pkgs with pkgutil; follow soname symlinks

**Decided:** `scripts/fetch-ktx.mjs` expands Darwin `.pkg` assets with `pkgutil --expand-full` only. It does not use `7z` for those packages. Linux and macOS library lookup follows soname symlinks (`libktx.so.4`, `libktx.4.dylib`) and matches the path suffix under the archive prefix (`KTX-Software-…/` or a component `Payload/`). This replaces the “or `7z x` the pkg” option in [ktx-cli/staging.md](ktx-cli/staging.md).

**Why:** CI failed on v4.4.2. `7z x` stops at the xar, so `usr/local/bin/ktx` stays inside a gzip+cpio Payload. The Linux tarball nests files under `KTX-Software-4.4.2-Linux-x86_64/`, and `lib/libktx.so.4` is a symlink to `libktx.so.4.4.2`. A walk that only accepts regular files never sees that name, which is the `NEEDED` entry.

## 2026-10-12 — Git LFS for binaries; SVGs stay in Git

**Decided:** Brand/Tauri icons and test-fixture binaries (*.png / *.jpg / *.ktx / *.ktx2, plus other binary patterns in .gitattributes) are Git LFS. *.svg is not LFS-tracked. Every ctions/checkout in ci.yml and elease.yml sets `lfs: true`. After converting blobs to pointers, push LFS objects with `git lfs push --all origin` if a normal push skipped them.

**Why:** Icon LFS blobs never reached GitHub (clone 404). Without `lfs: true`, CI saw pointer text and failed with Invalid PNG signature. Fixtures were committed as raw blobs while LFS rules applied (`should have been pointers`). Docs SVG placeholders are small text and do not belong in LFS.

## 2026-10-12 — Auto-install Rust check tools on new machines

**Decided:** Add `scripts/ensure-tools.mjs` and `npm run setup` / `npm run ensure-tools`. On a fresh checkout, `setup` installs `rustfmt` and `clippy` via rustup when missing, installs `cargo-machete` via `cargo install --locked` when missing, then runs `fetch-ktx`. Wire tool install + fetch into `precheck:rust` only. `pretauri` runs `fetch-ktx` alone (host triple). CI quality-gate jobs drop the explicit `fetch-ktx` step; release matrix builds keep `fetch-ktx -- --target <triple>` because tauri-action’s `npm run tauri` prehook cannot see the matrix target.

**Why:** A clean machine failed `npm run check` at `cargo machete` (`no such command`) after tests passed, and earlier failed Tauri builds without a staged sidecar. Putting `ensure-tools` on `pretauri` would compile machete on every release OS/arch build; release `check` had no machete step before, so `precheck:rust` also closes that gap.

## 2026-10-12 — Updater private key in 1Password Tech vault

**Decided:** Source of truth for the Tauri updater minisign private key is 1Password vault **Tech**, item **KTXForge Tauri updater signing key** (`notesPlain`; no key password). Commit only the public key in `tauri.conf.json`. Mirror the private key into GitHub secret `TAURI_SIGNING_PRIVATE_KEY` for Actions. Developers load the key via `op read` / `npm run tauri:build:signed` (`scripts/with-updater-signing-key.mjs`) and do not keep `~/.tauri/*.key` copies across machines. Document the flow in [updater-signing.md](updater-signing.md). Share access by granting Tech vault membership to tech leads, not by distributing key files.

**Why:** The previous GitHub-only secret could not be recovered for local signed builds or multi-PC setups. A shared vault keeps one recoverable copy for leads without initializing or syncing key files on every Windows/Linux environment.
