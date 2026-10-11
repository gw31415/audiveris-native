#!/usr/bin/env python3
"""Helpers for proofreading an audiveris-native MusicXML against the score image.

Standard library only (cropping uses Pillow when present, else ImageMagick).

  scoretool.py summary  SCORE.musicxml [--part P1] [--measures 3-8]
  scoretool.py check    SCORE.musicxml
  scoretool.py crops    DEBUG_DIR OUT_DIR IMAGE... [--split N] [--pad K]
  scoretool.py extract  SCORE.musicxml --measures 3-4 [--part P1]
  scoretool.py replace  SCORE.musicxml --part P1 --measure 3 FRAGMENT.xml [-o OUT]
  scoretool.py insert   SCORE.musicxml --part P1 --after 3 FRAGMENT.xml [-o OUT]
  scoretool.py delete   SCORE.musicxml --part P1 --measure 3 [-o OUT]
  scoretool.py split    SCORE.musicxml --measure 3 --at 2 [--part P1] [-o OUT]
  scoretool.py merge    SCORE.musicxml --measure 3 [--part P1] [-o OUT]
  scoretool.py renumber SCORE.musicxml [-o OUT]

Run "scoretool.py COMMAND -h" for details.
"""

import argparse
import json
import os
import re
import shutil
import signal
import subprocess
import sys
import textwrap
import xml.etree.ElementTree as ET
from fractions import Fraction

# ---------------------------------------------------------------- reading

TYPE_ABBR = {
    "maxima": "max", "long": "long", "breve": "brv", "whole": "w", "half": "h",
    "quarter": "q", "eighth": "8", "16th": "16", "32nd": "32", "64th": "64",
    "128th": "128", "256th": "256",
}
ALTER = {-2: "bb", -1: "b", 0: "", 1: "#", 2: "x"}
ACC = {
    "sharp": "#", "flat": "b", "natural": "n", "double-sharp": "x",
    "flat-flat": "bb", "sharp-sharp": "x", "natural-sharp": "n#", "natural-flat": "nb",
}
ARTIC = {
    "staccato": "stacc", "staccatissimo": "staccmo", "accent": "acc",
    "strong-accent": "marc", "tenuto": "ten", "detached-legato": "port",
    "spiccato": "spicc", "breath-mark": "breath", "caesura": "caes",
}


def parse_range(text):
    """'3-8' -> (3, 8), '5' -> (5, 5), None -> None."""
    if not text:
        return None
    a, _, b = text.partition("-")
    return int(a), int(b or a)


def measure_no(m):
    try:
        return int(re.match(r"-?\d+", m.get("number", "")).group())
    except (AttributeError, ValueError):
        return None


def in_range(m, rng):
    if rng is None:
        return True
    n = measure_no(m)
    return n is not None and rng[0] <= n <= rng[1]


def load(path):
    try:
        return ET.parse(path).getroot()
    except ET.ParseError as e:
        sys.exit(f"{path}: not well-formed XML: {e}")


def part_names(root):
    names = {}
    for sp in root.iter("score-part"):
        names[sp.get("id")] = (sp.findtext("part-name") or "").strip()
    return names


def note_token(n):
    if n.find("rest") is not None:
        r = n.find("rest")
        tok = "R" + ("(measure)" if r.get("measure") == "yes" else "")
        ds, do = r.findtext("display-step"), r.findtext("display-octave")
        if ds:
            tok += f"@{ds}{do}"
    elif n.find("unpitched") is not None:
        tok = "X"
    else:
        p = n.find("pitch")
        alter = int(round(float(p.findtext("alter") or 0)))
        tok = f"{p.findtext('step')}{ALTER.get(alter, '?')}{p.findtext('octave')}"
    acc = n.findtext("accidental")
    if acc:
        tok += "!" + ACC.get(acc, acc)
    if any(t.get("type") == "start" for t in n.findall("tie")):
        tok += "~"
    return tok


