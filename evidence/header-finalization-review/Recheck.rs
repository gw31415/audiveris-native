use audiveris_native::header_finalization::finalize_system;
use serde_json::{Value,json};
fn main(){
 let grid=json!({"bar_semantics":{"left_end_peaks":[],"systems":[]}});
 let base=json!({"staffs":[{"staff_id":1,"header_start":0,"header_stop":100,"clef_stop":25,"header_key":null,"header_time":null}],"sig_inters":[],"relations":[]});
 let mut p=base.clone();p["sig_inters"]=json!([
 {"id":1,"kind":"ClefInter","staff_id":1,"bounds":[1,1,5,30],"grade":0.1},
 {"id":2,"kind":"KeyInter","staff_id":1,"bounds":[40,1,5,30],"grade":0.4},
 {"id":3,"kind":"KeyInter","staff_id":1,"bounds":[40,1,5,30],"grade":0.4000000000000001}]);p["relations"]=json!([{ "kind":"ClefKeyRelation","source":1,"target":2},{"kind":"ClefKeyRelation","source":1,"target":3},{"kind":"Exclusion","source":2,"target":3}]);
 let out=finalize_system(&grid,&p,10).unwrap();println!("partition actual {:?}; source expected {:?}",out["staffs"][0]["clef"]["contextual_grade"].as_f64().unwrap(), ((1.+2.)*0.1)/(1.+2.*0.1));
 let mut p=base.clone();p["staffs"][0]["header_start"]=json!(i64::MAX);let r=std::panic::catch_unwind(||finalize_system(&grid,&p,10));println!("header_start_overflow_panics={}",r.is_err());
 let mut p=base;p["staffs"][0]["header_key"]=json!({"id":1});p["sig_inters"]=json!([
 {"id":1,"kind":"KeyInter","staff_id":1,"bounds":[40,1,10,30],"grade":0.4},
 {"id":2,"kind":"KeyAlterInter","staff_id":1,"bounds":[null,1,5,30],"grade":0.4},
 {"id":3,"kind":"KeyAlterInter","staff_id":1,"bounds":[45,1,5,30],"grade":0.4}]);p["relations"]=json!([{ "kind":"Containment","source":1,"target":2},{"kind":"Containment","source":1,"target":3}]);let r=std::panic::catch_unwind(||finalize_system(&grid,&p,10));println!("malformed_member_bounds_panics={}",r.is_err());
 let _=Value::Null;
}
