// SPDX-License-Identifier: AGPL-3.0-or-later
//! Selected HeadStemsCleaner.check; no automatic selection or STEMS epilog.
use crate::{head_stem_relation::{self, HeadSide, StemPortion}, shared_sig::{Graph, HeadStemAttachment, Key, Phase}};
use serde_json::{Value,json};
use std::collections::{BTreeMap,BTreeSet};
#[derive(Clone,Debug)]
pub struct Link { pub relation_id:usize, pub median:[f64;4], pub stem_grade:f64, pub target_ratio:f64, pub head_side:HeadSide, pub dy:f64 }
fn canonical(bounds:[i32;4], links:&[&Link])->Result<bool,String> {
 let (mut left,mut right)=(None,None);
 for l in links { if l.dy>0.2{return Ok(false)} match l.head_side {HeadSide::Left=>left=Some(*l),HeadSide::Right=>right=Some(*l)} }
 let (Some(left),Some(right))=(left,right) else{return Ok(false)};
 let center=(bounds[1]+bounds[3]/2) as f64;
 if center >=(left.median[1]+left.median[3])/2. || center <=(right.median[1]+right.median[3])/2. {return Ok(false)}
 Ok(head_stem_relation::stem_portion(bounds,left.median,bounds[1] as f64)?==StemPortion::StemTop && head_stem_relation::stem_portion(bounds,right.median,(bounds[1]+bounds[3]-1) as f64)?==StemPortion::StemBottom)
}
/// Caller supplies the relation list in original check order. Strict minimum
/// preserves the first contribution tie, using intrinsic stem grade only.
pub fn select_removals(bounds:[i32;4], ordered:&[Link])->Result<Vec<usize>,String> {
 if bounds[2]<=0||bounds[3]<=0||bounds.iter().any(|v|v.unsigned_abs()>4_000_000)||ordered.len()>100_000{return Err("invalid_stem_cleaner_input".into())}
 let mut ids=BTreeSet::new();
 for l in ordered {
  if l.relation_id==0||!ids.insert(l.relation_id)||!l.stem_grade.is_finite()||!(0.0..=1.0).contains(&l.stem_grade)||!l.target_ratio.is_finite()||!(1.0..=11.0).contains(&l.target_ratio)||!l.dy.is_finite()||l.dy.abs()>4_000_000.||l.median.iter().any(|v|!v.is_finite()||v.abs()>4_000_000.)||l.median[1]>=l.median[3] {return Err("unsupported_stem_cleaner_link".into())}
 }
 let mut links=ordered.iter().collect::<Vec<_>>();let mut removed=Vec::new();
 loop {
  if links.len()<2 || (links.len()==2&&canonical(bounds,&links)?){break}
  let mut lowest=f64::MAX;let mut worst=None;
  for (i,l) in links.iter().enumerate(){let contribution=l.stem_grade*(l.target_ratio-1.);if lowest>contribution{lowest=contribution;worst=Some(i)}}
  let i=worst.ok_or("unsupported_nonfinite_stem_cleaner_contribution")?;removed.push(links.remove(i).relation_id);
 }
 Ok(removed)
}
/// Selected nonmanual live head/stem keys; unrelated edges and vertices are
/// retained transactionally. Missing relations are skipped as in the original.
/// Parallel relations to one supplied stem need a separate ordering oracle.
pub fn clean_head(graph:&mut Graph,head:Key,ordered:&[Key])->Result<Value,String>{
 if head.system==0||head.phase!=Phase::Head||ordered.len()>100_000{return Err("invalid_stem_cleaner_head".into())}
 let snapshot=graph.snapshot();let vs=snapshot["inters"].as_array().ok_or("invalid_stem_cleaner_sig")?;
 let by_id=vs.iter().map(|v|(v["id"].as_u64().unwrap() as usize,v)).collect::<BTreeMap<_,_>>();
 let hid=graph.id(head).ok_or("missing_stem_cleaner_head")?;let h=by_id.get(&hid).ok_or("removed_stem_cleaner_head")?;
 if h["kind"]!="HeadInter"||h["manual"]==true||h["is_manual"]==true{return Err("unsupported_stem_cleaner_head".into())}
 let bounds:[i32;4]=serde_json::from_value(h["bounds"].clone()).map_err(|_|"invalid_stem_cleaner_head_bounds")?;
 let edges=snapshot["relations"].as_array().ok_or("invalid_stem_cleaner_edges")?;let mut links=Vec::new();let mut seen=BTreeSet::new();
 for k in ordered {
  if k.system!=head.system||k.phase!=Phase::Stem||!seen.insert(*k){return Err("invalid_stem_cleaner_order".into())}
  let id=graph.id(*k).ok_or("missing_stem_cleaner_stem")?;let s=by_id.get(&id).ok_or("removed_stem_cleaner_stem")?;
  if s["kind"]!="StemInter"||s["manual"]==true||s["is_manual"]==true{return Err("unsupported_stem_cleaner_stem".into())}
  let matches=edges.iter().filter(|e|e["kind"]=="HeadStemRelation"&&e["source"]==hid&&e["target"]==id).collect::<Vec<_>>();
  if matches.len()>1{return Err("unsupported_parallel_stem_cleaner_relations".into())}
  let Some(e)=matches.first() else{continue};if e["manual"]==true||e["is_manual"]==true{return Err("unsupported_manual_stem_cleaner_relation".into())}
  links.push(Link{relation_id:e["id"].as_u64().ok_or("invalid_stem_cleaner_relation_id")? as usize,median:serde_json::from_value(s["median"].clone()).map_err(|_|"invalid_stem_cleaner_median")?,stem_grade:s["grade"].as_f64().ok_or("invalid_stem_cleaner_grade")?,target_ratio:e["target_ratio"].as_f64().ok_or("invalid_stem_cleaner_ratio")?,head_side:serde_json::from_value(e["head_side"].clone()).map_err(|_|"invalid_stem_cleaner_side")?,dy:e["dy"].as_f64().ok_or("invalid_stem_cleaner_gap")?});
 }
 let removed=select_removals(bounds,&links)?;let removed_ids=removed.iter().copied().collect::<BTreeSet<_>>();
 let mut records=Vec::new();
 for v in vs {let key:Key=serde_json::from_value(v["origin"].clone()).map_err(|_|"invalid_stem_cleaner_origin")?;if key.system==head.system&&key.phase==Phase::Stem{let mut r=v.clone();r["id"]=json!(key.local);records.push(r)}}
 let mut attachments=Vec::new();
 for e in edges {if e["kind"]!="HeadStemRelation"||e["system_id"]!=head.system||removed_ids.contains(&(e["id"].as_u64().unwrap() as usize)){continue}
  let source=by_id.get(&(e["source"].as_u64().unwrap() as usize)).ok_or("missing_stem_cleaner_endpoint")?;let target=by_id.get(&(e["target"].as_u64().unwrap() as usize)).ok_or("missing_stem_cleaner_endpoint")?;
  let source:Key=serde_json::from_value(source["origin"].clone()).map_err(|_|"invalid_stem_cleaner_origin")?;let target:Key=serde_json::from_value(target["origin"].clone()).map_err(|_|"invalid_stem_cleaner_origin")?;
  attachments.push(HeadStemAttachment{local:e["local_relation_id"].as_u64().ok_or("invalid_stem_cleaner_relation_id")? as usize,head:source,stem_local:target.local,relation:e.clone()});
 }
 let mut next=graph.clone();next.sync_stems(head.system,&records,&attachments)?;*graph=next;
 Ok(json!({"removed_relation_ids":removed,"selected_cleaner_complete":true,"automatic_parts_selection_complete":false,"stems_complete":false,"heads_complete":false,"epilog_complete":false,"shared_sig":graph.snapshot()}))
}