def note_suffix(n):
    t = n.findtext("type")
    s = TYPE_ABBR.get(t, t or "?")
    s += "." * len(n.findall("dot"))
    tm = n.find("time-modification")
    if tm is not None:
        s += f"[{tm.findtext('actual-notes')}:{tm.findtext('normal-notes')}]"
    extra = []
    nt = n.find("notations")
    if nt is not None:
        for sl in nt.findall("slur"):
            extra.append("(" if sl.get("type") == "start" else ")" if sl.get("type") == "stop" else "")
        for a in nt.findall("articulations/*"):
            extra.append("." + ARTIC.get(a.tag, a.tag))
        for o in nt.findall("ornaments/*"):
            if o.tag != "accidental-mark":
                extra.append("." + o.tag)
        if nt.find("fermata") is not None:
            extra.append(".ferm")
        if nt.find("arpeggiate") is not None:
            extra.append(".arp")
        for tu in nt.findall("tuplet"):
            extra.append("{" if tu.get("type") == "start" else "}")
        tr = nt.find("ornaments/tremolo")
        if tr is not None:
            extra.append(f".trem{tr.text}")
    for ly in n.findall("lyric"):
        txt = ly.findtext("text") or ""
        syl = ly.findtext("syllabic") or "single"
        if syl in ("begin", "middle"):
            txt += "-"
        if syl in ("middle", "end"):
            txt = "-" + txt
        extra.append(f'"{txt}"' if ly.get("number", "1") == "1" else f'"{ly.get("number")}:{txt}"')
    return s + "".join(extra)


def describe_attributes(a):
    out = []
    key = a.find("key")
    if key is not None:
        out.append(f"key={key.findtext('fifths')}")
    time = a.find("time")
    if time is not None:
        sym = time.get("symbol")
        out.append(f"time={time.findtext('beats')}/{time.findtext('beat-type')}"
                   + (f"({sym})" if sym else ""))
    st = a.findtext("staves")
    if st:
        out.append(f"staves={st}")
    for c in a.findall("clef"):
        oct_ = c.findtext("clef-octave-change")
        out.append(f"clef{c.get('number', '')}={c.findtext('sign')}{c.findtext('line') or ''}"
                   + (f"{int(oct_):+d}" if oct_ else ""))
    ms = a.find("measure-style/multiple-rest")
    if ms is not None:
        out.append(f"multirest={ms.text}")
    return out


def describe_direction(d):
    out = []
    for dt in d.findall("direction-type"):
        for c in dt:
            if c.tag == "dynamics":
                out.append("dyn:" + "".join(x.tag for x in c))
            elif c.tag == "words":
                out.append(f'words:"{(c.text or "").strip()}"')
            elif c.tag == "wedge":
                out.append(f"wedge:{c.get('type')}")
            elif c.tag == "octave-shift":
                out.append(f"8va:{c.get('type')}{c.get('size', '')}")
            elif c.tag == "pedal":
                out.append(f"ped:{c.get('type')}")
            elif c.tag in ("segno", "coda"):
                out.append(c.tag)
            elif c.tag == "metronome":
                out.append(f"metro:{c.findtext('beat-unit')}={c.findtext('per-minute')}")
            else:
                out.append(c.tag)
    snd = d.find("sound")
    if snd is not None and snd.attrib:
        out.append("sound:" + ",".join(f"{k}={v}" for k, v in snd.attrib.items()))
    st = d.findtext("staff")
    pl = d.get("placement")
    return " ".join(out) + (f" (staff{st})" if st else "") + (f" [{pl}]" if pl else "")


