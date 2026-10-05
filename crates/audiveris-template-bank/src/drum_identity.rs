// SPDX-License-Identifier: AGPL-3.0-or-later
// Generated exact official drum-template catalog.
impl Bank {
    /// Exact, bounded catalog; missing family/shape/size never interpolates.
    pub fn load_drum_coverage(family: &str) -> Result<Self, Error> {
        match family {
            "Bravura" => Self::from_bytes(include_bytes!("../assets/drum/Bravura/manifest.json"), include_bytes!("../assets/drum/Bravura/templates.json"), Trust { manifest_sha256: "acbf60cdef0862f9fc113deff8af61ccef235ef4f6a1ebd19fca4423a3f8f430", upstream_commit: "9e1e55cd2746037d059345881c53e6a6754bffbd", font_sha256: "dca2d90c88437a701b1c2e71fa54e76f9fa41d7deee935d74dc871ea66ecfdd2", model_sha256: "327194c47116a8ebf5771d04cea08a4e563ec93d2955e1efd990a54a9245f29e", coverage_input_sha256: "228b1fa3135c706f4829bffbd32a059dba4a9131fd9d9ae4473de92010b46c45" }),
            "Leland" => Self::from_bytes(include_bytes!("../assets/drum/Leland/manifest.json"), include_bytes!("../assets/drum/Leland/templates.json"), Trust { manifest_sha256: "2434b19083294a77955ebe1a6f88fa5d23096c16706bf2856a8653efa6352216", upstream_commit: "9e1e55cd2746037d059345881c53e6a6754bffbd", font_sha256: "cacca69cb750d851023d52a8e41d3373904aa82afc7247085f37d4ee6823506c", model_sha256: "327194c47116a8ebf5771d04cea08a4e563ec93d2955e1efd990a54a9245f29e", coverage_input_sha256: "57e3093a7a7263685d7ce282d4be886c7af883aaa432c67d7f271fb11abb5867" }),
            "FinaleJazz" => Self::from_bytes(include_bytes!("../assets/drum/FinaleJazz/manifest.json"), include_bytes!("../assets/drum/FinaleJazz/templates.json"), Trust { manifest_sha256: "d39f3fd8c294ff0469c27099477ea18932faddb28fe40c29ed62145de1d14f47", upstream_commit: "9e1e55cd2746037d059345881c53e6a6754bffbd", font_sha256: "a8ea0408ef7628ca38ae29eaa1b4357d2b0bf11747f6eee3c5f9ed5cca358e08", model_sha256: "327194c47116a8ebf5771d04cea08a4e563ec93d2955e1efd990a54a9245f29e", coverage_input_sha256: "dcfaf815af13978ce4f180a6a0a997e15602950dd6a393a46ed72eaba481f8c6" }),
            _ => Err(Error::UnsupportedTemplate),
        }
    }
}
