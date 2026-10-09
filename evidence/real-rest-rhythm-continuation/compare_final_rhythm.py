#!/usr/bin/env python3
"""Compare saved original rhythm XML references with an actual native diagnostic; never feed recognition."""
import argparse,collections,hashlib,json
from pathlib import Path
import xml.etree.ElementTree as ET
import zipfile


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def token(system, part, measure, kind, staff, bounds):
    return [system,part,measure,kind,staff,*bounds]


def ordered_tokens(items):
    return sorted(items, key=lambda x:json.dumps(x,separators=(',',':')))


def source_book(path):
    with zipfile.ZipFile(path) as book:
        member='sheet#1/sheet#1.xml'
        raw=book.read(member)
    root=ET.fromstring(raw)
    stacks=[];measures=[];voices=[];chords=[];lookup={};incoming=collections.defaultdict(list)
    for system in root.findall('.//system'):
        sid=int(system.attrib['id']);elements=system.findall('stack')
        inters={int(e.attrib['id']):e for e in system.iter() if e.tag in ('head-chord','rest-chord')}
        for part in system.findall('part'):
            pid=int(part.attrib['id']);owned=part.findall('measure')
            if len(owned)!=len(elements):raise ValueError('source Part/Stack measure cardinality differs')
            for mi,measure in enumerate(owned,1):
                stack=elements[mi-1]
                if measure.get('id')!=stack.get('id'):raise ValueError('source positional Measure/Stack printed ID differs')
                for tag,kind in [('head-chords','HeadChordInter'),('rest-chords','RestChordInter')]:
                    for item in (measure.findtext(tag) or '').split():
                        cid=int(item);chord=inters[cid];b=chord.find('bounds')
                        key=token(sid,pid,mi,kind,int(chord.attrib['staff']),[int(b.attrib[k]) for k in ('x','y','w','h')])
                        if cid in lookup:raise ValueError('source Chord belongs to multiple serialized Measures')
                        lookup[cid]=key
                measures.append({'system':sid,'part':pid,'measure_index':mi,'expected':stack.get('expected'),'actual':stack.get('duration'),'excess':stack.get('excess'),'abnormal':measure.get('abnormal','false')=='true'})
        for mi,stack in enumerate(elements,1):
            slots=[{'id':int(s.attrib['id']),'time_offset':s.get('time-offset'),'x_offset':int(s.attrib['x-offset'])} for s in stack.findall('slot')]
            stacks.append({'system':sid,'measure_index':mi,'expected':stack.get('expected'),'actual':stack.get('duration'),'excess':stack.get('excess'),'slots':slots,'source_printed_id':stack.get('id'),'source_special':stack.get('special')})
        for part in system.findall('part'):
            pid=int(part.attrib['id'])
            for mi,measure in enumerate(part.findall('measure'),1):
                slot_times={int(s.attrib['id']):s.get('time-offset') for s in elements[mi-1].findall('slot')}
                for voice in measure.findall('voice'):
                    vid=int(voice.attrib['id']);entries=[];members=[]
                    rest=voice.get('measure-rest-chord',voice.get('whole-rest-chord'))
                    for entry in voice.findall('slots/entry'):
                        slot=int(entry.findtext('key'));value=entry.find('value');cid=int(value.attrib['chord']);key=lookup[cid]
                        if key[:3]!=[sid,pid,mi]:raise ValueError('source Voice references another Measure Chord')
                        entries.append({'slot_id':slot,'status':value.attrib['status'],'chord':key})
                        if value.attrib['status']=='BEGIN':
                            members.append(key)
                            incoming[(sid,mi,slot)].append(key)
                            chords.append({'chord':key,'voice_id':vid,'slot_id':slot,'time_offset':slot_times[slot]})
                    if len({json.dumps(k) for k in members})!=len(members):raise ValueError('source Chord has repeated BEGIN entries')
                    voices.append({'system':sid,'part':pid,'measure_index':mi,'id':vid,'excess':voice.get('excess'),'is_measure_rest':rest is not None,'measure_rest_chord':lookup[int(rest)] if rest else None,'chord_members':ordered_tokens(members),'slots':entries})
    for stack in stacks:
        for slot in stack['slots']:
            slot['incoming_chords']=ordered_tokens(incoming[(stack['system'],stack['measure_index'],slot['id'])])
    for measure in measures:
        stack=next(s for s in stacks if (s['system'],s['measure_index'])==(measure['system'],measure['measure_index']))
        measure['slots']=stack['slots']
    identities=list(lookup.values())
    if len({json.dumps(k) for k in identities})!=len(identities):raise ValueError('source bounds identity is ambiguous')
    return {'stacks':stacks,'measures':measures,'voices':voices,'chord_times':chords,'chord_membership':ordered_tokens(identities)},hashlib.sha256(raw).hexdigest()


