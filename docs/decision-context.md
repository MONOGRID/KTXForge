# Decision context

Architectural decisions for this app: why the code is shaped the way it is, and what forced it. Product behavior lives in [requirements.md](requirements.md). Agent coding rules live in [AGENTS.md](../AGENTS.md). Individual fixes belong in the git log.

Read the whole log before choosing or changing an approach.

**What belongs here:** a choice between real alternatives, a constraint that explains code someone would otherwise "simplify" (a tool bug, an OS quirk, a format rule), or a rule that spans modules. A bug fix, a UI tweak, or a default value does not belong here unless it changes one of those.

**How to change it:** when a new decision touches an existing entry, edit that entry so it describes the current state, and add a dated line to its **Revised** list saying what changed and why. A decision with no matching entry gets a new dated entry at the bottom. Each entry gives the date, what was decided, and why.

## 2026-10-04 — Embed the official `ktx` CLI

**Decided:** Ship the official `ktx` binary from [KTX-Software](https://github.com/KhronosGroup/KTX-Software) as a Tauri sidecar and drive it from Rust. Do not replace it with a crates.io KTX or Basis crate. Rust wraps the CLI where the tool stops:

- Convert JPEG, WebP, BMP, GIF, TIFF and Radiance HDR to PNG or EXR before `ktx create`. The CLI reads only PNG, EXR and raw.
- Encode JPEG on export. `ktx extract` writes only PNG, EXR or raw.
- Run `ktx create` / `ktx extract` off the UI thread and report exit code, stderr and progress.
- Parse `ktx info --format json` ([Khronos info schema](https://schema.khronos.org/ktx/info_v1.json)).
- Write the `KTXanimData` key after create (for BasisLZ, between `ktx create` and `ktx encode`). `ktx create` has `--layers` and no flag for that key.

**Why:** The spec requires every `ktx create` option, the official info report and validation, and extract per mip, face, layer and frame. The CLI already does all of that. In the 2026-10-04 crates.io survey each crate covered only part of it:

| Crate | Covers | Missing |
| --- | --- | --- |
| `ktx2` 0.5 | Pure-Rust KTX2 reader (header, DFD, key/value, mip index) | Writer, pixel decode |
| `ktx2-rw` 0.2 | Static libktx v4.4.0, create/transcode/write | ASTC block sizes, UASTC HDR, Zstd/Zlib levels, TF/primaries, mip filters, swizzle, PSNR/SSIM |
| `libktx-rs` 0.3 | libktx bindings | Five years behind KTX-Software |
| `basis-universal` 0.3 | ETC1S / UASTC LDR encode | UASTC HDR, ASTC encode, DFD, validation report |
| `basisu` 0.1 | Pure-Rust transcoder | Encoder |

Binding the missing surface from libktx means reimplementing `ktx create`'s color-conversion rules, format constraints and warnings. That duplicates the CLI and needs CMake plus a C++ build on every release target. The `ktx2` crate is still used for reading KTX2 metadata in Rust.

**Revisit when:** a maintained binding exposes the full create/info/extract surface against a current KTX-Software; or spawning processes and writing temp files blocks a requirement; or Khronos stops shipping a per-OS `ktx` executable.

## 2026-10-10 — Stay on KTX-Software v4.4.2 (defer CLI gaps)

**Decided:** Pin the embedded tool to v4.4.2 (`ktx-version.json`). Option types, argv and the rules engine are frozen against the captured help in [ktx-cli/](ktx-cli/). Gaps against [requirements.md](requirements.md) are deferred, not exposed in types or UI, until a deliberate pin bump:

- UASTC HDR codecs (`uastc-hdr-4x4`, `uastc-hdr-6x6i`) and every `--uastc-hdr-*` flag. HDR sources therefore cannot use `--encode`.
- Codec name `uastc-ldr-4x4`. v4.4.2 spells it `--encode uastc`.
- `--premultiply-alpha`.
- `ktx convert`. KTX1 → KTX2 uses the legacy `ktx2ktx2` binary (see sidecar packaging).
- Half-float `ktx extract --transcode` targets. Only `r8|rg8|rgb8|rgba8` exist.

v4.4.2 spellings that look wrong are real and stay: ASTC `--format` values carry the Vulkan `_BLOCK` suffix. The help text's `pg_eotf` and `abobergb` are typos; the parser rejects them and accepts `pq_eotf` and `adobergb`, which the app uses. `ktx extract` fails on an explicit `--layer 0` for a non-array texture, so `build_extract_args` omits `--layer` / `--face` / `--depth` (and `--level` for a single-level file) when the dimension is 1.

**Why:** 5.0.0 was a release candidate when the pin was set; the user chose a stable pin over the missing options. Names that only exist in 5.0 would fail at runtime. The gap list came from comparing every UI option with `ktx create --help` from the official 4.4.2 build.

**Revised:** 2026-10-14 — `pq_eotf` / `adobergb` replace the help-text typos after the e2e sweep showed `ktx create` rejects them.

## 2026-10-04 — Rust owns the work; Vue renders

**Decided:** Rust does every conversion, process spawn, parse and image preparation. Vue renders controls, previews, progress and inspector data, calls Tauri commands and listens for events. Consequences in code:

- Rust → TS payload types come from `ts-rs` into committed `src/bindings/`. `build.rs` sets `TS_RS_EXPORT_DIR` to an absolute path via `cargo:rustc-env`, because the relative path in `.cargo/config.toml` alone missed when the Cargo target dir is relocated (CI, sandbox).
- Previews cross IPC as PNG bytes in `tauri::ipc::Response`, not base64 or file URLs.
- Open/save dialogs, the native menu, window chrome and the updater are Rust-side.
- Two Vite HTML entries: `index.html` (encoder, window `main`) and `inspector.html`.
- Async commands never block a runtime worker: image decode, conversion, thumbnails, EXR tone-mapping, JPEG export and `KTXanimData` writes go through `run_blocking`; file I/O uses `tokio::fs`; dialogs use the callback API awaited over a channel, not `blocking_*`.

Frontend rules (strict TS, ESLint strict-type-checked, UnoCSS, VueUse first) are in [AGENTS.md](../AGENTS.md).

**Why:** Same rules engine and tool knowledge for UI and argv, testable in Rust, and no frontend duplicate that can drift. A blocking call inside an async command holds a worker for the whole decode or dialog.

## 2026-10-11 — Sidecar packaging: `ktx`, `ktx2ktx2`, `libktx`

**Decided:** `scripts/fetch-ktx.mjs` downloads the pinned release and stages `ktx` and `ktx2ktx2` as `bundle.externalBin` sidecars, plus the shared library through per-OS Tauri configs:

- **Windows:** `tauri.windows.conf.json` puts `binaries/ktx.dll` in `bundle.resources`, next to `ktx.exe`.
- **Linux:** `tauri.linux.conf.json` puts `libktx.so.4` in `bundle.resources` (lands in `/usr/lib/KTXForge/`); `fetch-ktx` sets the sidecar runpath to `$ORIGIN:$ORIGIN/../lib/KTXForge`. A missing `patchelf` is an error.
- **macOS:** `tauri.macos.conf.json` puts `libktx.4.dylib` in `bundle.macOS.frameworks`; `fetch-ktx` adds rpath `@executable_path/../Frameworks` with `install_name_tool` and re-signs the sidecar ad hoc.

The rpath step is idempotent and runs on already-staged binaries. Darwin `.pkg` assets are expanded with `pkgutil --expand-full`, not `7z`. Library lookup follows soname symlinks (`libktx.so.4` → `libktx.so.4.4.2`) and matches path suffixes under the archive prefix. Layout: [ktx-cli/staging.md](ktx-cli/staging.md#installed-layout).

**Why:** `externalBin` copies executables only. A Linux .deb/.rpm without the library made both sidecars exit 127; the AppImage only worked by picking up a host copy. Resources are Tauri's supported path for non-executables on Windows and Linux, and `/usr/lib/<productName>/` is the per-app directory Tauri owns. On macOS the bundler signs only frameworks and binaries, so a dylib in `Contents/MacOS` breaks signing and arm64 loading. `7z x` stops at the xar and leaves the binary inside a gzip+cpio Payload. The soname a binary `NEEDED`s is a symlink, so a walk over regular files never finds it.

**Revised:** 2026-10-12 — added `ktx2ktx2` (KTX1 preview/export only worked from a dev checkout) and shipped `libktx` in every installer.

## 2026-10-10 — Sidecar process lifecycle and temp workspace

**Decided:**

- `sidecar::run_ktx` maps a non-zero exit to `AppError::Ktx`. `run_ktx_collect` returns `KtxOutput` for any exit code, so `ktx info` JSON from an invalid file still reaches the inspector.
- Cancellation is a shared `CancellationHandle`. Sidecar children are tracked by PID: a cancelled, failed or dropped run kills its child, and `RunEvent::Exit` kills any survivors and drops the temp workspace.
- An encode job belongs to its queue output (`EncodeRequest.outputId`). Re-encoding that output, or `discard_outputs`, cancels the old job and deletes its directory.
- `job_id` comes from the webview, so `Workspace::job_dir` / `remove_job` accept only a parsed `uuid::Uuid` before any filesystem join.
- Anything that runs a sidecar in tests goes through the app's own resolution: the e2e driver sits next to the `ktx` that `tauri-build` copies into the target dir, and a missing sidecar fails the build.

**Why:** Tauri exits without running destructors, so every session left its temp directory behind and quitting mid-encode left `ktx` running. A plain `root.join(job_id)` lets an absolute path escape the workspace. Test helpers that returned early reported green without running; some looked only for the Windows binary.

**Revised:** 2026-10-14 — `test_support` removed with the unit tests; see the end-to-end suite entry.

## 2026-10-11 — Stage inputs before `ktx create`

**Decided:** `prepare_input` copies or converts every source into the job's `inputs/` dir when the original cannot go to `ktx create` as-is:

- Basenames with `,` or a leading `@` are staged as `input_<hash>.{png,exr}`. Converted files always use that safe name.
- Radiance `.hdr` is written as 32-bit float **scanline** EXR with the `exr` crate (ZIP16, increasing Y). Tiled `.exr` inputs are re-staged as scanline, keeping layers, channels and attributes (largest level only). Scanline PNG/EXR passes through.
- EXR is read with the `exr` crate, not `image`'s OpenEXR decoder.

**Why:** `ktx create` splits input paths at commas and treats `@file` as a response file. The Linux v4.4.2 `ktx` segfaults (exit 139) on any tiled EXR, and `image` always writes tiled RLE, so every Radiance encode crashed on Linux. `image` also rejected EXRs with unsorted channels that `exr` reads.

**Revised:** 2026-10-12 — scanline EXR staging.

## 2026-10-10 — Rust writes `KTXanimData`

**Decided:** After `ktx create`, Rust rebuilds the KTX2 to insert `KTXanimData` (`duration`, `timescale`, `loopCount`; frames are array layers). It refuses `layerCount < 2`. The rebuilt file lists mip data smallest → largest (level index still base-first), aligns each level to `lcm(block, 4)` uncompressed or 1 with supercompression, and pads to 8 only before SGD when SGD exists. The e2e suite runs `ktx validate` on the result.

A BasisLZ animation is encoded in two steps (`create_args::encodes_as_video`): `ktx create` with every flag except the codec ones writes uncompressed frames, Rust inserts the key, then `ktx encode --codec basis-lz` with the codec flags writes the output. UASTC and uncompressed animations keep the single `ktx create`.

**Why:** The CLI has no flag for the key. Largest-first levels and unconditional 8-byte padding failed validation with `error-4004`. libktx treats an ETC1S array that carries `KTXanimData` as video: the encoder then writes P-frames and the transcoder decodes the payload as a video bitstream. Adding the key after a plain-array encode left a file `ktx extract` could not transcode (`error-5007`). `ktx encode` loads the key from the input, so the payload is encoded as video.

**Revised:** 2026-10-12 — KTX2-valid level layout. 2026-10-14 — BasisLZ animation via `ktx encode` after the key is written.

## 2026-10-11 — Preview extraction: transcode choice, caching, concurrency

**Decided:**

- `ktx extract --transcode` target comes from the channel count implied by the DFD color model and channel types (`unpacked_channels_from_dfd`), not from the sample count.
- KTX1 files are converted once with `ktx2ktx2` into the inspector cache before extract.
- Two cache layers. Rust keeps extracted PNGs on disk and short-circuits `inspector_image` on a hit, reusing `TextureMeta` from `inspect` (no `ktx info` per slider tick). The inspector keeps object URLs in an LRU soft-capped at 32 entries; displayed viewport and depth-volume planes are refcount-pinned so eviction never revokes them.
- Process-wide extract semaphore sized by `extract_slots_for_cores` (~80% of logical cores, 1–16), exposed via `extract_concurrency`; the inspector fetch pool uses the same formula. Disk hits skip the semaphore.
- Scrubbing is latest-only: encoder sides use `createScrubGate`, the inspector coalesces interactive requests to the latest selection. After the viewport settles, idle warm enqueues the mip chain and the full layer / frame stack, nearest first, into spare slots.
- Fit and zoom use KTX header dimensions × mip, not the decoded image's `naturalWidth`.
- Only float formats (`SFLOAT` / `UFLOAT`) preview through EXR; `ktx extract` writes 16-bit UNORM as a 16-bit PNG. The EXR tone-map reads whatever R / G / B / A channels the file has (missing colour channels are 0, missing alpha is 1, as in the tool's PNGs for R and RG formats).

**Why:** BasisLZ packs RGB into one DFD sample, so counting samples picked `r8` / `rg8` and produced red normals or dropped blue. Scrubbing a 1024² × 200–400 layer UASTC+Zstd array spawned dozens of parallel extracts, locked the machine and used tens of GB. A single slot fixed that but left scrubbing at ~10% CPU; a fixed 3 left wide hosts idle. Using `naturalWidth` made Fit fall back to 1:1 for a frame on every `src` change. Matching `R16G16B16` / `R32` as "HDR" sent 16-bit UNORM to the EXR reader, and an RGB-only EXR reader rejected single-channel float files.

**Revised:** 2026-10-11 — one extract slot plus LRU 16 after the RAM blow-up. 2026-10-12 — 3 slots, then core-scaled slots, LRU 32, and idle warm of the full stack. 2026-10-14 — EXR only for float formats; any-channel EXR tone-map.

## 2026-10-11 — The options rules engine lives in Rust

**Decided:** All `ktx create` option knowledge is in Rust (`ktx/rules.rs`, `ktx/option_help.rs`, `ktx/presets.rs`):

- `rules::resolve` is the only place rules apply. It applies the output's input role to a copy of `CreateOptions`, then, looping to a fixed point, corrects values that only fail for the source size (`fit_values`) and clears every field a gate or source rule disables (HDR clears `--encode`, which disables BasisLZ fields). Each correction becomes an info warning in the report.
- Enablement is a declarative table: each `OptionKey` has `Gate`s (`Pred` + reason). Several gates are AND; `Any` is OR (mip filters need generate-mipmap **or** scale **or** width **or** height). Predicates see the options plus a `Context` (source class, role, level-0 size of the first input), so size rules such as "1D needs a 1-px-tall result" are gates too.
- `describe_options` reports per-field `enabled`, filtered `choices`, `help` per option and per choice, an `OptionSuggestion` when current values are invalid for the sources, and the source-aware `default_format`.
- `rules::resolve_for_encode` returns `ResolvedOptions` (its only constructor), and `build_create_args` accepts only that and emits every set field with no conditionals. A value hidden in the UI is never sent; it stays in the store and returns if the field reappears.
- Vue only renders: it shows a field when `FieldState.enabled` is true, treats a missing entry as disabled, applies suggestions, and keeps no predicate, help catalog, role mirror, or arity/compatibility logic. Unavailable options are hidden, not dimmed.

A staged-binary test runs resolved argv for every role and several hidden-value cases through the real `ktx create`.

**Why:** Rules had spread across five places and drifted. Enabling SSIM under UASTC and then setting encode to none hid the field but still emitted `--compare-ssim`, which `ktx create` rejects. A TS role mirror wrote `--levels N` into persistent options, so leaving Mip chain broke later single-image encodes. A predicate graph shipped to Vue would duplicate the engine.

**Revised:** 2026-10-11 — gate table; help text moved from TS to Rust; hide instead of dim. 2026-10-12 — single `resolve` pass and `ResolvedOptions`. 2026-10-14 — gates see a source `Context`; `fit_values` corrections inside the fixed-point loop.

## 2026-10-11 — Never offer a known-invalid encode

**Decided:** The GUI never lets the user select or keep a combination of sources and options the app knows `ktx create` will reject. The rules engine filters or clears it, Vue applies the suggestion, and `resolve_for_encode` refuses as a backstop. Hard errors remain only where omitting a flag cannot fix the input. Current rules:

- **Source vs format:** HDR sources offer only `R16G16B16[A16]_SFLOAT` and no ASTC or `--encode`; LDR sources hide half-float formats. LDR and HDR never mix in one output.
- **Transfer function vs format:** `*_SRGB` formats drop `--convert-tf linear` and list only sRGB aliases for `--assign-tf`; UNORM formats with an sRGB twin offer only `linear`. `VkFormat` has typed `is_srgb` / `has_srgb_twin` / `is_astc` / `is_hdr_float`.
- **Packs:** `validate_pack_inputs` returns an `InputValidationReport` (panel-level and per-input errors): role arity, same LDR/HDR, same level-0 size for array/cubemap/depth, square cubemap faces, mip-chain sizes `max(1, base >> i)` and count ≤ `floor(log2(max(w, h))) + 1`. Vue marks the Inputs panel, rows and Outputs queue; Encode all requires every report to be clean.
- **Resize and mips:** `--scale` is hidden while `--width` / `--height` are set (scale defaults to 1, so it would always clash). Cubemap faces resize only with `--scale`. `--levels` needs `--generate-mipmap` (otherwise `ktx` expects one input per level) and is capped at the resampled size's level count; a mip chain that would not fit its resampled size drops the resize. A `--scale` that shrinks the input to 0 px is dropped. `--mipmap-filter-scale` is raised to 1: below that the resampler can find no samples for a pixel and fails, depending on filter and size.
- **1D:** only when the resampled first input is 1 px tall, and not with `--encode` or ASTC (ASTC refuses 1D; BasisLZ / UASTC write a file `ktx validate` and `ktx extract` reject).
- **Codecs:** `--encode` is hidden and cleared (also by the suggestion) for ASTC formats. `--normal-mode` needs two or more channels (libktx refuses R8). The BasisLZ endpoint / selector limits and RDO thresholds are hidden while that RDO is disabled. Compare metrics are hidden for ASTC 3D textures.
- **Color:** `--normalize` needs a linear output transfer function. `--convert-tf` and `--convert-primaries` need an `--assign-tf` that `ktx` can decode (linear, sRGB, ITU family, PQ EOTF); `--convert-primaries` also needs a linear or sRGB output and, for HDR (EXR carries no primaries), `--assign-primaries`. With `--convert-tf` set, only decodable `--assign-tf` values are offered. Without it, ASTC UNORM formats accept only `--assign-tf linear` and formats with an sRGB twin no sRGB assign. `--fail-on-color-conversions` is offered only with `--assign-tf` and no conversion flag, since the app cannot read an input's own transfer function; `--fail-on-origin-changes` only without `--convert-texcoord-origin`. Each warn flag is hidden while its fail flag is set.
- **Origin:** `--convert-texcoord-origin` is hidden when any input level is 1 px tall (a 1×1 mip tail, an N×1 image): v4.4.2 segfaults flipping it.
- Soft `ktx create` warnings (for example an 8-bit PNG with no transfer function) do not block encode. Tone-mapping HDR to LDR for compression is out of scope.

**Why:** Each of these was a sidecar fatal the user had to decode from stderr: an EXR with `R8G8B8A8_SRGB`, `--convert-tf linear` with an sRGB format, mismatched screenshots packed into one array. The app already classifies sources, so it can prevent them. Also stated in [AGENTS.md](../AGENTS.md).

**Revised:** 2026-10-11 — sRGB/TF pairing became a hard rule. 2026-10-13 — pack validation. 2026-10-14 — resize, 1D, codec, color and origin rules from the e2e options sweep and the v4.4.2 `ktx create` / libktx sources.

## 2026-10-11 — Known option defaults are real values

**Decided:** When `ktx create` has a default for an option, `CreateOptions::default` and the store hold that value. No placeholder, "Default (…)" label, or "tool default" text standing in for an unset field. `None` means "omit the flag" and is used only where the CLI default is omission (`width`/`height`/`layers`/`depth`/`levels`, `max-endpoints`/`max-selectors`, color assign/convert). `scale` defaults to `1`. UASTC RDO sub-options are stored at their defaults but emitted only with `--uastc-rdo`. `threads` defaults to `available_parallelism`. Values: [ktx-cli/create.txt](ktx-cli/create.txt). Also in [AGENTS.md](../AGENTS.md).

**Why:** Placeholders looked like values while the field stayed `None`, so gates such as "scale is set" disagreed with what the user saw.

## 2026-10-11 — Encoder session: batch outputs with role-typed inputs

**Decided:** The encoder session is a queue of KTX **outputs**. Each output has its own `InputRole` (`Base2D`, `ArrayLayers`, `CubemapFaces`, `MipChain`, `DepthSlices`), its own input list (whose length is the arity; layers / faces / depth / levels are derived, not typed), and its own animation settings (array layers only). `CreateOptions` is shared by the whole queue. Drops route by panel: Outputs → one output per image; Inputs → append to the selected output; Preview → one image adds an output, several ask batch vs pack. Encode all runs at most 2 jobs in parallel. Changing an output's inputs resets its encode result. Compound shapes (cube arrays) are deferred.

**Why:** Dropping several albedos used to build one `ktx create` that expected one input and failed. The UI looked like a batch queue but packed everything into one file. With one global role, setting any output to array layers turned every single image into a one-layer array, and a cubemap output failed arity for the rest.

**Revised:** 2026-10-11 — per-panel drop targets. 2026-10-12 — role and animation moved from global to per output. 2026-10-13 — single-image preview drop skips the dialog.

## 2026-10-12 — Official encode presets in Rust

**Decided:** The default experience is "drop images, press Encode, get a good KTX2" without learning KTX or opening a terminal. An Official preset catalog in `ktx/presets.rs` makes that work: Color / Data / Normal / Mask × UASTC vs BasisLZ × RGBA/RGB where alpha applies. The new-session default is Color > UASTC RGBA (`R8G8B8A8_SRGB`, `--encode uastc`, Zstd 18). Data, Normal and Mask presets (UNORM formats) own `--assign-tf linear`; Color presets own no transfer flag. Rust owns definitions, apply and match (the frontend keeps no copy); a preset writes format, encode, color/normal flags, codec settings and supercompression, and never touches inputs, role or container shape. Vue shows the breadcrumb in the encoder header and shows **Custom** when an owned field diverges. Source rules still win over a preset. User-saved presets, ASTC and HDR presets are deferred.

**Why:** Uncompressed / no-encode defaults worked against the main reason people open the app. Artists need opinions on the common axes (alpha, quality vs size, color vs linear data vs normals vs masks); power users still get every control. Zstd 22 was the first default; 18 keeps most of the size win without the memory cost of levels above 20. `ktx create` reads an 8-bit PNG without a transfer chunk as sRGB and converts it to linear for a UNORM format, so the first linear presets turned mask value 13 into 1 and flattened normal maps; assigning `linear` stores the bytes as they are.

**Revised:** 2026-10-12 — catalog grew to twelve leaves; selector moved to the header. Later — default Zstd 22 → 18. 2026-10-14 — linear presets assign `linear` instead of letting `ktx` convert.

## 2026-10-11 — Shared viewer stack for encoder and inspector

**Decided:** Both windows use the same viewer building blocks under `src/shared`:

- `ImageViewport` + `useZoomPan`: checkerboard, Ctrl/Cmd+wheel zoom, drag pan, and a **sticky** window-scoped zoom mode (Fit, Actual, or manual) that survives output changes, scrubbing and remounts. Size changes never force Fit. In Actual mode a smaller mip shows smaller. The layer translate is snapped so the image origin lands on a whole device pixel, measured from the frame's viewport position.
- The encoder compares source and encoded output with a vertical wipe in one frame, not side by side. The wipe clips in frame space so the cut stays under the handle while panning. Both layers fill a shared canvas.
- `composeCubemapCross` composites the classic face cross on a canvas (encoder: one cross per wipe side; inspector: **Cross** or a single face).
- `DepthVolumeView` shows depth slices as a CSS 3D `preserve-3d` stack with drag orbit; the 2D viewport shows the highlighted slice. No WebGL.
- View menu zoom actions go only to the focused window (`document.hasFocus()`).
- Inspector-only display **Flip Y** (CSS, persisted with `useLocalStorage`, default off); export never bakes it in.

**Why:** One transform implementation for both windows. A wipe is a clearer quality compare than two boxes scaled separately. Clip-path on the transformed image moved the cut away from the handle. Re-fitting on every size change threw away the user's zoom while scrubbing. A CSS stack gives a spatial overview without a second rendering stack. Centring alone left the image at a fractional position (the frame itself sits at a sub-pixel layout offset), so Actual size resampled every texel.

**Revised:** 2026-10-11 — role-aware preview, CSS3D volume, sticky zoom. 2026-10-13 — Flip Y. 2026-10-14 — inspector Cross vs single face; Flip Y default off; device-pixel snapping.

## 2026-10-11 — UI components and theme

**Decided:** shadcn-vue (Reka UI, New York) styled through UnoCSS with `unocss-preset-shadcn` and `unocss-preset-animations`. `shadcn-vue init` is not run, because it pulls in Tailwind; an empty `tailwind.config.js` exists only so the CLI can add components under `src/components/ui/`. Component defaults are scaled to about two-thirds of stock. `theme.css` (loaded after `virtual:uno.css`) owns the palette: VS Code Dark+ panel surfaces, the logo's electric blue `#0870f8` as primary/ring, and a cool off-white light mode. Dark is the default, both in `useColorMode` and as `class="dark"` in the HTML so first paint matches. Icons come only from `@lucide/vue`. Session actions (Encode, Export) are full-width window footers. The UI was accepted as shipped on `dev` before Wave 7.

**Why:** Hand-rolled controls and native selects/scrollbars looked inconsistent in the webview. AGENTS.md keeps UnoCSS as the styling layer, and the community preset is the supported bridge. Stock shadcn heights wasted space in dense option and report lists. Stock neutral read flat; desktop users expect VS Code-like contrast. Light as the unset default flashed before the toggle mounted.

**Revised:** 2026-10-12 — brand green → logo blue; dark default; compact density.

## 2026-10-04 — Native menu owned by Rust

**Decided:** `menu.rs` defines the native menu (File, View, Help; plus the app and Edit menus on macOS). Open, Exit and Check for Updates run in Rust. Other items emit `menu://action` with a typed `MenuAction`, handled by the same code as the on-screen buttons. Vue syncs item enablement with `set_menu_item_enabled`. Exit is a normal item calling `app.exit(0)`. On Windows only, a capture-phase `keydown` listener (`bindWindowsMenuShortcuts`, tested in `menuShortcuts.test.ts`) maps the Ctrl chords to menu ids and calls `dispatch_menu_action`, which checks the live enabled flag and runs the shared `handle_menu_id`.

**Why:** Desktop apps expose every primary action with accelerators. Predefined Quit is unsupported on Linux. A macOS menu bar without the app and Edit menus loses copy, paste and hide. WebView2 swallows keys before `TranslateAcceleratorW`, so Windows accelerators shown in the menu did nothing (wry#451 / tauri#6365). Routing the fallback through Rust keeps enablement in one place.

**Revised:** 2026-10-11 — full action set; Windows accelerator fallback.

## 2026-10-04 — Window bounds persistence

**Decided:** Windows are created hidden and `tauri-plugin-window-state` (registered on the builder, before config windows exist) restores and shows them. Flags are size, position, maximized and visible on Windows, macOS and Linux X11. On Linux Wayland (detected from `WAYLAND_DISPLAY` / `WAYLAND_SOCKET`) the plugin keeps only `VISIBLE`; `window_bounds.rs` stores the GTK **content** size from `gtk_window_get_size` plus maximized in `.window-size.json`, restores with `set_size(LogicalSize)` (`with_saved_bounds` for runtime windows such as the inspector), and saves on every `Resized` that changes the stored floating size. Wayland position is not restored. Do not set `GDK_BACKEND=x11`.

**Why:** Hidden-then-show avoids a flash at 800×600. Registering inside `setup` is too late, because config windows exist before that hook. Wayland has no global coordinates, and on GNOME the configure size includes client-side decoration margins while `gtk_window_resize` takes the content size, so every launch grew the window. `gtk_window_get_size` round-trips exactly. Saving on change survives `tauri dev` killing the process without `RunEvent::Exit`.

**Revised:** 2026-10-12 — Wayland handled in `window_bounds.rs` (a first attempt with physical sizes did not stop the growth).

## 2026-10-12 — OS open routing

**Decided:** Installers register `.ktx2` (`image/ktx2`) and `.ktx` (`image/ktx`) associations. Opens are routed entirely in Rust (`os_open.rs` → `open_inspector_path`): argv on cold start, `tauri-plugin-single-instance` on a second launch, `RunEvent::Opened` on macOS. A cold OS open is inspector-only: window-state skips `main`, and `open_from_args_cold` always destroys `main` even if it was already shown (macOS also within a 2 s launch grace). A warm open only drops a hidden `main` and leaves a visible encoder alone. File → Open… that picks images always shows the encoder, recreating it if needed (`encoder_window::deliver_sources` queues sources in `PendingEncoderSources` for `take_pending_encoder_sources` on mount). KTX picks open the inspector.

**Why:** An "Open with" restored the encoder next to the inspector. A never-shown hidden `main` kept the process alive after the inspector closed. When window-state had already shown `main`, the "only if hidden" rule left it up. After an inspector-only start, Open emitted sources to a window that did not exist.

**Revised:** 2026-10-13 — inspector-only cold open. 2026-10-14 — cold open always destroys `main`; File → Open recreates the encoder.

## 2026-10-11 — Error and job feedback surfaces

**Decided:** No toasts (Sonner removed). Encode job outcomes use OS notifications: success → notification; failure → notification plus one native `dialog.message` alert. Other failures (save, inspect, rejected drop, invalid input) use a native alert only. Preview load failures stay silent in the viewer. Multi-file inspector export success opens a scrollable Vue modal (count, scope/format summary, written paths) instead of an inline path list in the footer — long “Export everything” results must not shift or crush the viewer layout.

**Why:** One encode failure used to appear as an exit code, a stderr dump and a toast. Notifications reach the user when the window is unfocused during a batch; modals are for errors that need acknowledgment. Requirements also forbid layout shift when results arrive; dumping dozens of export paths under the export bar broke that.

**Revised:** 2026-10-12 — Sonner removed entirely. 2026-10-14 — inspector export success → scrollable result modal (not inline footer list).

## 2026-10-10 — One quality gate: `npm run check`

**Decided:** `npm run check` runs before every commit, in every sub-agent task, and in CI. It covers Rust (fmt, clippy `-D warnings` over every target and feature, tests, `cargo machete`), web (vue-tsc, `tsc` on `e2e/`, ESLint on `src` and `e2e`, Stylelint, Vitest via `vitest run`), and fallow (`dead-code --fail-on-issues`, `dupes --threshold 3`, `health --min-score 85 --min-severity high`). Findings are fixed in code; the only suppression marks code written ahead of its consumer and names that task. Thresholds change only through this entry. Fallow is a local devDependency run with `npx`, with no git hook. `scripts/ensure-tools.mjs` (`npm run setup`, wired into `precheck:rust`) installs `rustfmt`, `clippy` and `cargo-machete` and stages the sidecar on a fresh machine. Release matrix builds run `fetch-ktx -- --target <triple>` themselves, because tauri-action's prehook cannot see the matrix target.

**Why:** Catch drift while the change is small; parallel agents especially duplicate helpers. Fallow covers TS, Vue and CSS only, so Rust relies on clippy and `cargo machete`. A hook that calls a global `fallow` breaks without a global install. A clean machine failed at `cargo machete` and at builds with no staged sidecar. Vitest replaced hand-run Node self-tests so those contracts run in CI.

**Revised:** 2026-10-12 — Vitest; `ensure-tools`. 2026-10-14 — the e2e code is typechecked, linted and fallow-checked here; the e2e run itself (`npm run e2e`) is not part of the gate yet.

## 2026-10-10 — Releases and updater signing

**Decided:** semantic-release runs on push to `main`. Development happens on `dev`; `beta` publishes prereleases. Conventional Commits pick the version (`feat` minor, `fix`/`perf` patch, `!` major), but nothing lint-enforces them. semantic-release writes `CHANGELOG.md`, bumps `package.json` (Tauri reads the version there) and `Cargo.toml`, tags, and creates a **draft** GitHub Release; the tauri-action matrix uploads to it and a final job publishes. The updater checks `releases/latest/download/latest.json`. The minisign private key lives in 1Password (vault Tech, item "KTXForge Tauri updater signing key", no password) and is mirrored to the `TAURI_SIGNING_PRIVATE_KEY` GitHub secret; local signed builds read it with `op` (`npm run tauri:build:signed`). Only the public key is committed. Flow: [updater-signing.md](updater-signing.md). `plugins.updater` in `tauri.conf.json` must be an object even when empty, or plugin init panics.

**Why:** The user wanted commit-driven changelogs with a familiar tool; release-please adds a release-PR step nobody asked for. The draft stays unpublished until every platform has built, so `releases/latest` never lacks artifacts. commitlint's line-length rule failed on semantic-release's own changelog commit and blocked a release. A GitHub-only secret could not be recovered for local signed builds.

**Revised:** 2026-10-11 — CI/release workflows; key generated. 2026-10-12 — key moved to 1Password. 2026-10-13 — husky and commitlint removed.

## 2026-10-12 — Git LFS for binaries

**Decided:** Icons, screenshots and test fixtures (`*.png`, `*.jpg`, `*.ktx`, `*.ktx2` and the other binary patterns in `.gitattributes`) are in Git LFS; `*.svg` is not. Every `actions/checkout` sets `lfs: true`. `npm run prepare` (`scripts/prepare-git.mjs`) unsets `core.hooksPath` and runs `git lfs install --local`, so the LFS hooks live in `.git/hooks`.

**Why:** Without `lfs: true`, CI read pointer text as PNGs. While husky owned `core.hooksPath`, the LFS hooks never ran, so pushes sent pointers without objects and GitHub rejected them (GH008).

**Revised:** 2026-10-12 — hooks under husky. 2026-10-13 — husky removed; standard LFS hooks.

## 2026-10-11 — Identity, license, icons

**Decided:** The product is **KTXForge**, identifier `com.monogrid.ktxforge`, repository `MONOGRID/KTXForge`, crates `ktxforge` / `ktxforge_lib`. License is MIT (copyright MONOGRID); the bundled KTX-Software notices ship in `src-tauri/licenses/`. Icons come from two masters through `npm run icons` (`scripts/generate-icons.mjs`): `assets/icon-simple.png` below 128 px, `assets/icon-full.png` at 128 px and above.

**Why:** The identifier keys the data directory and updates, so it had to be final before the first release (it replaced the scaffold name `KTX-GUI`). MIT is compatible with redistributing the Apache-2.0 KTX-Software binary. `tauri icon` takes one source, and the full mark is unreadable at taskbar sizes.

**Revised:** 2026-10-12 — MIT license.

## 2026-10-14 — End-to-end suite instead of unit tests

**Decided:** Behaviour is tested end to end in `e2e/` with Playwright Test (`npm run e2e`), and the Rust unit tests are removed except the few that check behaviour the suite cannot reach (OS "Open with" argv, workspace cleanup, natural frame order on drop). The suite checks outcomes, not functions:

1. Seeded random sessions click only the controls `describe_options` shows, pick only offered values, apply every suggestion like the store, and press Encode. `ktx create` must accept the command line.
2. Valid packs encode to the shape their role promises; invalid packs are flagged by `validate_pack_inputs` and refused before the sidecar.
3. A corpus written by Khronos' own `ktx create` (formats, codecs, shapes) plus fixtures (KTX1, animation, broken files) is inspected, previewed and exported; LDR previews must equal `ktx extract` texel for texel.
4. The real frontend runs in Chromium (Vite dev server) with `window.__TAURI_INTERNALS__` bridged to the same Rust: Fit, Actual, Zoom In/Out, Ctrl+wheel, pan, sticky zoom while scrubbing mips, Flip Y, and drop → Encode all → Save in the encoder.
5. Each option set in the rail must show up in the file (`ktx info`), the encoder preview must equal the file, and goldens (decoded images, header summaries) live in `e2e/__snapshots__` so a `ktx` pin bump shows every change.

The Rust side runs headless as `ktxforge-e2e` (cargo feature `e2e`, never in release builds): a `MockRuntime` app with the real shell plugin serving the app's commands by their IPC names over stdin/stdout, with `e2e_*` twins only for commands that open a native dialog. The pipeline functions (`sidecar`, `extract_ops`, `info::inspect`, `run_encode`, `export_to`, preview PNG helpers) are generic over `tauri::Runtime` so the same code runs under Wry and the mock. Khronos' `ktx` next to the driver is the oracle; tests never call app code to check app output.

**Why:** The unit tests restated implementation (argv strings, gate tables, help text, formulas) and broke on every refactor while missing real failures: the first sweep found command lines `ktx create` rejects, a BasisLZ animation that no longer decodes after `KTXanimData` is written, 16-bit UNORM and single-channel float files the inspector cannot preview, and an Actual-size view that is not pixel-aligned. Playwright cannot drive WebKitGTK, so the GUI runs in Chromium against the real backend instead of mocked IPC; window chrome (menus, dialogs, notifications, focus) is the only part answered by the test.

