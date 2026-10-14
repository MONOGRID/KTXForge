//! Path classification and natural frame ordering.

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::commands::SourceKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileKind {
    Image {
        kind: SourceKind,
        needs_conversion: bool,
    },
    Ktx1,
    Ktx2,
    Unsupported,
}

const KTX1_MAGIC: [u8; 12] = [
    0xAB, 0x4B, 0x54, 0x58, 0x20, 0x31, 0x31, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A,
];
const KTX2_MAGIC: [u8; 12] = [
    0xAB, 0x4B, 0x54, 0x58, 0x20, 0x32, 0x30, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A,
];
const PNG_MAGIC: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
const EXR_MAGIC: [u8; 4] = [0x76, 0x2F, 0x31, 0x01];

/// Classify by magic bytes first, then by extension.
pub fn classify(path: &Path) -> FileKind {
    if let Some(kind) = classify_by_magic(path) {
        return kind;
    }
    // Readable file whose magic is not KTX: do not trust a `.ktx` / `.ktx2` extension.
    if file_has_prefix(path) {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase())
            .unwrap_or_default();
        if ext == "ktx" || ext == "ktx2" {
            return FileKind::Unsupported;
        }
    }
    classify_by_extension(path)
}

fn file_has_prefix(path: &Path) -> bool {
    File::open(path)
        .and_then(|mut f| {
            let mut b = [0u8; 1];
            f.read(&mut b).map(|n| n > 0)
        })
        .unwrap_or(false)
}

fn classify_by_magic(path: &Path) -> Option<FileKind> {
    let mut file = File::open(path).ok()?;
    let mut buf = [0u8; 16];
    let n = file.read(&mut buf).ok()?;
    let bytes = &buf[..n];

    if bytes.starts_with(&KTX1_MAGIC) {
        return Some(FileKind::Ktx1);
    }
    if bytes.starts_with(&KTX2_MAGIC) {
        return Some(FileKind::Ktx2);
    }
    if bytes.starts_with(&PNG_MAGIC) {
        return Some(FileKind::Image {
            kind: SourceKind::Ldr,
            needs_conversion: false,
        });
    }
    if bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF {
        return Some(FileKind::Image {
            kind: SourceKind::Ldr,
            needs_conversion: true,
        });
    }
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some(FileKind::Image {
            kind: SourceKind::Ldr,
            needs_conversion: true,
        });
    }
    if bytes.len() >= 2 && &bytes[0..2] == b"BM" {
        return Some(FileKind::Image {
            kind: SourceKind::Ldr,
            needs_conversion: true,
        });
    }
    if bytes.len() >= 6 && (bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a")) {
        return Some(FileKind::Image {
            kind: SourceKind::Ldr,
            needs_conversion: true,
        });
    }
    if bytes.len() >= 4 && (bytes.starts_with(b"II*\0") || bytes.starts_with(b"MM\0*")) {
        return Some(FileKind::Image {
            kind: SourceKind::Ldr,
            needs_conversion: true,
        });
    }
    if bytes.starts_with(&EXR_MAGIC) {
        return Some(FileKind::Image {
            kind: SourceKind::Hdr,
            needs_conversion: false,
        });
    }
    if bytes.starts_with(b"#?RADIANCE") || bytes.starts_with(b"#?RGBE") {
        return Some(FileKind::Image {
            kind: SourceKind::Hdr,
            needs_conversion: true,
        });
    }
    None
}

fn classify_by_extension(path: &Path) -> FileKind {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "png" => FileKind::Image {
            kind: SourceKind::Ldr,
            needs_conversion: false,
        },
        "jpg" | "jpeg" | "webp" | "bmp" | "gif" | "tif" | "tiff" => FileKind::Image {
            kind: SourceKind::Ldr,
            needs_conversion: true,
        },
        "exr" => FileKind::Image {
            kind: SourceKind::Hdr,
            needs_conversion: false,
        },
        "hdr" | "rgbe" | "xyze" => FileKind::Image {
            kind: SourceKind::Hdr,
            needs_conversion: true,
        },
        "ktx" => FileKind::Ktx1,
        "ktx2" => FileKind::Ktx2,
        _ => FileKind::Unsupported,
    }
}

/// Natural sort by the last number in the file stem, then by name.
pub fn order_frames(mut paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths.sort_by(|a, b| {
        let ka = frame_sort_key(a);
        let kb = frame_sort_key(b);
        ka.cmp(&kb)
    });
    paths
}

fn frame_sort_key(path: &Path) -> (Option<u64>, String) {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_owned();
    let last_num = last_number_in_stem(stem);
    (last_num, name)
}

fn last_number_in_stem(stem: &str) -> Option<u64> {
    let mut best: Option<(usize, usize)> = None;
    let mut start: Option<usize> = None;
    for (i, c) in stem.char_indices() {
        if c.is_ascii_digit() {
            if start.is_none() {
                start = Some(i);
            }
        } else if let Some(s) = start.take() {
            best = Some((s, i));
        }
    }
    if let Some(s) = start {
        best = Some((s, stem.len()));
    }
    best.and_then(|(s, e)| stem[s..e].parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Dropped animation frames become layers in the order a person reads them.
    #[test]
    fn orders_frames_naturally() {
        let paths = vec![
            PathBuf::from("frame_10.png"),
            PathBuf::from("frame_2.png"),
            PathBuf::from("frame_1.png"),
        ];
        assert_eq!(
            order_frames(paths),
            vec![
                PathBuf::from("frame_1.png"),
                PathBuf::from("frame_2.png"),
                PathBuf::from("frame_10.png"),
            ]
        );
    }
}
