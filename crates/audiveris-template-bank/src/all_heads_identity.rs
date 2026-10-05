// SPDX-License-Identifier: AGPL-3.0-or-later
// Generated exact official all-head template catalog.
impl Bank {
    /// Exact, bounded catalog; missing family/shape/size never interpolates.
    pub fn load_all_head_coverage(family: &str) -> Result<Self, Error> {
        match family {
            "Bravura" => Self::from_bytes(include_bytes!("../assets/all-heads/Bravura/manifest.json"), include_bytes!("../assets/all-heads/Bravura/templates.json"), Trust { manifest_sha256: "a7dce23296db6f57fd1b1935e6e75b0c3c38c8226f33d3289a9ddb07d2255919", upstream_commit: "9e1e55cd2746037d059345881c53e6a6754bffbd", font_sha256: "dca2d90c88437a701b1c2e71fa54e76f9fa41d7deee935d74dc871ea66ecfdd2", model_sha256: "327194c47116a8ebf5771d04cea08a4e563ec93d2955e1efd990a54a9245f29e", coverage_input_sha256: "b75167f60d7194d0eb4dabf2dfa91ab7fb0ce0ab4fe5c82c2634fe5cf6a9c538" }),
            "Leland" => Self::from_bytes(include_bytes!("../assets/all-heads/Leland/manifest.json"), include_bytes!("../assets/all-heads/Leland/templates.json"), Trust { manifest_sha256: "de01e9a0b1588b3fc2cfbe0ccf2399c42336939f93ea2cb96c9ae8472facf4a7", upstream_commit: "9e1e55cd2746037d059345881c53e6a6754bffbd", font_sha256: "cacca69cb750d851023d52a8e41d3373904aa82afc7247085f37d4ee6823506c", model_sha256: "327194c47116a8ebf5771d04cea08a4e563ec93d2955e1efd990a54a9245f29e", coverage_input_sha256: "9a27ee70a863c6a3932bf3c749f1c1c7a65fa97c0508534b47a9d7f3d60099ff" }),
            "FinaleJazz" => Self::from_bytes(include_bytes!("../assets/all-heads/FinaleJazz/manifest.json"), include_bytes!("../assets/all-heads/FinaleJazz/templates.json"), Trust { manifest_sha256: "ac507ed4fcfb38704a076ca3912b1054169346ea5c483ff5e8e29024c276c0cd", upstream_commit: "9e1e55cd2746037d059345881c53e6a6754bffbd", font_sha256: "a8ea0408ef7628ca38ae29eaa1b4357d2b0bf11747f6eee3c5f9ed5cca358e08", model_sha256: "327194c47116a8ebf5771d04cea08a4e563ec93d2955e1efd990a54a9245f29e", coverage_input_sha256: "dbb7c47860d4145b573ee8a26a065d4f6831ad37d2716458f3e564a8bd87f62e" }),
            _ => Err(Error::UnsupportedTemplate),
        }
    }
}
