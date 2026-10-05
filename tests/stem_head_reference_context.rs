// SPDX-License-Identifier: AGPL-3.0-or-later
#[path = "../examples/support/stem_head_reference_context_controls.rs"]
mod controls;
use audiveris_native::stem_head_reference_context::{self, Head};
use audiveris_template_bank::Bank;
use serde_json::Value;
#[test]
fn original_xml_rounding_and_loaded_context_refs_from_native_laura_image_report() {
    let o: Value = serde_json::from_str(include_str!(
        "../fixtures/official-stem-head-reference-context/controls.json"
    ))
    .unwrap();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/official-stem-head-reference-context/native-laura-report.json.gz");
    let output=std::process::Command::new("python3").arg("-c").arg("import gzip,sys;sys.stdout.buffer.write(gzip.decompress(open(sys.argv[1],'rb').read()))").arg(path).output().unwrap();
    assert!(output.status.success());
    let page = serde_json::from_slice(&output.stdout).unwrap();
    let native = controls::observe(&o, &page);
    assert_eq!(native["head_references"].as_array().unwrap().len(), 1408);
    assert_eq!(
        native["XML_adapter_controls"].as_array().unwrap().len(),
        394
    );
    assert!(native["transient_reference_differences"].as_u64().unwrap() > 0);
}
#[test]
fn null_signed_zero_and_invalid_context_leave_live_reference_api_unchanged() {
    assert_eq!(stem_head_reference_context::to_xml(None).unwrap(), None);
    assert_eq!(
        stem_head_reference_context::to_xml(Some(-0.)).unwrap(),
        Some("-0".into())
    );
    assert_eq!(
        stem_head_reference_context::restored_dx([Some(-0.), None]).unwrap()[0]
            .unwrap()
            .to_bits(),
        (-0.0f64).to_bits()
    );
    for v in [f64::NAN, f64::INFINITY, 4_000_001.] {
        assert!(stem_head_reference_context::to_xml(Some(v)).is_err());
    }
    let bank = Bank::load_all_head_coverage("Bravura").unwrap();
    let h = Head {
        bank: &bank,
        point_size: 51,
        shape: "NOTEHEAD_BLACK",
        bounds: [40, 50, 15, 12],
        side: audiveris_native::head_stem_relation::HeadSide::Left,
        vertical: audiveris_native::head_stem_relation::VerticalSide::Top,
    };
    let raw = [Some(1. / 3.), Some(1. / 7.)];
    let saved = raw;
    let reference = stem_head_reference_context::reference(&h, raw).unwrap();
    assert_eq!(raw, saved);
    let current = audiveris_native::head_stem_relation::stem_reference_point(
        &bank, 51, h.shape, h.bounds, h.side, h.vertical, raw,
    )
    .unwrap();
    assert_ne!(current, reference);
    assert!(stem_head_reference_context::reference(&h, [Some(f64::NAN), None]).is_err());
}
