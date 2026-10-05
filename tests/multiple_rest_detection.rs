// SPDX-License-Identifier: AGPL-3.0-or-later
use audiveris_native::{
    multiple_rest_admission::StaffPosition,
    multiple_rest_detection::{BeamInput, discover},
    serif_peaks::Peak,
};
fn beam() -> BeamInput {
    BeamInput {
        id: 3,
        kind: "BeamInter",
        removed: false,
        bounds: [100, 149, 101, 8],
        median: [100., 153., 200., 153.],
        height: 8.,
        grade: 0.72,
        staff: Some(StaffPosition {
            id: 7,
            tablature: false,
            endpoint_pitches: [0., 0.],
            mid_line_thickness: 2.,
        }),
    }
}
fn peak(x: i32) -> Peak {
    Peak {
        start: x,
        stop: x + 3,
        top: 135,
        bottom: 171,
        impacts: [0.9; 6],
        grade: 0.7,
    }
}
#[test]
fn ordered_detection_selection_and_registration_geometry() {
    let mut queries = Vec::new();
    let plan = discover(&beam(), 16, 2, |staff, q| {
        queries.push((staff, q));
        Ok(if q.left {
            vec![peak(95), peak(100)]
        } else {
            vec![peak(198), peak(205)]
        })
    })
    .unwrap()
    .unwrap();
    assert_eq!(queries.len(), 2);
    assert_eq!(
        (
            queries[0].0,
            queries[0].1.x,
            queries[0].1.left,
            queries[0].1.added_chunk
        ),
        (7, 100, true, 6)
    );
    assert_eq!((queries[1].1.x, queries[1].1.left), (200, false));
    assert_eq!(plan.serifs[0].peak.start, 100);
    assert_eq!(plan.serifs[1].peak.start, 198);
    assert_eq!(plan.serifs[0].median, [102., 134.5, 102., 172.5]);
    assert_eq!(plan.serifs[0].width, 4.);
    assert_eq!((plan.beam_id, plan.staff_id, plan.grade), (3, 7, 0.72));
    assert_eq!(plan.median, beam().median);
    assert!(plan.serifs[0].left && !plan.serifs[1].left);
}
#[test]
fn admission_rejection_and_missing_side_leave_no_plan() {
    let mut b = beam();
    b.removed = true;
    assert!(
        discover(&b, 16, 2, |_, _| panic!("rejected beam searched"))
            .unwrap()
            .is_none()
    );
    b.removed = false;
    for missing_left in [false, true] {
        let mut visited = Vec::new();
        assert!(
            discover(&b, 16, 2, |_, q| {
                visited.push(q.left);
                Ok(if q.left == missing_left {
                    vec![]
                } else {
                    vec![peak(q.x)]
                })
            })
            .unwrap()
            .is_none()
        );
        assert_eq!(visited, [true, false]);
    }
}
