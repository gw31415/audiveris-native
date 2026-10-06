// SPDX-License-Identifier: AGPL-3.0-or-later
// Independent Rust predicate prototype; not connected to the image CLI yet.
fn too_close(rest:[i64;4],chord:[i64;4],dx:i64)->bool {
    rest[0]-dx < chord[0]+chord[2] && chord[0] < rest[0]+rest[2]+dx
        && rest[1] < chord[1]+chord[3] && chord[1] < rest[1]+rest[3]
        && ((chord[0] as f64+chord[2] as f64/2.)-(rest[0] as f64+rest[2] as f64/2.)).abs() < dx as f64
}
fn main() {
    let p=std::env::args().nth(1).expect("original observation path");
    let d:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap();
    let chord=serde_json::from_value(d["chord_bounds"].clone()).unwrap();
    let dx=(0.5*d["interline"].as_f64().unwrap()).round_ties_even() as i64;
    assert_eq!(dx,d["minimum_dx"].as_i64().unwrap());
    for c in d["cases"].as_array().unwrap() {
        let rest=serde_json::from_value(c["rest_bounds"].clone()).unwrap();
        assert_eq!(!too_close(rest,chord,dx),c["accepted"].as_bool().unwrap(),"{}",c["name"]);
        println!("{} original agrees",c["name"]);
    }
    println!("prototype5 conditions passed; actual image CLI integration pending");
}
