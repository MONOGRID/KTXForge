//! Static help copy for encoder options and enum choices (from `ktx create` docs).
//! Vue only renders these strings; it does not own the catalog.

use strum::IntoEnumIterator;

use super::options::{
    AstcQuality, ColorPrimaries, ConvertPrimaries, ConvertTransferFunction, EncodeCodec,
    MipmapFilter, MipmapWrap, OptionChoice, OptionKey, TexcoordOrigin, TransferFunction, VkFormat,
};

/// Short description of what an option does (label tooltip).
pub fn help_for(key: OptionKey) -> &'static str {
    match key {
        OptionKey::OneD => "Create a 1D texture. If unset, the texture is 2D or 3D.",
        OptionKey::Cubemap => {
            "Cubemap faces in order +X, −X, +Y, −Y, +Z, −Z. Texture-coordinate origin must be top-left."
        }
        OptionKey::Layers => {
            "Array texture layer count. Also the frame count when writing KTXanimData."
        }
        OptionKey::Depth => {
            "Base level depth in pixels. When set, the texture is 3D and an origin with -front or -back is required."
        }
        OptionKey::Levels => {
            "Mip level count. With generate-mipmap, builds this many levels; otherwise this many input images are required."
        }
        OptionKey::GenerateMipmap => {
            "Generate the mipmap chain during create. Full chain if levels is omitted. Not valid for SINT, UINT, 3D, or raw."
        }
        OptionKey::RuntimeMipmap => {
            "Mark the texture for client-side mipmap generation later. Mutually exclusive with generate-mipmap. Not valid with BasisLZ, UASTC, or ASTC encoding."
        }
        OptionKey::Width => {
            "Base level width. Required for raw; otherwise resamples the image and enables mipmap filter options."
        }
        OptionKey::Height => {
            "Base level height. Required for raw; otherwise resamples the image and enables mipmap filter options."
        }
        OptionKey::Scale => {
            "Scale images as they are loaded. Not valid with raw. Enables mipmap filter options."
        }
        OptionKey::Raw => "Treat input as raw pixel data. Width and height are required.",
        OptionKey::MipmapFilter => {
            "Filter used when generating mipmaps or resampling. Default lanczos4."
        }
        OptionKey::MipmapFilterScale => "Filter scale for mipmap/resample. Default 1.0.",
        OptionKey::MipmapWrap => {
            "How to sample pixels near image boundaries when filtering. Default clamp."
        }
        OptionKey::AssignTf => {
            "Force the created texture transfer function, ignoring the input file(s)."
        }
        OptionKey::ConvertTf => {
            "Convert input to the chosen transfer function (linear or sRGB aliases). Not valid with raw."
        }
        OptionKey::AssignPrimaries => "Force the created texture color primaries.",
        OptionKey::ConvertPrimaries => {
            "Convert input primaries. Not valid with raw or when assigned primaries are none."
        }
        OptionKey::AssignTexcoordOrigin => {
            "Force texture-coordinate origin (top/bottom-left; append -front/-back when depth is set)."
        }
        OptionKey::ConvertTexcoordOrigin => {
            "Convert origin (vertical flip only). Not valid with raw. Cubemaps must stay top-left."
        }
        OptionKey::FailOnColorConversions => "Error if any color conversion would run.",
        OptionKey::WarnOnColorConversions => {
            "Warn on every conversion, including visually lossless ones."
        }
        OptionKey::NoWarnOnColorConversions => {
            "Silence conversion warnings. Overrides the warn flag."
        }
        OptionKey::FailOnOriginChanges => "Error if an origin flip is required.",
        OptionKey::WarnOnOriginChanges => "Warn when an origin changes.",
        OptionKey::Swizzle => "KTX swizzle metadata written into the file. Four characters from rgba01.",
        OptionKey::InputSwizzle => {
            "Pre-swizzle input channels before create. Four characters from rgba01."
        }
        OptionKey::Normalize => {
            "Normalize input normals to unit length. Linear, 2+ components. Not valid with raw."
        }
        OptionKey::NormalMode => {
            "Normal-map encode mode. RGB(A) XYZ becomes X+Y in RGB+A. For BasisLZ disables endpoint and selector RDO."
        }
        OptionKey::Format => {
            "Vulkan format written into the KTX2 file (no VK_FORMAT_ prefix). With encode, must be an uncompressed R8… format."
        }
        OptionKey::Encode => "Encode with BasisLZ or UASTC after create. Case insensitive.",
        OptionKey::Qlevel => {
            "BasisLZ quality 1–255. Lower is smaller/faster/lower quality. Default 128 when endpoints and selectors are unset."
        }
        OptionKey::Clevel => "BasisLZ compression level 0–6 (default 1). Speed versus quality.",
        OptionKey::MaxEndpoints => {
            "BasisLZ max endpoints 1–16128. Takes effect only together with max-selectors."
        }
        OptionKey::MaxSelectors => "BasisLZ max selectors 1–16128.",
        OptionKey::EndpointRdoThreshold => {
            "BasisLZ endpoint RDO threshold (default 1.25, useful ~1.0–3.0). Overrides qlevel."
        }
        OptionKey::SelectorRdoThreshold => {
            "BasisLZ selector RDO threshold (default 1.25, useful ~1.0–3.0)."
        }
        OptionKey::NoEndpointRdo => "Disable BasisLZ endpoint RDO (on by default).",
        OptionKey::NoSelectorRdo => "Disable BasisLZ selector RDO (on by default).",
        OptionKey::UastcQuality => {
            "UASTC quality 0 (fastest) through 4 (very slow). Default 1."
        }
        OptionKey::UastcRdo => "Enable the UASTC RDO post-process.",
        OptionKey::UastcRdoL => {
            "UASTC RDO lambda 0.001–10 (default 1.0). Normals: try 0.25–0.75."
        }
        OptionKey::UastcRdoD => "UASTC RDO dictionary size 64–65536 bytes (default 4096).",
        OptionKey::UastcRdoB => {
            "UASTC RDO smooth-block error scale 1–300 (default 10). 1 disables."
        }
        OptionKey::UastcRdoS => {
            "UASTC RDO smooth-block standard deviation 0.01–65536 (default 18)."
        }
        OptionKey::UastcRdoF => "Do not favor simpler UASTC modes during RDO.",
        OptionKey::UastcRdoM => "Disable UASTC RDO multithreading (slightly higher compression, deterministic).",
        OptionKey::AstcQuality => {
            "ASTC search quality: fastest, fast, medium (default), thorough, exhaustive."
        }
        OptionKey::AstcPerceptual => {
            "Use perceptual error instead of RMS. For normal maps and RGB only."
        }
        OptionKey::Threads => {
            "Encoder thread count. Default is hardware concurrency for BasisLZ/UASTC, or 1."
        }
        OptionKey::NoSse => "Forbid SSE on BasisLZ and UASTC compressors.",
        OptionKey::CompareSsim => "Compute SSIM after encode (BasisLZ, UASTC, or ASTC).",
        OptionKey::ComparePsnr => "Compute PSNR after encode (BasisLZ, UASTC, or ASTC).",
        OptionKey::Zstd => {
            "Zstd supercompression level 1–22. Levels above 20 use much more memory. Not valid with BasisLZ. Recommended with UASTC."
        }
        OptionKey::Zlib => "Zlib supercompression level 1–9. Not valid with BasisLZ.",
    }
}

