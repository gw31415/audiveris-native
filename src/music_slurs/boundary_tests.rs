// SPDX-License-Identifier: AGPL-3.0-or-later
//! Declared scalar ownership controls, not image or source runtime observations.
use super::*;
use crate::{
    music_measures::{Chord, ChordKind, Measure},
    music_time::Fraction,
    music_voices::{self, Slot, SlotVoice, Voice},
    shared_sig::Phase,
};

struct Declared {
    graph: Graph,
    before_slur: Graph,
    state: State,
    chords: Vec<Chord>,
    measures: Vec<Measure>,
    voices: Vec<Voice>,
    slot: Slot,
}
fn key(phase: Phase, local: usize) -> Key {
    Key {
        system: 1,
        phase,
        local,
    }
}
fn declared() -> Declared {
    let points = (0..5)
        .map(|line| {
            vec![
                [10., 100. + line as f64 * 11.],
                [200., 100. + line as f64 * 11.],
            ]
        })
        .collect::<Vec<_>>();
    let staff = Arc::new(StaffGeometry {
        id: 1,
        system: 1,
        specific_interline: 11,
        abscissae: [10, 200],
        lines: points
            .iter()
            .map(|p| Arc::new(crate::curves::Curve::interpolate(p.clone()).unwrap()))
            .collect(),
    });
    let geometry = json!({
        "systems":[{"id":1,"coordinates":[10,100,191,144,44],"staff_ids":[1]}],
        "staffs":[{"id":1,"system_id":1,"left":10,"right":200,
            "lines":points.iter().map(|p|json!({"points":p})).collect::<Vec<_>>()}]
    });
    let area = crate::system_area_geometry::build(&geometry, 220, 200, 11)
        .unwrap()
        .system_areas
        .remove(0);
    let system = Arc::new(SystemGeometry {
        id: 1,
        left: 10,
        profile: 0,
        area: Arc::new(area),
        staffs: vec![staff.clone()],
    });
    let part = Arc::new(PartGeometry {
        id: 1,
        system: 1,
        staffs: vec![staff.clone()],
        bounds: [10, 90, 190, 65],
    });
    let measure = Arc::new(MeasureLayout {
        part: part.clone(),
        index: 1,
        stack: Arc::new(crate::music_measure_layout::Stack {
            system: 1,
            index: 1,
        }),
        skew: crate::sheet_skew::SheetSkew::new(0.).unwrap(),
        contained_left: None,
        effective_left: None,
        right: None,
    });
    let mut graph = Graph::default();
    let mut chords = Vec::new();
    for (index, x) in [30, 100].into_iter().enumerate() {
        let head = key(Phase::Head, index + 1);
        let chord = key(Phase::Chord, index + 1);
        graph
            .vertex(
                head,
                json!({"kind":"HeadInter","shape":"NOTEHEAD_BLACK",
            "bounds":[x,120,6,5],"staff_id":1,"grade":0.8,"manual":true}),
                false,
            )
            .unwrap();
        let id = graph
            .vertex(
                chord,
                json!({"kind":"HeadChordInter","staff_id":1,
            "part_id":1,"measure_index":1,"bounds":null,"member_ids":[],
            "voice":null,"time_offset":null,"manual":true,"dots_number":0}),
                false,
            )
            .unwrap();
        graph
            .relations(
                1,
                Phase::Chord,
                vec![(chord, head, json!({"kind":"Containment","manual":true}))],
            )
            .unwrap();
        chords.push(Chord {
            kind: ChordKind::Head,
            key: chord,
            id,
            stem: None,
            members: vec![graph.retained_inter(head).unwrap().0.clone()],
            measure: 0,
            bounds: [x, 120, 6, 5],
        });
    }
    let mut measures = vec![Measure {
        system: 1,
        part: 1,
        staff: 1,
        index: 1,
        left: 10,
        right: 200,
        geometry: Some(measure.clone()),
        timing: Default::default(),
        tuplets: vec![],
        stack_tuplets: vec![],
    }];
    graph.bind_dot_measures(&measures).unwrap();
    let mut voices = Vec::new();
    let vi = music_voices::create_voice(&mut graph, &mut voices, &measures, &chords[0]).unwrap();
    let slot = Slot {
        identity: Arc::new(()),
        id: 1,
        time_offset: Some(Fraction::new(7, 64).unwrap()),
        x_offset: 23,
        incoming_ids: vec![chords[0].id],
    };
    graph
        .set_chord_slot(chords[0].key, slot.id, &slot.identity)
        .unwrap();
    let mut body = graph.retained_inter(chords[0].key).unwrap().0.clone();
    body["time_offset"] = json!("7/64");
    graph.vertex(chords[0].key, body, false).unwrap();
    measures[0].timing.expected = Some(Fraction::new(3, 4).unwrap());
    measures[0].timing.slots.push(slot.clone());
    voices[vi].slots.insert(
        slot.id,
        SlotVoice {
            chord_id: chords[0].id,
            continuing: false,
        },
    );
    music_voices::check_duration(&graph, &mut voices[vi], &mut measures[0]).unwrap();
    assert_eq!(
        graph.chord_end_time(chords[0].key).unwrap(),
        Some("23/64".into())
    );
    assert_eq!(
        voices[vi].termination.map(crate::music_time::format_delta),
        Some("-25/64".into())
    );
    let before_slur = graph.clone();
    let glyph = GlyphPixels {
        left: 33,
        top: 112,
        width: 71,
        height: 7,
        orientation: Orientation::Vertical,
        runs: vec![
            Run {
                position: 0,
                start: 6,
                length: 1,
            },
            Run {
                position: 35,
                start: 0,
                length: 1,
            },
            Run {
                position: 70,
                start: 6,
                length: 1,
            },
        ],
    };
    let candidate = Arc::new(Candidate {
        components: vec![Arc::new(glyph.clone())],
        points: vec![
            fit::Pixel { x: 33, y: 118 },
            fit::Pixel { x: 68, y: 112 },
            fit::Pixel { x: 103, y: 118 },
        ],
        curve: lookup::Curve {
            bezier: [[33., 118.], [50., 108.], [86., 108.], [103., 118.]],
            above: true,
        },
        grade: 0.8,
        glyph,
    });
    let mut registry = beam_dispatch::Registry::default();
    let glyph_id = registry.register(&candidate.glyph).unwrap();
    let slur_key = key(Phase::Curve, 1);
    graph
        .vertex(
            slur_key,
            json!({"kind":"SlurInter","shape":"SLUR_ABOVE",
        "above":true,"curve":candidate.curve.bezier,"bounds":curve_bounds(candidate.curve),
        "glyph_id":glyph_id,"grade":candidate.grade,"manual":false,"is_tie":false}),
            false,
        )
        .unwrap();
    let relations = [lookup::Side::Left, lookup::Side::Right].map(|side| {
        Arc::new(Relation {
            side,
            euclidean: 1.,
        })
    });
    let mut relation_ids = [0; 2];
    for side in 0..2 {
        relation_ids[side] = graph
            .register_fresh_slur_relation(
                slur_key,
                key(Phase::Head, side + 1),
                relations[side].clone(),
            )
            .unwrap();
    }
    let slur = Arc::new(Slur {
        key: slur_key,
        system,
        part: part.clone(),
        endpoints: [0, 1].map(|side| Endpoint {
            head: key(Phase::Head, side + 1),
            chord: chords[side].key,
            staff: staff.clone(),
            measure: measure.clone(),
        }),
        relations,
        relation_ids,
        glyph_id,
        candidate,
    });
    let state = State {
        slurs: vec![slur.clone()],
        parts: vec![PartSlurs {
            part,
            slurs: vec![slur],
        }],
        raw_candidate_count: 1,
        clump_count: 1,
    };
    graph.set_fresh_slur_part(slur_key, 1).unwrap();
    Declared {
        graph,
        before_slur,
        state,
        chords,
        measures,
        voices,
        slot,
    }
}
fn observed(f: &Declared) -> Value {
    let slur = &f.state.slurs[0];
    json!({
        "graph":f.graph.slur_preservation_observation(),
        "clocks":f.graph.slur_chord_clock_observation(),
        "chords":format!("{:?}", f.chords),
        "chord_addresses":f.chords.iter().map(|c| c as *const _ as usize).collect::<Vec<_>>(),
        "voices":music_voices::projection(&f.voices, &f.measures),
        "voice_owners":f.voices.iter().map(|v| Arc::as_ptr(&v.identity) as usize).collect::<Vec<_>>(),
        "expected":f.measures[0].timing.expected,
        "excess":f.measures[0].timing.excess,
        "slots":music_voices::slots_projection(&f.measures[0].timing.slots),
        "slot_owner":Arc::as_ptr(&f.measures[0].timing.slots[0].identity) as usize,
        "slur_owner":Arc::as_ptr(slur) as usize,
        "candidate_owner":Arc::as_ptr(&slur.candidate) as usize,
        "part_owner":Arc::as_ptr(&slur.part) as usize,
        "system_owner":Arc::as_ptr(&slur.system) as usize,
        "endpoint_owners":slur.endpoints.iter().map(|e|json!([Arc::as_ptr(&e.staff) as usize,
            Arc::as_ptr(&e.measure) as usize, e.head, e.chord])).collect::<Vec<_>>(),
        "relations":slur.relations.iter().map(|r|json!([Arc::as_ptr(r) as usize,r.projection()])).collect::<Vec<_>>()
    })
}
#[test]
fn output_without_slur_visits_refuses_live_or_retained_owners_without_changing_clocks() {
    let f = declared();
    let before = observed(&f);
    let require = crate::music_xml::slur_boundary::require_no_slurs;
    let pending = "musicxml_slur_writer_required_for_retained_or_live_Slur";
    assert!(require(&f.before_slur, None).is_ok());
    assert_eq!(require(&f.graph, Some(&f.state)).unwrap_err(), pending);
    assert_eq!(require(&f.graph, None).unwrap_err(), pending);
    assert_eq!(
        require(&f.before_slur, Some(&f.state)).unwrap_err(),
        pending
    );
    let part_only = State {
        slurs: vec![],
        parts: vec![PartSlurs {
            part: f.state.parts[0].part.clone(),
            slurs: f.state.parts[0].slurs.clone(),
        }],
        raw_candidate_count: 1,
        clump_count: 1,
    };
    assert_eq!(
        require(&f.before_slur, Some(&part_only)).unwrap_err(),
        pending
    );
    assert_eq!(observed(&f), before);
    assert!(Arc::ptr_eq(&f.state.slurs[0], &f.state.parts[0].slurs[0]));
    assert!(Arc::ptr_eq(
        f.graph.chord_slot_identity(f.chords[0].key).unwrap(),
        &f.slot.identity
    ));
    assert!(Arc::ptr_eq(
        &f.graph
            .chord_voice_owner(f.chords[0].key, &f.voices)
            .unwrap()
            .unwrap()
            .identity,
        &f.voices[0].identity
    ));
    for (id, owner) in f.state.slurs[0]
        .relation_ids
        .iter()
        .zip(&f.state.slurs[0].relations)
    {
        assert!(Arc::ptr_eq(
            f.graph.fresh_slur_relation_owner(*id).unwrap(),
            owner
        ));
    }
}
#[test]
fn output_without_slur_visits_rejects_a_live_relation_even_without_a_live_slur_vertex() {
    let mut f = declared();
    let key = f.state.slurs[0].key;
    let body = f.graph.retained_inter(key).unwrap().0.clone();
    // Declared inconsistent SIG: vertex tombstoned while the actual edge is
    // still live. The export boundary must not trust a state-list-only check.
    f.graph.vertex(key, body, true).unwrap();
    let before = observed(&f);
    assert!(f.graph.live_inter_keys(1, "SlurInter").is_empty());
    assert_eq!(
        crate::music_xml::slur_boundary::require_no_slurs(&f.graph, None).unwrap_err(),
        "musicxml_slur_writer_required_for_retained_or_live_Slur"
    );
    assert_eq!(observed(&f), before);
}

