// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage,
    beam_candidates::SystemCandidates,
    beam_dispatch::Registry,
    beam_extend_to_beam::ExtensionScale,
    beam_extension_geometry::HorizontalSide,
    beam_extension_prefix::{SideDecision, initial_merge_prefix, side_prefix},
    beam_impacts::Impacts,
    beam_merge::MergeCandidate,
    beam_structure::BeamItem,
};
use serde_json::{Value, json};
fn decision(v: &SideDecision) -> Value {
    match v {
        SideDecision::Merged {
            beam,
            neighbor,
            appended,
        } => {
            json!({"beam":beam,"side":"LEFT","neighbor":neighbor,"merged":true,"appended":appended})
        }
        SideDecision::NeedDownstream {
            beam,
            side,
            neighbor,
            max_extension,
            parallel_eligible_after_stem_and_spot,
        } => {
            json!({"beam":beam,"side":if *side==HorizontalSide::Left{"LEFT"}else{"RIGHT"},"neighbor":neighbor,"merged":false,"max_extension":max_extension,"parallel_eligible_after_stem_and_spot":parallel_eligible_after_stem_and_spot})
        }
    }
}
fn snapshot(s: &SystemCandidates) -> Value {
    json!({"active_ids":s.inters().iter().filter(|v|!v.is_removed()).map(|v|v.id()).collect::<Vec<_>>(),"removed":s.inters().iter().map(|v|v.is_removed()).collect::<Vec<_>>(),"index_present":s.inters().iter().map(|v|s.active(v.id()).is_some()).collect::<Vec<_>>(),"edge_count":s.exclusions().len()})
}
#[test]
fn original_side_decisions_and_initial_snapshot_prefix_match() {
    let official: Value = serde_json::from_str(include_str!(
        "../fixtures/official-beam-extension-prefix.json"
    ))
    .unwrap();
    assert_eq!(official["native_outputs_used"], false);
    let cases = official["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 37);
    let mut merge_count = 0;
    let mut skipped = 0;
    let mut completed = 0;
    for (index, c) in cases.iter().enumerate() {
        assert_eq!(c["side_order"], json!(["LEFT", "RIGHT"]));
        let input = &c["input"];
        let seed = GrayImage::new(160, 96, vec![0; 160 * 96]).unwrap();
        let mut pixels = vec![255; 160 * 96];
        for p in input["black_pixels"].as_array().unwrap() {
            pixels[p[1].as_u64().unwrap() as usize * 160 + p[0].as_u64().unwrap() as usize] = 0;
        }
        let image = GrayImage::new(160, 96, pixels).unwrap();
        let mut s = SystemCandidates::empty(1);
        let mut r = Registry::default();
        for b in input["beams"].as_array().unwrap() {
            s.register_merge(
                &seed,
                &mut r,
                &MergeCandidate {
                    item: BeamItem {
                        median: std::array::from_fn(|i| b["median"][i].as_f64().unwrap()),
                        height: 8.,
                    },
                    is_small: false,
                    impacts: Impacts::new([1.; 6]).unwrap(),
                },
            )
            .unwrap();
        }
        if let Some(id) = input["pre_removed"].as_u64() {
            s.remove(id as usize).unwrap();
        }
        let initial_ids = s
            .inters()
            .iter()
            .filter(|v| !v.is_removed())
            .map(|v| v.id())
            .collect::<Vec<_>>();
        assert_eq!(json!(initial_ids), c["initial_ids"], "{index} SIG order");
        let scale = ExtensionScale {
            interline: 16,
            standard_height: 8.,
            small_height: None,
        };
        let decisions = if input["mode"] == "prefix" {
            let got = initial_merge_prefix(&mut s, &image, &mut r, scale).unwrap();
            assert_eq!(json!(got.initial_ids), c["initial_ids"]);
            assert_eq!(
                json!(got.skipped_removed),
                c["skipped_removed"],
                "{index} skips"
            );
            assert_eq!(
                json!(got.stopped_for_downstream),
                c["stopped_for_downstream"],
                "{index} stop"
            );
            skipped += got.skipped_removed.len();
            completed += usize::from(!got.stopped_for_downstream);
            got.decisions
        } else {
            let before = s.clone();
            let reg = r.snapshot();
            let side = if input["side"] == "LEFT" {
                HorizontalSide::Left
            } else {
                HorizontalSide::Right
            };
            let got = side_prefix(
                &mut s,
                &image,
                &mut r,
                input["target"].as_u64().unwrap() as usize,
                side,
                scale,
            )
            .unwrap();
            if matches!(got, SideDecision::NeedDownstream { .. }) {
                assert_eq!(s, before);
                assert_eq!(r.snapshot(), reg);
            }
            vec![got]
        };
        merge_count += decisions
            .iter()
            .filter(|v| matches!(v, SideDecision::Merged { .. }))
            .count();
        assert_eq!(
            json!(decisions.iter().map(decision).collect::<Vec<_>>()),
            c["decisions"],
            "{index} decision"
        );
        for key in ["active_ids", "removed", "index_present", "edge_count"] {
            assert_eq!(snapshot(&s)[key], c["after"][key], "{index} state {key}");
        }
        let ids = s.inters().iter().map(|v| v.glyph_id()).collect::<Vec<_>>();
        for i in 0..ids.len() {
            for j in 0..ids.len() {
                assert_eq!(
                    ids[i] == ids[j],
                    c["after"]["glyph_ids"][i] == c["after"]["glyph_ids"][j],
                    "{index} glyph identity"
                );
            }
        }
    }
    assert!(merge_count > 0 && skipped > 0 && completed > 0);
}
#[test]
fn empty_prefix_finishes_and_invalid_target_is_a_noop() {
    let image = GrayImage::new(20, 20, vec![0; 400]).unwrap();
    let mut s = SystemCandidates::empty(1);
    let mut r = Registry::default();
    let scale = ExtensionScale {
        interline: 16,
        standard_height: 8.,
        small_height: None,
    };
    let prefix = initial_merge_prefix(&mut s, &image, &mut r, scale).unwrap();
    assert!(!prefix.stopped_for_downstream && prefix.initial_ids.is_empty());
    let before = s.clone();
    let reg = r.snapshot();
    assert!(side_prefix(&mut s, &image, &mut r, 1, HorizontalSide::Right, scale).is_err());
    assert_eq!(s, before);
    assert_eq!(r.snapshot(), reg);
}
