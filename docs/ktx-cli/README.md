# KTX-Software v4.4.2 CLI surface

Captured from the official Windows x64 release of [KTX-Software v4.4.2](https://github.com/KhronosGroup/KTX-Software/releases/tag/v4.4.2) on 2026-10-10.

| File | Source |
| --- | --- |
| `version.txt` | `ktx --version` → `ktx version: v4.4.2` |
| `help.txt` | `ktx --help` |
| `create.txt` | `ktx create --help` |
| `encode.txt` | `ktx encode --help` |
| `extract.txt` | `ktx extract --help` |
| `info.txt` | `ktx info --help` |
| `validate.txt` | `ktx validate --help` |
| `transcode.txt` | `ktx transcode --help` |
| `deflate.txt` | `ktx deflate --help` |
| `convert.txt` | `ktx convert --help` (command **does not exist**; top-level usage is what the tool printed) |

Installer: `KTX-Software-4.4.2-Windows-x64.exe` (SHA-1 `5323B8DD2431DCD4C22AA869015942294BEA99E7`), extracted with `7z x`.

## Spec vs `ktx create` (requirements.md UI options)

Status values: **present** (same spelling), **spelled differently**, **missing**.

### Container

| Spec option | Status | Notes |
| --- | --- | --- |
| `--1d` | present | |
| `--cubemap` | present | |
| `--layers` | present | |
| `--depth` | present | |
| `--levels` | present | |
| `--generate-mipmap` | present | |
| `--runtime-mipmap` | present | |
| `--width`, `--height` | present | |
| `--scale` | present | |
| `--raw` | present | |

### Mipmap / resample filters

| Spec option | Status | Notes |
| --- | --- | --- |
| `--mipmap-filter` | present | All listed filter names match |
| `--mipmap-filter-scale` | present | |
| `--mipmap-wrap` | present | `wrap` \| `reflect` \| `clamp` |

### Color, orientation, channels

| Spec option | Status | Notes |
| --- | --- | --- |
| `--assign-tf` | present | Value list includes `pg_eotf` and `abobergb` **as printed** (tool spelling, not typos in our capture) |
| `--convert-tf` | present | `linear` \| `srgb` (+ sRGB aliases) |
| `--assign-primaries` | present | |
| `--convert-primaries` | present | |
| `--assign-texcoord-origin` | present | |
| `--convert-texcoord-origin` | present | |
| `--fail-on-color-conversions` | present | |
| `--warn-on-color-conversions` | present | |
| `--no-warn-on-color-conversions` | present | |
| `--fail-on-origin-changes` | present | |
| `--warn-on-origin-changes` | present | |
| `--swizzle` | present | |
| `--input-swizzle` | present | |
| `--normalize` | present | |
| `--premultiply-alpha` | **missing** | Not in `create.txt` at all |
| `--normal-mode` | present | Under “Encode common options” |

Deprecated aliases also present: `--assign-oetf`, `--convert-oetf` (same as `--assign-tf` / `--convert-tf`).

### Format and codecs

| Spec item | Status | Notes |
| --- | --- | --- |
| `--format` required | present | Accepts VkFormat names without `VK_FORMAT_` |
| Uncompressed + `--encode` list (`R8_*` … `R8G8B8A8_*`) | present | Listed under `--format` when used with `--encode` |
| `R16G16B16_SFLOAT`, `R16G16B16A16_SFLOAT` | **not listed** in that with-encode list | Help still says format is any VkFormat name; HDR needs a live create smoke test (T0.3) |
| ASTC via `--format` ASTC_* | spelled differently | Help does not list names. Live create accepts Vulkan names with `_BLOCK` suffix, e.g. `ASTC_6x6_SRGB_BLOCK`. Spec short names like `ASTC_6x6_SRGB` are **rejected**. |
| `--encode basis-lz` | present | |
| `--encode uastc-ldr-4x4` | **missing** | CLI value is `uastc` only |
| `--encode uastc` | present (spelled differently) | Spec treats `uastc` as deprecated alias; **v4.4.2 only offers `uastc`** |
| `--encode uastc-hdr-4x4` | **missing** | |
| `--encode uastc-hdr-6x6i` | **missing** | |

### BasisLZ

| Spec option | Status |
| --- | --- |
| `--qlevel`, `--clevel`, `--max-endpoints`, `--max-selectors`, `--endpoint-rdo-threshold`, `--selector-rdo-threshold`, `--no-endpoint-rdo`, `--no-selector-rdo` | present |

### UASTC LDR

| Spec option | Status | Notes |
| --- | --- | --- |
| `--uastc-quality` | present | Help default is **1**; spec says default 2 |
| `--uastc-rdo`, `--uastc-rdo-l/d/b/s/f/m` | present | |

### UASTC HDR

| Spec option | Status |
| --- | --- |
| `--uastc-hdr-uber-mode` | **missing** |
| `--uastc-hdr-ultra-quant` | **missing** |
| `--uastc-hdr-favor-astc` | **missing** |
| `--uastc-hdr-lambda` | **missing** |
| `--uastc-hdr-6x6i-level` | **missing** |

### ASTC / shared encoder / supercompression

| Spec option | Status |
| --- | --- |
| `--astc-quality`, `--astc-perceptual` | present |
| `--threads`, `--no-sse`, `--compare-ssim`, `--compare-psnr` | present |
| `--zstd`, `--zlib` | present |

## Other commands vs architecture notes

| Spec expectation | v4.4.2 fact |
| --- | --- |
| `ktx convert` (KTX1→KTX2) | **Missing** as a unified subcommand. Legacy `ktx2ktx2.exe` ships beside `ktx.exe` in the Windows package. |
| `ktx extract --transcode` half-float targets | **Missing**. Targets are `etc-rgb` \| `etc-rgba` \| `eac-r11` \| `eac-rg11` \| `bc1` \| `bc3` \| `bc4` \| `bc5` \| `bc7` \| `astc` \| `r8` \| `rg8` \| `rgb8` \| `rgba8` only. |
| `ktx info` for KTX1 | Supported (text; JSON rejected for KTX1 per release notes / help). |

## Gaps for the orchestrator / user

These are required by [requirements.md](../requirements.md) but absent or different in **v4.4.2**:

1. **Encode codecs:** only `basis-lz` and `uastc`. No `uastc-ldr-4x4`, `uastc-hdr-4x4`, or `uastc-hdr-6x6i`. Live create: `--encode uastc` with `R16G16B16A16_SFLOAT` is rejected (“Only R8, RG8, RGB8, or RGBA8 UNORM and SRGB formats can be encoded”).
2. **All UASTC HDR CLI flags** listed in the spec.
3. **`--premultiply-alpha`**.
4. **`ktx convert`** subcommand (legacy `ktx2ktx2` remains).
5. **Half-float extract transcode targets** for HDR preview/export path.
6. **ASTC format names:** UI/spec short names need the Vulkan `_BLOCK` suffix when passed to `ktx create` (e.g. `ASTC_6x6_SRGB_BLOCK`).

Suspect spellings from the spec (`pg_eotf`, `abobergb`) are **confirmed present** in this binary’s help; keep those strings.

**Decision needed:** drop / defer the missing options for the v4.4.2 pin, or move the pin to KTX-Software 5.0 (still RC as of the plan’s survey). Until the user answers, later waves should expose only flags that exist in these captures.