#[test]
fn typed_slur_support_leaves_head_grades_and_actual_clock_owners_unchanged() {
    type Contextualize = fn(&mut Graph) -> Result<(), String>;
    let passes: [(&str, Contextualize); 2] = [
        ("Head/Stem", Graph::contextualize_head_stems),
        ("Head/Stem/Flag", Graph::contextualize_head_stem_flags),
    ];
    let without_head_context = |mut observation: Value| {
        for node in observation["graph"]["sig"]["inters"]
            .as_array_mut()
            .unwrap()
        {
            if node["kind"] == "HeadInter" {
                node.as_object_mut().unwrap().remove("contextual_grade");
            }
        }
        observation
    };
    for (name, contextualize) in passes {
        let mut f = declared();
        let before = without_head_context(observed(&f));
        let mut baseline = f.before_slur.clone();
        contextualize(&mut baseline).unwrap();
        contextualize(&mut f.graph).unwrap();
        let slur = &f.state.slurs[0];
        for side in 0..2 {
            let head = slur.endpoints[side].head;
            let want = &baseline.retained_inter(head).unwrap().0["contextual_grade"];
            assert_eq!(want, &json!(0.8), "{name}: declared unsupported Head");
            assert_eq!(
                &f.graph.retained_inter(head).unwrap().0["contextual_grade"],
                want,
                "{name}: Slur support has zero contribution to its Head"
            );
            let selected = &slur.relations[side];
            assert_eq!(selected.projection()["source_ratio"], json!(6.0));
            assert_eq!(selected.projection()["target_ratio"], json!(1.0));
            assert_eq!(
                f.graph
                    .fresh_slur_relation_id(slur.key, head, selected.side)
                    .unwrap(),
                slur.relation_ids[side]
            );
            assert!(Arc::ptr_eq(
                f.graph
                    .fresh_slur_relation_owner(slur.relation_ids[side])
                    .unwrap(),
                selected
            ));
        }
        // Only projected Head context may be populated. The actual selected
        // relation and every non-context owner/clock field must survive intact.
        assert_eq!(without_head_context(observed(&f)), before, "{name}");
        assert!(Arc::ptr_eq(
            f.graph.chord_slot_identity(f.chords[0].key).unwrap(),
            &f.slot.identity
        ));
        assert!(Arc::ptr_eq(
            &f.graph
                .chord_voice_owner(f.chords[0].key, &f.voices)
                .unwrap()
                .unwrap()
                .identity,
            &f.voices[0].identity
        ));
    }
}

