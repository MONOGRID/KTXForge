# Acceptance — Wave 7

Spec coverage and smoke results for [requirements.md](requirements.md). UI chrome is **accepted** as shipped on `dev` (see decision-context 2026-10-12). Deferred CLI gaps stay on KTX-Software v4.4.2 per [ktx-cli/README.md](ktx-cli/README.md).

Verified: 2026-10-12. Quality gate: `npm run check` (includes `vitest run`). `fallow suppressions`: none.

---

## Product requirements

| # | Requirement | Where | Verified | Status |
| --- | --- | --- | --- | --- |
| 1 | Embedded KTX-Software `ktx` | `scripts/fetch-ktx.mjs`, `ktx-version.json`, `tauri.conf.json` `externalBin` | `npm run fetch-ktx`; staged `ktx --version` → v4.4.2; CI matrix | pass |
| 2 | GitHub releases | `.github/workflows/release.yml`, `.releaserc.json` | CI/release workflows on repo | pass |
| 3 | Auto-update | `src-tauri/src/updater.rs`, `commands/update.rs`, `UpdateBanner.vue`, `plugins.updater` | Config + runtime skip in debug; first public release updater path → smoke N/A | pass |
| 4 | Drag-and-drop encode | `drop.rs`, `OutputsQueue.vue`, `InputsPanel.vue`, `DropOverlay.vue`, `DropIntentDialog.vue` | T7.2 Linux smoke; Rust classify tests | pass |
| 5 | Every create option + preview | `CreateOptions` / `rules.rs` / option sections; `PreviewPane.vue` | `ktx::rules::tests::*`, `create_args` flag coverage; UI accepted | pass (deferred flags below) |
| 6 | KTX inspector window | `inspector_window.rs`, `InspectorApp.vue`, report panels | T7.2; `info_types` sample deserialize tests | pass |
| 7 | Multi-frame + anim metadata | `AnimationSection.vue`, `ktx/anim.rs`, encode pipeline | `writes_anim_data_*`, `encode_animation_frames_writes_anim_key` | pass |
| 8 | Mipmap preview | `MipStrip.vue`, `PreviewIndexSliders.vue`, `inspector_image` | T7.2; extract/inspect staged-binary tests | pass |
| 9 | Responsive UI | async commands, scrub gate, extract concurrency | Vitest `scrubGate`; decision-context extract slots | pass |
| 10 | Heavy work in Rust | `src-tauri/src/{ktx,image,commands}` | Architecture + fallow/TS not calling shell | pass |
| 11 | Vue visualization only | `src/api/commands.ts`, `events.ts` | Code review / API wrappers | pass |

---

## Encode options (v4.4.2 surface)

| Option / group | Rust | Vue | Verified | Status |
| --- | --- | --- | --- | --- |
| Container (`1d`, cubemap, layers†, depth, levels, generate/runtime mipmap, width/height, scale, raw) | `options.rs`, `rules.rs`, `create_args.rs` | `ContainerSection.vue`, `FilterSection.vue`, `InputsPanel.vue` (†layers from input role) | `report_has_one_entry_per_option_key`, create_args cases, rules tests | pass |
| Mip filters / wrap / scale | same | `FilterSection.vue` | `mipmap_filters_ignored_unless_resampling` | pass |
| Color / origin / swizzle / normalize / normal-mode | same | `ColorSection.vue` | color/origin rules tests | pass |
| `--premultiply-alpha` | — | — | not in v4.4.2 `create.txt` | deferred ([ktx-cli/README.md](ktx-cli/README.md)) |
| Format + encode (`basis-lz` \| `uastc`) | `options.rs` | `FormatSection.vue` | serde/cli enum tests; encode fixture tests | pass |
| Spec `uastc-ldr-4x4` name | mapped to CLI `uastc` | Format encode picker | pin decision | pass (spelling) |
| UASTC HDR codecs + `--uastc-hdr-*` | omitted | no section | pin decision | deferred |
| BasisLZ flags | `BasisLzOptions` | `BasisLzSection.vue` | `basis_lz_options_only_when_selected` | pass |
| UASTC LDR flags | `UastcLdrOptions` | `UastcLdrSection.vue` | rules + create_args | pass |
| ASTC quality / perceptual | `AstcOptions` | `AstcSection.vue` | `astc_options_need_astc_format`; `_BLOCK` formats | pass |
| Threads / no-sse / SSIM / PSNR | `EncoderOptions` | `EncoderFlagsSection.vue` | `compare_metrics_need_encoding`, metrics parser | pass |
| Zstd / Zlib | `Supercompression` | `SupercompressionSection.vue` | `rejects_zstd_with_basis_lz` | pass |
| Animation (`KTXanimData`) | `anim.rs`, `AnimationOptions` | `AnimationSection.vue` | anim write + validate tests | pass |
| Invalid combo refusal | `rules::validate`, create_args | hide via `OptionField` / report | mixed LDR/HDR, HDR format rules tests | pass |

---

## Inspector / export

| Area | Where | Verified | Status |
| --- | --- | --- | --- |
| Status / header / level index / index / DFD / known KV / other KV / Basis global / KTX1 text | `info.rs`, `info_types.rs`; `StatusPanel`, `HeaderPanel`, `LevelIndexTable`, `IndexPanel`, `DfdPanel`, `KnownMetadataPanel`, `KeyValueTable`, `BasisGlobalDataPanel`, `Ktx1TextPanel` | sample JSON deserialize; `anim_info_has_ktx_anim_data` | pass |
| Image viewer, mips, faces, layers, depth, frame player | `ImageViewer`, `MipStrip`, `SubresourceSelectors`, `FramePlayer`; `imageCache.ts` | Vitest LRU/slots; T7.2 | pass |
| Export PNG/JPEG/EXR | `commands/export.rs`, `ExportPanel.vue` | `export_anim_frames_as_jpeg_via_staged_binary` | pass |
| Windows menu shortcut fallback | `menu.rs`, `menuShortcuts.ts` | Vitest `menuShortcuts.test.ts` | pass |

---

## Frontend unit tests (Vitest)

| Suite | Covers |
| --- | --- |
| `src/shared/scrubGate.test.ts` | Latest-only scrub coalesce under stampede |
| `src/shared/menuShortcuts.test.ts` | Matcher chords + sync with `menu.rs` / apps |
| `src/inspector/imageMemoryLru.test.ts` | Soft-cap LRU, pins, `extractSlotsForCores` |

---

## Smoke (T7.2)

| Step | Linux x64 | macOS arm64 | Windows x64 | Notes |
| --- | --- | --- | --- | --- |
| 1. App starts, product KTXForge, no system `ktx` required | pass | pending hardware | pending hardware | Local `.deb` + `npm run tauri dev` on Linux 2026-10-12; sidecar v4.4.2 staged |
| 2. Encode PNG / JPEG / EXR; multi-input anim or array | pass (dev + cargo encode/anim tests) | pending | pending | Full interactive batch UI exercised in prior accepted UI work on Linux; CI `--include-ignored` covers staged-binary encode |
| 3. Inspector fixtures, mips, layers/frames, anim play, truncated validation | pass (dev + inspect tests) | pending | pending | Fixtures in `src-tauri/tests/fixtures/` |
| 4. Export PNG and JPEG from inspector footer | pass (dev + export JPEG test) | pending | pending | |
| 5. In-app updater from prior install | N/A — first release | N/A | N/A | Re-run after first `main` publish |

Cross-OS installer smoke for macOS/Windows remains **pending** until those machines run the checklist; Linux is the recorded pass baseline. CI already builds/checks on `windows-latest`, `macos-latest`, and `ubuntu-22.04`.
