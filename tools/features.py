#!/usr/bin/env python3
"""Per-feature precision / recall of score markings against a reference.

Usage: features.py REFERENCE(.mscx|.musicxml) OUTPUT.musicxml [OUTPUT2 ...]

Features are counted per part (bags of tokens, independent of measures):
dynamics, articulations, fermatas, ornaments, grace notes, ties, slurs,
hairpins (wedges), arpeggios, tremolos and lyric syllables.
"""
import sys
import xml.etree.ElementTree as ET
from collections import Counter

ART = {
    'staccato': 'staccato', 'accent': 'accent', 'tenuto': 'tenuto',
    'staccatissimo': 'staccatissimo', 'marcato': 'marcato', 'strong-accent': 'marcato',
}


def mscx_features(path):
    score = ET.parse(path).getroot().find('Score')
    staves = {s.get('id'): s for s in score.findall('Staff')}
    parts = []
    for part in score.findall('Part'):
        c = Counter()
        for st in part.findall('Staff'):
            for el in staves[st.get('id')].iter():
                t = el.tag
                if t == 'Dynamic':
                    c['dyn:' + (el.findtext('subtype') or '')] += 1
                elif t == 'Articulation':
                    sub = (el.findtext('subtype') or '').lower()
                    if sub.startswith('ornament'):
                        c['orn:' + ('trill' if 'trill' in sub else 'mordent' if 'mordent' in sub else 'turn' if 'turn' in sub else sub)] += 1
                        continue
                    for k, v in [('staccatissimo', 'staccatissimo'), ('staccato', 'staccato'), ('accent', 'accent'),
                                 ('tenuto', 'tenuto'), ('marcato', 'marcato')]:
                        if k in sub and not (k == 'staccato' and 'staccatissimo' in sub):
                            c['art:' + v] += 1
                elif t == 'Fermata':
                    c['fermata'] += 1
                elif t == 'Ornament':
                    sub = (el.findtext('subtype') or '').lower()
                    c['orn:' + ('trill' if 'trill' in sub else 'mordent' if 'mordent' in sub else 'turn' if 'turn' in sub else sub)] += 1
                elif t in ('acciaccatura', 'appoggiatura', 'grace4', 'grace16', 'grace32'):
                    c['grace'] += 1
                elif t == 'Spanner' and el.find('next') is not None:
                    kind = el.get('type')
                    if kind == 'Tie':
                        c['tie'] += 1
                    elif kind == 'Slur':
                        c['slur'] += 1
                    elif kind == 'HairPin':
                        sub = el.find('HairPin').findtext('subtype') if el.find('HairPin') is not None else None
                        if sub in ('0', '1'):
                            c['wedge:' + ('cresc' if sub == '0' else 'dim')] += 1
                elif t == 'Arpeggio':
                    c['arpeggio'] += 1
                elif t == 'Tremolo':
                    c['tremolo'] += 1
                elif t == 'Lyrics':
                    c['lyric:' + (el.findtext('text') or '').strip().lower()] += 1
        parts.append(c)
    return parts


def xml_features(path):
    root = ET.parse(path).getroot()
    parts = []
    for part in root.findall('part'):
        c = Counter()
        for el in part.iter():
            t = el.tag
            if t == 'dynamics':
                for d in el:
                    c['dyn:' + d.tag] += 1
            elif t == 'articulations':
                for a in el:
                    if a.tag in ART:
                        c['art:' + ART[a.tag]] += 1
            elif t == 'fermata':
                c['fermata'] += 1
            elif t == 'ornaments':
                for o in el:
                    tag = o.tag
                    if 'trill' in tag:
                        c['orn:trill'] += 1
                    elif 'mordent' in tag:
                        c['orn:mordent'] += 1
                    elif 'turn' in tag:
                        c['orn:turn'] += 1
                    elif tag == 'tremolo':
                        c['tremolo'] += 1
            elif t == 'grace':
                c['grace'] += 1
            elif t == 'tied' and el.get('type') == 'start':
                c['tie'] += 1
            elif t == 'slur' and el.get('type') == 'start':
                c['slur'] += 1
            elif t == 'wedge' and el.get('type') in ('crescendo', 'diminuendo'):
                c['wedge:' + ('cresc' if el.get('type') == 'crescendo' else 'dim')] += 1
            elif t == 'arpeggiate':
                c['arpeggio'] += 1
            elif t == 'lyric':
                c['lyric:' + (el.findtext('text') or '').strip().lower()] += 1
        parts.append(c)
    return parts


def load(p):
    return mscx_features(p) if p.endswith('.mscx') else xml_features(p)


def family(token):
    return token.split(':')[0]


def main():
    ref = load(sys.argv[1])
    out = []
    for p in sys.argv[2:]:
        for i, c in enumerate(load(p)):
            if i < len(out):
                out[i] += c
            else:
                out.append(c)
    stats = {}
    for i in range(max(len(ref), len(out))):
        r = ref[i] if i < len(ref) else Counter()
        o = out[i] if i < len(out) else Counter()
        for tok in set(r) | set(o):
            f = family(tok)
            s = stats.setdefault(f, [0, 0, 0])  # matched, ref, out
            s[0] += min(r[tok], o[tok])
            s[1] += r[tok]
            s[2] += o[tok]
    for f in sorted(stats):
        m, r, o = stats[f]
        print(f"FEAT {f:10s} ref {r:5d} out {o:5d} match {m:5d} recall {m / r if r else 0:.3f} precision {m / o if o else 0:.3f}")


main()
