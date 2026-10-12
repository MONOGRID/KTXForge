use std::path::PathBuf;

fn main() {
    // Ensure ts-rs writes next to the Vue app even when Cargo's target dir is relocated.
    let bindings = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../src/bindings");
    let bindings = bindings.canonicalize().unwrap_or(bindings);
    println!("cargo:rustc-env=TS_RS_EXPORT_DIR={}", bindings.display());
    // Tests run the staged `binaries/<tool>-<triple>` sidecars directly.
    let target = std::env::var("TARGET").expect("cargo sets TARGET for build scripts");
    println!("cargo:rustc-env=KTXFORGE_TARGET_TRIPLE={target}");
    tauri_build::build();
}