/// Choice-level help for enum options. Empty for non-enum keys.
pub fn choices_for(key: OptionKey) -> Vec<OptionChoice> {
    match key {
        OptionKey::Format => VkFormat::iter().map(vk_format_choice).collect(),
        OptionKey::Encode => EncodeCodec::iter().map(encode_choice).collect(),
        OptionKey::MipmapFilter => MipmapFilter::iter().map(mipmap_filter_choice).collect(),
        OptionKey::MipmapWrap => MipmapWrap::iter().map(mipmap_wrap_choice).collect(),
        OptionKey::AssignTf => TransferFunction::iter().map(transfer_fn_choice).collect(),
        OptionKey::ConvertTf => ConvertTransferFunction::iter()
            .map(convert_tf_choice)
            .collect(),
        OptionKey::AssignPrimaries => ColorPrimaries::iter().map(primaries_choice).collect(),
        OptionKey::ConvertPrimaries => ConvertPrimaries::iter()
            .map(convert_primaries_choice)
            .collect(),
        OptionKey::AssignTexcoordOrigin | OptionKey::ConvertTexcoordOrigin => {
            TexcoordOrigin::iter().map(texcoord_origin_choice).collect()
        }
        OptionKey::AstcQuality => AstcQuality::iter().map(astc_quality_choice).collect(),
        _ => Vec::new(),
    }
}

