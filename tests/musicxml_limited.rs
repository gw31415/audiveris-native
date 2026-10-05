// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{GrayImage, music_xml};
use serde_json::Value;
use std::{fs, process::Command};

#[test]
fn recognized_bass_and_alto_headers_reach_native_pitch_and_musicxml() {
    let f = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/official-image-musicxml-clefs");
    for name in ["bass-header-quarter", "alto-header-quarter"] {
        let oracle: Value =
            serde_json::from_slice(&fs::read(f.join(format!("{name}-observed.json"))).unwrap())
                .unwrap();
        let image = GrayImage::decode(&fs::read(f.join(format!("{name}.png"))).unwrap()).unwrap();
        let result = music_xml::recognize(&image).unwrap();
        let report = &result.owner.owner.prepared.report;
        let events = report["events"].as_array().unwrap();
        assert_eq!(events.len(), oracle["events"].as_array().unwrap().len());
        for (n, o) in events.iter().zip(oracle["events"].as_array().unwrap()) {
            for field in [
                "measure_number",
                "step",
                "octave",
                "alter",
                "duration_quarters",
                "voice",
                "type",
                "rest",
            ] {
                assert_eq!(n[field], o[field], "{name}:{field}");
            }
        }
        let nodes = report["shared_sig"]["inters"].as_array().unwrap();
        let clef = nodes.iter().find(|n| n["kind"] == "ClefInter").unwrap();
        for field in ["shape", "pitch", "bounds", "frozen"] {
            assert_eq!(
                clef[field], oracle["original_clefs"][0][field],
                "{name}:{field}"
            );
        }
        assert_eq!(clef["origin"]["phase"], "MUSIC_HEADER");
        assert!(events.iter().all(|e| e["effective_clef_id"] == clef["id"]));
        let c = &oracle["clef_events"][0];
        let xml = format!(
            "<clef><sign>{}</sign><line>{}</line></clef>",
            c["sign"].as_str().unwrap(),
            c["line"]
        );
        assert_eq!(result.xml.matches(&xml).count(), 1);
        assert_eq!(result.xml.matches("<clef>").count(), 1);
        assert_eq!(report["musicxml_complete"], false);
        assert_eq!(report["stems_complete"], false);
    }
}

