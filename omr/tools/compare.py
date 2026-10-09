#!/usr/bin/env python3
"""Compare two MusicXML files event by event (per part, measure, staff).

Events are (onset, pitch-or-rest, duration) in quarter-note units. Voice numbers
and layout are ignored. Prints a per-measure diff and an overall match score.
"""
import sys
import xml.etree.ElementTree as ET
from fractions import Fraction
from difflib import SequenceMatcher

STEP = {'C': 0, 'D': 2, 'E': 4, 'F': 5, 'G': 7, 'A': 9, 'B': 11}


def events(path):
    root = ET.parse(path).getroot()
    parts = []
    for part in root.findall('part'):
        div = 1
        measures = []
        for m in part.findall('measure'):
            t = Fraction(0)
            last_start = Fraction(0)
            evs = []
            for el in m:
                if el.tag == 'attributes':
                    d = el.find('divisions')
                    if d is not None:
                        div = int(d.text)
                elif el.tag == 'backup':
                    t -= Fraction(int(el.find('duration').text), div)
                elif el.tag == 'forward':
                    t += Fraction(int(el.find('duration').text), div)
                elif el.tag == 'note':
                    if el.find('grace') is not None:
                        continue
                    dur = Fraction(int(el.find('duration').text), div)
                    start = last_start if el.find('chord') is not None else t
                    staff = el.findtext('staff', '1')
                    if el.find('rest') is not None:
                        what = 'rest'
                    else:
                        p = el.find('pitch')
                        midi = 12 * (int(p.findtext('octave')) + 1) + STEP[p.findtext('step')] + int(float(p.findtext('alter', '0')))
                        what = midi
                    evs.append((staff, start, what, dur))
                    if el.find('chord') is None:
                        last_start = t
                        t += dur
            measures.append(sorted(evs, key=lambda e: (e[0], e[1], str(e[2]))))
        parts.append(measures)
    return parts


DUR = {'long': 16, 'breve': 8, 'whole': 4, 'half': 2, 'quarter': 1, 'eighth': Fraction(1, 2), '16th': Fraction(1, 4),
       '32nd': Fraction(1, 8), '64th': Fraction(1, 16), '128th': Fraction(1, 32)}
GRACE = {'appoggiatura', 'acciaccatura', 'grace4', 'grace16', 'grace32', 'grace8after', 'grace16after', 'grace32after'}


def mscx_events(path):
    """MuseScore 3/4 source: parts -> measures -> (staff-in-part, onset, midi|rest, duration)."""
    score = ET.parse(path).getroot().find('Score')
    staves = {s.get('id'): s for s in score.findall('Staff')}
    parts = []
    for part in score.findall('Part'):
        ids = [s.get('id') for s in part.findall('Staff')]
        measures = []
        for k, sid in enumerate(ids):
            for mi, m in enumerate(staves[sid].findall('Measure')):
                while len(measures) <= mi:
                    measures.append([])
                for voice in m.findall('voice'):
                    t = Fraction(0)
                    ratio = Fraction(1)
                    for el in voice:
                        if el.tag == 'Tuplet':
                            ratio = Fraction(int(el.findtext('normalNotes')), int(el.findtext('actualNotes')))
                        elif el.tag == 'endTuplet':
                            ratio = Fraction(1)
                        elif el.tag == 'location':
                            t += Fraction(el.findtext('fractions')) * 4
                        elif el.tag in ('Chord', 'Rest'):
                            if any(el.find(g) is not None for g in GRACE):
                                continue
                            dt = el.findtext('durationType')
                            if dt == 'measure':
                                d = Fraction(el.findtext('duration')) * 4
                            else:
                                d = Fraction(DUR[dt])
                                dots = int(el.findtext('dots', '0'))
                                d = d * (2 - Fraction(1, 2 ** dots)) * ratio
                            if el.tag == 'Rest':
                                measures[mi].append((str(k + 1), t, 'rest', d))
                            else:
                                for n in el.findall('Note'):
                                    measures[mi].append((str(k + 1), t, int(n.findtext('pitch')), d))
                            t += d
        parts.append([sorted(m, key=lambda e: (e[0], e[1], str(e[2]))) for m in measures])
    return parts


def load(path):
    return mscx_events(path) if path.endswith('.mscx') else events(path)


def name(e):
    staff, start, what, dur = e
    w = 'R' if what == 'rest' else ['C', 'C#', 'D', 'Eb', 'E', 'F', 'F#', 'G', 'Ab', 'A', 'Bb', 'B'][what % 12] + str(what // 12 - 1)
    return f"s{staff}@{float(start):g}:{w}/{float(dur):g}"


def main():
    a, b = load(sys.argv[1]), load(sys.argv[2])
    total = matched = 0
    for pi in range(max(len(a), len(b))):
        pa = a[pi] if pi < len(a) else []
        pb = b[pi] if pi < len(b) else []
        print(f"== part {pi + 1}: {len(pa)} vs {len(pb)} measures")
        for mi in range(max(len(pa), len(pb))):
            ea = [name(e) for e in (pa[mi] if mi < len(pa) else [])]
            eb = [name(e) for e in (pb[mi] if mi < len(pb) else [])]
            sm = SequenceMatcher(None, ea, eb, autojunk=False)
            m = sum(bl.size for bl in sm.get_matching_blocks())
            total += max(len(ea), len(eb))
            matched += m
            if ea != eb:
                print(f"  m{mi + 1}: only-ref {[e for e in ea if e not in eb]}  only-test {[e for e in eb if e not in ea]}")
    print(f"MATCH {matched}/{total} = {matched / max(total, 1):.3f}")


main()