fn choice(value: &str, help: &'static str) -> OptionChoice {
    OptionChoice {
        value: value.to_owned(),
        help: help.to_owned(),
    }
}

fn encode_choice(v: EncodeCodec) -> OptionChoice {
    match v {
        EncodeCodec::BasisLz => choice(
            v.cli(),
            "ETC1S / BasisLZ universal GPU texture. Smaller files; R→RRR, RG→RRRG.",
        ),
        EncodeCodec::Uastc => choice(
            v.cli(),
            "UASTC LDR 4×4. Higher quality than BasisLZ; often paired with Zstd.",
        ),
    }
}

fn mipmap_filter_choice(v: MipmapFilter) -> OptionChoice {
    match v {
        MipmapFilter::Box => choice(v.cli(), "Box / averaging filter. Fastest, softest result."),
        MipmapFilter::Tent => choice(v.cli(), "Tent (linear/triangle) filter."),
        MipmapFilter::Bell => choice(v.cli(), "Bell curve filter."),
        MipmapFilter::BSpline => choice(v.cli(), "Cubic B-spline. Smooth, slightly soft."),
        MipmapFilter::Mitchell => choice(
            v.cli(),
            "Mitchell–Netravali. Good balance of sharpness and ringing.",
        ),
        MipmapFilter::Blackman => choice(v.cli(), "Blackman-windowed sinc."),
        MipmapFilter::Lanczos3 => choice(
            v.cli(),
            "Lanczos with 3 lobes. Sharper than box; some ringing.",
        ),
        MipmapFilter::Lanczos4 => choice(
            v.cli(),
            "Lanczos with 4 lobes. Default; sharp with controlled ringing.",
        ),
        MipmapFilter::Lanczos6 => choice(v.cli(), "Lanczos with 6 lobes. Sharper; more ringing."),
        MipmapFilter::Lanczos12 => choice(
            v.cli(),
            "Lanczos with 12 lobes. Very sharp; strong ringing risk.",
        ),
        MipmapFilter::Kaiser => choice(v.cli(), "Kaiser-windowed sinc."),
        MipmapFilter::Gaussian => choice(v.cli(), "Gaussian blur. Soft, little ringing."),
        MipmapFilter::Catmullrom => choice(v.cli(), "Catmull–Rom spline. Sharp interpolator."),
        MipmapFilter::QuadraticInterp => choice(v.cli(), "Quadratic interpolation filter."),
        MipmapFilter::QuadraticApprox => choice(v.cli(), "Quadratic approximation filter."),
        MipmapFilter::QuadraticMix => choice(v.cli(), "Quadratic mix filter."),
    }
}

fn mipmap_wrap_choice(v: MipmapWrap) -> OptionChoice {
    match v {
        MipmapWrap::Wrap => choice(v.cli(), "Tile / wrap at edges (periodic)."),
        MipmapWrap::Reflect => choice(v.cli(), "Mirror at edges."),
        MipmapWrap::Clamp => choice(v.cli(), "Clamp to edge pixels. Default."),
    }
}

