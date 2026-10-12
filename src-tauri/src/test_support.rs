//! Helpers shared by unit tests.

use std::path::PathBuf;

use crate::ktx::sidecar::Sidecar;

pub fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Staged sidecar for the host target.
///
/// `tauri-build` refuses to compile without the `bundle.externalBin` files, so a
/// missing binary is a broken setup: fail loudly instead of skipping the test.
pub fn staged_sidecar(tool: Sidecar) -> PathBuf {
    let name = format!(
        "{}-{}{}",
        tool.name(),
        env!("KTXFORGE_TARGET_TRIPLE"),
        std::env::consts::EXE_SUFFIX
    );
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("binaries")
        .join(name);
    assert!(
        path.is_file(),
        "{} is not staged; run `npm run fetch-ktx`",
        path.display()
    );
    path
}

pub fn staged_ktx() -> PathBuf {
    staged_sidecar(Sidecar::Ktx)
}
