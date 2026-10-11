pub mod anim;
pub mod create_args;
pub mod extract_args;
pub mod extract_ops;
pub mod info;
pub mod info_types;
pub mod input_role;
pub mod ktx1;
pub mod metrics;
pub mod option_help;
pub mod options;
pub mod rules;
pub mod sidecar;

// Re-exports for later waves (engines/commands). Allowed unused while stubs remain.
#[allow(unused_imports)]
pub use info_types::{
    ImageSelection, InspectKind, InspectReport, Ktx1Header, Ktx2Info, PreviewKind,
};
#[allow(unused_imports)]
pub use options::{
    AnimationOptions, CreateOptions, EncodeCodec, FieldState, OptionKey, OptionReport,
    OptionSuggestion, OptionWarning, SourcesClass, VkFormat, WarningLevel,
};
