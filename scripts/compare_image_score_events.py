#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Development-only musical event comparison against an independent MuseScore3 score.
This does not provide notes to the native recognizer. Voice/notation acceptance follows separately.
"""
import argparse
from collections import Counter
from fractions import Fraction
import hashlib,json
from pathlib import Path
import xml.etree.ElementTree as ET
DURATIONS={'long':Fraction(4),'breve':Fraction(2),'whole':Fraction(1),'half':Fraction(1,2),'quarter':Fraction(1,4),'eighth':Fraction(1,8),'16th':Fraction(1,16),'32nd':Fraction(1,32),'64th':Fraction(1,64),'128th':Fraction(1,128)}
def truth_events(path):
    root=ET.parse(path).getroot()
    if root.tag!='museScore' or not root.attrib.get('version','').startswith('3.'):
        raise ValueError('independent MuseScore3 truth required')
    score=root.find('Score');events=[];meters={};voices={}
    for staff in score.findall('Staff'):
        sid=int(staff.attrib['id']);meter=None
        for mi,measure in enumerate(staff.findall('Measure'),1):
            for vi,voice in enumerate(measure.findall('voice'),1):
                clock=Fraction(0)
                for item in voice:
                    if item.tag=='TimeSig':meter=(int(item.findtext('sigN')),int(item.findtext('sigD')))
                    if item.tag not in ('Chord','Rest'):continue
                    if item.find('Tuplet') is not None:raise ValueError('truth tuplet interpretation pending')
                    duration_type=item.findtext('durationType')
                    if duration_type=='measure':
                        if meter is None:raise ValueError('truth whole-rest meter required')
                        duration=Fraction(*meter)
                    else:duration=DURATIONS[duration_type]
                    dots=int(item.findtext('dots','0'));duration*=sum((Fraction(1,2**i) for i in range(dots+1)),Fraction())
                    grace=any(item.find(tag) is not None for tag in ('appoggiatura','acciaccatura','grace4','grace16','grace32'))
                    if grace:duration=Fraction(0)
                    pitches=[int(n.findtext('pitch')) for n in item.findall('Note')] if item.tag=='Chord' else [None]
                    if not pitches:raise ValueError('truth chord without notes')
                    for pitch in pitches:events.append((sid,mi,str(clock),pitch,str(duration),grace));voices.setdefault(sid,set()).add(vi)
                    if not grace:clock+=duration
                if meter is not None:meters[(sid,mi)]=meter
    return events,meters,{str(k):len(v) for k,v in voices.items()}
def xml_events(path):
    root=ET.parse(path).getroot()
    if root.tag!='score-partwise':raise ValueError('MusicXML score-partwise required')
    events=[];meters={};voices={};base=0
    semitones={'C':0,'D':2,'E':4,'F':5,'G':7,'A':9,'B':11}
    for part in root.findall('part'):
        divisions=None;meter=None;staff_count=1
        for mi,measure in enumerate(part.findall('measure'),1):
            clock=Fraction(0);last_onset=None
            for item in measure:
                if item.tag=='attributes':
                    if item.findtext('divisions') is not None:divisions=int(item.findtext('divisions'))
                    if item.findtext('staves') is not None:staff_count=int(item.findtext('staves'))
                    time=item.find('time')
                    if time is not None:meter=(int(time.findtext('beats')),int(time.findtext('beat-type')))
                elif item.tag in ('backup','forward'):
                    if not divisions:raise ValueError('MusicXML divisions required')
                    delta=Fraction(int(item.findtext('duration')),4*divisions);clock+=delta if item.tag=='forward' else -delta
                    if clock<0:raise ValueError('MusicXML negative measure position')
                elif item.tag=='note':
                    if not divisions:raise ValueError('MusicXML divisions required')
                    staff=int(item.findtext('staff','1'));staff_count=max(staff_count,staff);sid=base+staff
                    grace=item.find('grace') is not None
                    duration=Fraction(0) if grace else Fraction(int(item.findtext('duration')),4*divisions)
                    if item.find('chord') is None:last_onset=clock
                    if last_onset is None:raise ValueError('MusicXML chord without lead note')
                    pitch=item.find('pitch')
                    midi=None if pitch is None else (int(pitch.findtext('octave'))+1)*12+semitones[pitch.findtext('step')]+int(pitch.findtext('alter','0'))
                    if midi is None and item.find('rest') is None:raise ValueError('unpitched MusicXML comparison pending')
                    events.append((sid,mi,str(last_onset),midi,str(duration),grace));voices.setdefault(sid,set()).add(item.findtext('voice','1'))
                    if item.find('chord') is None and not grace:clock+=duration
            if meter is not None:
                for staff in range(1,staff_count+1):meters[(base+staff,mi)]=meter
        base+=staff_count
    return events,meters,{str(k):len(v) for k,v in voices.items()}
def compare(truth,recognized):
    expected,tm,tv=truth_events(truth);actual,am,av=xml_events(recognized);a=Counter(actual);e=Counter(expected);matched=sum((a&e).values());precision=matched/len(actual) if actual else 0.;recall=matched/len(expected) if expected else 0.
    return dict(truth_sha256=hashlib.sha256(truth.read_bytes()).hexdigest(),recognized_sha256=hashlib.sha256(recognized.read_bytes()).hexdigest(),scope='exact staff/measure/onset/MIDI/duration/grace multiset; voice continuity, chord grouping and notation relations not yet scored',truth_events=len(expected),recognized_events=len(actual),matched_events=matched,precision=precision,recall=recall,F1=2*precision*recall/(precision+recall) if precision+recall else 0.,missing_events=list((e-a).elements()),extra_events=list((a-e).elements()),meter_matches=tm==am,truth_voice_counts=tv,recognized_voice_counts=av,quality_accepted=False)
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('truth',type=Path);p.add_argument('recognized',type=Path);p.add_argument('output',type=Path);a=p.parse_args();v=compare(a.truth,a.recognized)
    with a.output.open('x') as f:json.dump(v,f,indent=2);f.write('\n')
    print(json.dumps({k:v[k] for k in ('truth_events','recognized_events','matched_events','precision','recall','F1','meter_matches','quality_accepted')}))
if __name__=='__main__':main()