#[test]
fn slur_chord_getters_preserve_cold_geometry_warm_cache_and_actual_clock_owners() {
    let mut f = declared();
    let chord = f.chords[0].key;
    let stem = key(Phase::Stem, 1);
    f.graph
        .vertex(
            stem,
            json!({"kind":"StemInter","shape":"STEM","bounds":[34,90,2,34],
                "grade":0.8,"manual":true}),
            false,
        )
        .unwrap();
    f.graph
        .relations(
            1,
            Phase::Chord,
            vec![(
                chord,
                stem,
                json!({"kind":"ChordStemRelation","manual":true}),
            )],
        )
        .unwrap();
    f.chords[0].stem = Some(stem);
    f.chords[0].bounds = [30, 90, 6, 35];
    let before = observed(&f);
    let chord_node = |snapshot: &Value| {
        snapshot["inters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["id"] == f.chords[0].id)
            .unwrap()
            .clone()
    };
    assert!(chord_node(&f.graph.snapshot())["retained_chord_cache"].is_null());
    let cold = f.graph.retained_or_live_chord_getters(chord).unwrap();
    assert_eq!(cold.stem, Some(stem));
    assert_eq!(cold.bounds, Some([30, 90, 6, 35]));
    assert_eq!(cold.head, Some([35, 122]));
    assert_eq!(cold.tail, Some([35, 90]));
    assert_eq!(observed(&f), before);
    assert!(chord_node(&f.graph.snapshot())["retained_chord_cache"].is_null());
    assert_eq!(f.slot.time_offset, Some(Fraction::new(7, 64).unwrap()));
    assert_eq!(
        f.measures[0].timing.expected,
        Some(Fraction::new(3, 4).unwrap())
    );
    assert_eq!(f.graph.chord_end_time(chord).unwrap(), Some("23/64".into()));
    assert!(Arc::ptr_eq(
        f.graph.chord_slot_identity(chord).unwrap(),
        &f.slot.identity
    ));
    assert!(Arc::ptr_eq(
        &f.graph
            .chord_voice_owner(chord, &f.voices)
            .unwrap()
            .unwrap()
            .identity,
        &f.voices[0].identity
    ));

    // Declared pre-invalidation state: the edge already names the Stem while
    // old geometry remains cached. This setup is not a callback invocation.
    f.graph
        .bind_recognition_chord_caches(&[(chord, Some(stem))])
        .unwrap();
    f.graph
        .declare_chord_geometry(chord, [30, 120, 6, 5], [33, 122], [33, 122])
        .unwrap();
    let warm_before = observed(&f);
    let warm = f.graph.retained_or_live_chord_getters(chord).unwrap();
    assert_eq!(warm.stem, Some(stem));
    assert_eq!(warm.bounds, Some([30, 120, 6, 5]));
    assert_eq!(warm.head, Some([33, 122]));
    assert_eq!(warm.tail, Some([33, 122]));
    assert_eq!(observed(&f), warm_before);

    // Declared damaged live endpoint: geometry alone cannot certify a Stem.
    let mut stem_body = f.graph.retained_inter(stem).unwrap().0.clone();
    stem_body["kind"] = json!("HeadInter");
    f.graph.vertex(stem, stem_body, false).unwrap();
    let damaged_before = observed(&f);
    assert_eq!(
        f.graph.retained_or_live_chord_getters(chord).unwrap_err(),
        "typed_owned_chord_identity_required"
    );
    assert_eq!(observed(&f), damaged_before);
}
