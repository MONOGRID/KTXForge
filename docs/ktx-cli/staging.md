# Staging layout for embedded `ktx` (v4.4.2)

How CI / `npm run fetch-ktx` (T2.4) should download, extract, and stage the sidecar for Tauri `bundle.externalBin: ["binaries/ktx"]`.

Staged names follow Tauri’s convention: `src-tauri/binaries/ktx-<target-triple>[.exe]`, plus shared libraries where the OS loader needs them next to the executable (or at a documented rpath).

## Per-target table

| Target triple | Release asset | SHA-1 | Extract command | `ktx` path in archive | Libraries | Stage destination | Rpath / load fix | License path | Verified |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `x86_64-pc-windows-msvc` | `KTX-Software-4.4.2-Windows-x64.exe` | `5323B8DD2431DCD4C22AA869015942294BEA99E7` | `7z x <asset> -o<dir>` (NSIS). Fallback: silent install `/S /D=<dir>` | `bin/ktx.exe` | `bin/ktx.dll` (required). `bin/ktx-jni.dll` not needed for CLI. | `binaries/ktx-x86_64-pc-windows-msvc.exe` + copy `ktx.dll` beside it **and** list `ktx.dll` under `bundle.resources` so the installed app keeps it next to the exe | Windows loads DLLs from the exe directory; no rpath | Not packaged as a loose `LICENSE` file in the NSIS tree; use upstream [LICENSE.md](https://github.com/KhronosGroup/KTX-Software/blob/v4.4.2/LICENSE.md) (commit into `src-tauri/licenses/`) | **Local (this spike)** |
| `aarch64-pc-windows-msvc` | `KTX-Software-4.4.2-Windows-arm64.exe` | *(fetch `.sha1` companion or compute in T2.4)* | Same as x64 NSIS | Expect `bin/ktx.exe`, `bin/ktx.dll` | Same pattern | `binaries/ktx-aarch64-pc-windows-msvc.exe` + `ktx.dll` | Same | Same | **verify in CI** (no arm64 Windows host here) |
| `aarch64-apple-darwin` | `KTX-Software-4.4.2-Darwin-arm64.pkg` | companion `.sha1` if published; else hash in T2.4 | `pkgutil --expand-full <asset> <dir>` (do not stop at `7z x`; that leaves gzip+cpio `Payload` blobs) | `usr/local/bin/ktx` inside the tools component `Payload/` | `usr/local/lib/libktx.4.dylib` (symlink to `libktx.4.4.2.dylib`) inside the library component `Payload/` | `binaries/ktx-aarch64-apple-darwin` + `libktx.4.dylib` beside it | Binary loads `@rpath/libktx.4.dylib` and already has rpath `@executable_path`, so the sibling dylib is enough | Same upstream LICENSE.md | **archive listing (2026-10-11)** |
| `x86_64-apple-darwin` | `KTX-Software-4.4.2-Darwin-x86_64.pkg` | same | same | same | same | `binaries/ktx-x86_64-apple-darwin` + dylib | same | same | same packaging as arm64 |
| `x86_64-unknown-linux-gnu` | `KTX-Software-4.4.2-Linux-x86_64.tar.bz2` (prefer over `.deb`/`.rpm` for a flat tree) | companion `*.tar.bz2.sha1` on the release | `tar -xjf <asset> -C <dir>` | `KTX-Software-4.4.2-Linux-x86_64/bin/ktx` (regular file) | `…/lib/libktx.so.4` is a symlink to `libktx.so.4.4.2`. `NEEDED` is `libktx.so.4` | `binaries/ktx-x86_64-unknown-linux-gnu` + copy the symlink target’s bytes as `libktx.so.4` beside it | `patchelf --set-rpath '$ORIGIN'` (the binary has no `$ORIGIN` of its own) | `LICENSE*` / `share/doc/...` inside tarball if present; else upstream LICENSE.md | **archive listing (2026-10-11)** |
| `aarch64-unknown-linux-gnu` | `KTX-Software-4.4.2-Linux-arm64.tar.bz2` | companion `.sha1` | same | same | same | `binaries/ktx-aarch64-unknown-linux-gnu` + libs | same | same | **verify in CI** |

## Windows layout (observed)

After `7z x KTX-Software-4.4.2-Windows-x64.exe`:

```text
bin/ktx.exe
bin/ktx.dll          ← required at runtime next to ktx.exe
bin/ktx-jni.dll      ← Java; do not stage
bin/toktx.exe, ktxinfo.exe, ktx2ktx2.exe, …  ← legacy; not the sidecar
share/doc/KTX-Software/html/…  ← HTML docs; not required at runtime
```

`ktx.exe --version` succeeds when `cwd` or PATH includes `bin` (DLL side-by-side).

Tauri staging recommendation for Windows:

1. Copy `bin/ktx.exe` → `src-tauri/binaries/ktx-x86_64-pc-windows-msvc.exe`
2. Copy `bin/ktx.dll` → `src-tauri/binaries/ktx.dll` **and** ensure the bundler places it next to the final app executable (e.g. `bundle.resources` entry pointing at that DLL). Sidecar rename does not rename the DLL; the loader still looks for `ktx.dll`.

## macOS / Linux (archive listing, 2026-10-11)

`7z x` on a Darwin `.pkg` only unwraps the xar. `usr/local/bin/ktx` lives inside `KTX-Software-…-tools.pkg/Payload` (gzip, then cpio). Expand with `pkgutil --expand-full`. The library component has `libktx.4.4.2.dylib` plus soname symlink `libktx.4.dylib`. The tool loads `@rpath/libktx.4.dylib` with rpath `@executable_path`.

The Linux `.tar.bz2` has a top-level `KTX-Software-4.4.2-Linux-<arch>/` directory. `bin/ktx` is a regular file. `lib/libktx.so.4` is a symlink to `libktx.so.4.4.2`, and that soname is what the binary’s `NEEDED` entry names. Copy the symlink’s target bytes to `libktx.so.4` (a directory walk that skips symlinks will not see it).

## License

The Windows NSIS tree did not expose a top-level `LICENSE` file next to `bin/`. T2.4 should commit Khronos’s Apache-2.0-oriented project license from the v4.4.2 tag (`LICENSE.md` / `LICENSES/*`) into `src-tauri/licenses/KTX-Software-LICENSE.md` (and note Ericsson `etcdec` special case from that file).
