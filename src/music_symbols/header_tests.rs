// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
pub(crate) fn assert_real_headers(owner: &crate::stem_image_owned::OwnedItems) {
    let source: Value = serde_json::from_str(include_str!(
        "../../fixtures/official-symbol-headers/image-observed.json"
    ))
    .unwrap();
    let mut mask = GrayImage {
        width: source["width"].as_u64().unwrap() as usize,
        height: source["height"].as_u64().unwrap() as usize,
        pixels: vec![
            0;
            (source["width"].as_u64().unwrap() * source["height"].as_u64().unwrap())
                as usize
        ],
    };
    let graph = owner.graph.snapshot();
    let mut count = 0;
    for header in owner.upstream_report["headers"]["staffs"]
        .as_array()
        .unwrap()
    {
        let staff = owner
            .staffs
            .iter()
            .find(|staff| {
                Some(staff.id) == header["staff_id"].as_u64()
                    && Some(staff.system) == header["system_id"].as_u64()
            })
            .unwrap();
        assert!(owner.parts.iter().any(|part| {
            part.staffs
                .iter()
                .any(|held| std::sync::Arc::ptr_eq(held, staff))
        }));
        let expected = source["rectangles"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["staff"] == staff.id && r["system"] == staff.system)
            .unwrap();
        assert_eq!(header["start"], expected["start"]);
        assert_eq!(header["stop"], expected["stop"]);
        assert_eq!(json!(owner.interline), expected["interline"]);
        let rectangle = header_rectangle(
            staff,
            header["start"].as_i64().unwrap(),
            header["stop"].as_i64().unwrap(),
            owner.interline,
        )
        .unwrap();
        assert_eq!(json!(rectangle), expected["bounds"]);
        erase_rectangle(&mut mask, rectangle);
        count += 1;
    }
    assert_eq!(count, 6);
    assert_eq!(
        format!("{:x}", Sha256::digest(&mask.pixels)),
        source["mask_sha256"].as_str().unwrap()
    );
    assert_eq!(owner.graph.snapshot(), graph);
}