def cmd_summary(args):
    root = load(args.score)
    names = part_names(root)
    rng = parse_range(args.measures)
    title = root.findtext("work/work-title") or root.findtext("movement-title") or ""
    if not args.part and rng is None:
        print(f"title: {title}")
        for c in root.findall("identification/creator"):
            print(f"{c.get('type')}: {c.text}")
        print("parts: " + ", ".join(f"{k}={v!r}" for k, v in names.items()))
    for part in root.findall("part"):
        pid = part.get("id")
        if args.part and pid != args.part:
            continue
        system = 0
        for m in part.findall("measure"):
            if m.find("print[@new-system='yes']") is not None or system == 0:
                system += 1
            if not in_range(m, rng):
                continue
            head = [f"{pid} m{m.get('number')}", f"S{system}"]
            if m.get("implicit") == "yes":
                head.append("implicit")
            lines = []
            voices = {}
            order = []
            prev_key = None
            for el in m:
                if el.tag == "attributes":
                    head += describe_attributes(el)
                elif el.tag == "direction":
                    lines.append("  dir " + describe_direction(el))
                elif el.tag == "barline":
                    bits = [el.get("location", "right")]
                    if el.findtext("bar-style"):
                        bits.append(el.findtext("bar-style"))
                    r = el.find("repeat")
                    if r is not None:
                        bits.append(f"repeat-{r.get('direction')}")
                    e = el.find("ending")
                    if e is not None:
                        bits.append(f"ending{e.get('number')}-{e.get('type')}")
                    lines.append("  bar " + " ".join(bits))
                elif el.tag == "note":
                    key = (el.findtext("staff") or "1", el.findtext("voice") or "1")
                    if key not in voices:
                        voices[key] = []
                        order.append(key)
                    tok = note_token(el)
                    suf = note_suffix(el)
                    grace = el.find("grace") is not None
                    if el.find("chord") is not None and voices[key] and prev_key == key:
                        voices[key][-1][0].append(tok)
                    else:
                        voices[key].append([[tok], suf, grace])
                    prev_key = key
            print(" ".join(head))
            for key in order:
                toks = []
                for pitches, suf, grace in voices[key]:
                    body = "+".join(pitches)
                    if len(pitches) > 1:
                        body = f"<{body}>"
                    toks.append(("g:" if grace else "") + body + ":" + suf)
                print(f"  st{key[0]} v{key[1]}: " + " ".join(toks))
            for line in lines:
                print(line)


# ---------------------------------------------------------------- checking

def cmd_check(args):
    root = load(args.score)
    problems = []
    counts = {}
    first_time = None
    times = {}  # part -> [(measure number, "beats/beat-type")]
    for part in root.findall("part"):
        pid = part.get("id")
        measures = part.findall("measure")
        counts[pid] = len(measures)
        divisions, beats, beat_type = None, None, None
        if measures and measures[0].find("attributes/time") is None:
            problems.append(f"{pid} m{measures[0].get('number')}: no <time> in the first measure"
                            + (" (checking with the first part's)" if first_time else ""))
            if first_time:
                beats, beat_type = first_time
        seen_numbers = set()
        open_ties = {}
        for idx, m in enumerate(measures):
            num = m.get("number")
            if num in seen_numbers:
                problems.append(f"{pid} m{num}: duplicate measure number")
            seen_numbers.add(num)
            pos = Fraction(0)
            voice_end = {}
            max_pos = Fraction(0)
            for el in m:
                if el.tag == "attributes":
                    if el.findtext("divisions"):
                        divisions = int(el.findtext("divisions"))
                    t = el.find("time")
                    if t is not None and t.findtext("beats"):
                        times.setdefault(pid, []).append(
                            (num, f"{t.findtext('beats')}/{t.findtext('beat-type')}"))
                        try:
                            beats = Fraction(sum(int(x) for x in t.findtext("beats").split("+")))
                            beat_type = int(t.findtext("beat-type"))
                        except ValueError:
                            pass
                elif el.tag == "backup":
                    pos -= int(el.findtext("duration") or 0)
                elif el.tag == "forward":
                    pos += int(el.findtext("duration") or 0)
                    v = el.findtext("voice")
                    if v:
                        voice_end[v] = max(voice_end.get(v, 0), pos)
                elif el.tag == "note":
                    if el.find("grace") is not None:
                        continue
                    d = int(el.findtext("duration") or 0)
                    v = el.findtext("voice") or "1"
                    if el.find("chord") is None:
                        pos += d
                    voice_end[v] = max(voice_end.get(v, 0), pos)
                    if el.find("chord") is None and el.find("rest") is None and divisions:
                        # nominal duration from type/dots/tuplet vs encoded duration
                        nominal = nominal_duration(el, divisions)
                        if nominal is not None and nominal != d:
                            problems.append(f"{pid} m{num} v{v}: duration {d} does not match "
                                            f"type {note_token(el)} {note_suffix(el)} (= {nominal})")
                    p = el.find("pitch")
                    if p is not None:
                        key = (el.findtext("staff") or "1", p.findtext("step"),
                               p.findtext("octave"), p.findtext("alter") or "0")
                        types = {t.get("type") for t in el.findall("tie")}
                        if "stop" in types:
                            open_ties.pop(key, None)
                        if "start" in types:
                            open_ties[key] = num
                if pos < 0:
                    problems.append(f"{pid} m{num}: backup before the start of the measure")
                    pos = Fraction(0)
                max_pos = max(max_pos, pos)
            if divisions and beats and beat_type:
                expected = beats * 4 * divisions / beat_type
                multirest = m.find("attributes/measure-style/multiple-rest") is not None
                for v, end in sorted(voice_end.items()):
                    if end > expected:
                        problems.append(f"{pid} m{num} v{v}: overfull "
                                        f"({fmt_beats(end, divisions)} of {fmt_beats(expected, divisions)} quarters)")
                    elif end < expected and not multirest:
                        first_or_last = idx == 0 or idx == len(measures) - 1
                        tag = " (pickup/last measure?)" if first_or_last or m.get("implicit") == "yes" else ""
                        problems.append(f"{pid} m{num} v{v}: underfull "
                                        f"({fmt_beats(end, divisions)} of {fmt_beats(expected, divisions)} quarters){tag}")
            elif idx == 0 and not divisions:
                problems.append(f"{pid}: no <divisions> in the first measure")
            if idx == 0 and first_time is None and beats:
                first_time = (beats, beat_type)
        for key, num in open_ties.items():
            problems.append(f"{pid} m{num}: tie on {key[1]}{key[2]} (staff {key[0]}) never stopped")
    if len(set(counts.values())) > 1:
        problems.append("measure counts differ between parts: "
                        + ", ".join(f"{k}={v}" for k, v in counts.items()))
    for p in problems:
        print(p)
    # One line per distinct time-signature sequence (normally all parts agree).
    seqs = {}
    for pid, seq in times.items():
        seqs.setdefault(tuple(seq), []).append(pid)
    for seq, pids in seqs.items():
        label = "time signatures" + (f" ({','.join(pids)})" if len(seqs) > 1 else "")
        print(f"{label}: " + ", ".join(f"{t} from m{n}" for n, t in seq))
    flagged = {re.match(r"\S+ m(\S+?):?(?: |$)", p).group(1) for p in problems
               if re.match(r"\S+ m\S+", p)}
    print(f"{len(problems)} issue(s) in {len(flagged)} measure(s); measures per part: "
          + ", ".join(f"{k}={v}" for k, v in counts.items()))
    return 1 if problems else 0


