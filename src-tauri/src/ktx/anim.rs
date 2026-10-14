//! Write / replace `KTXanimData` in a KTX2 file.

use std::path::Path;

use ktx2::{Header, LevelIndex, Reader};

use crate::error::AppError;
use crate::ktx::options::AnimationOptions;

const ANIM_KEY: &str = "KTXanimData";

/// Add or replace the 12-byte `KTXanimData` key (duration, timescale, loopCount LE u32).
/// Rejects when `layerCount < 2`. Rewrites via temp file + rename.
pub fn write_anim_data(path: &Path, anim: &AnimationOptions) -> Result<(), AppError> {
    let bytes = std::fs::read(path).map_err(|e| AppError::io(e.to_string()))?;
    let reader = Reader::new(bytes.as_slice())
        .map_err(|e| AppError::invalid_input(format!("invalid KTX2: {e}")))?;
    let header = reader.header();
    if header.layer_count < 2 {
        return Err(AppError::invalid_input(
            "KTXanimData requires layerCount >= 2",
        ));
    }

    let level_count = header.level_count.max(1) as usize;
    let mut levels = Vec::with_capacity(level_count);
    for i in 0..level_count {
        let start = Header::LENGTH + i * LevelIndex::LENGTH;
        let entry: [u8; LevelIndex::LENGTH] = bytes
            .get(start..start + LevelIndex::LENGTH)
            .and_then(|s| s.try_into().ok())
            .ok_or_else(|| AppError::invalid_input("level index truncated"))?;
        levels.push(LevelIndex::from_bytes(&entry));
    }

    let dfd_start = header.index.dfd_byte_offset as usize;
    let dfd_end = dfd_start + header.index.dfd_byte_length as usize;
    let dfd = bytes
        .get(dfd_start..dfd_end)
        .ok_or_else(|| AppError::invalid_input("DFD out of range"))?
        .to_vec();

    let mut pairs: Vec<(String, Vec<u8>)> = reader
        .key_value_data()
        .filter(|(k, _)| *k != ANIM_KEY)
        .map(|(k, v)| (k.to_owned(), v.to_vec()))
        .collect();

    let mut anim_value = Vec::with_capacity(12);
    anim_value.extend_from_slice(&anim.duration.to_le_bytes());
    anim_value.extend_from_slice(&anim.timescale.to_le_bytes());
    anim_value.extend_from_slice(&anim.loop_count.to_le_bytes());
    pairs.push((ANIM_KEY.to_owned(), anim_value));
    // Spec: keys sorted by byte value.
    pairs.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    let new_kvd = encode_kvd(&pairs);

    let sgd = if header.index.sgd_byte_length > 0 {
        let s = header.index.sgd_byte_offset as usize;
        let e = s + header.index.sgd_byte_length as usize;
        bytes
            .get(s..e)
            .ok_or_else(|| AppError::invalid_input("SGD out of range"))?
            .to_vec()
    } else {
        Vec::new()
    };

    let level_payloads: Vec<Vec<u8>> = levels
        .iter()
        .map(|lv| {
            let s = lv.byte_offset as usize;
            let e = s + lv.byte_length as usize;
            bytes
                .get(s..e)
                .ok_or_else(|| AppError::invalid_input("level data out of range"))
                .map(|b| b.to_vec())
        })
        .collect::<Result<_, _>>()?;

    // KTX2: no supercompression → lcm(texel_block_size, 4); else 1.
    // Infer from offsets of a file that already validated (common values 4/8/16).
    let level_align = level_data_alignment(header.supercompression_scheme.is_some(), &levels);

    let mut new_header = header;
    let mut out = Vec::new();
    out.extend_from_slice(&new_header.as_bytes());
    let level_index_pos = out.len();
    out.resize(out.len() + level_count * LevelIndex::LENGTH, 0);

    new_header.index.dfd_byte_offset = out.len() as u32;
    new_header.index.dfd_byte_length = dfd.len() as u32;
    out.extend_from_slice(&dfd);

    new_header.index.kvd_byte_offset = out.len() as u32;
    new_header.index.kvd_byte_length = new_kvd.len() as u32;
    out.extend_from_slice(&new_kvd);

    // SGD is 8-byte aligned when present. Do not pad when absent — validate
    // requires exact level byteOffsets after KVD.
    if sgd.is_empty() {
        new_header.index.sgd_byte_offset = 0;
        new_header.index.sgd_byte_length = 0;
    } else {
        while out.len() % 8 != 0 {
            out.push(0);
        }
        new_header.index.sgd_byte_offset = out.len() as u64;
        new_header.index.sgd_byte_length = sgd.len() as u64;
        out.extend_from_slice(&sgd);
    }

    // Mip level array: smallest (levelCount-1) → largest (level 0). Index stays base-first.
    let mut new_levels = levels;
    for i in (0..level_count).rev() {
        while out.len() % level_align != 0 {
            out.push(0);
        }
        let payload = &level_payloads[i];
        new_levels[i].byte_offset = out.len() as u64;
        new_levels[i].byte_length = payload.len() as u64;
        out.extend_from_slice(payload);
    }

    out[..Header::LENGTH].copy_from_slice(&new_header.as_bytes());
    for (i, lv) in new_levels.iter().enumerate() {
        let pos = level_index_pos + i * LevelIndex::LENGTH;
        out[pos..pos + LevelIndex::LENGTH].copy_from_slice(&lv.as_bytes());
    }

    write_atomic(path, &out)
}

/// Required mip-level alignment from the KTX2 spec; inferred from existing offsets.
fn level_data_alignment(has_supercompression: bool, levels: &[LevelIndex]) -> usize {
    if has_supercompression {
        return 1;
    }
    for &align in &[16usize, 8, 4] {
        if levels
            .iter()
            .all(|l| (l.byte_offset as usize).is_multiple_of(align))
        {
            return align;
        }
    }
    4
}

fn encode_kvd(pairs: &[(String, Vec<u8>)]) -> Vec<u8> {
    let mut out = Vec::new();
    for (key, value) in pairs {
        let mut kv = Vec::new();
        kv.extend_from_slice(key.as_bytes());
        kv.push(0);
        kv.extend_from_slice(value);
        let len = kv.len() as u32;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&kv);
        while out.len() % 4 != 0 {
            out.push(0);
        }
    }
    out
}

fn write_atomic(path: &Path, data: &[u8]) -> Result<(), AppError> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut tmp = tempfile::Builder::new()
        .prefix("ktx-anim-")
        .suffix(".ktx2")
        .tempfile_in(parent)
        .map_err(|e| AppError::io(e.to_string()))?;
    use std::io::Write;
    tmp.write_all(data)
        .map_err(|e| AppError::io(e.to_string()))?;
    tmp.flush().map_err(|e| AppError::io(e.to_string()))?;
    let tmp_path = tmp.path().to_path_buf();
    tmp.persist(path).map_err(|e| {
        AppError::io(format!(
            "rename {} -> {}: {}",
            tmp_path.display(),
            path.display(),
            e.error
        ))
    })?;
    Ok(())
}
