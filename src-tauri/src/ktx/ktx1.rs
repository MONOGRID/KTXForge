//! KTX1 64-byte header reader (both endiannesses).

use crate::error::AppError;
use crate::ktx::info_types::Ktx1Header;

const KTX1_MAGIC: [u8; 12] = [
    0xAB, 0x4B, 0x54, 0x58, 0x20, 0x31, 0x31, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A,
];
const HEADER_LEN: usize = 64;
const ENDIAN_REF: u32 = 0x0403_0201;

/// Parse the 64-byte KTX1 header from the start of `bytes`.
pub fn read_header(bytes: &[u8]) -> Result<Ktx1Header, AppError> {
    if bytes.len() < HEADER_LEN {
        return Err(AppError::invalid_input("KTX1 header truncated"));
    }
    if bytes[..12] != KTX1_MAGIC {
        return Err(AppError::invalid_input("not a KTX1 file"));
    }
    let endian_raw = u32::from_le_bytes(bytes[12..16].try_into().unwrap());
    let little_endian = if endian_raw == ENDIAN_REF {
        true
    } else if endian_raw.swap_bytes() == ENDIAN_REF {
        false
    } else {
        return Err(AppError::invalid_input("invalid KTX1 endianness mark"));
    };

    let read_u32 = |offset: usize| -> u32 {
        let raw = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        if little_endian {
            raw
        } else {
            raw.swap_bytes()
        }
    };

    Ok(Ktx1Header {
        little_endian,
        gl_type: read_u32(16),
        gl_type_size: read_u32(20),
        gl_format: read_u32(24),
        gl_internal_format: read_u32(28),
        gl_base_internal_format: read_u32(32),
        pixel_width: read_u32(36),
        pixel_height: read_u32(40),
        pixel_depth: read_u32(44),
        number_of_array_elements: read_u32(48),
        number_of_faces: read_u32(52),
        number_of_mipmap_levels: read_u32(56),
        bytes_of_key_value_data: read_u32(60),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn reads_legacy_ktx_header() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/legacy.ktx");
        let bytes = std::fs::read(path).unwrap();
        let h = read_header(&bytes).unwrap();
        assert!(h.little_endian);
        assert!(h.pixel_width > 0);
        assert!(h.pixel_height > 0);
        assert!(h.number_of_faces >= 1);
    }
}
