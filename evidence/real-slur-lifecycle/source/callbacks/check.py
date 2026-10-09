# SPDX-License-Identifier: AGPL-3.0-or-later
"""Assert recorded original observable contracts; no native implementation is executed."""
from pathlib import Path
import json,collections,hashlib,difflib
p=Path(__file__).parent;x=json.loads((p/'observed.json').read_text());events=x['events'];checks=[]
def check(name,ok):
 checks.append({'contract':name,'ok':bool(ok)})
 if not ok:raise AssertionError(name)
check('No Slur existed before fresh CURVES',x['before_slurs']==[])
check('Six final original Slurs',len(x['after_slurs'])==6)
check('No observer failures',x['observer_errors']==[])
check('Actual prior Chord identity, cached bounds, clocks, expectedDuration and owner rows unchanged',x['before_chord_owners']==x['after_chord_owners'])
held={};sequence=['sheet/curve/ClumpPruner$ClumpLinker.doLink.entry','sig/inter/SlurInter.added.entry','sig/inter/SlurInter.added.return','sig/relation/SlurHeadRelation.added.entry','sig/relation/SlurHeadRelation.added.return','sig/relation/SlurHeadRelation.added.entry','sig/relation/SlurHeadRelation.added.return','sheet/curve/ClumpPruner$ClumpLinker.doLink.return']
for k in range(6):
 es=events[8*k:8*k+8];label=es[0]['slur']['object'];check(f'{label} exact fresh vertex/LEFT/RIGHT callback order',[e['hook'] for e in es]==sequence)
 check(f'{label} one actual Slur object through pending and inserted states',all(e['slur']['object']==label for e in es))
 check(f'{label} abnormal false,true,true,false changes at original returns',[e['slur']['abnormal'] for e in [es[0],es[2],es[4],es[6]]]==[False,True,True,False])
 check(f'{label} no Part cache or membership during vertex/edge callbacks',all(e['slur']['part_raw'] is None and e['slur']['actual_part_memberships']==[] for e in es))
 check(f'{label} original callbacks enter after edge insertion',[len(es[i]['slur']['live_head_relations']) for i in [3,5]]==[1,2])
 check(f'{label} explicit LEFT then RIGHT sides',[es[i]['relation']['side'] for i in [3,5]]==['LEFT','RIGHT'])
 check(f'{label} both original pending relation objects reused',all(es[i]['event_edge_is_receiver'] and es[i]['same_pending_relation'] for i in [3,4,5,6]))
 check(f'{label} original relation identity retained at doLink return',[a['relation']['object']for a in es[0]['pending_links']]==[a['object']for a in es[-1]['slur']['live_head_relations']])
 check(f'{label} grade/ratios/manual/nontie contracts',all(r['grade']==1.0 and r['source_ratio']==6.0 and r['target_ratio']==1.0 and not r['manual']for r in es[-1]['slur']['live_head_relations']) and all(not e['slur']['manual'] and not e['slur']['tie']for e in es))
 held[label]=es[0]
check('All clump vertex/edge lifecycles precede dispatch entry',events[48]['hook']=='sheet/curve/SlursBuilder.dispatchToParts.entry')
dispatch_order=[s['object']for s in events[48]['all_slurs']]
for k,label in enumerate(dispatch_order):
 es=events[49+4*k:53+4*k];check(f'{label} Part add precedes setPart',[e['hook']for e in es]==['sheet/Part.addSlur.entry','sheet/Part.addSlur.return','sig/inter/AbstractInter.setPart.entry','sig/inter/AbstractInter.setPart.return'])
 check(f'{label} dispatch source system/SIG encounter order',all(e['slur']['object']==label for e in es))
 check(f'{label} membership exists before cache assignment',[len(e['slur']['actual_part_memberships'])for e in es]==[0,1,1,1] and [e['slur']['part_raw'] is None for e in es]==[True,True,True,False])
 check(f'{label} first actual endpoint Part is both collection and cache owner',es[1]['receiver']==es[3]['slur']['part_raw']==es[0]['slur']['live_head_relations'][0]['target']['part'])
 check(f'{label} same Slur object once in actual Part collection',es[3]['slur']['actual_part_memberships'][0]['identity_occurrences']==1)
check('Dispatch return follows all Parts',events[73]['hook']=='sheet/curve/SlursBuilder.dispatchToParts.return' and len(events)==74)
check('All final relations are the originally selected objects',all(s['live_head_relations']==next(e['slur']['live_head_relations']for e in reversed(events)if e.get('slur',{}).get('object')==s['object'])for s in x['after_slurs']))
prov=json.loads((p/'PROVENANCE.json').read_text());check('Pre-CURVES Book unchanged',prov['book_sha256_before']==prov['book_sha256_after'])
audit=json.loads((p/'instrumentation-audit.json').read_text());check('All original bodies retained in executed overlays',audit['differences']==0 and audit['total_inserted_observation_calls']==12)
summary={'scope':'Original runtime contract assertions, not native parity assertions. One fresh CURVES stage on persisted CHORDS checkpoint, not fresh upstream recognition. Manual/tie/orphan branches absent. Chord timing fields are null in this fresh stage.','fresh_slurs':6,'actual_pending_relations_reused':12,'observed_events':len(events),'chord_owners_unchanged':len(x['before_chord_owners']),'contract_checks':len(checks),'failed_checks':0,'checks':checks,'construction_order':[events[i]['slur']['id']for i in range(7,48,8)],'dispatch_order':[s['id']for s in events[48]['all_slurs']],'native_integration_compared':False,'automatic_stage_only':True,'upstream_checkpoint_book_sha256':prov['book_sha256_before']}
(p/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
base=Path('/tmp/slur-native-hookup-unit/music_slurs.rs').read_text();old='''    let mut parts: Vec<PartSlurs> = Vec::new();
    for slur in &slurs {
        graph.set_fresh_slur_part(slur.key, slur.part.id)?;
        if let Some(p) = parts.iter_mut().find(|p| Arc::ptr_eq(&p.part, &slur.part)) {
            p.slurs.push(Arc::clone(slur));
        } else {
            parts.push(PartSlurs {
                part: Arc::clone(&slur.part),
                slurs: vec![Arc::clone(slur)],
            });
        }
        let actual = &registry.get(slur.glyph_id)?.glyph;'''
new='''    let mut parts: Vec<PartSlurs> = Vec::new();
    for system in &m.owner.systems {
        for slur in slurs.iter().filter(|slur| Arc::ptr_eq(&slur.system, system)) {
            // The source appends the same Slur while its cached Part is still null.
            if let Some(p) = parts.iter_mut().find(|p| Arc::ptr_eq(&p.part, &slur.part)) {
                p.slurs.push(Arc::clone(slur));
            } else {
                parts.push(PartSlurs {
                    part: Arc::clone(&slur.part),
                    slurs: vec![Arc::clone(slur)],
                });
            }
            graph.set_fresh_slur_part(slur.key, slur.part.id)?;
        }
    }
    for slur in &slurs {
        let actual = &registry.get(slur.glyph_id)?.glyph;'''
assert old in base
updated=base.replace(old,new);(p/'music_slurs-observed-dispatch.rs').write_text(updated)
(p/'E-part-dispatch-observed-order.patch').write_text(''.join(difflib.unified_diff(base.splitlines(True),updated.splitlines(True),fromfile='a/src/music_slurs.rs',tofile='b/src/music_slurs.rs')))
print(json.dumps({k:summary[k]for k in ['fresh_slurs','actual_pending_relations_reused','observed_events','chord_owners_unchanged','contract_checks','failed_checks','construction_order','dispatch_order']},indent=2))