fn transfer_fn_choice(v: TransferFunction) -> OptionChoice {
    match v {
        TransferFunction::Linear => choice(v.cli(), "Linear light (no gamma curve)."),
        TransferFunction::Srgb => choice(v.cli(), "sRGB transfer function."),
        TransferFunction::SrgbEotf => choice(v.cli(), "sRGB EOTF alias."),
        TransferFunction::Scrgb => choice(v.cli(), "scRGB transfer function."),
        TransferFunction::ScrgbEotf => choice(v.cli(), "scRGB EOTF alias."),
        TransferFunction::Itu => choice(v.cli(), "ITU transfer function."),
        TransferFunction::ItuOetf => choice(v.cli(), "ITU OETF."),
        TransferFunction::Bt601 => choice(v.cli(), "BT.601 transfer."),
        TransferFunction::Bt601Oetf => choice(v.cli(), "BT.601 OETF."),
        TransferFunction::Bt709 => choice(v.cli(), "BT.709 transfer."),
        TransferFunction::Bt709Oetf => choice(v.cli(), "BT.709 OETF."),
        TransferFunction::Bt2020 => choice(v.cli(), "BT.2020 transfer."),
        TransferFunction::Bt2020Oetf => choice(v.cli(), "BT.2020 OETF."),
        TransferFunction::Smpte170m => choice(v.cli(), "SMPTE 170M transfer."),
        TransferFunction::Smpte170mOetf => choice(v.cli(), "SMPTE 170M OETF."),
        TransferFunction::Smpte170mEotf => choice(v.cli(), "SMPTE 170M EOTF."),
        TransferFunction::Ntsc => choice(v.cli(), "NTSC transfer."),
        TransferFunction::NtscEotf => choice(v.cli(), "NTSC EOTF."),
        TransferFunction::Slog => choice(v.cli(), "S-Log transfer."),
        TransferFunction::SlogOetf => choice(v.cli(), "S-Log OETF."),
        TransferFunction::Slog2 => choice(v.cli(), "S-Log2 transfer."),
        TransferFunction::Slog2Oetf => choice(v.cli(), "S-Log2 OETF."),
        TransferFunction::Bt1886 => choice(v.cli(), "BT.1886 transfer."),
        TransferFunction::Bt1886Eotf => choice(v.cli(), "BT.1886 EOTF."),
        TransferFunction::HlgOetf => choice(v.cli(), "Hybrid Log-Gamma OETF."),
        TransferFunction::HlgEotf => choice(v.cli(), "Hybrid Log-Gamma EOTF."),
        TransferFunction::PqOetf => choice(v.cli(), "Perceptual Quantizer (PQ) OETF."),
        TransferFunction::PqEotf => choice(v.cli(), "Perceptual Quantizer (PQ) EOTF."),
        TransferFunction::Dcip3 => choice(v.cli(), "DCI-P3 transfer."),
        TransferFunction::Dcip3Eotf => choice(v.cli(), "DCI-P3 EOTF."),
        TransferFunction::PalOetf => choice(v.cli(), "PAL OETF."),
        TransferFunction::Pal625Eotf => choice(v.cli(), "PAL 625 EOTF."),
        TransferFunction::St240 => choice(v.cli(), "SMPTE ST 240 transfer."),
        TransferFunction::St240Oetf => choice(v.cli(), "SMPTE ST 240 OETF."),
        TransferFunction::St240Eotf => choice(v.cli(), "SMPTE ST 240 EOTF."),
        TransferFunction::Acescc => choice(v.cli(), "ACEScc log transfer."),
        TransferFunction::AcesccOetf => choice(v.cli(), "ACEScc OETF."),
        TransferFunction::Acescct => choice(v.cli(), "ACEScct transfer."),
        TransferFunction::AcescctOetf => choice(v.cli(), "ACEScct OETF."),
        TransferFunction::Adobergb => choice(v.cli(), "Adobe RGB transfer."),
        TransferFunction::AdobergbEotf => choice(v.cli(), "Adobe RGB EOTF."),
    }
}

fn convert_tf_choice(v: ConvertTransferFunction) -> OptionChoice {
    match v {
        ConvertTransferFunction::Linear => choice(v.cli(), "Convert pixels to linear light."),
        ConvertTransferFunction::Srgb => choice(v.cli(), "Convert pixels to sRGB."),
        ConvertTransferFunction::SrgbEotf => choice(v.cli(), "sRGB EOTF alias for convert."),
        ConvertTransferFunction::Scrgb => choice(v.cli(), "scRGB alias for convert."),
        ConvertTransferFunction::ScrgbEotf => choice(v.cli(), "scRGB EOTF alias for convert."),
    }
}

