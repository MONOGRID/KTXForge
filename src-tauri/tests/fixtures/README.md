# Test fixtures (KTX-Software v4.4.2)

Small (≤ 64×64) inputs and KTX outputs for the end-to-end suite (`e2e/`). Generated against the official Windows x64 `ktx` from v4.4.2. Do not hand-edit binaries; regenerate from a scratch script if a fixture must change.

Total size is under 2 MB (~90 KiB as of Wave 0).

## Sources

| File | Provenance |
| --- | --- |
| `rgba.png` | Synthetic 64×64 RGBA PNG |
| `rgb.jpg` | Solid 48×48 JPEG via System.Drawing |
| `gray.png` | Synthetic 32×32 greyscale PNG |
| `hdr.exr` | Minimal OpenEXR float RGBA 32×32 with alphabetically sorted channels, **scanline** ZIP (rewritten Wave 3 for channel order; re-encoded 2026-10-12 with `oiiotool --nosoftwareattrib hdr.exr -d float --scanline --compression zip`, pixels unchanged per `idiff`, because the Linux v4.4.2 `ktx create` segfaults on the earlier tiled RLE layout) |
| `odd_size.png` | Synthetic 37×19 RGBA PNG |
| `tiled.exr` | 32×32 **tiled** RLE OpenEXR, every texel (0.25, 4, 8, 0.5), written with the `exr` crate. The Linux v4.4.2 `ktx create` segfaults on tiled EXR; the app must re-stage it as scanline. |
| `frames/frame_001.png` … `frame_004.png` | 32×32 solid colors |
| `cube/face_0.png` … `face_5.png` | Cubemap faces in +X…−Z order |

## KTX outputs

| File | Command / notes |
| --- | --- |
| `rgba8_srgb.ktx2` | `ktx create --format R8G8B8A8_SRGB rgba.png rgba8_srgb.ktx2` |
| `basis_lz.ktx2` | `ktx create --format R8G8B8A8_SRGB --encode basis-lz --generate-mipmap rgba.png basis_lz.ktx2` |
| `uastc_zstd.ktx2` | `ktx create --format R8G8B8A8_SRGB --encode uastc --zstd 3 rgba.png uastc_zstd.ktx2` |
| `astc_6x6_srgb.ktx2` | `ktx create --format ASTC_6x6_SRGB_BLOCK --astc-quality fastest rgba.png astc_6x6_srgb.ktx2` |
| `cubemap.ktx2` | `ktx create --format R8G8B8A8_SRGB --cubemap cube/face_*.png cubemap.ktx2` |
| `array4_mips.ktx2` | `ktx create --format R8G8B8A8_SRGB --layers 4 --generate-mipmap frames/* array4_mips.ktx2` |
| `hdr_uastc.ktx2` | `ktx create --format R16G16B16A16_SFLOAT hdr.exr hdr_uastc.ktx2` — **uncompressed HDR**. v4.4.2 refuses `--encode uastc` on float formats (UASTC HDR codecs are not in this release). |
| `anim.ktx2` | `ktx create --format R8G8B8A8_SRGB --layers 4 frames/* anim.ktx2`, then hand-written `KTXanimData` (`duration=1`, `timescale=10`, `loopCount=0`) |
| `legacy.ktx` | `toktx legacy.ktx rgba.png` (KTX1) |
| `truncated.ktx2` | First 200 bytes of `basis_lz.ktx2` |
| `not_ktx.ktx2` | Plain text file |

Matching `ktx info` samples live in [docs/ktx-cli/samples/](../../../docs/ktx-cli/samples/).
