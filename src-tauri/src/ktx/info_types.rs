//! Types for `ktx info --format json` (info schema) and inspector reports.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ImageSelection {
    pub level: u32,
    pub face: u32,
    pub layer: u32,
    pub depth_slice: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PreviewKind {
    Png,
    TonemappedFromExr,
    Unavailable { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct Ktx1Header {
    pub little_endian: bool,
    pub gl_type: u32,
    pub gl_type_size: u32,
    pub gl_format: u32,
    pub gl_internal_format: u32,
    pub gl_base_internal_format: u32,
    pub pixel_width: u32,
    pub pixel_height: u32,
    pub pixel_depth: u32,
    pub number_of_array_elements: u32,
    pub number_of_faces: u32,
    pub number_of_mipmap_levels: u32,
    pub bytes_of_key_value_data: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum InspectKind {
    Ktx2 { info: Box<Ktx2Info> },
    Ktx1 { text: String, header: Ktx1Header },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct InspectReport {
    pub path: String,
    #[ts(type = "number")]
    pub file_size: u64,
    pub kind: InspectKind,
    pub preview: PreviewKind,
}

/// One entry from `ktx info` `messages` ([info_v0](https://schema.khronos.org/ktx/info_v0.json)).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ValidationMessage {
    pub id: u32,
    /// Severity: `fatal` | `error` | `warning` (JSON field name is `type`).
    #[serde(rename = "type")]
    pub type_: String,
    pub message: String,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ByteRange {
    pub byte_offset: u64,
    pub byte_length: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct LevelIndexEntry {
    pub byte_offset: u64,
    pub byte_length: u64,
    pub uncompressed_byte_length: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct Ktx2Header {
    pub identifier: String,
    pub vk_format: String,
    pub type_size: u32,
    pub pixel_width: u32,
    pub pixel_height: u32,
    pub pixel_depth: u32,
    pub layer_count: u32,
    pub face_count: u32,
    pub level_count: u32,
    pub supercompression_scheme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct Ktx2Index {
    pub data_format_descriptor: ByteRange,
    pub key_value_data: ByteRange,
    pub supercompression_global_data: ByteRange,
    pub levels: Vec<LevelIndexEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct DfdSample {
    pub qualifiers: Vec<String>,
    pub channel_type: String,
    pub bit_length: u32,
    pub bit_offset: u32,
    pub sample_position: Vec<u32>,
    /// May be a bit-pattern interpreted as signed (HDR float formats).
    pub sample_lower: i64,
    pub sample_upper: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct DfdBlock {
    pub vendor_id: String,
    pub descriptor_type: String,
    pub version_number: String,
    pub descriptor_block_size: u32,
    pub flags: Vec<String>,
    pub transfer_function: String,
    pub color_primaries: String,
    pub color_model: String,
    pub texel_block_dimension: Vec<u32>,
    pub bytes_plane: Vec<u32>,
    pub samples: Vec<DfdSample>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct DataFormatDescriptor {
    pub total_size: u32,
    pub blocks: Vec<DfdBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct KtxAnimDataInfo {
    pub duration: u32,
    pub timescale: u32,
    #[serde(rename = "loopCount")]
    pub loop_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct KtxGlFormat {
    pub internal_format: Option<u32>,
    pub format: Option<u32>,
    #[serde(rename = "type")]
    pub type_: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct CubemapIncomplete {
    pub px: Option<bool>,
    pub nx: Option<bool>,
    pub py: Option<bool>,
    pub ny: Option<bool>,
    pub pz: Option<bool>,
    pub nz: Option<bool>,
}

/// Value of an unknown or free-form key/value entry.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(untagged)]
pub enum KeyValueValue {
    Text(String),
    Hex { hex: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct KeyValueEntry {
    pub key: String,
    pub value: KeyValueValue,
}

/// Flexible JSON value for keys not modeled as known metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RawKeyValue {
    Text(String),
    Number(serde_json::Number),
    Bool(bool),
    Object(serde_json::Map<String, serde_json::Value>),
    Array(Vec<serde_json::Value>),
    Null,
}

/// Known KTX metadata keys plus remaining pairs as a list for the UI.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct KeyValueData {
    #[serde(rename = "KTXorientation", default)]
    pub orientation: Option<String>,
    #[serde(rename = "KTXswizzle", default)]
    pub swizzle: Option<String>,
    #[serde(rename = "KTXwriter", default)]
    pub writer: Option<String>,
    #[serde(rename = "KTXwriterScParams", default)]
    pub writer_sc_params: Option<String>,
    #[serde(rename = "KTXastcDecodeMode", default)]
    pub astc_decode_mode: Option<String>,
    #[serde(rename = "KTXglFormat", default)]
    pub gl_format: Option<KtxGlFormat>,
    #[serde(rename = "KTXdxgiFormat__", default)]
    pub dxgi_format: Option<u32>,
    #[serde(rename = "KTXmetalPixelFormat", default)]
    pub metal_pixel_format: Option<u32>,
    #[serde(rename = "KTXcubemapIncomplete", default)]
    pub cubemap_incomplete: Option<CubemapIncomplete>,
    #[serde(rename = "KTXanimData", default)]
    pub anim_data: Option<KtxAnimDataInfo>,
    /// Keys outside the known set (or unparsed leftovers). Populated after deserialize when needed.
    #[serde(default)]
    pub other: Vec<KeyValueEntry>,
    /// Catch-all for unknown keys during JSON parse; not exported to TS.
    #[serde(flatten)]
    #[serde(skip_serializing)]
    #[ts(skip)]
    pub unknown: HashMap<String, RawKeyValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct BasisLzImageSlice {
    pub image_flags: Vec<String>,
    pub rgb_slice_byte_length: u32,
    pub rgb_slice_byte_offset: u32,
    pub alpha_slice_byte_length: u32,
    pub alpha_slice_byte_offset: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct BasisLzGlobalData {
    #[serde(rename = "type")]
    pub type_: String,
    pub endpoint_count: u32,
    pub selector_count: u32,
    pub endpoints_byte_length: u32,
    pub selectors_byte_length: u32,
    pub tables_byte_length: u32,
    pub extended_byte_length: u32,
    pub images: Vec<BasisLzImageSlice>,
}

/// Full `ktx info --format json` document for KTX2.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct Ktx2Info {
    pub valid: bool,
    pub messages: Vec<ValidationMessage>,
    pub header: Ktx2Header,
    pub index: Ktx2Index,
    pub data_format_descriptor: DataFormatDescriptor,
    pub key_value_data: KeyValueData,
    #[serde(default)]
    pub supercompression_global_data: Option<BasisLzGlobalData>,
}

impl Ktx2Info {
    /// Move unrecognized key/value keys into `other` as text entries.
    pub fn normalize_other_keys(&mut self) {
        let unknown = std::mem::take(&mut self.key_value_data.unknown);
        for (key, value) in unknown {
            let text = match value {
                RawKeyValue::Text(s) => s,
                RawKeyValue::Number(n) => n.to_string(),
                RawKeyValue::Bool(b) => b.to_string(),
                RawKeyValue::Null => String::new(),
                RawKeyValue::Object(map) => {
                    serde_json::to_string(&map).unwrap_or_else(|_| "{}".to_owned())
                }
                RawKeyValue::Array(arr) => {
                    serde_json::to_string(&arr).unwrap_or_else(|_| "[]".to_owned())
                }
            };
            self.key_value_data.other.push(KeyValueEntry {
                key,
                value: KeyValueValue::Text(text),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn samples_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../docs/ktx-cli/samples")
    }

    #[test]
    fn deserializes_every_info_json_sample() {
        let dir = samples_dir();
        let mut count = 0u32;
        for entry in std::fs::read_dir(&dir).expect("samples dir") {
            let entry = entry.expect("entry");
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            if !path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(".info.json"))
            {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("read sample");
            let mut info: Ktx2Info = serde_json::from_str(&text)
                .unwrap_or_else(|e| panic!("failed to parse {}: {e}", path.display()));
            info.normalize_other_keys();
            count += 1;
        }
        assert!(
            count >= 8,
            "expected at least 8 info.json samples, got {count}"
        );
    }

    #[test]
    fn anim_info_has_ktx_anim_data() {
        let path = samples_dir().join("anim.info.json");
        let text = std::fs::read_to_string(path).unwrap();
        let info: Ktx2Info = serde_json::from_str(&text).unwrap();
        let anim = info.key_value_data.anim_data.expect("KTXanimData present");
        assert_eq!(anim.duration, 1);
        assert_eq!(anim.timescale, 10);
        assert_eq!(anim.loop_count, 0);
    }

    #[test]
    fn validation_message_id_is_integer_per_info_v0() {
        let path = samples_dir().join("basis_lz_with_warning.info.json");
        let text = std::fs::read_to_string(path).unwrap();
        let info: Ktx2Info = serde_json::from_str(&text).unwrap();
        assert_eq!(info.messages.len(), 1);
        let msg = &info.messages[0];
        assert_eq!(msg.id, 6030);
        assert_eq!(msg.type_, "warning");
        assert!(!msg.message.is_empty());
        assert!(!msg.details.is_empty());
    }
}
