// SPDX-License-Identifier: AGPL-3.0-or-later
use super::beam_rest_clear_tests::{frame, prepare_control};
use super::beam_rest_linked_tests::chord_key;
use super::*;
use crate::music_measure_slots::{ClockState, MeasureSlot, compound};

fn original() -> Value {
    let mut d: Value = serde_json::from_str(include_str!(
        "../../../fixtures/official-narrow-slot-owners/observed.json"
    ))
    .unwrap();
    let rs = d["relation_sets"].as_array().unwrap().clone();
    for f in d["frame_sets"].as_array_mut().unwrap() {
        f["owned"]["relations"] = rs[f["owned"]["relations"].as_u64().unwrap() as usize].clone();
    }
    let fs = d["frame_sets"].as_array().unwrap().clone();
    for c in d["cases"].as_array_mut().unwrap() {
        for p in c["phases"].as_array_mut().unwrap() {
            p["state"] = fs[p["state"].as_u64().unwrap() as usize].clone();
        }
    }
    d
}
#[test]
fn original_shared_narrow_owner_and_reused_numbers_keep_independent_nullable_exact_clocks() {
    for case in original()["cases"].as_array().unwrap() {
        let mut c = prepare_control(case);
        let narrow = case["declared_narrow"]
            .as_array()
            .unwrap()
            .iter()
            .map(|spec| MeasureSlot {
                identity: Arc::new(()),
                id: spec["id"].as_u64().unwrap() as usize,
                chord_ids: spec["chords"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|name| {
                        c.owner
                            .owner
                            .graph
                            .id(chord_key(&c, name.as_str().unwrap()))
                            .unwrap()
                    })
                    .collect(),
            })
            .collect::<Vec<_>>();
        let names = ["narrow0", "narrow1", "narrow2"];
        let centers = c
            .owner
            .chords
            .iter()
            .map(|ch| (ch.id, ch.bounds[1] + ch.bounds[3] / 2))
            .collect::<BTreeMap<_, _>>();
        let compounds = case["declared_compounds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|spec| {
                let members = spec["members"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|name| {
                        narrow[names.iter().position(|&n| json!(n) == *name).unwrap()].clone()
                    })
                    .collect::<Vec<_>>();
                compound(spec["id"].as_u64().unwrap() as usize, &members, &centers).unwrap()
            })
            .collect::<Vec<_>>();
        let narrow_name = |member: &MeasureSlot| {
            names[narrow
                .iter()
                .position(|n| Arc::ptr_eq(&member.identity, &n.identity))
                .unwrap()]
        };
        for slot in &compounds {
            for &id in &slot.chord_ids {
                let key = c.owner.chords.iter().find(|ch| ch.id == id).unwrap().key;
                c.owner
                    .owner
                    .graph
                    .set_chord_slot(key, slot.id, &slot.identity)
                    .unwrap();
            }
        }
        let mut clocks = ClockState::new(&compounds).unwrap();
        let before = c.owner.owner.graph.snapshot();
        let physical = c.owner.chords.as_ptr();
        let slots = c.owner.measures[0].timing.slots.as_ptr();
        let report = c.owner.owner.prepared.report.clone();
        for phase in case["phases"].as_array().unwrap() {
            match phase["phase"].as_str().unwrap() {
                "created" => {}
                "shared_exact" => clocks
                    .set_owner(&narrow[0], Some(Fraction::new(7, 64).unwrap()))
                    .unwrap(),
                "distinct_quarter" => clocks
                    .set_owner(&narrow[1], Some(Fraction::new(1, 4).unwrap()))
                    .unwrap(),
                "second_exact" => clocks
                    .set_owner(&narrow[2], Some(Fraction::new(3, 16).unwrap()))
                    .unwrap(),
                "shared_null" => clocks.set_owner(&narrow[0], None).unwrap(),
                _ => panic!("unknown source phase"),
            }
            let ns = narrow.iter().enumerate().map(|(i,n)|json!({"label":names[i],"id":n.id,"chords":n.chord_ids.iter().map(|id|c.labels[id].clone()).collect::<Vec<_>>(),"time":clocks.get_owner(n).unwrap().map(crate::music_time::format_fraction)})).collect::<Vec<_>>();
            let cs = compounds.iter().enumerate().map(|(i,s)|{
                let first = phase["narrow"].as_array().unwrap().iter().find(|n|n["label"]==narrow_name(&s.members[0])).unwrap();
                assert_eq!(json!(clocks.first_time(s).unwrap().map(crate::music_time::format_fraction)),first["time"]);
                for (view,row) in s.members.iter().zip(clocks.projection(s).unwrap().as_array().unwrap()) {
                    assert_eq!(row["time_offset"],phase["narrow"].as_array().unwrap().iter().find(|n|n["label"]==narrow_name(view)).unwrap()["time"]);
                }
                json!({"label":format!("compound{i}"),"id":s.id,"members":s.members.iter().map(narrow_name).collect::<Vec<_>>(),"chords":s.chord_ids.iter().map(|id|c.labels[id].clone()).collect::<Vec<_>>(),"lookup":(["chord0","chord1"].iter().map(|name|{let id=c.owner.owner.graph.id(chord_key(&c,name)).unwrap();json!({"chord":name,"narrow":s.narrow_owner(id).map(narrow_name)})}).collect::<Vec<_>>())})
            }).collect::<Vec<_>>();
            let bindings = ["chord0","chord1"].iter().map(|name|{
                let key = chord_key(&c,name);
                let identity = c.owner.owner.graph.chord_slot_identity(key).unwrap();
                json!({"chord":name,"slot":format!("compound{}",compounds.iter().position(|s|Arc::ptr_eq(identity,&s.identity)).unwrap())})
            }).collect::<Vec<_>>();
            assert_eq!(json!(ns), phase["narrow"]);
            assert_eq!(json!(cs), phase["compounds"]);
            assert_eq!(json!(bindings), phase["bindings"]);
            let mut source = phase["state"].clone();
            source["owned"]["relations"] = json!([]);
            assert_eq!(frame(&mut c), source);
            assert_eq!(c.owner.owner.graph.snapshot(), before);
            assert_eq!(c.owner.chords.as_ptr(), physical);
            assert_eq!(c.owner.measures[0].timing.slots.as_ptr(), slots);
            assert_eq!(c.owner.owner.prepared.report, report);
            assert_eq!(
                clocks.get(7).unwrap_err(),
                "music_candidate_ambiguous_narrow_number"
            );
            assert_eq!(
                clocks.set(7, None).unwrap_err(),
                "music_candidate_ambiguous_narrow_number"
            );
        }
        let mut same_number = narrow[0].clone();
        same_number.identity = Arc::new(());
        assert_eq!(
            clocks.get_owner(&same_number).unwrap_err(),
            "music_candidate_owned_narrow_required"
        );
        let mut staged = clocks.clone();
        staged
            .set_owner(&narrow[0], Some(Fraction::new(7, 64).unwrap()))
            .unwrap();
        assert_eq!(clocks.get_owner(&narrow[0]).unwrap(), None);
        assert_eq!(
            staged.get_owner(&compounds[0].members[0]).unwrap(),
            Some(Fraction::new(7, 64).unwrap())
        );
    }
}
