// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage, Run, clusters, filament_merge, scale,
    sections::{self, Orientation, Section, SectionId},
    vertical,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
fn fixture(n: &str) -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(format!(
            "{}/fixtures/official-grid-second-pass/{n}.json",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap(),
    )
    .unwrap()
}
fn original_sections(g: &Value) -> Vec<Section> {
    g["sections"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            let runs: Vec<_> = s["runs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| Run {
                    start: r[0].as_u64().unwrap() as usize,
                    length: r[1].as_u64().unwrap() as usize,
                })
                .collect();
            let weight = runs.iter().map(|r| r.length).sum();
            Section {
                id: SectionId(s["id"].as_u64().unwrap() as u32),
                orientation: Orientation::Horizontal,
                first_pos: s["first_pos"].as_u64().unwrap() as usize,
                runs,
                weight,
            }
        })
        .collect()
}
fn pixel_key(line: &Value, ss: &[Section]) -> BTreeSet<(usize, usize)> {
    let by: BTreeMap<_, _> = ss.iter().map(|s| (s.id.0, s)).collect();
    let mut ps = BTreeSet::new();
    for id in line["section_ids"].as_array().unwrap() {
        let s = by[&(id.as_u64().unwrap() as u32)];
        for (y, r) in s.runs.iter().enumerate() {
            for x in r.start..r.start + r.length {
                ps.insert((x, s.first_pos + y));
            }
        }
    }
    ps
}
fn groups(v: &Value, ss: &[Section]) -> BTreeSet<BTreeSet<(usize, usize)>> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|r| pixel_key(r, ss))
        .collect()
}
fn compare_staffs(a: &Value, e: &Value, ss: &[Section], original: &[Section]) {
    let aa = a["staffs"].as_array().unwrap();
    let ee = e["staffs"].as_array().unwrap();
    assert_eq!(aa.len(), ee.len());
    for (a, e) in aa.iter().zip(ee) {
        for k in ["id", "interline", "small", "short", "left", "right"] {
            if matches!(k, "left" | "right") {
                assert!((a[k].as_f64().unwrap() - e[k].as_f64().unwrap()).abs() < 1e-12);
            } else {
                assert_eq!(a[k], e[k], "{k}");
            }
        }
        let al = a["lines"].as_array().unwrap();
        let el = e["lines"].as_array().unwrap();
        assert_eq!(al.len(), 5);
        assert_eq!(al.len(), el.len());
        for (a, e) in al.iter().zip(el) {
            assert_eq!(pixel_key(a, ss), pixel_key(e, original));
            let ap = a["points"].as_array().unwrap();
            let ep = e["points"].as_array().unwrap();
            assert_eq!(ap.len(), ep.len());
            for (p, q) in ap.iter().zip(ep) {
                for i in 0..2 {
                    assert!(
                        (p[i].as_f64().unwrap() - q[i].as_f64().unwrap()).abs() < 1e-12,
                        "point {p} != {q}"
                    );
                }
            }
        }
    }
    assert_eq!(a["second_pass_ran"], e["second_pass_ran"]);
}
fn image_and_scale() -> (GrayImage, scale::Scale) {
    let i = GrayImage::decode(
        &std::fs::read(format!(
            "{}/fixtures/official-head-actual-pages/images/Bravura-cue.png",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap(),
    )
    .unwrap();
    let b = vertical::binarize(&i, |_, _, _| {});
    let sc = scale::analyze(&b).result.unwrap();
    (b, sc)
}
#[test]
fn original_second_pass_initial_cue_staffs_and_discarded_contract() {
    let (_, sc) = image_and_scale();
    for name in ["cue", "main-lines-only", "empty"] {
        let g = fixture(name);
        let ss = original_sections(&g);
        let factory = json!({"filaments":g["input_filaments"],"line_filtering":{"retained_ids":g["input_filaments"].as_array().unwrap().iter().map(|r|r["id"].clone()).collect::<Vec<_>>(),"global_slope":g["global_slope"],"too_few_after_curvature":false}});
        let a =
            clusters::analyze_two_pass(g["width"].as_u64().unwrap() as usize, &sc, &ss, &factory);
        assert_eq!(a["status"], "initial_staffs", "{a}");
        compare_staffs(&a, &g, &ss, &ss);
        assert_eq!(
            groups(&a["first_discarded"], &ss),
            groups(&g["first_discarded"], &ss)
        );
        assert_eq!(
            groups(&a["second_discarded"], &ss),
            groups(&g["second_discarded"], &ss)
        );
        let ids = a["second_input"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["id"].as_u64().unwrap())
            .collect::<Vec<_>>();
        assert!(ids.windows(2).all(|p| p[0] < p[1]));
        for k in ["grid_complete", "heads_complete", "stems_complete"] {
            assert_eq!(a[k], false);
        }
    }
}
#[test]
fn actual_native_cue_factory_reaches_original_initial_staffs_only() {
    let (b, sc) = image_and_scale();
    let prep = sections::prepare(&b, &sc).unwrap();
    let factory = filament_merge::analyze(b.width, &sc, &prep.horizontal);
    let a = clusters::analyze_two_pass(b.width, &sc, &prep.horizontal, &factory);
    let e = fixture("cue");
    assert_eq!(a["status"], "initial_staffs", "{a}");
    compare_staffs(&a, &e, &prep.horizontal, &original_sections(&e));
    assert_eq!(a["staffs"].as_array().unwrap().len(), 4);
    assert_eq!(
        a["staffs"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|s| s["small"] == true)
            .count(),
        1
    );
    assert_eq!(
        clusters::analyze(&sc, &prep.horizontal, &factory)["error"],
        "multiple_interline_requires_second_pass"
    );
}
#[test]
fn original_length_consistency_equality_empty_and_second_pass_guards() {
    let g = fixture("cue");
    assert_eq!(g["length_consistency"]["max_diff_length_ratio"], 0.5);
    for c in g["length_consistency"]["cases"].as_array().unwrap() {
        let lengths = c["lengths"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n.as_u64().unwrap() as usize)
            .collect::<Vec<_>>();
        assert_eq!(
            clusters::cluster_lengths_consistent(&lengths).unwrap(),
            c["consistent"]
        );
    }
    assert!(clusters::cluster_lengths_consistent(&[]).is_err());
    assert!(clusters::cluster_lengths_consistent(&[0, 400]).is_err());
    let (b, mut sc) = image_and_scale();
    let empty = json!({"combs":[],"filaments":[],"line_filtering":{"retained_ids":[],"global_slope":0.,"too_few_after_curvature":false}});
    assert_eq!(
        clusters::analyze_two_pass(0, &sc, &[], &empty)["error"],
        "invalid_second_pass_scale_or_width"
    );
    let malformed = json!({"filaments":[{"id":0,"section_ids":[999999]}],"line_filtering":{"retained_ids":[0]}});
    assert_eq!(
        clusters::analyze_two_pass(b.width, &sc, &[], &malformed)["error"],
        "invalid_second_pass_filaments"
    );
    let work_limit = json!({"filaments":[],"line_filtering":{"retained_ids":vec![serde_json::Value::Null;100_000]}});
    assert_eq!(
        clusters::analyze_two_pass(b.width, &sc, &[], &work_limit)["error"],
        "unsupported_second_pass_work_limit"
    );
    let bad_section = Section {
        id: SectionId(1),
        orientation: Orientation::Horizontal,
        first_pos: 0,
        runs: vec![],
        weight: 0,
    };
    assert_eq!(
        clusters::analyze_two_pass(b.width, &sc, &[bad_section], &empty)["error"],
        "invalid_second_pass_sections"
    );

    sc.small_interline = None;
    assert_eq!(
        clusters::analyze_two_pass(b.width, &sc, &[], &empty),
        clusters::analyze(
            &sc,
            &[],
            &json!({"filaments":[],"combs":[],"line_filtering":{"retained_ids":[],"global_slope":0.,"too_few_after_curvature":false}})
        )
    );
}
