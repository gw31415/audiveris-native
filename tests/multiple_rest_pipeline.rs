// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    GrayImage, Run,
    beam_candidates::SystemCandidates,
    beam_dispatch::Registry,
    beam_groups,
    beam_impacts::Impacts,
    beam_merge::MergeCandidate,
    beam_structure::BeamItem,
    multiple_rest_pipeline::{self, Context},
    sections::{Orientation, Section, SectionId},
    serif_peaks::StaffGeometry,
};
use serde_json::Value;
#[test]
fn uninterrupted_original_complete_rest_conversions_match_integrated_native_pass() {
    let v: Value = serde_json::from_str(include_str!(
        "../fixtures/official-multiple-rest-conversion.json"
    ))
    .unwrap();
    assert_eq!(v["native_outputs_used"], false);
    assert_eq!(v["cases"].as_array().unwrap().len(), 11);
    let mut converted = 0;
    for (index, c) in v["cases"].as_array().unwrap().iter().enumerate() {
        let g = &c["geometry"];
        let width = g["width"].as_u64().unwrap() as usize;
        let height = g["height"].as_u64().unwrap() as usize;
        let mut pixels = vec![255; width * height];
        for r in c["rectangles"].as_array().unwrap() {
            let b: [usize; 4] = std::array::from_fn(|n| r[n].as_u64().unwrap() as usize);
            for y in b[1]..b[1] + b[3] {
                for x in b[0]..b[0] + b[2] {
                    pixels[y * width + x] = 0;
                }
            }
        }
        let image = GrayImage::new(width, height, pixels).unwrap();
        let beam = &c["beam"];
        let mut registry = Registry::default();
        let mut state = SystemCandidates::empty(1);
        state
            .register_merge(
                &image,
                &mut registry,
                &MergeCandidate {
                    item: BeamItem {
                        median: std::array::from_fn(|n| beam["median"][n].as_f64().unwrap()),
                        height: beam["height"].as_f64().unwrap(),
                    },
                    is_small: false,
                    impacts: Impacts {
                        values: [1.; 6],
                        grade: beam["grade"].as_f64().unwrap(),
                        non_finite: false,
                    },
                },
            )
            .unwrap();
        assert_eq!(
            serde_json::json!(state.active(1).unwrap().bounds),
            beam["bounds"],
            "case {index} source bounds"
        );
        let mut groups = beam_groups::populate_system(&state, 16).unwrap();
        let sections = c["sections"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(id, s)| {
                let runs = s["runs"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|r| Run {
                        start: r[0].as_u64().unwrap() as usize,
                        length: r[1].as_u64().unwrap() as usize,
                    })
                    .collect::<Vec<_>>();
                Section {
                    id: SectionId(id as u32 + 1),
                    orientation: Orientation::Vertical,
                    first_pos: s["first_pos"].as_u64().unwrap() as usize,
                    weight: runs.iter().map(|r| r.length).sum(),
                    runs,
                }
            })
            .collect::<Vec<_>>();
        let rows = g["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| [r[0].as_i64().unwrap() as i32, r[1].as_i64().unwrap() as i32])
            .collect::<Vec<_>>();
        let middle = vec![g["middle"].as_i64().unwrap() as i32; width];
        let thickness = g["line_thickness"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n.as_f64().unwrap())
            .collect::<Vec<_>>();
        let il = g["interline"].as_u64().unwrap() as usize;
        let fore = g["fore"].as_u64().unwrap() as usize;
        let staff_id = beam["staff_id"].as_u64().unwrap();
        let staffs = [(
            staff_id,
            StaffGeometry {
                rows: &rows,
                middle: &middle,
                abscissas: std::array::from_fn(|n| g["abscissas"][n].as_i64().unwrap() as i32),
                interline: il,
                fore,
                line_thickness: &thickness,
            },
        )];
        let mut native_staff = serde_json::json!({"id":staff_id,"system_id":1,"specific_interline":il,"left":g["abscissas"][0],"right":g["abscissas"][1],"lines":[]});
        for (n, thickness) in thickness.iter().enumerate() {
            let y = rows[0][0] + n as i32 * il as i32;
            native_staff["lines"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!({"points":[[0,y],[width,y]],"thickness":thickness}));
        }
        let mut mapping_grid = serde_json::json!({"systems":[{"id":1,"area":{"x_range":[0,width],"north_neighbor_last_line":null,"south_neighbor_first_line":null}}],"staffs":[native_staff],"vlag":[],"hlag":[]});
        for (n, section) in sections.iter().enumerate() {
            mapping_grid["vlag"].as_array_mut().unwrap().push(serde_json::json!({"id":n+1,"first_pos":section.first_pos,"runs":section.runs.iter().map(|r|[r.start,r.length]).collect::<Vec<_>>() }));
        }
        let mapping = audiveris_native::native_rest_mapping::Mapping::from_grid(
            &mapping_grid,
            width,
            height,
            1,
        )
        .unwrap();
        let measured = mapping.position(state.active(1).unwrap()).unwrap();
        if let Some(measured) = measured {
            assert_eq!(
                serde_json::json!(measured.endpoint_pitches),
                beam["endpoint_pitches"],
                "case {index} native pitches"
            );
        }
        let mut shared = audiveris_native::shared_sig::Graph::default();
        let graph = multiple_rest_pipeline::process_shared(
            &mut state,
            &mut groups,
            &mut registry,
            Context {
                image: &image,
                interline: il,
                fore,
                sections: &sections,
                staffs: &staffs,
            },
            |candidate| mapping.position(candidate).unwrap(),
            &mut shared,
        )
        .unwrap();
        let snapshot = shared.snapshot();
        let active_ids = snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v["id"].as_u64().unwrap())
            .collect::<std::collections::BTreeSet<_>>();
        for edge in snapshot["relations"].as_array().unwrap() {
            assert!(active_ids.contains(&edge["source"].as_u64().unwrap()));
            assert!(active_ids.contains(&edge["target"].as_u64().unwrap()));
        }
        assert_eq!(
            snapshot["inters"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|v| v["kind"] == "MultipleRestInter")
                .count(),
            graph.rests.len()
        );
        assert_eq!(
            snapshot["relations"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|v| v["kind"] == "MultipleRestSerifRelation")
                .count(),
            graph.relations.len()
        );
        let rests=graph.rests.iter().map(|r|serde_json::json!({"median":r.median,"height":r.height,"grade":r.grade,"staff_id":r.staff_id})).collect::<Vec<_>>();
        assert_eq!(serde_json::json!(rests), c["rests"], "case {index} rests");
        assert_eq!(
            state.active(1).is_none(),
            c["removed"].as_bool().unwrap(),
            "case {index} removed"
        );
        assert_eq!(
            serde_json::json!(
                groups
                    .groups
                    .iter()
                    .map(|g| g.members.clone())
                    .collect::<Vec<_>>()
            ),
            c["groups"],
            "case {index} groups"
        );
        assert_eq!(
            serde_json::json!(
                graph
                    .relations
                    .iter()
                    .map(|r| if r.left { "LEFT" } else { "RIGHT" })
                    .collect::<Vec<_>>()
            ),
            c["relations"],
            "case {index} relations"
        );
        assert_eq!(graph.serifs.len(), c["serifs"].as_array().unwrap().len());
        for (serif, expected) in graph.serifs.iter().zip(c["serifs"].as_array().unwrap()) {
            assert_eq!(
                serde_json::json!(serif.plan.median),
                expected["median"],
                "case {index} serif median"
            );
            assert_eq!(serde_json::json!(serif.plan.width), expected["width"]);
            assert!(
                (serif.plan.peak.grade - expected["grade"].as_f64().unwrap()).abs() < 1e-10,
                "case {index} serif grade"
            );
            let glyph = &registry.get(serif.glyph_id).unwrap().glyph;
            let mut pixels = Vec::new();
            for run in &glyph.runs {
                for y in run.start..run.start + run.length {
                    pixels.push([glyph.left + run.position as i32, glyph.top + y as i32]);
                }
            }
            pixels.sort_by_key(|p| (p[1], p[0]));
            assert_eq!(
                serde_json::json!({"bounds":[glyph.left,glyph.top,glyph.width as i32,glyph.height as i32],"pixels":pixels}),
                expected["glyph"],
                "case {index} glyph"
            );
        }
        converted += graph.rests.len();
    }
    assert_eq!(converted, 6);
}
