# KTXForge

Desktop GUI for [KTX](https://www.khronos.org/ktx/) textures — encode ordinary images to KTX2 and inspect `.ktx` / `.ktx2` files without installing Khronos tools separately.

KTXForge embeds the official [KTX-Software](https://github.com/KhronosGroup/KTX-Software) `ktx` CLI and surfaces **every documented `ktx create` option** (plus inspector detail from `ktx info` / extract). The UI hides or auto-corrects known-invalid combinations so you do not have to learn the sidecar’s fatal messages by trial and error.

Local-only: nothing is uploaded. Runs on Windows, macOS, and Linux.

## Screenshots

Replace the SVGs under [`docs/screenshots/`](docs/screenshots/) with real captures when ready. Each image below is a placeholder; the caption is what the final shot should show.

### Encoder — main window

![Encoder main window: outputs queue, wipe preview, and encode options rail](docs/screenshots/encoder-main.svg)

*Placeholder for `docs/screenshots/encoder-main.png`.* Full encoder layout: **KTX outputs** queue (top-left), optional **Inputs** packing list (bottom-left), center **source ↔ encoded wipe** preview with mip/layer/face/depth controls when relevant, and the right **shared encode options** rail (format, codec, mips, color, Basis/UASTC/ASTC panels).

### Encoder — options and batch queue

![Encoder options rail and batch encode progress](docs/screenshots/encoder-options.svg)

*Placeholder for `docs/screenshots/encoder-options.png`.* Close-up of the options rail with a codec panel expanded (e.g. BasisLZ or UASTC), tooltips visible if useful, plus the batch **Done i / N** queue modal or Encode bar showing parallel progress and Cancel.

### Inspector — report and viewport

![Inspector with image viewport and full KTX info report](docs/screenshots/inspector-main.svg)

*Placeholder for `docs/screenshots/inspector-main.png`.* Inspector window: left **mip / layer / face / depth** selectors and pan-zoom viewport (cubemap cross or depth volume when applicable); right scrollable **info report** (header, DFD, key/value metadata, validation messages) and **Export** to PNG/JPEG.

## Features

### Encoder

- Drag-and-drop ordinary images onto the main window (panel-aware drops: outputs, inputs, or preview).
- **Batch of KTX outputs** — one encode job per output, shared options, limited-parallel queue with Cancel.
- Pack multiple inputs into one KTX as **array layers**, **cubemap faces**, **mip chain**, or **depth slices**.
- **Every `ktx create` option** in the UI (tooltips from the tool docs); unavailable options are hidden, not left dangling disabled.
- Formats and codecs: uncompressed Vulkan layouts, **BasisLZ**, **UASTC LDR/HDR**, **ASTC** block sizes, **Zstd / Zlib** supercompression.
- Color / orientation / channels: transfer functions, primaries, texcoord origin, swizzle, normalize, premultiply, normal-mode, conversion warn/fail flags.
- Mipmap generate / runtime mips, resample filters, scale / width / height, 1D / cubemap / layers / depth / levels / raw.
- Source vs encoded **wipe preview**, mip scrubbing, cubemap cross, 3D depth volume orbit.
- Everyday formats the CLI rejects (JPEG, WebP, BMP, GIF, TIFF, …) are converted in Rust before `ktx create`.
- Animation packing with **`KTXanimData`** written after a successful create.
- Save selected or all completed results; native alert + OS notification on encode failure.

### Inspector

- Drop `.ktx` / `.ktx2` to open a dedicated inspector window.
- Full **`ktx info`** report (JSON for KTX2, text + best-effort header for KTX1), including validation messages.
- Browse **mips**, **layers / frames**, **faces**, and **depth** slices; pan / zoom; View menu zoom controls.
- Known metadata keys (`KTXorientation`, `KTXswizzle`, `KTXwriter`, `KTXanimData`, platform format keys, …).
- Export the selected subresource as **PNG** or **JPEG**.
- Cubemap cross and CSS3D depth-volume views when the file has faces or depth.

### Platform

- Embedded, pinned `ktx` sidecar — no system install required.
- Auto-update from GitHub Releases (when configured).
- Heavy work in Rust; Vue is visualization and controls only.

## Development

```bash
npm ci
npm run setup      # rustfmt/clippy, cargo-machete, fetch pinned ktx sidecar
npm run tauri dev
```

Prerequisites: Node LTS and [Rust via rustup](https://rustup.rs) (stable). On Linux you also need the usual Tauri system packages (`libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `patchelf`). `npm run setup` installs `rustfmt`, `clippy`, and `cargo-machete` when missing, then stages the `ktx` sidecar. The same tool install + fetch runs via `precheck:rust` before `npm run check`; `pretauri` only runs `fetch-ktx` (host triple) so release cross-builds stay fast.

Quality gate before commit (Rust fmt/clippy/tests/machete, Vue build/lint/style, Vitest, fallow):

```bash
npm run check
```

Frontend unit tests alone:

```bash
npm run test          # vitest run
npm run test:watch    # vitest
```

Acceptance checklist: [`docs/acceptance.md`](docs/acceptance.md).

Local release-style builds that produce updater signatures need the private key from 1Password (Tech vault), not a key file on disk. See [docs/updater-signing.md](docs/updater-signing.md):

```bash
npm run tauri:build:signed
```

### Commits and releases

Commits follow [Conventional Commits](https://www.conventionalcommits.org/). On `main` / `beta`, [semantic-release](https://semantic-release.gitbook.io/) derives the version (`feat` → minor, `fix` / `perf` → patch, `!` or `BREAKING CHANGE:` → major; other types do not release), updates [`CHANGELOG.md`](CHANGELOG.md), tags, and publishes GitHub Release artifacts via Actions.

Preview the next version locally (needs a `GITHUB_TOKEN` with repo access):

```bash
npx semantic-release --dry-run --no-ci
```

Releases run only in CI (`npm run release` / the Release workflow). To bump the embedded `ktx` pin, edit `ktx-version.json` in a `feat:` or `fix:` commit.

Product requirements live in [`docs/requirements.md`](docs/requirements.md). Decision log: [`docs/decision-context.md`](docs/decision-context.md).

## License

See repository license files. The embedded KTX-Software binary ships under its own Khronos license terms.
