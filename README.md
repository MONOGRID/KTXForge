# 🔥 KTXForge

Encode [KTX](https://www.khronos.org/ktx/) textures without learning the format or opening a terminal — drop ordinary images, press Encode, move on.

For people who want the result, not a `ktx create` degree. Sensible defaults and a rules engine hide invalid option combos; every documented option is still there when you need it. Inspect `.ktx` / `.ktx2` the same way: drop a file, no CLI.

The official [KTX-Software](https://github.com/KhronosGroup/KTX-Software) tools ship inside the app, so nothing else is installed. Local-only. Windows, macOS, and Linux.

## 📸 Highlights

### Encode with a wipe preview

Batch KTX outputs on the left, original vs encoded wipe in the center, and the full `ktx create` options rail on the right.

![Encoder: outputs queue, original vs encoded wipe preview, and format / mipmap options](docs/screenshots/encoder-main.png)

### Codec and option help in place

Every encode choice carries a tooltip from the tool docs — here UASTC vs BasisLZ — so you pick a codec without leaving the rail.

![Format rail: Encode dropdown open on UASTC with an inline codec tooltip](docs/screenshots/encoder-options.png)

### Inspect KTX1 / KTX2 in a dedicated window

Drop a `.ktx` / `.ktx2` to browse mips, layers, and faces while the sidebar shows validation status and the full header / level-index report.

![Inspector: image viewport with mip and layer scrubbers, plus status and KTX header panels](docs/screenshots/inspector-main.png)

## ✨ Features

### 🖼️ Encoder

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

### 🔍 Inspector

- Drop `.ktx` / `.ktx2`, or open them via the OS (“Open with” / double-click on an installed build), to open a dedicated inspector window.
- Full **`ktx info`** report (JSON for KTX2, text + best-effort header for KTX1), including validation messages.
- Browse **mips**, **layers / frames**, **faces**, and **depth** slices; pan / zoom; View menu zoom controls.
- Known metadata keys (`KTXorientation`, `KTXswizzle`, `KTXwriter`, `KTXanimData`, platform format keys, …).
- Export the selected subresource as **PNG** or **JPEG**.
- Cubemap cross and CSS3D depth-volume views when the file has faces or depth.

### 💻 Platform

- Embedded, pinned `ktx` sidecar — no system install required.
- Installed builds register as a viewer for `.ktx` / `.ktx2` (OS file associations).
- Auto-update from GitHub Releases (when configured).
- Heavy work in Rust; Vue is visualization and controls only.

## 🛠️ Development

```bash
npm ci
npm run setup      # rustfmt/clippy, cargo-machete, fetch pinned ktx sidecar
npm run tauri dev
```

Prerequisites: Node LTS, [Git LFS](https://git-lfs.com) (`git lfs install` once per machine), and [Rust via rustup](https://rustup.rs) (stable). On Linux you also need the usual Tauri system packages (`libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `patchelf`). `npm ci` / `npm install` runs husky and configures LFS filters; push hooks under `.husky/` upload LFS objects automatically. `npm run setup` installs `rustfmt`, `clippy`, and `cargo-machete` when missing, then stages the `ktx` and `ktx2ktx2` sidecars. The same tool install + fetch runs via `precheck:rust` before `npm run check`; `pretauri` only runs `fetch-ktx` (host triple) so release cross-builds stay fast.

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

### 📦 Commits and releases

Commits follow [Conventional Commits](https://www.conventionalcommits.org/). On `main` / `beta`, [semantic-release](https://semantic-release.gitbook.io/) derives the version (`feat` → minor, `fix` / `perf` → patch, `!` or `BREAKING CHANGE:` → major; other types do not release), updates [`CHANGELOG.md`](CHANGELOG.md), tags, and publishes GitHub Release artifacts via Actions.

Preview the next version locally (needs a `GITHUB_TOKEN` with repo access):

```bash
npx semantic-release --dry-run --no-ci
```

Releases run only in CI (`npm run release` / the Release workflow). To bump the embedded `ktx` pin, edit `ktx-version.json` in a `feat:` or `fix:` commit.

Product requirements live in [`docs/requirements.md`](docs/requirements.md). Decision log: [`docs/decision-context.md`](docs/decision-context.md).

## 📄 License

KTXForge is [MIT](LICENSE). The embedded [KTX-Software](https://github.com/KhronosGroup/KTX-Software) `ktx` sidecar ships under its own terms (primarily Apache-2.0); see [`src-tauri/licenses/KTX-Software-LICENSE.md`](src-tauri/licenses/KTX-Software-LICENSE.md).
