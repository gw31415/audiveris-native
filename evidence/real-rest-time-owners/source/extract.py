# SPDX-License-Identifier: AGPL-3.0-or-later
"""Project declared original observations; never recognition input."""
import hashlib,json,pathlib
out=pathlib.Path(__file__).parent
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def semantic(s,p):
    ch=s['chord']; voice=s['voice']; relation=s['containment']
    return {
        'rest':{'removed':s['rest']['removed'],'registered':s['rest']['registered'],'manual':s['rest']['manual']},
        'chord':{'removed':ch['removed'],'registered':ch['registered'],'manual':ch['manual'],'time_offset':ch['time_offset'],'member_count':len(ch['registered_member_objects']),
            'same_identity':ch['object']==p['chord']['object'],'same_bounds':ch['bounds']==p['chord']['bounds'],'same_bounds_object':ch['bounds_object']==p['chord']['bounds_object'],
            'same_staff':ch['raw_staff_object']==p['chord']['raw_staff_object'],'same_raw_part':ch['raw_part_object']==p['chord']['raw_part_object'],'same_measure':ch['measure_object']==p['chord']['measure_object'],
            'same_voice':ch['voice_object']==p['voice']['object'],'same_slot':ch['slot_object']==p['slot']['object'],'same_time_offset_object':ch['time_offset_object']==p['chord']['time_offset_object']},
        'staff_contains_rest':s['staff']['contains_rest'],
        'measure_contains_chord':s['measure']['contains_chord'],
        'measure_contains_voice':voice['object'] in s['measure']['voice_objects'],
        'same_actual_owner_chain':all(s[k]['object']==p[k]['object']for k in ('staff','part','measure','stack')) and s['staff']['part_object']==s['part']['object'] and s['measure']['part_object']==s['part']['object'] and s['measure']['stack_object']==s['stack']['object'] and s['part']['contains_measure'] and s['stack']['contains_measure'],
        'expected_duration':s['stack']['expected_duration'],'same_expected_object':s['stack']['expected_duration_object']==p['stack']['expected_duration_object'],'stack_excess':s['stack']['excess'],
        'voice':{'same_identity':voice['object']==p['voice']['object'],'id':voice['id'],'same_measure':voice['measure_object']==p['voice']['measure_object'],'chord_count':len(voice['chord_objects']),
            'slot_count':len(voice['slot_entries']),'slot_entries_unchanged':voice['slot_entries']==p['voice']['slot_entries'],'same_slot_table':voice['slot_table_object']==p['voice']['slot_table_object'],
            'termination':voice['termination'],'same_termination_object':voice['termination_object']==p['voice']['termination_object'],'excess':voice['excess'],'same_excess_object':voice['excess_object']==p['voice']['excess_object']},
        'slot':{'same_identity':s['slot']['object']==p['slot']['object'],'time_offset':s['slot']['time_offset'],'same_time_offset_object':s['slot']['time_offset_object']==p['slot']['time_offset_object'],'same_stack':s['slot']['stack_object']==p['slot']['stack_object'],'incoming_same_chord':s['slot']['incoming_objects']==[ch['object']],'retained_in_stack':s['slot']['object']in s['stack']['slot_objects']},
        'containment':{'registered':relation['registered'],'manual':relation['manual'],'same_identity':relation['object']==p['containment']['object']}}
def phase(e,p):
    if e['hook']=='sheet/Staff.removeNote.return':return 'StaffRemoved','RestInter'
    if e['hook']=='sheet/rhythm/Measure.removeInter.return':return 'MeasureRemoved','RestChordInter'
    if e['hook']=='GraphListener.edgeRemoved':return 'EdgeRemoved','RestChordInter'
    if e['hook']=='GraphListener.vertexRemoved':return 'VertexRemoved','RestChordInter'if e['vertex_object']==p['chord']['object']else'RestInter'
    return None
controls=[]
for name in ('underfill','excess'):
    path=out/(name+'-observed.json'); d=json.loads(path.read_text());p=d['prepared'];assert not d['errors']
    expected_termination='-25/64' if name=='underfill' else '7/64'
    all_states=[p]+[e['state']for e in d['events']]+[d['after']]
    for s in all_states:
        q=semantic(s,p)
        assert q['chord']['time_offset']=='7/64' and q['voice']['termination']==expected_termination
        assert q['voice']['excess']==(None if name=='underfill' else'7/64')
        assert all(v for k,v in q['chord'].items()if k.startswith('same_'))
        assert all(v for k,v in q['voice'].items()if k.startswith('same_')) and q['voice']['slot_entries_unchanged']
        assert all(q['slot'][k]for k in ('same_identity','same_time_offset_object','same_stack','incoming_same_chord','retained_in_stack'))
        assert q['same_actual_owner_chain'] and q['same_expected_object'] and q['containment']['same_identity']
        assert q['rest']['manual'] and q['chord']['manual'] and q['containment']['manual']
    events=[]
    for e in d['events']:
        ph=phase(e,p)
        if ph:events.append({'phase':ph[0],'subject_kind':ph[1],'source_hook':e['hook'],'state':semantic(e['state'],p)})
    assert [(e['phase'],e['subject_kind'])for e in events]==[('StaffRemoved','RestInter'),('MeasureRemoved','RestChordInter'),('EdgeRemoved','RestChordInter'),('VertexRemoved','RestChordInter'),('VertexRemoved','RestInter')]
    callback=[e for e in d['events']if e['hook'].startswith('sig/relation/Containment.removed.')]
    assert len(callback)==2 and callback[0]['state']==callback[1]['state']
    assert not any(e['hook'].startswith('sig/inter/AbstractChordInter.invalidateCache')for e in d['events'])
    controls.append({'name':name,'raw_observation_path':str(path),'raw_observation_sha256':sha(path),'original_event_count':len(d['events']),'all_phase_clock_manual_and_owner_invariants':True,
        'initial_raw_rest_part':d['pre_declaration']['rest']['raw_part_object'],'initial_raw_chord_part':d['pre_declaration']['chord']['raw_part_object'],
        'declared_expected':p['stack']['expected_duration'],'prepared':semantic(p,p),'events':events,'after':semantic(d['after'],p),
        'containment_callback_entry_equals_return':True,'chord_cache_invalidation_during_removal':False,
        'all_source_hooks':[e['hook']for e in d['events']]})
result={'source_pin':'9e1e55cd2746037d059345881c53e6a6754bffbd','scope':'Two declared original singleton quarter-rest extensive-removal controls on reloaded actual source owners; no fresh image/ordinary LINKS-clock/Java-rollback claim. Source coordinates select the test-owned original object only; never production selectors. Identity comparisons are intra-run.',
    'selected':{'source_system':2,'source_rest':4436,'source_chord':4485,'shape':'QUARTER_REST','bounds':[1122,879,10,32],'staff':4,'part':1},'controls':controls}
(out/'summary.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({'controls':len(controls),'events_per_control':[c['original_event_count']for c in controls],'summary_bytes':(out/'summary.json').stat().st_size,'all_checks_passed':True}))