TYPE_QUARTERS = {
    "maxima": Fraction(32), "long": Fraction(16), "breve": Fraction(8), "whole": Fraction(4),
    "half": Fraction(2), "quarter": Fraction(1), "eighth": Fraction(1, 2), "16th": Fraction(1, 4),
    "32nd": Fraction(1, 8), "64th": Fraction(1, 16), "128th": Fraction(1, 32), "256th": Fraction(1, 64),
}


def nominal_duration(n, divisions):
    q = TYPE_QUARTERS.get(n.findtext("type") or "")
    if q is None:
        return None
    total, add = q, q
    for _ in n.findall("dot"):
        add /= 2
        total += add
    tm = n.find("time-modification")
    if tm is not None:
        total = total * int(tm.findtext("normal-notes")) / int(tm.findtext("actual-notes"))
    d = total * divisions
    return int(d) if d.denominator == 1 else None


def fmt_beats(d, divisions):
    q = Fraction(d) / divisions
    return str(q.numerator) if q.denominator == 1 else f"{q.numerator}/{q.denominator}"


# ---------------------------------------------------------------- cropping

def staff_bounds(staff, x0, x1):
    ys = []
    for line in (staff["lines"][0], staff["lines"][-1]):
        for x in (x0, (x0 + x1) / 2, x1):
            t = (x - line["x0"]) / line["xs"]
            c = line["c"]
            ys.append(c[0] + c[1] * t + c[2] * t * t)
    return min(ys), max(ys)