#[test]
fn image_frozen_header_time_reaches_owned_sig_meter_and_music_xml() {
    for (directory, image_name, numerator, event_count, whole_shape) in [
        (
            "official-image-musicxml-header-time",
            "g-major-header-time-quarter.png",
            4,
            8,
            None,
        ),
        (
            "official-image-musicxml-header-meters",
            "g-major-header-three-four.png",
            3,
            6,
            None,
        ),
        (
            "official-image-musicxml-header-common",
            "g-major-header-common.png",
            4,
            8,
            Some("COMMON_TIME"),
        ),
    ] {
        let f = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join(directory);
        let oracle: Value =
            serde_json::from_slice(&fs::read(f.join("observed.json")).unwrap()).unwrap();
        let image = GrayImage::decode(&fs::read(f.join(image_name)).unwrap()).unwrap();
        let result = music_xml::recognize(&image).unwrap();
        let report = &result.owner.owner.prepared.report;
        assert_eq!(report["events"].as_array().unwrap().len(), event_count);
        for (native, original) in report["events"]
            .as_array()
            .unwrap()
            .iter()
            .zip(oracle["events"].as_array().unwrap())
        {
            for field in [
                "measure_number",
                "step",
                "octave",
                "alter",
                "duration_quarters",
                "voice",
                "type",
                "rest",
            ] {
                assert_eq!(native[field], original[field], "{field}");
            }
        }
        let sig = &report["shared_sig"];
        let nodes = sig["inters"].as_array().unwrap();
        let time_kind = if whole_shape.is_some() {
            "TimeWholeInter"
        } else {
            "TimePairInter"
        };
        let pair = nodes.iter().find(|n| n["kind"] == time_kind).unwrap();
        assert_eq!(pair["origin"]["phase"], "MUSIC_TIME");
        if let Some(shape) = whole_shape {
            assert_eq!(pair["shape"], shape);
            assert!(!pair["glyph"].is_null());
            assert!(
                !nodes
                    .iter()
                    .any(|n| n["kind"] == "TimePairInter" || n["kind"] == "TimeNumberInter")
            );
        } else {
            assert!(pair["glyph"].is_null());
            assert_eq!(pair["member_ids"].as_array().unwrap().len(), 2);
            assert!(!nodes.iter().any(|n| n["kind"] == "TimeWholeInter"));
        }
        for original in oracle["original_nodes"].as_array().unwrap() {
            let n = nodes
                .iter()
                .find(|n| n["kind"] == original["kind"] && n["bounds"] == original["bounds"])
                .unwrap();
            assert_eq!(n["frozen"], true);
            assert_eq!(
                (n["grade"].as_f64().unwrap() * 1000.).round_ties_even(),
                original["grade"].as_f64().unwrap() * 1000.
            );
            let effective = n["contextual_grade"]
                .as_f64()
                .unwrap_or(n["grade"].as_f64().unwrap());
            assert_eq!(
                (effective * 1000.).round_ties_even(),
                original["contextual_grade"].as_f64().unwrap() * 1000.
            );
            if original["kind"] == "TimeNumberInter" {
                assert_eq!(n["value"], original["value"]);
                assert_eq!(n["shape"], original["shape"]);
                assert_eq!(n["side"], original["side"]);
            }
            if !original["glyph"].is_null() {
                assert_eq!(n["glyph"], original["glyph"]);
                let gid = n["glyph_id"].as_u64().unwrap() as usize;
                assert_eq!(
                    audiveris_native::beam_dispatch::glyph_json(
                        &result.owner.owner.prepared.registry.get(gid).unwrap().glyph
                    ),
                    original["glyph"]
                );
                if original["kind"] == "TimeNumberInter" {
                    assert!(
                        sig["relations"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|r| r["kind"] == "Containment"
                                && r["source"] == pair["id"]
                                && r["target"] == n["id"])
                    );
                }
            }
        }
        if whole_shape.is_none() {
            let top = nodes
                .iter()
                .find(|n| n["kind"] == "TimeNumberInter" && n["side"] == "TOP")
                .unwrap();
            let bottom = nodes
                .iter()
                .find(|n| n["kind"] == "TimeNumberInter" && n["side"] == "BOTTOM")
                .unwrap();
            assert!(
                sig["relations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|r| r["kind"] == "TimeTopBottomRelation"
                        && r["source"] == top["id"]
                        && r["target"] == bottom["id"])
            );
        }
        let symbol = if whole_shape.is_some() {
            " symbol=\"common\""
        } else {
            ""
        };
        assert!(
            report["measures"]
                .as_array()
                .unwrap()
                .iter()
                .all(|m| m["numerator"] == numerator && m["denominator"] == 4)
        );
        assert_eq!(
            result
                .xml
                .matches(&format!(
                    "<time{symbol}><beats>{numerator}</beats><beat-type>4</beat-type></time>"
                ))
                .count(),
            1
        );
        assert!(
            result
                .xml
                .contains("<pitch><step>F</step><alter>1</alter><octave>5</octave></pitch>")
        );
        if numerator == 3 {
            // Remove the final head and stem, preserving the original staff
            // foreground. An underfilled 3/4 measure must refuse, not pad or
            // fall back to the old hard-coded four-quarter admission.
            let mut missing =
                GrayImage::new(image.width, image.height, image.pixels.clone()).unwrap();
            for y in 190..239 {
                for x in 1108..1135 {
                    if ![220, 221, 231, 232, 242, 243, 253, 254, 264, 265].contains(&y) {
                        missing.pixels[y * missing.width + x] = 255;
                    }
                }
            }
            let error = music_xml::recognize(&missing).err().unwrap();
            assert!(
                error.starts_with("musicxml_incomplete_or_excess_measure"),
                "{error}"
            );
            assert!(error.ends_with("quarters=2:expected=3"), "{error}");
            for (native, original) in report["measures"]
                .as_array()
                .unwrap()
                .iter()
                .zip(oracle["measures"].as_array().unwrap())
            {
                assert_eq!(native["expected_duration"], original["expected_duration"]);
                assert_eq!(native["actual_duration"], original["actual_duration"]);
                let offsets = report["events"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|e| e["measure_number"] == native["number"])
                    .map(|e| e["time_offset"].clone())
                    .collect::<Vec<_>>();
                assert_eq!(serde_json::json!(offsets), original["time_offsets"]);
            }
        }
        for flag in [
            "heads_complete",
            "stems_complete",
            "musicxml_complete",
            "epilog_complete",
            "automatic_raster_pipeline_complete",
        ] {
            assert_eq!(report[flag], false);
        }
    }
}