def native_state(document):
    if document.get('failed_stage')=='ordinary_native_page_rhythm':raise ValueError('native ordinary rhythm failed; after_rhythm is rollback, not completed rhythm')
    state=document['after_rhythm'];graph=state['graph'];lookup={}
    for chord in graph['inters']:
        if chord['kind'] not in ('HeadChordInter','RestChordInter'):continue
        lookup[chord['id']]=token(chord['origin']['system'],chord['part_id'],chord['measure_index'],chord['kind'],chord['staff_id'],chord['bounds'])
    identities=list(lookup.values())
    if len({json.dumps(k) for k in identities})!=len(identities):raise ValueError('native bounds identity is ambiguous')
    measures=[];voices=[];stacks=[];clocks=[]
    stack_keys=[]
    for measure in state['measures']:
        key=(measure['system'],measure['index'])
        if key not in stack_keys:stack_keys.append(key)
    if len(stack_keys)!=len(state['stacks']):raise ValueError('native Stack/Measure scope count mismatch')
    def slots(values):
        return [{'id':s['id'],'time_offset':s['time_offset'],'x_offset':s['x_offset'],'incoming_chords':ordered_tokens([lookup[c] for c in s['incoming_chord_ids']])} for s in values]
    for key,stack in zip(stack_keys,state['stacks']):
        if stack is None:raise ValueError('native stack rhythm not completed')
        stacks.append({'system':key[0],'measure_index':key[1],'expected':stack['expected'],'actual':stack['actual'],'excess':stack['excess'],'slots':slots(stack['slots'])})
    for measure in state['measures']:
        measures.append({'system':measure['system'],'part':measure['part'],'measure_index':measure['index'],'expected':measure['expected'],'actual':measure['actual'],'excess':measure['excess'],'abnormal':measure['abnormal'],'slots':slots(measure['slots'])})
    for voice in state['voices']:
        rest=voice.get('measure_rest_chord_id')
        voices.append({'system':voice['system_id'],'part':voice['part_id'],'measure_index':voice['measure_index'],'id':voice['id'],'excess':voice['excess'],'is_measure_rest':voice['is_measure_rest'],'measure_rest_chord':lookup[rest] if rest is not None else None,'chord_members':ordered_tokens([lookup[c] for c in voice['chord_ids']]),'slots':[{'slot_id':s['slot_id'],'status':s['status'],'chord':lookup[s['chord_id']]} for s in voice['slots']]})
    for chord in graph['inters']:
        if chord['id'] in lookup:
            clocks.append({'chord':lookup[chord['id']],'voice_id':chord.get('voice'),'slot_id':chord.get('slot_id'),'time_offset':chord.get('time_offset')})
    return {'stacks':stacks,'measures':measures,'voices':voices,'chord_times':clocks,'chord_membership':ordered_tokens(identities)}


