## [1.1.3](https://github.com/MONOGRID/KTXForge/compare/v1.1.2...v1.1.3) (2026-10-19)

### Bug Fixes

* re-sign bundled libktx dylib ad hoc so macOS sidecars can load it ([bf07216](https://github.com/MONOGRID/KTXForge/commit/bf0721655232959511ab21dabece152affb22c6a))

## [1.1.2](https://github.com/MONOGRID/KTXForge/compare/v1.1.1...v1.1.2) (2026-10-17)

### Bug Fixes

* OS open no longer quits on Windows, menu survives closing a window, update banner only offers real updates ([2f58999](https://github.com/MONOGRID/KTXForge/commit/2f58999416e0186552f264196aacb014cbae8d96))
* View zoom menu and shortcuts work in a newly opened inspector ([09f5817](https://github.com/MONOGRID/KTXForge/commit/09f5817f31245b7d9394432f5ed7923bb38bb770))

## [1.1.1](https://github.com/MONOGRID/KTXForge/compare/v1.1.0...v1.1.1) (2026-10-14)

### Bug Fixes

* "Fit" mode horizontal centering in low res images now works correctly ([2534569](https://github.com/MONOGRID/KTXForge/commit/2534569ddb44b9aa18482f6d380117073a51d088))
* BasisLZ animations, 16-bit previews, 1:1 view and linear presets ([6c5c0c1](https://github.com/MONOGRID/KTXForge/commit/6c5c0c1cf711828dc359c3469c597547d9e78deb))
* correctly export all cubemap faces when "all frames at current levels" is selected in inspector ([4ee6106](https://github.com/MONOGRID/KTXForge/commit/4ee61066e10d89935bf18608cafd0073757a4c1a))
* cubemap faces selector now is mutually exclusive with cross ([609511c](https://github.com/MONOGRID/KTXForge/commit/609511c9a66939d9bdfee0300a9b48fcb97273e7))
* encoder-inspector interoperability fixed, for example encoder can be opened by inspector "Open.." ([8489569](https://github.com/MONOGRID/KTXForge/commit/8489569f3a1b95cd9753eff47a65f8cf802338b8))
* export logs with dedicated UI, no longer clutters main UI ([96a910a](https://github.com/MONOGRID/KTXForge/commit/96a910ae6e163ba5a75a5f4a7009a7f3820b67ee))
* native menu appears on inspector window on os file association open ([faaf6b9](https://github.com/MONOGRID/KTXForge/commit/faaf6b91c41520fac6c700f593ecd0cf03c61e93))
* options rules never build a ktx create command line the tool rejects ([c021c7e](https://github.com/MONOGRID/KTXForge/commit/c021c7ea4803f124aac1a11fd8b86a07ccdb18d4))
* when opening small files in big viewports and using "Fit" , the images are now centered correctly ([af6a788](https://github.com/MONOGRID/KTXForge/commit/af6a7889c42641e84ef1ad4c6b01e6e267ead120))

## [1.1.0](https://github.com/MONOGRID/KTXForge/compare/v1.0.0...v1.1.0) (2026-10-13)

### Features

* enhance inspector functionality with vertical flip option ([6d68514](https://github.com/MONOGRID/KTXForge/commit/6d68514ab88d5983b40c3e1328869a1445705b3a))
* implement multi-input pack validation for KTX encoding ([af32193](https://github.com/MONOGRID/KTXForge/commit/af321937f0ebd2e297b3c425aaf59bef542b688e))

### Bug Fixes

* fix menu zoom functionality ([a344465](https://github.com/MONOGRID/KTXForge/commit/a34446528006c73a4bfedf77741487fb19716be6))
* wayland startup quirk ([6b8fbe0](https://github.com/MONOGRID/KTXForge/commit/6b8fbe02c3356e8363bdf9dd0230ce08d3f24405))
* zstd 18 is the new default for presets, balance between memory consumption and high compression ([fcae0cf](https://github.com/MONOGRID/KTXForge/commit/fcae0cffde15e1ad9801c7920f57e182f1dbaf8c))

## 1.0.0 (2026-10-13)

### Features

* add icon support and update favicon links ([58c0dd2](https://github.com/MONOGRID/KTXForge/commit/58c0dd200d0b90e099d0b29892a288e314d11af8))
* add OS file associations for `.ktx` and `.ktx2` and enhance inspector opening methods ([3f8e2ec](https://github.com/MONOGRID/KTXForge/commit/3f8e2ec58c62fac3547296cc42a6603f87ed5a8e))
* add Tauri dialog and notification plugins, enhance error handling ([5ae9b5f](https://github.com/MONOGRID/KTXForge/commit/5ae9b5feae163d7eff1bfcd7ae68a6cc10967520))
* dark mode default ([6f9b042](https://github.com/MONOGRID/KTXForge/commit/6f9b0425908ef22e5d18e4369eff48daf1bfd886))
* encode presets UI ([b4ebb88](https://github.com/MONOGRID/KTXForge/commit/b4ebb88b0f342c2603ca158c8755c88e6b480226))
* **encoder:** add CSS3D depth volume preview and inspector support ([8e65667](https://github.com/MONOGRID/KTXForge/commit/8e6566727bb577c07b2a7a069297f817efb55011))
* **encoder:** animation encode bar and preview [T5.4] ([e4d90b8](https://github.com/MONOGRID/KTXForge/commit/e4d90b822c8cd7664fab65c396b8529fb9885b53))
* **encoder:** container filter color option sections [T5.2] ([62db313](https://github.com/MONOGRID/KTXForge/commit/62db3132cae66f413e111835180c1cfbfb5034a5))
* **encoder:** container filter color option sections [T5.2] ([2d9f549](https://github.com/MONOGRID/KTXForge/commit/2d9f5492402808f107c74fcb084e7316dd6dfaf1))
* **encoder:** enforce strict transfer function rules for sRGB formats ([cc03839](https://github.com/MONOGRID/KTXForge/commit/cc0383931f44d45376c5c6f67fea028db484319f))
* **encoder:** enforce valid encoding combinations and enhance option handling ([dd98cf1](https://github.com/MONOGRID/KTXForge/commit/dd98cf1cea7be826fe4776589fcc112b9de81a51))
* **encoder:** enhance menu integration and tooltip support ([e205454](https://github.com/MONOGRID/KTXForge/commit/e20545455ea51c9d8a8749ef8e916a71da27f954))
* **encoder:** enhance preview pane speed ([a607a03](https://github.com/MONOGRID/KTXForge/commit/a607a0360b7da00a98a8a2b090c5cf9751d03b87))
* **encoder:** enhance UI with resizable panels and new input components ([d14234a](https://github.com/MONOGRID/KTXForge/commit/d14234a9eb265890e91fe2d3b6754ed1a6a9b1dc))
* **encoder:** format codec encoder option sections [T5.3] ([1626660](https://github.com/MONOGRID/KTXForge/commit/1626660d03a88decfec18648742b2fbad59de861))
* **encoder:** format codec encoder option sections [T5.3] ([29dc4a2](https://github.com/MONOGRID/KTXForge/commit/29dc4a20d9b56ef2e528e498d627df56fea03b35))
* **encoder:** implement batch outputs and role-typed inputs for enhanced encoding workflow ([c58b803](https://github.com/MONOGRID/KTXForge/commit/c58b8035e625ba896bbc7f4eb80137dc4c85bd33))
* **encoder:** implement role-aware preview with cubemap cross wipe and generated mip sliders ([342a97d](https://github.com/MONOGRID/KTXForge/commit/342a97d4e89298523933321998bdfed9d02c3f81))
* **encoder:** refine option handling and UI enhancements ([80bae19](https://github.com/MONOGRID/KTXForge/commit/80bae1902967411ca9a479160c39439daaee53f5))
* **encoder:** state store and shell layout [T5.1] ([f5060c7](https://github.com/MONOGRID/KTXForge/commit/f5060c7c6b6839af1104672966a2c5f60d8dd517))
* **inspector:** enhance viewer functionality with zoom controls and layout improvements ([1b65744](https://github.com/MONOGRID/KTXForge/commit/1b6574489673df76b015a9a17ddf5baa5d68b28f))
* **inspector:** export panel [T5.7] ([20a8461](https://github.com/MONOGRID/KTXForge/commit/20a8461d6985991a7c3076e686845923f5c66cf2))
* **inspector:** export panel [T5.7] ([b200b63](https://github.com/MONOGRID/KTXForge/commit/b200b6333df98df829c9dfaf64d81e88ccb936aa))
* **inspector:** mip browser selectors and frame player [T5.6] ([e5f27cb](https://github.com/MONOGRID/KTXForge/commit/e5f27cb5e711e09e57608c18dccb6480cc4ba369))
* **inspector:** mip browser selectors and frame player [T5.6] ([fb0be01](https://github.com/MONOGRID/KTXForge/commit/fb0be01c4e635a9d1b486881b13e60173e38828c))
* **inspector:** report panels and store [T5.5] ([413a5fc](https://github.com/MONOGRID/KTXForge/commit/413a5fc6bda59d1e74e49492cf649984ce80d511))
* **inspector:** report panels and store [T5.5] ([e0bf81b](https://github.com/MONOGRID/KTXForge/commit/e0bf81bad9bbf9b35775a8585c5d7dc8f958ae56))
* integrate vue-sonner for enhanced error handling and notifications ([e55b176](https://github.com/MONOGRID/KTXForge/commit/e55b1768667ee158ad9386b80fb881ede12b026b))
* moad icons ([1a03569](https://github.com/MONOGRID/KTXForge/commit/1a035697ec0bff619e29c44bfd832462779cc2c7))
* os native notifications and modals ([7dfa11b](https://github.com/MONOGRID/KTXForge/commit/7dfa11b8f9e537dfe4d39c8b9118838adc09842f))
* rename project to KTXForge and update related files ([6bb7854](https://github.com/MONOGRID/KTXForge/commit/6bb7854a8472e097d5758381932be6430769abe7))
* reorganize some layout ([cea0ce1](https://github.com/MONOGRID/KTXForge/commit/cea0ce1ece49e05f43fddaaf6056e291668abd53))
* smaller ui components ([9cbf345](https://github.com/MONOGRID/KTXForge/commit/9cbf34512bb32ebc15aff15f30b331bb61b5cd2e))
* **ui:** integrate shadcn-vue with UnoCSS for UI components ([34c512d](https://github.com/MONOGRID/KTXForge/commit/34c512d8ce9bb58d65ca4b2a0e82e0a421509db5))
* unify input role handling and enhance options resolution for KTX encoding ([80fef81](https://github.com/MONOGRID/KTXForge/commit/80fef81bcab00f456b657db8e0c4e231b745cf71))
* **updater:** update banner UI [T5.8] ([ad9aeed](https://github.com/MONOGRID/KTXForge/commit/ad9aeed0edc51788be9ad46adb008f7de5c32b5f))
* **updater:** update banner UI [T5.8] ([fad5b6a](https://github.com/MONOGRID/KTXForge/commit/fad5b6aab493d0d08480b6087090202cbd49597d))
* **viewer:** implement shared zoom and pan functionality across inspector and encoder ([4829a40](https://github.com/MONOGRID/KTXForge/commit/4829a40066cf176a44aaf4d91518aedad53ed05d))
* wave 1 and 2 ([2ece054](https://github.com/MONOGRID/KTXForge/commit/2ece0548520d196350f8af9eee1b5fd319784da3))
* wave 2 ([d4d102c](https://github.com/MONOGRID/KTXForge/commit/d4d102c45725add2d7e23794e957d5120dba9b3d))
* wave 2 ([4dc2388](https://github.com/MONOGRID/KTXForge/commit/4dc2388a35ed73e0fb45b25264070da651270c47))
* wave 3 ([4bcdb87](https://github.com/MONOGRID/KTXForge/commit/4bcdb871c225b98ebb50e9a5dcb419f474073472))
* wave 4 ([f45b857](https://github.com/MONOGRID/KTXForge/commit/f45b8573d050f8b580fc4477da5e84528c567296))

### Bug Fixes

* **anim:** ensure KTX2 mip levels are emitted in correct order and alignment ([32819bf](https://github.com/MONOGRID/KTXForge/commit/32819bf9037405a438ae656b90f8deff24d23ba5))
* **build:** ktx asset extraction ([35c1734](https://github.com/MONOGRID/KTXForge/commit/35c17344a9f7c157c972221ef24a49a7b3b2ae4e))
* bundle ktx2ktx2, clean up sidecars and temp files on exit ([c266be2](https://github.com/MONOGRID/KTXForge/commit/c266be29e8f79a31757a38aa86d17aa545bebdb1))
* bundle libktx beside the ktx sidecars in installers ([025fdc5](https://github.com/MONOGRID/KTXForge/commit/025fdc5559ab04b00b5433e671e5a6fd3f194f98))
* enhance error handling and update UI components for better accessibility ([3cde182](https://github.com/MONOGRID/KTXForge/commit/3cde182558ec6791e06279b75c9dc1f62f89eec3))
* **inspector:** mount ExportPanel and clear ErrorBox marker ([68aeaa8](https://github.com/MONOGRID/KTXForge/commit/68aeaa8f6b9afe072548200f828c86f59f5a6959))
* **inspector:** wire viewer to T5.5 store API ([96bd53d](https://github.com/MONOGRID/KTXForge/commit/96bd53d3efaf6ca49de8f2d59c086edff705e243))
* keyboard shortcuts should now properly work ([c736aa4](https://github.com/MONOGRID/KTXForge/commit/c736aa4ba03204c4fcec0e812c4e20892a1971a4))
* pass quality gate ([11eb390](https://github.com/MONOGRID/KTXForge/commit/11eb390d23fd73ad8d40d2d3002ad5a639090901))
* stage HDR inputs as scanline EXR so Linux ktx stops crashing ([aa5fde9](https://github.com/MONOGRID/KTXForge/commit/aa5fde9bc5f5bf55593e72c220d6ce8584fb7933))
* stop window growing on each launch under Wayland ([66044b6](https://github.com/MONOGRID/KTXForge/commit/66044b62716b67722da19ac33dfd6efc24fb2314))
* switch component not sending values properly ([7826838](https://github.com/MONOGRID/KTXForge/commit/7826838080e8d004dd71b6519e8b69a5f3665e92))

### Performance

* levels scrub faster ([5a8bde5](https://github.com/MONOGRID/KTXForge/commit/5a8bde5f95557e06d81bd92b9ac951fb11c53024))

# Changelog