fn primaries_choice(v: ColorPrimaries) -> OptionChoice {
    match v {
        ColorPrimaries::None => choice(v.cli(), "No color primaries. Blocks convert-primaries."),
        ColorPrimaries::Bt709 => choice(v.cli(), "BT.709 / Rec.709 primaries."),
        ColorPrimaries::Srgb => choice(v.cli(), "sRGB primaries (same as BT.709)."),
        ColorPrimaries::Bt601Ebu => choice(v.cli(), "BT.601 EBU (625-line) primaries."),
        ColorPrimaries::Bt601Smpte => choice(v.cli(), "BT.601 SMPTE (525-line) primaries."),
        ColorPrimaries::Bt2020 => choice(v.cli(), "BT.2020 / Rec.2020 wide gamut."),
        ColorPrimaries::Ciexyz => choice(v.cli(), "CIE XYZ primaries."),
        ColorPrimaries::Aces => choice(v.cli(), "ACES AP0 primaries."),
        ColorPrimaries::Acescc => choice(v.cli(), "ACEScc / AP1-related primaries."),
        ColorPrimaries::Ntsc1953 => choice(v.cli(), "NTSC 1953 primaries."),
        ColorPrimaries::Pal525 => choice(v.cli(), "PAL 525 primaries."),
        ColorPrimaries::Displayp3 => choice(v.cli(), "Display P3 primaries."),
        ColorPrimaries::Adobergb => choice(v.cli(), "Adobe RGB primaries."),
    }
}

fn convert_primaries_choice(v: ConvertPrimaries) -> OptionChoice {
    // Same meanings as assign; convert actually transforms pixels.
    match v {
        ConvertPrimaries::Bt709 => choice(v.cli(), "Convert to BT.709 / Rec.709 primaries."),
        ConvertPrimaries::Srgb => choice(v.cli(), "Convert to sRGB primaries."),
        ConvertPrimaries::Bt601Ebu => choice(v.cli(), "Convert to BT.601 EBU primaries."),
        ConvertPrimaries::Bt601Smpte => choice(v.cli(), "Convert to BT.601 SMPTE primaries."),
        ConvertPrimaries::Bt2020 => choice(v.cli(), "Convert to BT.2020 primaries."),
        ConvertPrimaries::Ciexyz => choice(v.cli(), "Convert to CIE XYZ primaries."),
        ConvertPrimaries::Aces => choice(v.cli(), "Convert to ACES AP0 primaries."),
        ConvertPrimaries::Acescc => choice(v.cli(), "Convert to ACEScc / AP1-related primaries."),
        ConvertPrimaries::Ntsc1953 => choice(v.cli(), "Convert to NTSC 1953 primaries."),
        ConvertPrimaries::Pal525 => choice(v.cli(), "Convert to PAL 525 primaries."),
        ConvertPrimaries::Displayp3 => choice(v.cli(), "Convert to Display P3 primaries."),
        ConvertPrimaries::Adobergb => choice(v.cli(), "Convert to Adobe RGB primaries."),
    }
}

fn texcoord_origin_choice(v: TexcoordOrigin) -> OptionChoice {
    match v {
        TexcoordOrigin::TopLeft => choice(v.cli(), "s=0,t=0 at top-left. Required for cubemaps."),
        TexcoordOrigin::BottomLeft => choice(v.cli(), "s=0,t=0 at bottom-left (OpenGL-style)."),
        TexcoordOrigin::TopLeftFront => {
            choice(v.cli(), "3D: top-left on the front face (depth set).")
        }
        TexcoordOrigin::TopLeftBack => {
            choice(v.cli(), "3D: top-left on the back face (depth set).")
        }
        TexcoordOrigin::BottomLeftFront => {
            choice(v.cli(), "3D: bottom-left on the front face (depth set).")
        }
        TexcoordOrigin::BottomLeftBack => {
            choice(v.cli(), "3D: bottom-left on the back face (depth set).")
        }
    }
}

fn astc_quality_choice(v: AstcQuality) -> OptionChoice {
    match v {
        AstcQuality::Fastest => choice(v.cli(), "Quality ≈ 0. Fastest encode, lowest quality."),
        AstcQuality::Fast => choice(v.cli(), "Quality ≈ 10. Quick encode."),
        AstcQuality::Medium => choice(v.cli(), "Quality ≈ 60. Default balance."),
        AstcQuality::Thorough => choice(v.cli(), "Quality ≈ 98. Slow, high quality."),
        AstcQuality::Exhaustive => choice(v.cli(), "Quality ≈ 100. Slowest; fullest search."),
    }
}