def compare(source,native):
    result={};spec={'stacks':['system','measure_index'],'measures':['system','part','measure_index'],'voices':['system','part','measure_index','id'],'chord_times':['chord']}
    for section,keys in spec.items():
        def index(rows):
            values={}
            for row in rows:
                key=json.dumps([row[k] for k in keys],separators=(',',':'))
                if key in values:raise ValueError('duplicate '+section+' comparison key '+key)
                values[key]=row
            return values
        expected=index(source[section]);actual=index(native[section]);diff=[];matched=0
        for key in sorted(expected.keys()|actual.keys()):
            a=expected.get(key);b=actual.get(key)
            if a is None or b is None:
                diff.append({'key':json.loads(key),'source':a,'native':b});continue
            fields={k:{'source':v,'native':b.get(k)} for k,v in a.items() if not k.startswith('source_') and b.get(k)!=v}
            if fields:diff.append({'key':json.loads(key),'fields':fields})
            else:matched+=1
        result[section]={'source_count':len(expected),'native_count':len(actual),'matched':matched,'differences':diff}
    result['chord_membership']={'source_count':len(source['chord_membership']),'native_count':len(native['chord_membership']),'equal':source['chord_membership']==native['chord_membership']}
    return result


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--original-book',type=Path,default=Path('/tmp/real-rest-original-v1/der-winter-gray.omr'))
    parser.add_argument('--native',type=Path)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args();source,member_hash=source_book(args.original_book)
    result={'schema':'audiveris-native.saved-original-final-rhythm-comparison.v1','source_book_sha256':digest(args.original_book),'source_sheet_member_sha256':member_hash,'helper_sha256':digest(Path(__file__)),'source':source,
      'qualifications':['Offline test-only comparison. No original observations are supplied to native recognition or used to rebuild runtime owners.',
      'Source Chord identities use actual serialized Measure membership and shape-independent kind/Staff/bounds. Cross-run numeric Inter IDs are never joined. Ambiguous geometry is rejected.',
      'Source Chord times/Voice IDs/shared Slot IDs are read through persisted Voice BEGIN-to-Stack-Slot references, not fresh original getters. BEGIN and CONTINUE are compared separately.',
      'Source Measure expected/actual/excess and shared slots come from its serialized Stack association. Printed IDs may be pickup-renumbered; comparison uses physical System-local index.',
      'Native diagnostic Stack entries lack owner scope fields. Their array order is associated with first-encounter unique (system,index) in actual native Measure diagnostic; no production owner is changed.',
      'Incoming Chord membership and Voice member identities are compared as sets; serialized source does not preserve all original runtime encounter order.',
      'Transient source Voice termination/kind/startingStaff, exact Chord duration/end getter values, callback identity and automatic recognition quality are not proven by the saved book. Native values must not be used to fill missing source fields.'],
      'source_runtime_fields_unavailable':['Voice.termination','Voice.kind','Voice.startingStaff','Chord.duration','Chord.endTime','Java runtime object identities'],
      'used_as_native_recognition_input':False,'quality_parity_claimed':False}
    status=0
    if args.native:
        result['native_artifact_sha256']=digest(args.native)
        document=json.loads(args.native.read_text())
        try:
            native=native_state(document);result['native']=native;result['comparison']=compare(source,native)
            result['compared']=True
            result['all_serialized_contracts_equal']=all(not result['comparison'][s]['differences'] for s in ['stacks','measures','voices','chord_times']) and result['comparison']['chord_membership']['equal']
            status=0 if result['all_serialized_contracts_equal'] else 1
        except (ValueError,KeyError) as error:
            result['compared']=False;result['comparison_blocker']=str(error);status=2
    args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({'output':str(args.output),'source_counts':{k:len(v) for k,v in source.items()},'compared':result.get('compared',False),'all_serialized_contracts_equal':result.get('all_serialized_contracts_equal'),'blocker':result.get('comparison_blocker')}))
    raise SystemExit(status)

if __name__=='__main__':main()
