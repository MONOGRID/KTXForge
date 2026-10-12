# Staging layout for embedded `ktx` (v4.4.2)

How CI / `npm run fetch-ktx` (T2.4) downloads, extracts, and stages the sidecars for Tauri `bundle.externalBin: ["binaries/ktx", "binaries/ktx2ktx2"]`, and how the shared library they load ships in the installed app.

Staged names follow Tauri’s convention: `src-tauri/binaries/ktx-<target-triple>[.exe]`, plus the shared library beside them under its own name. `externalBin` does not carry that library into installers; the per-OS config files in [Installed layout](#installed-layout) do.

## Per-target table

| Target triple | Release asset | SHA-1 | Extract command | `ktx` path in archive | Libraries | Stage destination | Rpath / load fix | License path | Verified |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `x86_64-pc-windows-msvc` | `KTX-Software-4.4.2-Windows-x64.exe` | `5323B8DD2431DCD4C22AA869015942294BEA99E7` | `7z x <asset> -o<dir>` (NSIS). Fallback: silent install `/S /D=<dir>` | `bin/ktx.exe` | `bin/ktx.dll` (required). `bin/ktx-jni.dll` not needed for CLI. | `binaries/ktx-x86_64-pc-windows-msvc.exe` + `binaries/ktx.dll`; `tauri.windows.conf.json` `bundle.resources` installs `ktx.dll` next to the exe | Windows loads DLLs from the exe directory; no rpath | Not packaged as a loose `LICENSE` file in the NSIS tree; use upstream [LICENSE.md](https://github.com/KhronosGroup/KTX-Software/blob/v4.4.2/LICENSE.md) (commit into `src-tauri/licenses/`) | **Local (this spike)** |
| `aarch64-pc-windows-msvc` | `KTX-Software-4.4.2-Windows-arm64.exe` | *(fetch `.sha1` companion or compute in T2.4)* | Same as x64 NSIS | Expect `bin/ktx.exe`, `bin/ktx.dll` | Same pattern | `binaries/ktx-aarch64-pc-windows-msvc.exe` + `ktx.dll` | Same | Same | **verify in CI** (no arm64 Windows host here) |
| `aarch64-apple-darwin` | `KTX-Software-4.4.2-Darwin-arm64.pkg` | companion `.sha1` if published; else hash in T2.4 | `pkgutil --expand-full <asset> <dir>` (do not stop at `7z x`; that leaves gzip+cpio `Payload` blobs) | `usr/local/bin/ktx` inside the tools component `Payload/` | `usr/local/lib/libktx.4.dylib` (symlink to `libktx.4.4.2.dylib`) inside the library component `Payload/` | `binaries/ktx-aarch64-apple-darwin` + `binaries/libktx.4.dylib`; `tauri.macos.conf.json` `bundle.macOS.frameworks` puts the dylib in `Contents/Frameworks` | Binary loads `@rpath/libktx.4.dylib` and has rpath `@executable_path`. `fetch-ktx` adds `@executable_path/../Frameworks` with `install_name_tool`, then re-signs ad hoc (`codesign --force --sign -`) | Same upstream LICENSE.md | **archive listing (2026-10-11)** |
| `x86_64-apple-darwin` | `KTX-Software-4.4.2-Darwin-x86_64.pkg` | same | same | same | same | `binaries/ktx-x86_64-apple-darwin` + dylib | same | same | same packaging as arm64 |
| `x86_64-unknown-linux-gnu` | `KTX-Software-4.4.2-Linux-x86_64.tar.bz2` (prefer over `.deb`/`.rpm` for a flat tree) | companion `*.tar.bz2.sha1` on the release | `tar -xjf <asset> -C <dir>` | `KTX-Software-4.4.2-Linux-x86_64/bin/ktx` (regular file) | `…/lib/libktx.so.4` is a symlink to `libktx.so.4.4.2`. `NEEDED` is `libktx.so.4` | `binaries/ktx-x86_64-unknown-linux-gnu` + copy the symlink target’s bytes as `binaries/libktx.so.4`; `tauri.linux.conf.json` `bundle.resources` installs it in `/usr/lib/KTXForge/` | `patchelf --set-rpath '$ORIGIN:$ORIGIN/../lib/KTXForge'` (the binary has no rpath of its own) | `LICENSE*` / `share/doc/...` inside tarball if present; else upstream LICENSE.md | **archive listing (2026-10-11)** |
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
2. Copy `bin/ktx.dll` → `src-tauri/binaries/ktx.dll`. `tauri.windows.conf.json` maps it as a resource to `ktx.dll`, which the installers place next to the app executable and the sidecars. Sidecar rename does not rename the DLL; the loader still looks for `ktx.dll`.

## macOS / Linux (archive listing, 2026-10-11)

`7z x` on a Darwin `.pkg` only unwraps the xar. `usr/local/bin/ktx` lives inside `KTX-Software-…-tools.pkg/Payload` (gzip, then cpio). Expand with `pkgutil --expand-full`. The library component has `libktx.4.4.2.dylib` plus soname symlink `libktx.4.dylib`. The tool loads `@rpath/libktx.4.dylib` with rpath `@executable_path`.

The Linux `.tar.bz2` has a top-level `KTX-Software-4.4.2-Linux-<arch>/` directory. `bin/ktx` is a regular file. `lib/libktx.so.4` is a symlink to `libktx.so.4.4.2`, and that soname is what the binary’s `NEEDED` entry names. Copy the symlink’s target bytes to `libktx.so.4` (a directory walk that skips symlinks will not see it).

## `ktx2ktx2` (KTX1 → KTX2)

`ktx extract` reads only KTX2, so the inspector converts legacy `.ktx` files with `ktx2ktx2` from the same release. It is a second `bundle.externalBin` entry, staged as `binaries/ktx2ktx2-<triple>[.exe]`. Every release keeps it in the same directory as `ktx` (`bin/`, or `usr/local/bin/` in the macOS tools payload), so `fetch-ktx` derives its archive path from `ktxPath`. It links the same `libktx` and gets the same rpath fix. It prints `--version` on stderr.

## Installed layout

Tauri copies `externalBin` sidecars next to the app executable, but nothing in `externalBin` brings the library along. Each OS has its own config file (merged over `tauri.conf.json` by the CLI for that target) because the library name differs:

| OS | Config | Sidecars installed at | Library installed at | How the loader finds it |
| --- | --- | --- | --- | --- |
| Windows (MSI, NSIS) | `tauri.windows.conf.json`: `bundle.resources` `{"binaries/ktx.dll": "ktx.dll"}` | install dir | install dir (resources root is the install dir) | DLL search starts in the exe directory |
| macOS (.app, .dmg) | `tauri.macos.conf.json`: `bundle.macOS.frameworks` `["binaries/libktx.4.dylib"]` | `Contents/MacOS/` | `Contents/Frameworks/` (signed with the bundle identity) | rpath `@executable_path/../Frameworks`, added by `fetch-ktx` |
| Linux (.deb, .rpm, AppImage) | `tauri.linux.conf.json`: `bundle.resources` `{"binaries/libktx.so.4": "libktx.so.4"}` | `/usr/bin/` | `/usr/lib/KTXForge/` (resource dir named after `productName`) | runpath `$ORIGIN/../lib/KTXForge`, set by `fetch-ktx` |

The same entries cover `tauri dev` and `cargo test`: `tauri-build` copies resources into `target/<profile>/` (found through `$ORIGIN` / the exe directory) and macOS frameworks into `target/Frameworks/` (found through `@executable_path/../Frameworks`). Placing the macOS dylib in `Contents/MacOS` through `bundle.macOS.files` would also match the original rpath, but the bundler does not sign files copied that way, so the bundle signature would not cover it.

linuxdeploy (AppImage) also copies `libktx.so.4` into `usr/lib/` and rewrites the sidecar runpath to `$ORIGIN/../lib`. It resolves the library through our runpath, so it uses the bundled copy. On a machine with a system `libktx` it would fall back to that one, which hides a missing bundle entry.

`fetch-ktx` re-applies the rpath on every run when it differs, so binaries staged before a layout change are updated without a re-download. If you change `productName`, re-run `npm run fetch-ktx`.

**Verified (2026-10-12, Linux x86_64):** `npm run tauri build` with the system `libktx` hidden (`bwrap --ro-bind /dev/null /usr/lib/libktx.so.4.4.2`). `ktx --version` and `ktx2ktx2 --version` run from the extracted .deb, .rpm and AppImage trees. Before the fix, the .deb and .rpm shipped no library and both sidecars exited 127. Windows and macOS layouts follow the bundler source and still need a check on those hosts.

## License

The Windows NSIS tree did not expose a top-level `LICENSE` file next to `bin/`. T2.4 should commit Khronos’s Apache-2.0-oriented project license from the v4.4.2 tag (`LICENSE.md` / `LICENSES/*`) into `src-tauri/licenses/KTX-Software-LICENSE.md` (and note Ericsson `etcdec` special case from that file).