fn vk_format_choice(v: VkFormat) -> OptionChoice {
    match v {
        VkFormat::R8Unorm => choice(v.cli(), "8-bit single channel, linear UNORM."),
        VkFormat::R8Srgb => choice(v.cli(), "8-bit single channel, sRGB."),
        VkFormat::R8G8Unorm => choice(v.cli(), "8-bit RG, linear UNORM."),
        VkFormat::R8G8Srgb => choice(v.cli(), "8-bit RG, sRGB."),
        VkFormat::R8G8B8Unorm => choice(v.cli(), "8-bit RGB, linear UNORM."),
        VkFormat::R8G8B8Srgb => choice(v.cli(), "8-bit RGB, sRGB."),
        VkFormat::R8G8B8A8Unorm => choice(v.cli(), "8-bit RGBA, linear UNORM."),
        VkFormat::R8G8B8A8Srgb => choice(v.cli(), "8-bit RGBA, sRGB. Common LDR default."),
        VkFormat::R16G16B16Sfloat => choice(v.cli(), "16-bit float RGB (HDR). Typically from EXR."),
        VkFormat::R16G16B16A16Sfloat => {
            choice(v.cli(), "16-bit float RGBA (HDR). Typically from EXR.")
        }
        VkFormat::Astc4x4UnormBlock => choice(
            v.cli(),
            "ASTC 4×4 block, linear. Highest ASTC quality / largest.",
        ),
        VkFormat::Astc4x4SrgbBlock => choice(v.cli(), "ASTC 4×4 block, sRGB."),
        VkFormat::Astc5x4UnormBlock => choice(v.cli(), "ASTC 5×4 block, linear."),
        VkFormat::Astc5x4SrgbBlock => choice(v.cli(), "ASTC 5×4 block, sRGB."),
        VkFormat::Astc5x5UnormBlock => choice(v.cli(), "ASTC 5×5 block, linear."),
        VkFormat::Astc5x5SrgbBlock => choice(v.cli(), "ASTC 5×5 block, sRGB."),
        VkFormat::Astc6x5UnormBlock => choice(v.cli(), "ASTC 6×5 block, linear."),
        VkFormat::Astc6x5SrgbBlock => choice(v.cli(), "ASTC 6×5 block, sRGB."),
        VkFormat::Astc6x6UnormBlock => choice(v.cli(), "ASTC 6×6 block, linear."),
        VkFormat::Astc6x6SrgbBlock => choice(v.cli(), "ASTC 6×6 block, sRGB."),
        VkFormat::Astc8x5UnormBlock => choice(v.cli(), "ASTC 8×5 block, linear."),
        VkFormat::Astc8x5SrgbBlock => choice(v.cli(), "ASTC 8×5 block, sRGB."),
        VkFormat::Astc8x6UnormBlock => choice(v.cli(), "ASTC 8×6 block, linear."),
        VkFormat::Astc8x6SrgbBlock => choice(v.cli(), "ASTC 8×6 block, sRGB."),
        VkFormat::Astc8x8UnormBlock => choice(v.cli(), "ASTC 8×8 block, linear."),
        VkFormat::Astc8x8SrgbBlock => choice(v.cli(), "ASTC 8×8 block, sRGB."),
        VkFormat::Astc10x5UnormBlock => choice(v.cli(), "ASTC 10×5 block, linear."),
        VkFormat::Astc10x5SrgbBlock => choice(v.cli(), "ASTC 10×5 block, sRGB."),
        VkFormat::Astc10x6UnormBlock => choice(v.cli(), "ASTC 10×6 block, linear."),
        VkFormat::Astc10x6SrgbBlock => choice(v.cli(), "ASTC 10×6 block, sRGB."),
        VkFormat::Astc10x8UnormBlock => choice(v.cli(), "ASTC 10×8 block, linear."),
        VkFormat::Astc10x8SrgbBlock => choice(v.cli(), "ASTC 10×8 block, sRGB."),
        VkFormat::Astc10x10UnormBlock => choice(v.cli(), "ASTC 10×10 block, linear."),
        VkFormat::Astc10x10SrgbBlock => choice(v.cli(), "ASTC 10×10 block, sRGB."),
        VkFormat::Astc12x10UnormBlock => choice(v.cli(), "ASTC 12×10 block, linear."),
        VkFormat::Astc12x10SrgbBlock => choice(v.cli(), "ASTC 12×10 block, sRGB."),
        VkFormat::Astc12x12UnormBlock => choice(
            v.cli(),
            "ASTC 12×12 block, linear. Smallest ASTC / lowest quality.",
        ),
        VkFormat::Astc12x12SrgbBlock => choice(v.cli(), "ASTC 12×12 block, sRGB."),
    }
}