#[test]
fn image_frozen_header_key_reaches_owned_sig_note_alter_and_music_xml() {
    let f = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/official-image-musicxml-header");
    let o: Value = serde_json::from_slice(&fs::read(f.join("observed.json")).unwrap()).unwrap();
    let im = GrayImage::decode(&fs::read(f.join("g-major-header-quarter.png")).unwrap()).unwrap();
    let result = music_xml::recognize(&im).unwrap();
    let report = &result.owner.owner.prepared.report;
    for (e, expected) in report["events"]
        .as_array()
        .unwrap()
        .iter()
        .zip(o["events"].as_array().unwrap())
    {
        for field in [
            "measure_number",
            "step",
            "octave",
            "alter",
            "duration_quarters",
            "voice",
            "type",
            "rest",
        ] {
            assert_eq!(e[field], expected[field], "{field}");
        }
        assert_eq!(e["effective_key_fifths"], 1);
    }
    assert_eq!(
        report["events"].as_array().unwrap().len(),
        o["events"].as_array().unwrap().len()
    );
    assert_eq!(report["events"][6]["step"], "F");
    assert_eq!(report["events"][6]["alter"], 1);
    assert_eq!(
        result.xml.matches("<key><fifths>1</fifths></key>").count(),
        1
    );
    assert!(
        result
            .xml
            .contains("<pitch><step>F</step><alter>1</alter><octave>5</octave></pitch>")
    );
    assert!(result.xml.contains("<time><beats>4</beats><beat-type>4</beat-type></time><clef><sign>G</sign><line>2</line></clef>"));
    let nodes = report["shared_sig"]["inters"].as_array().unwrap();
    for original in o["original_nodes"].as_array().unwrap() {
        let n = nodes
            .iter()
            .find(|n| n["kind"] == original["kind"] && n["bounds"] == original["bounds"])
            .unwrap();
        assert_eq!(n["frozen"] == true, original["frozen"] == true);
        assert_eq!(
            (n["grade"].as_f64().unwrap() * 1000.).round_ties_even(),
            original["grade"].as_f64().unwrap() * 1000.
        );
        if original["kind"] == "KeyInter" {
            assert_eq!(n["fifths"], original["fifths"]);
            let ids = n["member_ids"].as_array().unwrap();
            assert_eq!(ids.len(), 1);
            assert!(
                report["shared_sig"]["relations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|r| r["kind"] == "Containment"
                        && r["source"] == n["id"]
                        && r["target"] == ids[0])
            );
            assert!(
                report["shared_sig"]["relations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|r| r["kind"] == "ClefKeyRelation" && r["target"] == n["id"])
            );
            assert!(
                report["events"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|e| e["effective_key_id"] == n["id"])
            );
        }
        if original["kind"] == "KeyAlterInter" {
            assert_eq!(n["glyph"], original["glyph"]);
            let gid = n["glyph_id"].as_u64().unwrap() as usize;
            assert_eq!(
                audiveris_native::beam_dispatch::glyph_json(
                    &result.owner.owner.prepared.registry.get(gid).unwrap().glyph
                ),
                original["glyph"]
            );
        }
    }
    let reduction =
        &report["symbols"]["measures_head_chords"]["reduction"]["orphan_ledger_consistency"];
    let removed = reduction["removed"].as_array().unwrap();
    assert_eq!(removed.len(), 1);
    let original = &o["orphan_ledger_saved_state"]["before_reduction"][0];
    assert_eq!(original["has_head_or_next_ledger"], false);
    assert_eq!(removed[0]["index"], original["index"]);
    assert!(
        report["shared_sig"]["tombstones"]
            .as_array()
            .unwrap()
            .iter()
            .any(|n| n["id"] == removed[0]["id"])
    );
    assert!(!nodes.iter().any(|n| n["kind"] == "LedgerInter"));
    let gid = removed[0]["glyph_id"].as_u64().unwrap() as usize;
    assert_eq!(
        audiveris_native::beam_dispatch::glyph_json(
            &result.owner.owner.prepared.registry.get(gid).unwrap().glyph
        ),
        original["glyph"]
    );
    assert_eq!(report["heads_complete"], false);
    assert_eq!(report["stems_complete"], false);
    assert_eq!(report["musicxml_complete"], false);
    assert_eq!(report["epilog_complete"], false);
}

#[test]
fn interrupted_quarter_rest_compound_registers_owned_parts_and_matches_original() {
    let f = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/official-image-musicxml-compound");
    let observed: Value =
        serde_json::from_slice(&fs::read(f.join("observed.json")).unwrap()).unwrap();
    let image = GrayImage::decode(&fs::read(f.join("compound-quarter-rest.png")).unwrap()).unwrap();
    let result = music_xml::recognize(&image).unwrap();
    let report = &result.owner.owner.prepared.report;
    let events = report["events"].as_array().unwrap();
    assert_eq!(events.len(), 8);
    for (e, o) in events.iter().zip(
        observed["independent_original_full_image_export"]["events"]
            .as_array()
            .unwrap(),
    ) {
        for field in [
            "measure_number",
            "step",
            "octave",
            "rest",
            "type",
            "duration_quarters",
            "voice",
        ] {
            assert_eq!(e[field], o[field], "{field}");
        }
    }
    let symbols = &report["symbols"];
    let trial = symbols["classifier_trials"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["part_count"] == 2 && t["bounds"] == serde_json::json!([391, 234, 13, 38]))
        .unwrap();
    assert_eq!(trial["part_glyph_ids"].as_array().unwrap().len(), 2);
    let registry = &result.owner.owner.prepared.registry;
    let compound = &registry
        .get(trial["glyph_id"].as_u64().unwrap() as usize)
        .unwrap()
        .glyph;
    assert_eq!(
        audiveris_native::beam_dispatch::glyph_json(compound),
        observed["compound_glyph"]
    );
    let part_weight: usize = trial["part_glyph_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| {
            registry
                .get(id.as_u64().unwrap() as usize)
                .unwrap()
                .glyph
                .runs
                .iter()
                .map(|r| r.length)
                .sum::<usize>()
        })
        .sum();
    assert_eq!(part_weight, trial["weight"].as_u64().unwrap() as usize);
    for id in trial["created_inter_ids"].as_array().unwrap() {
        let n = report["shared_sig"]["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == *id)
            .unwrap();
        assert_eq!(n["shape"], "QUARTER_REST");
        assert_eq!(n["glyph_id"], trial["glyph_id"]);
    }
    for original in observed["after_LINKS"].as_array().unwrap() {
        let n = report["shared_sig"]["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["kind"] == original["kind"] && n["bounds"] == original["bounds"])
            .unwrap();
        assert!(
            (n["grade"].as_f64().unwrap() * 1000.).round_ties_even()
                == original["grade"].as_f64().unwrap() * 1000.
        );
    }
    assert!(
        symbols["limited_links"]["symbol_conflict_reduction"]["removed_conflict_symbol_ids"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(report["heads_complete"], false);
    assert_eq!(report["stems_complete"], false);
    assert_eq!(report["musicxml_complete"], false);
}

#[test]
fn ledger_quarter_image_retains_canonical_ledger_and_matches_original_export() {
    let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/official-image-musicxml-ledger");
    let observed: Value =
        serde_json::from_slice(&fs::read(fixtures.join("observed.json")).unwrap()).unwrap();
    let image = GrayImage::decode(&fs::read(fixtures.join("ledger-quarter.png")).unwrap()).unwrap();
    let result = music_xml::recognize(&image).unwrap();
    let report = &result.owner.owner.prepared.report;
    let events = report["events"].as_array().unwrap();
    assert_eq!(events.len(), 8);
    for (event, original) in events.iter().zip(observed["events"].as_array().unwrap()) {
        for field in [
            "step",
            "octave",
            "rest",
            "duration_quarters",
            "offset_quarters",
            "type",
            "voice",
        ] {
            assert_eq!(event[field], original[field], "{field}");
        }
        assert_eq!(event["measure_number"], original["measure"]);
    }
    let cluster: Value =
        serde_json::from_slice(&fs::read(fixtures.join("cluster-observed.json")).unwrap()).unwrap();
    assert_eq!(cluster["concrete_ledger_dot_allowed"], false);
    let trials = report["symbols"]["classifier_trials"].as_array().unwrap();
    for original in cluster["trials"].as_array().unwrap() {
        let native = trials
            .iter()
            .find(|t| t["bounds"] == original["bounds"] && t["part_count"] == original["parts"])
            .unwrap();
        assert_eq!(native["weight"], original["weight"]);
        let expected = original["evaluations_above_0_15"].as_array().unwrap();
        let actual = native["evaluations"].as_array().unwrap();
        assert_eq!(actual.len(), expected.len());
        for (a, b) in actual.iter().zip(expected) {
            assert_eq!(a["shape"], b["shape"]);
            assert!((a["score"].as_f64().unwrap() - b["score"].as_f64().unwrap()).abs() < 1e-12);
        }
    }
    let ledger = report["shared_sig"]["inters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["kind"] == "LedgerInter")
        .unwrap();
    assert_eq!(ledger["bounds"], serde_json::json!([272, 275, 21, 2]));
    let id = ledger["glyph_id"].as_u64().unwrap() as usize;
    assert_eq!(
        audiveris_native::beam_dispatch::glyph_json(
            &result.owner.owner.prepared.registry.get(id).unwrap().glyph
        ),
        ledger["glyph"]
    );
    assert_eq!(report["heads_complete"], false);
    assert_eq!(report["stems_complete"], false);
    assert_eq!(report["musicxml_complete"], false);
}

#[test]
fn half_rest_orphan_beams_links_and_refusal_match_original_boundaries() {
    let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/official-image-musicxml-half-rest");
    for y in [239, 241] {
        let name = format!("half-rest-y{y}");
        let image =
            GrayImage::decode(&fs::read(fixtures.join(format!("{name}.png"))).unwrap()).unwrap();
        let result = music_xml::recognize(&image).unwrap();
        let report = &result.owner.owner.prepared.report;
        let events = report["events"].as_array().unwrap();
        assert_eq!(events.len(), 5);
        assert_eq!(result.xml.matches("<rest/>").count(), 1);
        let observed: Value = serde_json::from_slice(
            &fs::read(fixtures.join(format!("{name}-observed.json"))).unwrap(),
        )
        .unwrap();
        for original in observed["systems"][0]["notes"].as_array().unwrap() {
            let e = events
                .iter()
                .find(|e| e["shape"] == original["shape"] && e["bounds"] == original["bounds"])
                .unwrap();
            for field in [
                "rest",
                "step",
                "octave",
                "duration",
                "time_offset",
                "voice",
                "dots",
                "measure_index",
            ] {
                assert_eq!(e[field], original[field], "{name}:{field}");
            }
        }
        assert_eq!(events[0]["shape"], "HALF_REST");
        assert_eq!(events[0]["duration_quarters"], 2);
        assert_eq!(events[1]["time_offset"], "1/2");
        assert_eq!(report["musicxml_complete"], false);
        assert_eq!(report["epilog_complete"], false);
        assert_eq!(
            result.owner.layout()["rest_chords"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert!(
            report["shared_sig"]["inters"]
                .as_array()
                .unwrap()
                .iter()
                .all(|n| !matches!(n["kind"].as_str(), Some("BeamInter" | "BeamHookInter")))
        );
    }
    let weak = GrayImage::decode(&fs::read(fixtures.join("half-rest-y240.png")).unwrap()).unwrap();
    assert!(
        music_xml::recognize(&weak)
            .err()
            .unwrap()
            .starts_with("musicxml_incomplete_or_excess_measure")
    );
    let beamed = GrayImage::decode(&fs::read(fixtures.join("beamed-notes.png")).unwrap()).unwrap();
    assert!(
        music_xml::recognize(&beamed)
            .err()
            .unwrap()
            .starts_with("music_beam_")
    );
}

#[test]
fn batch_cli_exports_independent_scores_system_break_and_preserves_outputs_on_refusal() {
    let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/official-image-musicxml-limited");
    let root = std::env::temp_dir().join(format!(
        "audiveris-musicxml-batch-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    let names = ["quarter-rest", "half-two-measures", "quarter-half-break"];
    let images = names.map(|name| fixtures.join(format!("{name}.png")));
    let output = Command::new(env!("CARGO_BIN_EXE_audiveris-musicxml"))
        .args(["--output-dir"])
        .arg(&root)
        .args(["--part-name", "Flute & <lead>"])
        .args(&images)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap().lines().count(), 3);
    let truth: Value = serde_json::from_str(include_str!(
        "../fixtures/official-image-musicxml-limited/MULTISCORE-TRUTH.json"
    ))
    .unwrap();
    let mut originals = Vec::new();
    for name in names {
        let xml = fs::read_to_string(root.join(format!("{name}.musicxml"))).unwrap();
        assert!(xml.contains("<part-name>Flute &amp; &lt;lead&gt;</part-name>"));
        let report_bytes = fs::read(root.join(format!("{name}.report.json"))).unwrap();
        let report: Value = serde_json::from_slice(&report_bytes).unwrap();
        let events = report["events"].as_array().unwrap();
        assert_eq!(events.len(), truth[name].as_array().unwrap().len());
        assert_eq!(xml.matches("<note>").count(), events.len());
        for (e, declared) in events.iter().zip(truth[name].as_array().unwrap()) {
            let pitch = if e["rest"] == true {
                "rest".into()
            } else {
                format!("{}{}", e["step"].as_str().unwrap(), e["octave"])
            };
            assert_eq!(pitch, declared[0].as_str().unwrap());
            assert_eq!(e["duration_quarters"], declared[1]);
        }
        let oracle: Value = serde_json::from_str(
            &fs::read_to_string(fixtures.join(format!("{name}-observed.json"))).unwrap(),
        )
        .unwrap();
        for sys in oracle["systems"].as_array().unwrap() {
            for o in sys["notes"].as_array().unwrap() {
                let e = events
                    .iter()
                    .find(|e| {
                        e["system_id"] == sys["system"]
                            && e["bounds"] == o["bounds"]
                            && e["shape"] == o["shape"]
                    })
                    .unwrap();
                for field in [
                    "rest",
                    "step",
                    "octave",
                    "duration",
                    "time_offset",
                    "voice",
                    "dots",
                    "measure_index",
                ] {
                    assert_eq!(e[field], o[field], "{name}:{field}");
                }
            }
        }
        if name == "quarter-half-break" {
            assert_eq!(report["measures"].as_array().unwrap().len(), 4);
            assert_eq!(xml.matches("<print new-system=\"yes\"/>").count(), 1);
            assert_eq!(report["measures"][2]["system_id"], 2);
            // PartwiseBuilder.isNewClef compares meaning across distinct header IDs.
            assert_eq!(xml.matches("<clef>").count(), 1);
        }
        assert_eq!(report["part_name"], "Flute & <lead>");
        assert_eq!(report["musicxml_complete"], false);
        originals.push((name, xml, report_bytes));
    }
    let repeat = Command::new(env!("CARGO_BIN_EXE_audiveris-musicxml"))
        .arg("--output-dir")
        .arg(&root)
        .args(&images)
        .output()
        .unwrap();
    assert!(!repeat.status.success());
    for (name, xml, bytes) in originals {
        assert_eq!(
            fs::read_to_string(root.join(format!("{name}.musicxml"))).unwrap(),
            xml
        );
        assert_eq!(
            fs::read(root.join(format!("{name}.report.json"))).unwrap(),
            bytes
        );
    }
    let partial = root.join("partial");
    fs::create_dir(&partial).unwrap();
    let rejected = Command::new(env!("CARGO_BIN_EXE_audiveris-musicxml"))
        .arg("--output-dir")
        .arg(&partial)
        .arg(&images[0])
        .arg(fixtures.join("central-rest-ambiguous.png"))
        .output()
        .unwrap();
    assert!(!rejected.status.success());
    assert!(
        String::from_utf8_lossy(&rejected.stderr)
            .contains("music_filter_weak_head_optionals_pending")
    );
    assert!(partial.join("quarter-rest.musicxml").exists());
    assert!(partial.join("quarter-rest.report.json").exists());
    assert!(!partial.join("central-rest-ambiguous.musicxml").exists());
    assert!(!partial.join("central-rest-ambiguous.report.json").exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn actual_images_preserve_rests_half_notes_measure_totals_and_refuse_missing_meter() {
    let truth: Value = serde_json::from_str(include_str!(
        "../fixtures/official-image-musicxml-limited/TRUTH.json"
    ))
    .unwrap();
    for (name, png, observed) in [
        (
            "quarter-rest",
            include_bytes!("../fixtures/official-image-musicxml-limited/quarter-rest.png")
                .as_slice(),
            include_str!("../fixtures/official-image-musicxml-limited/quarter-rest-observed.json"),
        ),
        (
            "half-two-measures",
            include_bytes!("../fixtures/official-image-musicxml-limited/half-two-measures.png")
                .as_slice(),
            include_str!(
                "../fixtures/official-image-musicxml-limited/half-two-measures-observed.json"
            ),
        ),
    ] {
        let result = music_xml::recognize(&GrayImage::decode(png).unwrap()).unwrap();
        let report = &result.owner.owner.prepared.report;
        let events = report["events"].as_array().unwrap();
        assert_eq!(events.len(), truth[name].as_array().unwrap().len());
        for (e, declared) in events.iter().zip(truth[name].as_array().unwrap()) {
            let pitch = if e["rest"] == true {
                "rest".into()
            } else {
                format!("{}{}", e["step"].as_str().unwrap(), e["octave"])
            };
            assert_eq!(pitch, declared[0].as_str().unwrap());
            assert_eq!(e["duration_quarters"], declared[1]);
        }
        let oracle: Value = serde_json::from_str(observed).unwrap();
        for sys in oracle["systems"].as_array().unwrap() {
            for o in sys["notes"].as_array().unwrap() {
                let e = events
                    .iter()
                    .find(|e| {
                        e["system_id"] == sys["system"]
                            && e["bounds"] == o["bounds"]
                            && e["shape"] == o["shape"]
                    })
                    .unwrap();
                for field in [
                    "rest",
                    "step",
                    "octave",
                    "duration",
                    "time_offset",
                    "voice",
                    "dots",
                    "measure_index",
                ] {
                    assert_eq!(e[field], o[field], "{name}:{field}");
                }
            }
        }
        for measure in report["measures"].as_array().unwrap() {
            let sum: u64 = events
                .iter()
                .filter(|e| e["measure_number"] == measure["number"])
                .map(|e| e["duration_quarters"].as_u64().unwrap())
                .sum();
            assert_eq!(sum, 4);
        }
        let rest_count = events.iter().filter(|e| e["rest"] == true).count();
        assert_eq!(result.xml.matches("<rest/>").count(), rest_count);
        assert_eq!(
            result.owner.layout()["rest_chords"]
                .as_array()
                .unwrap()
                .len(),
            rest_count
        );
        assert_eq!(
            result.owner.layout()["head_chords"]
                .as_array()
                .unwrap()
                .len(),
            events.len() - rest_count
        );
        assert_eq!(report["musicxml_complete"], false);
        assert_eq!(report["stems_complete"], false);
    }
    let image = GrayImage::decode(include_bytes!(
        "../fixtures/official-image-musicxml-limited/no-meter.png"
    ))
    .unwrap();
    let err = music_xml::recognize(&image).err().unwrap();
    assert_eq!(err, "musicxml_recognized_meter_required_no_default");
}