def image_size(path):
    try:
        from PIL import Image
        with Image.open(path) as im:
            return im.size
    except ImportError:
        pass
    tool = shutil.which("magick") or shutil.which("identify")
    if not tool:
        sys.exit("cropping needs Pillow (pip install pillow) or ImageMagick")
    cmd = [tool] + (["identify"] if tool.endswith("magick") else []) + ["-format", "%w %h", path + "[0]"]
    w, h = subprocess.run(cmd, check=True, capture_output=True, text=True).stdout.split()
    return int(w), int(h)


def save_crop(image_path, box, out_path, upscale, shrink=1):
    """Grayscale crop (box None: whole image), zoomed by upscale or reduced by shrink."""
    try:
        from PIL import Image
        with Image.open(image_path) as im:
            c = im.convert("L")
            if box:
                c = c.crop(box)
            if upscale > 1:
                c = c.resize((c.width * upscale, c.height * upscale), Image.LANCZOS)
            if shrink > 1:
                c = c.resize((max(1, c.width // shrink), max(1, c.height // shrink)), Image.LANCZOS)
            c.save(out_path, optimize=True)
        return
    except ImportError:
        pass
    tool = shutil.which("magick") or shutil.which("convert")
    if not tool:
        sys.exit("cropping needs Pillow (pip install pillow) or ImageMagick")
    cmd = [tool, image_path, "-colorspace", "Gray"]
    if box:
        x0, y0, x1, y1 = box
        cmd += ["-crop", f"{x1 - x0}x{y1 - y0}+{x0}+{y0}", "+repage"]
    if upscale > 1:
        cmd += ["-resize", f"{upscale * 100}%"]
    if shrink > 1:
        cmd += ["-resize", f"{100 / shrink:.2f}%"]
    subprocess.run(cmd + [out_path], check=True)


def cmd_crops(args):
    os.makedirs(args.out, exist_ok=True)
    index = []
    sys_no = 0
    for page_no, image in enumerate(args.images, 1):
        stem = os.path.splitext(os.path.basename(image))[0]
        jpath = os.path.join(args.debug, stem + ".json")
        if not os.path.exists(jpath):
            sys.exit(f"{jpath} missing: run audiveris-native with --debug {args.debug} on {image}")
        with open(jpath) as f:
            d = json.load(f)
        inter = d["scale"]["interline"]
        upscale = args.upscale or (2 if inter < 16 else 1)
        width, height = image_size(image)
        pad = int(args.pad * inter)
        overview = f"page-{page_no}.png"
        save_crop(image, None, os.path.join(args.out, overview), 1, max(1, -(-width // 1600)))
        print(f"{overview}: page {page_no} overview ({image})")
        for system in d["systems"]:
            sys_no += 1
            staves = [d["staves"][i] for i in system["staves"]]
            left, right = system["left"], system["right"]
            top = min(staff_bounds(s, left, right)[0] for s in staves)
            bottom = max(staff_bounds(s, left, right)[1] for s in staves)
            y0, y1 = max(0, int(top) - pad), min(height, int(bottom) + pad)
            bars = [b["x0"] for b in system["boundaries"]]
            # header (clef/key/time) sits between the system start and the first bar
            if not bars or bars[0] - left > 2 * inter:
                bars = [left] + bars
            measures = max(0, len(bars) - 1)
            name = f"S{sys_no:02d}.png"
            box = (max(0, left - 2 * inter), y0, min(width, right + 2 * inter), y1)
            save_crop(image, box, os.path.join(args.out, name), 1)
            entry = {"system": sys_no, "page": page_no, "image": image, "file": name,
                     "box": box, "measures_detected": measures, "staves": len(staves),
                     "parts": [len(p) for p in system["parts"]], "pieces": []}
            if args.split and measures > 0:
                for k in range(0, measures, args.split):
                    j = min(measures, k + args.split)
                    bx0 = max(0, bars[k] - inter)
                    bx1 = min(width, bars[j] + inter)
                    piece = f"S{sys_no:02d}-{k + 1}-{j}.png"
                    save_crop(image, (bx0, y0, bx1, y1), os.path.join(args.out, piece), upscale)
                    entry["pieces"].append({"file": piece, "measures_in_system": [k + 1, j]})
            index.append(entry)
    with open(os.path.join(args.out, "index.json"), "w") as f:
        json.dump(index, f, indent=1)
    for e in index:
        pieces = ", ".join(p["file"] for p in e["pieces"])
        print(f"{e['file']}: page {e['page']}, {e['staves']} staves, "
              f"{e['measures_detected']} measures detected" + (f" -> {pieces}" if pieces else ""))


# ---------------------------------------------------------------- editing

PART_RE = r'<part\s+id="{}"\s*>'
MEASURE_RE = r'<measure\b[^>]*\bnumber="{}"[^>]*>'


def part_span(text, pid):
    m = re.search(PART_RE.format(re.escape(pid)), text)
    if not m:
        sys.exit(f"part {pid} not found")
    end = text.index("</part>", m.end())
    return m.end(), end


def measure_span(text, pid, number):
    p0, p1 = part_span(text, pid)
    m = re.compile(MEASURE_RE.format(re.escape(str(number)))).search(text, p0, p1)
    if not m:
        sys.exit(f"measure {number} not found in part {pid}")
    end = text.index("</measure>", m.end()) + len("</measure>")
    # include the indentation and the newline so the file stays tidy
    start = text.rfind("\n", 0, m.start()) + 1
    if text[start:m.start()].strip():
        start = m.start()
    if text[end:end + 1] == "\n":
        end += 1
    return start, end


def read_fragment(path):
    with open(path) if path != "-" else sys.stdin as f:
        frag = f.read().strip("\n")
    try:
        wrapper = ET.fromstring(f"<wrap>{frag}</wrap>")
    except ET.ParseError as e:
        sys.exit(f"fragment is not well-formed: {e}")
    if not len(wrapper) or any(el.tag != "measure" for el in wrapper):
        sys.exit("fragment must contain one or more <measure> elements")
    frag = textwrap.dedent(frag)
    return "".join("    " + line if line.strip() else line for line in frag.splitlines(True)) + "\n"


def write_checked(args, text):
    try:
        ET.fromstring(text.split("?>", 1)[-1] if text.startswith("<?xml") else text)
    except ET.ParseError:
        try:
            ET.fromstring(re.sub(r"<!DOCTYPE[^>]*>", "", text.split("?>", 1)[-1]))
        except ET.ParseError as e:
            sys.exit(f"result is not well-formed, nothing written: {e}")
    out = args.output or args.score
    with open(out, "w", encoding="utf-8") as f:
        f.write(text)
    print(f"wrote {out}")


def read_text(path):
    with open(path, encoding="utf-8") as f:
        return f.read()


def cmd_extract(args):
    text = read_text(args.score)
    rng = parse_range(args.measures)
    pids = [args.part] if args.part else re.findall(r'<part\s+id="([^"]+)"', text)
    for pid in pids:
        p0, p1 = part_span(text, pid)
        print(f"<!-- part {pid} -->")
        for m in re.finditer(r"<measure\b[^>]*>", text[p0:p1]):
            num = re.search(r'number="(-?\d+)', m.group())
            if num and rng[0] <= int(num.group(1)) <= rng[1]:
                s, e = measure_span(text, pid, re.search(r'number="([^"]+)"', m.group()).group(1))
                sys.stdout.write(text[s:e])


def cmd_replace(args):
    text = read_text(args.score)
    s, e = measure_span(text, args.part, args.measure)
    write_checked(args, text[:s] + read_fragment(args.fragment) + text[e:])


def cmd_insert(args):
    text = read_text(args.score)
    _, e = measure_span(text, args.part, args.after)
    write_checked(args, text[:e] + read_fragment(args.fragment) + text[e:])


def cmd_delete(args):
    text = read_text(args.score)
    s, e = measure_span(text, args.part, args.measure)
    write_checked(args, text[:s] + text[e:])


def renumber_text(text):
    out, pos = [], 0
    for pm in re.finditer(r'<part\s+id="[^"]+"\s*>', text):
        p1 = text.index("</part>", pm.end())
        first = re.compile(r'<measure\b[^>]*\bnumber="(-?\d+)').search(text, pm.end(), p1)
        n = int(first.group(1)) if first else 1
        for mm in re.compile(r'(<measure\b[^>]*\bnumber=")([^"]*)(")').finditer(text, pm.end(), p1):
            out.append(text[pos:mm.start()])
            out.append(f"{mm.group(1)}{n}{mm.group(3)}")
            pos = mm.end()
            n += 1
    out.append(text[pos:])
    return "".join(out)


def cmd_renumber(args):
    write_checked(args, renumber_text(read_text(args.score)))


def timed_items(measure):
    """(onset in divisions, element) for the children of a <measure>, without
    <backup>/<forward> (positions are rebuilt by rebuild_measure), and the end."""
    items, pos, last_onset, end = [], 0, 0, 0
    for el in measure:
        if el.tag == "backup":
            pos -= int(el.findtext("duration") or 0)
        elif el.tag == "forward":
            pos += int(el.findtext("duration") or 0)
        elif el.tag == "note":
            if el.find("chord") is not None:
                items.append((last_onset, el))
            else:
                items.append((pos, el))
                last_onset = pos
                if el.find("grace") is None:
                    pos += int(el.findtext("duration") or 0)
        else:
            items.append((pos, el))
        end = max(end, pos)
    return items, end


def rebuild_measure(attrib, items):
    """A <measure> holding items (onset, element), with <backup>/<forward>
    inserted wherever the next element does not start at the current position."""
    m = ET.Element("measure", attrib)
    pos = 0
    for onset, el in items:
        chord = el.tag == "note" and el.find("chord") is not None
        if not chord and onset != pos:
            gap = ET.SubElement(m, "forward" if onset > pos else "backup")
            ET.SubElement(gap, "duration").text = str(abs(onset - pos))
            pos = onset
        m.append(el)
        if el.tag == "note" and not chord and el.find("grace") is None:
            pos += int(el.findtext("duration") or 0)
    return m


def measure_xml(m):
    ET.indent(m, space="  ", level=2)
    return "    " + ET.tostring(m, encoding="unicode", short_empty_elements=False).rstrip() + "\n"


def divisions_at(text, pid, number):
    """Last <divisions> of the part up to and including measure NUMBER."""
    p0, _ = part_span(text, pid)
    _, e = measure_span(text, pid, number)
    found = re.findall(r"<divisions>(\d+)</divisions>", text[p0:e])
    return int(found[-1]) if found else 1


def part_ids(text, part):
    return [part] if part else re.findall(r'<part\s+id="([^"]+)"', text)


def cmd_split(args):
    text = read_text(args.score)
    at_q = Fraction(args.at)
    for pid in part_ids(text, args.part):
        s, e = measure_span(text, pid, args.measure)
        m = ET.fromstring(text[s:e])
        at = at_q * divisions_at(text, pid, args.measure)
        if at.denominator != 1:
            sys.exit(f"{pid}: --at {args.at} is not a whole number of divisions")
        at = int(at)
        items, _ = timed_items(m)
        first, second = [], []
        side_of_onset = {}
        for onset, el in items:
            if el.tag == "barline" and el.get("location") == "right":
                second.append((0, el))
            elif el.tag in ("attributes", "print") or (el.tag == "barline" and onset == 0):
                first.append((onset, el))
            elif onset < at or (el.tag == "note" and el.find("chord") is not None
                               and side_of_onset.get(onset) == "first"):
                if el.tag == "note" and el.find("grace") is None and el.find("chord") is None \
                        and onset + int(el.findtext("duration") or 0) > at:
                    print(f"{pid} m{args.measure}: note at {onset} crosses the split; "
                          "left in the first measure, fix it by hand", file=sys.stderr)
                side_of_onset[onset] = "first"
                first.append((onset, el))
            else:
                second.append((onset - at, el))
        attrib = dict(m.attrib)
        attrib.pop("implicit", None)
        a = rebuild_measure(dict(m.attrib), first)
        b = rebuild_measure(dict(attrib, number=f"{args.measure}X"), second)
        text = text[:s] + measure_xml(a) + measure_xml(b) + text[e:]
    write_checked(args, renumber_text(text))


def cmd_merge(args):
    text = read_text(args.score)
    for pid in part_ids(text, args.part):
        s, e = measure_span(text, pid, args.measure)
        m1 = ET.fromstring(text[s:e])
        nxt = re.compile(r"<measure\b[^>]*>").search(text, e, part_span(text, pid)[1])
        if not nxt:
            sys.exit(f"{pid} m{args.measure}: no next measure to merge")
        number = re.search(r'number="([^"]+)"', nxt.group()).group(1)
        s2, e2 = measure_span(text, pid, number)
        m2 = ET.fromstring(text[s2:e2])
        items1, end1 = timed_items(m1)
        items2, _ = timed_items(m2)
        items = [(o, el) for o, el in items1 if not (el.tag == "barline" and el.get("location") == "right")]
        for o, el in items2:
            if el.tag == "print" or (el.tag == "barline" and el.get("location") == "left"):
                continue
            items.append((o + end1, el))
        text = text[:s] + measure_xml(rebuild_measure(dict(m1.attrib), items)) + text[e2:]
    write_checked(args, renumber_text(text))


# ---------------------------------------------------------------- main

def main():
    if hasattr(signal, "SIGPIPE"):
        signal.signal(signal.SIGPIPE, signal.SIG_DFL)  # quiet under "| head"
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)

    p = sub.add_parser("summary", help="compact per-measure listing of a MusicXML file")
    p.add_argument("score")
    p.add_argument("--part")
    p.add_argument("--measures", help="e.g. 3-8")
    p.set_defaults(func=cmd_summary)

    p = sub.add_parser("check", help="report measures whose voices do not fill the time signature, "
                                     "dangling ties, part measure count mismatches")
    p.add_argument("score")
    p.set_defaults(func=cmd_check)

    p = sub.add_parser("crops", help="crop the original page images per system (and per N measures) "
                                     "using audiveris-native --debug output")
    p.add_argument("debug", help="the --debug directory")
    p.add_argument("out", help="output directory for the crops")
    p.add_argument("images", nargs="+", help="original page images, in page order")
    p.add_argument("--split", type=int, default=2, help="measures per zoomed crop (0: none, default 2)")
    p.add_argument("--pad", type=float, default=6, help="margin above/below the system in interlines")
    p.add_argument("--upscale", type=int, default=0, help="zoom factor for measure crops (default: auto)")
    p.set_defaults(func=cmd_crops)

    p = sub.add_parser("extract", help="print the raw <measure> XML of a measure range")
    p.add_argument("score")
    p.add_argument("--measures", required=True)
    p.add_argument("--part")
    p.set_defaults(func=cmd_extract)

    for name, helptext in (("replace", "replace one measure with the <measure> element(s) of FRAGMENT"),
                           ("insert", "insert the <measure> element(s) of FRAGMENT after a measure"),
                           ("delete", "delete one measure")):
        p = sub.add_parser(name, help=helptext)
        p.add_argument("score")
        p.add_argument("--part", required=True)
        if name == "insert":
            p.add_argument("--after", required=True)
        else:
            p.add_argument("--measure", required=True)
        if name != "delete":
            p.add_argument("fragment", help="file with <measure> element(s), or - for stdin")
        p.add_argument("-o", "--output", help="write here instead of overwriting SCORE")
        p.set_defaults(func={"replace": cmd_replace, "insert": cmd_insert, "delete": cmd_delete}[name])

    p = sub.add_parser("split", help="split a measure in two at a position (barline missed by the OMR); "
                                     "all parts unless --part, then renumber")
    p.add_argument("score")
    p.add_argument("--measure", required=True)
    p.add_argument("--at", required=True, help="split position in quarter notes from the measure start, e.g. 2 or 3/2")
    p.add_argument("--part")
    p.add_argument("-o", "--output")
    p.set_defaults(func=cmd_split)

    p = sub.add_parser("merge", help="merge a measure with the next one (extra barline read by the OMR); "
                                     "all parts unless --part, then renumber")
    p.add_argument("score")
    p.add_argument("--measure", required=True)
    p.add_argument("--part")
    p.add_argument("-o", "--output")
    p.set_defaults(func=cmd_merge)

    p = sub.add_parser("renumber", help="renumber the measures of every part consecutively")
    p.add_argument("score")
    p.add_argument("-o", "--output")
    p.set_defaults(func=cmd_renumber)

    args = ap.parse_args()
    sys.exit(args.func(args) or 0)


if __name__ == "__main__":
    main()
