# KTXForge — encoder and inspector

Requirements for **KTXForge**. This document records the product requirements and the constraints taken from the current Khronos and Tauri documentation. It is the source of truth before implementation.

KTXForge exists so artists and engine folks can produce and inspect KTX without memorizing codec/format rules or running Khronos CLIs. The GUI is the product: drop ordinary images, press Encode, get a good KTX2. The embedded `ktx` sidecar is how that product stays correct — not the reason the app exists.

The app is a local desktop tool. It does not upload textures. The installed app must run on Windows, macOS, and Linux with no KTX tools installed on the machine.

## Product requirements

1. **Embedded KTX software.** The official Khronos [KTX-Software](https://github.com/KhronosGroup/KTX-Software) `ktx` tool is downloaded at build time and embedded in the app for Windows, macOS, and Linux. End users install nothing else.
2. **GitHub releases.** The repository lives on GitHub. GitHub Actions build and publish release artifacts.
3. **Auto-update.** The installed app checks GitHub Releases and updates itself through the Tauri updater.
4. **Drag and drop encode.** Dropping ordinary images opens the encoder. The user encodes them to KTX2.
5. **Every encoding option in the UI, plus a preview.** Every option documented for `ktx create` is visible. A small preview of the encoded result is shown when the tool can produce a viewable image.
6. **KTX inspector in its own window.** Dropping a `.ktx` or `.ktx2` file, or opening one via the OS (“Open with” / double-click when the installed app is registered), opens a separate inspector window. The user can export the image as PNG or JPEG and can see every detail the format reports.
7. **Multi-frame encode and decode.** Multiple frames can be packed into one KTX file and unpacked again, including animation metadata.
8. **Mipmap preview.** The inspector shows each mip level, not only the base image.
9. **Modern, snappy interface.** The UI stays responsive while encoding, extracting, and updating.
10. **Heavy work in Rust.** File conversion, process control, parsing, and image preparation run in Rust.
11. **JS and HTML are visualization only.** Vue renders controls, previews, and inspector data. It does not encode, decode, parse KTX, or shell out to the KTX tool.

## Architecture

| Layer | Owns |
| --- | --- |
| Vue / HTML / CSS | Layout, controls, drag-and-drop affordance, previews, progress, inspector display |
| Rust | Commands, sidecar lifecycle, image conversion, `ktx` argument building, stdout/stderr capture, JSON parsing, temporary files, export to JPEG |
| Embedded `ktx` | Create, encode, extract, transcode, info, validate |

The frontend calls Tauri commands and listens for progress events. It never receives a shell, a binary path, or a raw command line it can edit. Rust builds the `ktx` argument list from a typed options struct.

Drag-and-drop uses Tauri's file-drop events (`over`, `drop`, `cancel`), which provide absolute paths. Rust classifies each path. Image drops stay on the encoder window. KTX drops open the inspector. The webview drag-drop handler stays enabled so those paths are delivered. HTML5 drop is not the file pipeline.

Installed builds register OS file associations for `.ktx` and `.ktx2` (`bundle.fileAssociations`). Opening a file that way routes through the same Rust inspector open path: Windows/Linux pass the path on argv (cold start in `setup`, warm start via `tauri-plugin-single-instance`); macOS also delivers `RunEvent::Opened` file URLs. Associations apply to bundled installs, not `tauri dev`. A cold “Open with” / double-click opens **only the inspector** (the encoder window is destroyed even if window-state had already shown it). If the encoder is already open in a running instance, a second OS open adds or focuses the inspector and leaves the encoder as it is. File → Open… from either window that picks images always shows and focuses the encoder (recreating it when a cold OS-open had destroyed `main`); KTX picks still open the inspector.

The inspector is a second webview window, created from Rust with `WebviewWindowBuilder` on an async command. On Windows that builder deadlocks inside a synchronous command, so window creation stays async. The window label is stable and reused if the inspector is already open.

## Embedded KTX software

Current KTX-Software ships one CLI, `ktx`, with subcommands. The older tools are aliases and are scheduled for removal (legacy tools go away in the 4.5 line; the live tool reference already describes the unified CLI).

| `ktx` command | Role in this app | Replaces |
| --- | --- | --- |
| `create` | Encode images to KTX2 | `toktx` |
| `encode` | Re-encode an existing KTX2 | `ktxsc` |
| `extract` | Decode selected images to PNG or EXR | — |
| `info` | Inspector report | `ktxinfo` |
| `validate` | Called implicitly by `info` on KTX2 | `ktx2check` |
| `transcode` | Used through `extract --transcode` for previews | — |
| `convert` | KTX1 to KTX2 when a preview or export needs it | `ktx2ktx2` |
| `deflate` | Zstd or Zlib on an existing file | `ktxsc` |

`ktx create` accepts PNG, EXR, and raw input. It does not accept JPEG, WebP, BMP, GIF, or TIFF. Rust converts those to PNG (LDR) or EXR (HDR) before calling `ktx`. `ktx extract` writes PNG, EXR, or raw. It does not write JPEG. Rust encodes JPEG from the extracted image when the user picks JPEG.

Official release packages are installers and archives (`exe`, macOS package, Linux `deb` / `rpm` / `tar.bz2`), not a single loose binary. CI downloads the pinned KTX-Software release for each target, extracts the `ktx` executable, and stages it for Tauri's sidecar bundling. The app does not shell out to a system `ktx`.

Tauri looks up sidecars by target triple. The staged names follow `ktx-<triple>` (plus `.exe` on Windows), for example:

- `ktx-x86_64-pc-windows-msvc.exe`
- `ktx-aarch64-pc-windows-msvc.exe`
- `ktx-x86_64-apple-darwin`
- `ktx-aarch64-apple-darwin`
- `ktx-x86_64-unknown-linux-gnu`
- `ktx-aarch64-unknown-linux-gnu`

`externalBin` lists the path without the triple or extension. Rust starts the binary through the shell plugin sidecar API. The KTX-Software license ships with the app.

The pinned tool version is recorded in the repo and shown in the UI. Bumping it is a deliberate change, not a floating "latest" download on the user's machine.

## Encoder

Drop one or more ordinary images onto the main window. The encoder session is a **batch of KTX outputs**, not a single packed file by default. Each drop panel highlights on its own (not the whole window):

- **Top-left — KTX outputs:** queue of independent outputs. Drop here → one new output per image. Selectable rows; Add / delete; no reorder. A **+** on each row opens Inputs for that output and picks more images.
- **Bottom-left — Inputs:** shown only when the selected output has (or is about to have) multiple inputs. Role selector is array layers, cubemap faces, mip chain, or depth slices (single-image Base 2D is implicit when Inputs is hidden). List length **is** the arity (`--layers`, `--cubemap` + 6 faces, `--levels`, `--depth`). Drop here → append inputs for the selected output. Known-invalid packs (wrong count, mixed LDR/HDR, mismatched sizes, non-square cubemap faces, mip levels that do not follow `max(1, base>>i)`) are marked in the Inputs UI — red panel chrome with a tooltip for list-wide problems, red row chrome with a tooltip for a single bad input — and **Encode all** stays disabled. Do not rely on a `ktx create` fatal for those checks.
- **Center — Preview** for the selected output (source vs encoded wipe). Drop one image here → one new KTX output. Drop several → dialog: separate KTX outputs, or pack into one KTX with a chosen input kind.
- **Header — Official preset** selector beside the product name (Handbrake-style breadcrumb + nested Color / Data / Normal / Mask menu). Applies to every output in the queue.
- **Right — Shared encode options** for every output in the queue. New outputs incompatible with the first (e.g. HDR vs LDR) are rejected.

Encode runs a limited-parallel queue with a modal (`Done i / N`, Cancel queue). Save can write the selected result or all completed results to a folder.

The UI stays usable and shows progress and the tool's stderr if a command fails.

`ktx create` exit codes are surfaced as:

| Code | Meaning |
| --- | --- |
| 0 | Success |
| 1 | Command line error |
| 2 | IO failure |
| 3 | Invalid input file |
| 4 | Runtime or library error |
| 5 | Unsupported state or operation |
| 6 | Feature not implemented yet |

### Output the tool can produce

`ktx create` writes KTX2. Required `--format` names match Vulkan format names without the `VK_FORMAT_` prefix.

Uncompressed formats accepted together with `--encode`:

- `R8_UNORM`, `R8_SRGB`
- `R8G8_UNORM`, `R8G8_SRGB`
- `R8G8B8_UNORM`, `R8G8B8_SRGB`
- `R8G8B8A8_UNORM`, `R8G8B8A8_SRGB`
- `R16G16B16_SFLOAT`, `R16G16B16A16_SFLOAT` (HDR, from EXR half-float sources)

`--encode` codecs:

- `basis-lz` — ETC1S / BasisLZ. R becomes RRR. RG becomes RRRG.
- `uastc-ldr-4x4` — UASTC LDR 4×4. `uastc` is a deprecated alias and is not offered as its own choice.
- `uastc-hdr-4x4` — UASTC HDR 4×4. Input must be half-float RGB or RGBA.
- `uastc-hdr-6x6i` — UASTC HDR 6×6 intermediate. Same input rule.

ASTC is selected by setting `--format` to an ASTC Vulkan format. The tool builds an intermediate `R8G8B8` or `R8G8B8A8` texture (sRGB or UNORM to match the ASTC format) and encodes it. LDR ASTC block sizes:

`4x4`, `5x4`, `5x5`, `6x5`, `6x6`, `8x5`, `8x6`, `8x8`, `10x5`, `10x6`, `10x8`, `10x10`, `12x10`, `12x12`, each UNORM and sRGB.

Supercompression:

- `--zstd <1-22>` — not valid with BasisLZ. Levels above 20 use much more memory. Recommended together with UASTC.
- `--zlib <1-9>` — not valid with BasisLZ.

### Official encode presets

The product ships an **Official** preset tree of opinionated content recipes (not cubemap/array arrangement): Color / Data / Normal / Mask × UASTC vs BasisLZ × alpha where it applies. Selecting a preset fills format, encode, color/normal flags, codec knobs, and supercompression. **Inputs role/list and container shape stay untouched.** The full options rail stays visible afterward. Editing any field the active preset owns switches the selector to **Custom** until another Official preset is chosen. User-saved presets are a later feature.

| Preset | Intent |
| --- | --- |
| Color > UASTC RGBA | Product default: `R8G8B8A8_SRGB`, `--encode uastc`, `--zstd 18` |
| Color > UASTC RGB | Color without alpha (`R8G8B8_SRGB`, UASTC, zstd 18) |
| Color > BasisLZ RGBA | Smaller color with alpha (`R8G8B8A8_SRGB`, BasisLZ, no zstd) |
| Color > BasisLZ RGB | Smaller color without alpha (`R8G8B8_SRGB`, BasisLZ, no zstd) |
| Data > UASTC RGBA | Linear data with alpha (`R8G8B8A8_UNORM`, UASTC, zstd 18) |
| Data > UASTC RGB | Linear data without alpha (`R8G8B8_UNORM`, UASTC, zstd 18) |
| Data > BasisLZ RGBA | Smaller linear data with alpha (BasisLZ, no zstd) |
| Data > BasisLZ RGB | Smaller linear data without alpha (BasisLZ, no zstd) |
| Normal > UASTC | High-quality normals (UNORM, `--normal-mode`, `--normalize`, UASTC RDO λ 0.5, zstd 18) |
| Normal > BasisLZ | Smaller normals (UNORM, `--normal-mode`, `--normalize`, BasisLZ, no zstd) |
| Mask > UASTC R8 | Single-channel mask (`R8_UNORM`, UASTC, zstd 18) |
| Mask > BasisLZ R8 | Smaller single-channel mask (`R8_UNORM`, BasisLZ, no zstd) |

UASTC presets use zstd 18. BasisLZ presets omit supercompression (invalid with BasisLZ). The zstd level control’s own help notes that levels above 20 use much more memory. Source compatibility rules still win (e.g. HDR clearing UASTC → Custom).

### Options that must appear in the UI

Every option below is available in the UI. Options the current codec, format, or texture shape cannot use are **hidden** (not shown disabled). Whole option panels hide when every control inside is unavailable. Dependent detail controls follow enable-then-reveal: the master switch stays visible when it can be used; linked options appear only while that feature applies (for example mipmap filter controls only after generate-mipmap, scale, width, or height). Each control has a short tooltip from the tool docs. The product default for a new session is Color > UASTC RGBA (not uncompressed / no-encode). Other individual flag defaults match the tool when a preset does not set them.

The UI must never offer a combination of inputs and options that the app already knows will fail at encode time (for example HDR / EXR float with an 8-bit or ASTC format, or LDR PNG with a half-float format, or mixed LDR and HDR sources in one encode). Incompatible choices are hidden or auto-corrected to a safe default; encode is refused if a known-invalid combination reaches the backend.

**Container**

| Option | Values / notes |
| --- | --- |
| `--1d` | 1D texture |
| `--cubemap` | Faces in order +X, −X, +Y, −Y, +Z, −Z. Origin must be top-left |
| `--layers` | Array texture / animation frame count. **Derived** from the Inputs panel when role is Array layers (list length); omitted otherwise. Not a free number field in the options rail. |
| `--depth` | 3D texture. Requires a front/back texture-coordinate origin |
| `--levels` | Mip count. With generate-mipmap, this many levels are built. Otherwise this many input images are required |
| `--generate-mipmap` | Full chain if `--levels` is omitted. Not valid for SINT, UINT, 3D, or raw |
| `--runtime-mipmap` | Client generates mips later. Mutually exclusive with `--generate-mipmap`. Not valid with BasisLZ, UASTC, or ASTC encoding (uncompressed only) |
| `--width`, `--height` | Required for raw. Otherwise resamples. Enables the mipmap filter options |
| `--scale` | Scale on load. Not valid with raw |
| `--raw` | Raw input. Width and height required |

**Mipmap / resample filters** (used by generate-mipmap, scale, width, height)

| Option | Values |
| --- | --- |
| `--mipmap-filter` | `box`, `tent`, `bell`, `b-spline`, `mitchell`, `blackman`, `lanczos3`, `lanczos4` (default), `lanczos6`, `lanczos12`, `kaiser`, `gaussian`, `catmullrom`, `quadratic_interp`, `quadratic_approx`, `quadratic_mix` |
| `--mipmap-filter-scale` | Default `1.0` |
| `--mipmap-wrap` | `wrap`, `reflect`, `clamp` (default) |

Mip generation is avoided when the output transfer function is non-linear and is not sRGB. The UI says so when that combination is selected. The resampler only decodes sRGB, so generating mips for other non-linear transfer functions is blocked.

**Color, orientation, channels**

| Option | Values / notes |
| --- | --- |
| `--assign-tf` | `linear`, `srgb`, `srgb_eotf`, `scrgb`, `scrgb_eotf`, `itu`, `itu_oetf`, `bt601`, `bt601_oetf`, `bt709`, `bt709_oetf`, `bt2020`, `bt2020_oetf`, `smpte170m`, `smpte170m_oetf`, `smpte170m_eotf`, `ntsc`, `ntsc_eotf`, `slog`, `slog_oetf`, `slog2`, `slog2_oetf`, `bt1886`, `bt1886_eotf`, `hlg_oetf`, `hlg_eotf`, `pq_oetf`, `pg_eotf`, `dcip3`, `dcip3_eotf`, `pal_oetf`, `pal625_eotf`, `st240`, `st240_oetf`, `st240_eotf`, `acescc`, `acescc_oetf`, `acescct`, `acescct_oetf`, `abobergb`, `adobergb_eotf`. The tool page spells two of these `pg_eotf` and `abobergb`; confirm against `ktx create --help` when the binary is pinned |
| `--convert-tf` | `linear` or `srgb` (`srgb_eotf`, `scrgb`, `scrgb_eotf` are sRGB aliases). Not valid with raw |
| `--assign-primaries` | `none`, `bt709`, `srgb`, `bt601_ebu`, `bt601_smpte`, `bt2020`, `ciexyz`, `aces`, `acescc`, `ntsc1953`, `pal525`, `displayp3`, `adobergb` |
| `--convert-primaries` | Same list except `none`. Not valid with raw, and not valid when assigned primaries are `none` |
| `--assign-texcoord-origin` | `top-left` or `bottom-left`, plus `-front` or `-back` when depth is set |
| `--convert-texcoord-origin` | Same corners. Not valid with raw. Cubemaps must stay top-left. Only a vertical flip is supported |
| `--fail-on-color-conversions` | Error if any conversion would run |
| `--warn-on-color-conversions` | Warn on every conversion, including visually lossless ones |
| `--no-warn-on-color-conversions` | Silence conversion warnings. Overrides the warn flag |
| `--fail-on-origin-changes` | Error if an origin flip is required |
| `--warn-on-origin-changes` | Warn when an origin changes |
| `--swizzle` | Four characters from `rgba01` |
| `--input-swizzle` | Pre-swizzle channels, same alphabet |
| `--normalize` | Unit-length normals. Linear, 2+ components. Not valid with raw or premultiply |
| `--premultiply-alpha` | Premultiply and set the metadata flag. Not valid with normalize or raw |
| `--normal-mode` | Linear, 2+ components. RGB(A) XYZ becomes X+Y in RGB+A. For BasisLZ this disables endpoint and selector RDO |

`*_SRGB` formats require an sRGB output transfer function — the UI hides `linear` (and other non-sRGB assign values) and auto-clears an invalid choice; encode refuses as a backstop. Non-sRGB formats that have an sRGB twin reject an sRGB `--convert-tf` the same way (only `linear` is offered). Implicit conversion: sRGB formats convert non-sRGB input to sRGB; other formats convert non-linear input to linear. Supported implicit or explicit conversion inputs are linear, sRGB, ITU (BT.601, BT.709, BT.2020, SMPTE 170M), and PQ. Supported conversion outputs are linear and sRGB. sRGB to linear is treated as visually lossy. Encoding to ASTC, BasisLZ, or UASTC is discouraged unless the data is linear or sRGB. Soft color-space notes still appear as warnings next to the controls.

**BasisLZ** (`--encode basis-lz`)

| Option | Range / default |
| --- | --- |
| `--qlevel` | 1–255. Lower is smaller, faster, lower quality. Default 128 when endpoints and selectors are unset |
| `--clevel` | 0–6, default 1. Speed versus quality. Prefer `qlevel` first |
| `--max-endpoints` | 1–16128, default unset. Takes effect only together with `--max-selectors` |
| `--max-selectors` | 1–16128, default unset |
| `--endpoint-rdo-threshold` | Default 1.25. Useful range about 1.0–3.0. Overrides `qlevel` |
| `--selector-rdo-threshold` | Default 1.25. Same range |
| `--no-endpoint-rdo` | Off by default (RDO on) |
| `--no-selector-rdo` | Off by default (RDO on) |

If `qlevel`, `max-endpoints`, and `max-selectors` are all set, `qlevel` is ignored. `qlevel` only picks the RDO thresholds when it is above 128.

**UASTC LDR**

| Option | Range / default |
| --- | --- |
| `--uastc-quality` | 0 fastest (~43.45 dB), 1, 2 default (~47.47 dB), 3, 4 very slow (~48.24 dB) |
| `--uastc-rdo` | Enable the RDO post-process |
| `--uastc-rdo-l` | Lambda 0.001–10, default 1.0. Normals: try 0.25–0.75 |
| `--uastc-rdo-d` | Dictionary 64–65536 bytes, default 4096 |
| `--uastc-rdo-b` | Smooth-block error scale 1–300, default 10. 1 disables |
| `--uastc-rdo-s` | Smooth-block stddev 0.01–65536, default 18 |
| `--uastc-rdo-f` | Do not favor simpler modes |
| `--uastc-rdo-m` | Disable RDO multithreading |

**UASTC HDR**

| Option | Notes |
| --- | --- |
| `--uastc-hdr-uber-mode` | 4×4 only. Slower, slightly higher quality. Can hurt BC6H transcode quality |
| `--uastc-hdr-ultra-quant` | 4×4 only. Better quantized endpoints, slower |
| `--uastc-hdr-favor-astc` | 4×4 only. Favor ASTC HDR over BC6H |
| `--uastc-hdr-lambda` | RDO. Higher is smaller and lower quality. Try 100–20000 |
| `--uastc-hdr-6x6i-level` | 0–12, default 2. 6×6i only |

**ASTC**

| Option | Values |
| --- | --- |
| `--astc-quality` | `fastest` (0), `fast` (10), `medium` (60, default), `thorough` (98), `exhaustive` (100) |
| `--astc-perceptual` | Perceptual error instead of RMS. Normal maps and RGB only |

**Shared encoder flags**

| Option | Notes |
| --- | --- |
| `--threads` | Default is hardware concurrency for BasisLZ, or 1 |
| `--no-sse` | BasisLZ and UASTC only |
| `--compare-ssim` | Print SSIM. BasisLZ, UASTC, or ASTC |
| `--compare-psnr` | Print PSNR. Same codecs |

SSIM and PSNR, when requested, are shown next to the preview.

### Encode preview

The center Preview is role-aware and always starts from the selected output’s **inputs** (encode is not required to browse).

| Input role | Before encode | After encode |
|---|---|---|
| Base 2D | Single source | Wipe source vs encoded; **Mip** slider when the KTX has more than one level |
| Array layers | **Layer** slider over inputs | Wipe that layer vs the matching extract; **Mip** slider when levels > 1 |
| Mip chain | **Mip** slider over inputs | Wipe that input vs encoded level (generate-mipmap is off for this role) |
| Cubemap faces | Classic face **cross** (empty cells if fewer than six) | Same cross on both sides of **one** wipe; **Mip** slider when levels > 1 |
| Depth slices | Oblique **CSS 3D volume** of all input slices; **Depth** slider highlights one slice | Same volume (source stack); wipe panel compares the highlighted slice vs extract |

Wipe compare always shows input vs output in the same frame. When scrubbing a **generated** mip, the left side stays the full-resolution input(s) and the right side is the extracted mip so resolution loss is visible. Cubemap uses one composite cross per side under a single wipe handle (not per-face wipes). Depth slices use a `perspective` / `preserve-3d` stack for spatial overview (drag to orbit; double-click resets); the wipe stays a 2D compare of the highlighted slice (not a clipped duplicate volume).

After a successful encode, Rust extracts viewable PNG slices (`encoded_preview` with an `ImageSelection`):

- BasisLZ and UASTC: `ktx extract --transcode` to `r8`, `rg8`, `rgb8`, `rgba8`, or a half-float target chosen from the channel count, then a PNG for the webview.
- Uncompressed PNG-exportable formats: `ktx extract` of the selected level/face/layer/depth.
- HDR: Rust tone-maps the EXR extract to a PNG for the preview and labels it as a preview, not the stored data.
- The preview also shows byte size of the KTX2 file, dimensions, codec, mip count, layer count, and face count.

Block-compressed transcode targets (`etc-rgb`, `etc-rgba`, `eac-r11`, `eac-rg11`, `bc1`, `bc3`, `bc4`, `bc5`, `bc7`, `astc`) can only be extracted as raw, so they are not the preview path. The preview always uses an unpacked target the webview can show.

## Inspector

Dropping `.ktx` or `.ktx2`, or opening one through the OS file association, opens the inspector window for that file. KTX2 details come from `ktx info --format json`, which follows the [info schema](https://schema.khronos.org/ktx/info_v1.json). The command also validates KTX2 and returns errors and warnings. Invalid files still show a best-effort report.

The viewer flips preview images vertically by default (**Flip Y**, matching a bottom-left / OpenGL-style origin). A switch in the Viewer header turns that off; the choice persists for the next session. Export writes the extracted pixels without this display flip.

The inspector shows every field the schema can return:

- **Status:** `valid`, and each message (`fatal`, `error`, `warning`) with id, message, and details.
- **Header:** identifier, `vkFormat`, `typeSize`, pixel width / height / depth, `layerCount`, `faceCount`, `levelCount`, supercompression scheme.
- **Level index:** for each mip, byte offset, compressed byte length, uncompressed byte length.
- **Index offsets:** data format descriptor, key/value data, supercompression global data.
- **Data format descriptor:** vendor, descriptor type, version, flags, transfer function, color primaries, color model, texel block dimension, bytes per plane, and each sample (channel, qualifiers, bit offset, bit length, sample position, sample lower/upper).
- **Known metadata:** `KTXorientation`, `KTXswizzle`, `KTXwriter`, `KTXwriterScParams`, `KTXastcDecodeMode`, `KTXglFormat` (internal format, format, type), `KTXdxgiFormat__`, `KTXmetalPixelFormat`, `KTXcubemapIncomplete` (which of the six faces exist), `KTXanimData` (`duration`, `timescale`, `loopCount`).
- **Other key/value pairs** the file contains, including ones outside the known set.
- **BasisLZ global data** when present: endpoint and selector counts, table sizes, and per-image slice offsets and flags.

`ktx info` prints text for KTX1. JSON is rejected for KTX1. The inspector still opens KTX1 files and shows the text report plus whatever header fields Rust can present. Export of a KTX1 file goes through `ktx convert` to KTX2 first when extract cannot read it directly.

### Export

The user saves the selected image, the selected mip, one frame, or all frames.

- PNG and JPEG for LDR exports.
- EXR remains available for HDR, SINT, UINT, and other formats the tool will not put in a PNG.
- JPEG is produced in Rust from the extracted image. Quality is a visible control.
- Multi-image extract names follow the tool: `output_level{n}_face{n}_layer{n}_depth{n}.<ext>`, with each suffix omitted when that dimension has size 1.

Extract selection uses `--level`, `--layer`, `--face`, `--depth`, or `--all`.

### Mipmap preview

The inspector lists every mip level from the header (`levelCount` and the level index). Selecting a level extracts that level and shows it at its real pixel size and scaled up so it stays readable. Dimensions for level `i` are the base size shifted by `i`, with a minimum of 1. Cubemap faces, array layers, and 3D slices stay selectable while browsing mips.

### 3D depth volume preview

When `pixelDepth` at the selected mip is greater than 1, the inspector shows the same oblique **CSS 3D volume** stack as the encoder Preview (`perspective` / `preserve-3d` slice planes). The **Depth** slider highlights the active plane and the 2D viewport below shows that slice. Slice bitmaps come from `inspector_image` for each depth index at the current level / face / layer. Drag on the volume to orbit; double-click resets the angle.

### Cubemap cross preview

When `faceCount` is greater than 1, the inspector shows the same classic face **cross** composite as the encoder Preview (shared `composeCubemapCross`). Face buttons still set the export / subresource selection; the viewport stays on the full cross.

### Multi-frame

In KTX2 a frame is one array layer. A file is an animation when `layerCount` is the frame count and `KTXanimData` is present.

`KTXanimData` is three little-endian `uint32` values:

| Field | Meaning |
| --- | --- |
| `duration` | Time units per frame |
| `timescale` | Time units per second |
| `loopCount` | `0` loops forever, `1` plays once, `n` plays `n` times |

Frame length in seconds is `duration / timescale`. Sequence length is that value times `layerCount`.

Encoding multiple frames:

- Set Inputs role to **Array layers**, select (or create) one KTX output, and drop/reorder frame images in the bottom list.
- Rust calls `ktx create --layers <count>` with one image per layer, in order (`count` = list length).
- The UI exposes duration, timescale, and loop count when that role is active and the selected output has ≥ 2 layers.
- The current `ktx create` reference has no flag for `KTXanimData`. Rust writes that key onto the file after a successful create, using the 12-byte layout above, so the encoded file is a real animation and not only an array texture. The inspector reads the key back via `ktx info`.

Decoding:

- The inspector shows frame count, frame duration, timescale, loop count, and sequence length.
- The user can step frames and play them.
- Export can write one frame or the whole sequence as PNG or JPEG files.

Cubemaps (6 faces), cube arrays, and 3D depth slices are shown with their own selectors. They are not treated as animation unless `KTXanimData` is present.

## Interface

The interface is a desktop app, not a web page stretched into a window: clear type, tight spacing, immediate hover and press states, and no layout shift when results arrive. Long encodes show progress on the window that started them and never block the other window. Drop targets highlight on `over` and reset on `cancel`. Errors from the tool appear next to the action that failed, with the exit code and stderr.

The main window is the encoder. The inspector is a separate window so a file can stay open while another encode runs.

## GitHub Actions and auto-update

Releases are built with [`tauri-apps/tauri-action`](https://v2.tauri.app/distribute/pipelines/github/) on a matrix that matches the embedded `ktx` binaries:

- macOS Apple silicon (`aarch64-apple-darwin`)
- macOS Intel (`x86_64-apple-darwin`)
- Ubuntu x64
- Ubuntu arm64 (public runners)
- Windows x64

The workflow needs `contents: write`. It checks out the repo, installs Linux webkit and packaging packages, sets up Node and Rust, installs frontend dependencies, downloads and stages the pinned `ktx` binaries, then runs `tauri-action` to build and attach artifacts to a GitHub Release. Releases are automatic: on every push to `main`, semantic-release derives the version from Conventional Commit messages, updates `CHANGELOG.md`, tags the commit, and creates a draft release; the build matrix attaches the artifacts; the workflow publishes the draft once every platform has built. Commits that are not `feat`, `fix`, `perf`, or breaking produce no release. The repository is `https://github.com/MONOGRID/KTXForge`. The updater signing private key’s source of truth is the 1Password **Tech** vault item (see [updater-signing.md](updater-signing.md)). CI uses a GitHub Actions secret (`TAURI_SIGNING_PRIVATE_KEY` and, if set, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`). The private key is never committed and is not kept as a file on developer machines.

Updater configuration, from the Tauri v2 updater plugin:

- `bundle.createUpdaterArtifacts` is `true`.
- `plugins.updater.pubkey` is the public half of the minisign key pair from `tauri signer generate`. The private key is what signs release artifacts. Losing it means existing installs can never update again.
- `plugins.updater.endpoints` includes `https://github.com/MONOGRID/KTXForge/releases/latest/download/latest.json`, which `tauri-action` generates.
- Windows `installMode` is `passive` (small progress UI, no extra prompts).
- On launch, Rust checks for an update. The UI shows version notes and progress, then relaunches after install. Signature checks stay on. They cannot be disabled.

Update artifacts the bundler produces: AppImage plus `.sig` on Linux, `.app.tar.gz` plus `.sig` on macOS, NSIS `setup.exe` plus `.sig` on Windows. `latest.json` carries `version`, optional `notes` and `pub_date`, and per-platform `url` plus the signature contents. Platform keys are `linux-x86_64`, `linux-aarch64`, `darwin-x86_64`, `darwin-aarch64`, and `windows-x86_64`.

macOS builds that are not Apple-notarized set an ad-hoc signing identity so Apple silicon downloads from GitHub are not marked damaged. Windows and macOS distribution signing stay optional follow-ups. They are separate from updater minisign.

## Out of scope for the first version

- Mobile (the updater plugin does not cover Android or iOS).
- A network service, account, or texture upload.
- Reimplementing Basis, UASTC, or ASTC in Rust or JavaScript.
- Editing arbitrary GPU block formats in the webview. Preview and PNG/JPEG export go through `ktx` transcode and extract.

## Sources

- [KTX-Software README](https://github.com/KhronosGroup/KTX-Software) and [releases](https://github.com/KhronosGroup/KTX-Software/releases) (packages surveyed at v4.4.2; tool behavior taken from the current tools reference)
- [KTX tools reference](https://github.khronos.org/KTX-Software/ktxtools/ktx.html): [`create`](https://github.khronos.org/KTX-Software/ktxtools/ktx_create.html), [`encode`](https://github.khronos.org/KTX-Software/ktxtools/ktx_encode.html), [`extract`](https://github.khronos.org/KTX-Software/ktxtools/ktx_extract.html), [`info`](https://github.khronos.org/KTX-Software/ktxtools/ktx_info.html)
- [KTX info JSON schema](https://schema.khronos.org/ktx/info_v1.json)
- [KTX 2.0 fragment URI](https://registry.khronos.org/KTX/specs/2.0/ktx-frag.html) (`KTXanimData`: a frame is an array layer)
- [Tauri v2 updater](https://v2.tauri.app/plugin/updater/)
- [Tauri v2 GitHub releases](https://v2.tauri.app/distribute/pipelines/github/)
- [Tauri v2 sidecars](https://v2.tauri.app/develop/sidecar/)
- [Tauri drag-and-drop](https://v2.tauri.app/reference/javascript/api/namespacewebviewwindow/) and [`WebviewWindowBuilder`](https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindowBuilder.html)
