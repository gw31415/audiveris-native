#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Compare independent MuseScore 3 truth with recognized MusicXML structure.

Development evidence only: neither this score nor its annotations feed recognition.
Voice labels and slur numbers resolve membership, but are absent from compared keys.
Unsupported timing and relation encodings fail explicitly instead of scoring omissions.
"""

import argparse
from collections import Counter, defaultdict
from dataclasses import dataclass, field
from fractions import Fraction
import hashlib
import json
from pathlib import Path
import xml.etree.ElementTree as ET

from compare_image_score_events import DURATIONS


class UnsupportedScore(ValueError):
    pass


@dataclass
class Group:
    staff: int
    measure: int
    onset: Fraction
    duration: Fraction
    grace: bool
    pitches: list
    voice: str
    order: int
    element: object = None
    xml_slurs: list = field(default_factory=list)

    def key(self):
        return (self.staff, self.measure, str(self.onset), str(self.duration),
                self.grace, tuple(sorted(self.pitches, key=str)))

    def events(self):
        return [(self.staff, self.measure, str(self.onset), pitch,
                 str(self.duration), self.grace) for pitch in self.pitches]


@dataclass
class Score:
    groups: list = field(default_factory=list)
    measures: list = field(default_factory=list)
    parts: list = field(default_factory=list)
    slurs: list = field(default_factory=list)
    slur_endpoints: list = field(default_factory=list)
    issues: list = field(default_factory=list)
    tie_endpoints: list = field(default_factory=list)
    unscored: Counter = field(default_factory=Counter)

    def categories(self):
        events = [event for group in self.groups for event in group.events()]
        partitions, successors = voice_structure(self.groups)
        return {
            "events": events,
            "rests": [event for event in events if event[3] is None],
            "chord_groups": [group.key() for group in self.groups],
            "multiple_note_chords": [group.key() for group in self.groups if len(group.pitches) > 1],
            "within_measure_voice_partitions": partitions,
            "within_measure_voice_successors": successors,
            "meters": [row[:3] for row in self.measures],
            "measure_extents": [(row[0], row[1], row[3]) for row in self.measures],
            "pickup_markers": [(row[0], row[1], row[4]) for row in self.measures],
            "part_staff_membership": self.parts,
            "slur_endpoints": self.slur_endpoints,
            "slur_relations": self.slurs,
            "tie_endpoints": self.tie_endpoints,
        }


def voice_structure(groups):
    voices = defaultdict(list)
    for group in groups:
        voices[(group.staff, group.measure, group.voice)].append(group)
    partitions = []
    successors = []
    for members in voices.values():
        ordered = sorted(members, key=lambda group: (group.onset, group.order))
        keys = tuple(group.key() for group in ordered)
        partitions.append(keys)
        successors.extend(zip(keys, keys[1:]))
    return partitions, successors


def single_stream_boundaries(score):
    """Observe only boundaries whose local streams need no identity assignment."""
    members = defaultdict(list)
    for group in score.groups:
        members[(group.staff, group.measure)].append(group)
    measures = {(row[0], row[1]) for row in score.measures}
    streams = {}
    for key in measures:
        groups = sorted(members[key], key=lambda group: (group.onset, group.order))
        voices = {group.voice for group in groups}
        reasons = []
        if not groups:
            reasons.append("empty_measure")
        if len(voices) > 1:
            reasons.append("multiple_local_streams_without_cross_measure_identity_evidence")
        end = Fraction()
        if len(voices) == 1:
            for group in groups:
                if group.onset < end:
                    reasons.append("overlapping_groups_in_local_stream")
                    break
                if group.duration < 0 or (not group.grace and group.duration == 0):
                    reasons.append("nonpositive_ordinary_duration")
                    break
                end = group.onset + group.duration
        streams[key] = {
            "voice_count": len(voices),
            "group_count": len(groups),
            "reasons": reasons,
            "first": groups[0].key() if groups else None,
            "last": groups[-1].key() if groups else None,
        }
    boundaries = {}
    for staff, measure in sorted(measures):
        if (staff, measure + 1) not in measures:
            continue
        left, right = streams[(staff, measure)], streams[(staff, measure + 1)]
        eligible = not left["reasons"] and not right["reasons"]
        boundaries[(staff, measure, measure + 1)] = {
            "eligible": eligible,
            "left": left,
            "right": right,
            "successor": (left["last"], right["first"]) if eligible else None,
        }
    return boundaries


def compare_boundary_continuity(expected, actual):
    expected_boundaries = single_stream_boundaries(expected)
    actual_boundaries = single_stream_boundaries(actual)
    expected_edges, actual_edges, observations = [], [], []
    for boundary in sorted(expected_boundaries.keys() | actual_boundaries.keys()):
        left, right = expected_boundaries.get(boundary), actual_boundaries.get(boundary)
        comparable = left is not None and right is not None and left["eligible"] and right["eligible"]
        if comparable:
            expected_edges.append(left["successor"])
            actual_edges.append(right["successor"])
        observations.append({"staff": boundary[0], "measures": boundary[1:],
                             "scored": comparable, "truth": left, "recognized": right})
    coverage = {
        "truth_boundaries": len(expected_boundaries),
        "recognized_boundaries": len(actual_boundaries),
        "truth_single_stream_boundaries": sum(item["eligible"] for item in expected_boundaries.values()),
        "recognized_single_stream_boundaries": sum(item["eligible"] for item in actual_boundaries.values()),
        "jointly_scored_boundaries": len(expected_edges),
        "unscored_boundary_union": sum(not item["scored"] for item in observations),
        "available_relation_observations": {
            "truth_cross_measure_slurs": sum(start[1] != stop[1] for start, stop in expected.slurs),
            "recognized_cross_measure_slurs": sum(start[1] != stop[1] for start, stop in actual.slurs),
            "truth_tie_endpoints": len(expected.tie_endpoints),
            "recognized_tie_endpoints": len(actual.tie_endpoints),
            "used_for_polyphonic_identity_assignment": False,
        },
        "observations": observations,
        "qualification": "Only adjacent measures on the same staff with one nonoverlapping local stream on each side in BOTH scores are scored. Local voice labels may change freely across the barline. The compared pair is the last complete group before the boundary and the first after it, retaining pitch, duration and grace identity. Multiple streams, empty measures and missing boundaries remain unscored; no pitch-distance or reused-label heuristic supplies continuity. This does not prove polyphonic voice identity across measures.",
    }
    return metric(expected_edges, actual_edges), coverage


def fraction_text(element, name, default="0"):
    return Fraction(element.findtext(name, default))


def truth_duration(item, meter, measure):
    if item.find("Tuplet") is not None:
        raise UnsupportedScore("MuseScore tuplet timing is not implemented")
    if any(item.find(name) is not None for name in ("staffMove", "duration")):
        raise UnsupportedScore("MuseScore cross-staff or explicit duration is not implemented")
    kind = item.findtext("durationType")
    if kind == "measure":
        if meter is None:
            raise UnsupportedScore("whole-measure rest needs a meter")
        duration = Fraction(measure.attrib.get("len", str(Fraction(*meter))))
    elif kind in DURATIONS:
        duration = DURATIONS[kind]
    else:
        raise UnsupportedScore("unsupported MuseScore duration: " + str(kind))
    dots = int(item.findtext("dots", "0"))
    duration *= sum((Fraction(1, 2**i) for i in range(dots + 1)), Fraction())
    grace = any(item.find(name) is not None for name in (
        "appoggiatura", "acciaccatura", "grace4", "grace16", "grace32"))
    return (Fraction() if grace else duration), grace


def _location(group, location):
    allowed = {"measures", "fractions"}
    if any(child.tag not in allowed for child in location):
        raise UnsupportedScore("cross-staff/voice/note MuseScore spanner locations are not implemented")
    return (group.staff, group.measure + int(location.findtext("measures", "0")),
            group.onset + fraction_text(location, "fractions"), group.voice)


def _truth_slurs(score):
    by_location = defaultdict(list)
    starts = []
    stops = Counter()
    for group in score.groups:
        here = (group.staff, group.measure, group.onset, group.voice)
        by_location[here].append(group)
        for spanner in group.element.findall("Spanner"):
            kind = spanner.attrib.get("type")
            if kind != "Slur":
                score.unscored["MuseScore Spanner:" + str(kind)] += 1
                continue
            next_location = spanner.find("next/location")
            prev_location = spanner.find("prev/location")
            if next_location is None and prev_location is None:
                score.issues.append(("slur_without_location", group.key()))
            if next_location is not None:
                starts.append((group, _location(group, next_location)))
                score.slur_endpoints.append(("start", group.key()))
            if prev_location is not None:
                stops[(here, _location(group, prev_location))] += 1
                score.slur_endpoints.append(("stop", group.key()))
    for start, destination in starts:
        candidates = by_location[destination]
        if len(candidates) != 1:
            score.issues.append(("unresolved_or_ambiguous_slur_end", start.key(),
                                 tuple(str(value) for value in destination), len(candidates)))
            continue
        end = candidates[0]
        here = (start.staff, start.measure, start.onset, start.voice)
        reciprocal = (destination, here)
        if stops[reciprocal] == 0:
            score.issues.append(("missing_reciprocal_slur_stop", start.key(), end.key()))
            continue
        stops[reciprocal] -= 1
        score.slurs.append((start.key(), end.key()))
    for (here, source), count in stops.items():
        if count:
            score.issues.append(("unmatched_slur_stop", tuple(map(str, here)),
                                 tuple(map(str, source)), count))


def truth_score(path):
    root = ET.parse(path).getroot()
    if root.tag != "museScore" or not root.attrib.get("version", "").startswith("3."):
        raise UnsupportedScore("MuseScore 3 truth required")
    score = Score()
    body = root.find("Score")
    if body is None:
        raise UnsupportedScore("MuseScore Score element required")
    # A tie-bearing reference needs a separately observed MuseScore note-location
    # contract. Counting it as an absent relation would fabricate a quality score.
    if any(sp.attrib.get("type") == "Tie" for sp in body.iter("Spanner")) or body.findall(".//Tie"):
        raise UnsupportedScore("MuseScore tie endpoint resolution needs a real reference fixture")
    if body.findall(".//Note/Spanner") or body.findall(".//endSpanner"):
        raise UnsupportedScore("MuseScore note-level or legacy spanner locations are not implemented")
    staves = body.findall("Staff")
    staff_ids = {staff.attrib["id"]: index for index, staff in enumerate(staves, 1)}
    for part in body.findall("Part"):
        score.parts.append(tuple(staff_ids[staff.attrib["id"]] for staff in part.findall("Staff")))
    for staff in staves:
        sid = staff_ids[staff.attrib["id"]]
        meter = None
        for mi, measure in enumerate(staff.findall("Measure"), 1):
            end = Fraction()
            for vi, voice in enumerate(measure.findall("voice"), 1):
                clock = Fraction()
                for order, item in enumerate(voice):
                    if item.tag == "TimeSig":
                        meter = (int(item.findtext("sigN")), int(item.findtext("sigD")))
                    elif item.tag in ("location", "tick", "Tuplet", "endTuplet", "RepeatMeasure"):
                        raise UnsupportedScore("MuseScore voice timing element: " + item.tag)
                    elif item.tag in ("Chord", "Rest"):
                        duration, grace = truth_duration(item, meter, measure)
                        pitches = [int(note.findtext("pitch")) for note in item.findall("Note")] if item.tag == "Chord" else [None]
                        if not pitches:
                            raise UnsupportedScore("MuseScore chord without notes")
                        score.groups.append(Group(sid, mi, clock, duration, grace,
                                                  pitches, str(vi), order, item))
                        clock += duration
                    elif item.tag == "Spanner":
                        score.unscored["MuseScore Spanner:" + str(item.attrib.get("type"))] += 1
                end = max(end, clock)
            irregular = measure.findtext("irregular", "0") == "1"
            if "len" in measure.attrib:
                if meter is None:
                    raise UnsupportedScore("explicit MuseScore measure length needs a meter")
                irregular |= Fraction(measure.attrib["len"]) != Fraction(*meter)
            score.measures.append((sid, mi, meter, str(end), irregular))
    _truth_slurs(score)
    return score


def _xml_midi(note):
    pitch = note.find("pitch")
    if pitch is None:
        if note.find("rest") is None:
            raise UnsupportedScore("MusicXML unpitched notes are not implemented")
        return None
    alter = Fraction(pitch.findtext("alter", "0"))
    midi = 12 * (int(pitch.findtext("octave")) + 1) + {
        "C": 0, "D": 2, "E": 4, "F": 5, "G": 7, "A": 9, "B": 11,
    }[pitch.findtext("step")] + alter
    if midi.denominator != 1:
        raise UnsupportedScore("microtonal MusicXML is not implemented")
    return int(midi)


def _xml_slurs(score, groups):
    active = {}
    for group in sorted(groups, key=lambda g: (g.measure, g.onset, g.order)):
        for number, action in group.xml_slurs:
            score.slur_endpoints.append((action, group.key()))
            if action == "start":
                if number in active:
                    score.issues.append(("overlapping_slur_start", active[number].key(), group.key()))
                active[number] = group
            elif action == "stop":
                if number not in active:
                    score.issues.append(("unmatched_slur_stop", group.key()))
                else:
                    score.slurs.append((active.pop(number).key(), group.key()))
            else:
                raise UnsupportedScore("MusicXML slur action is not implemented: " + str(action))
    for group in active.values():
        score.issues.append(("unclosed_slur_start", group.key()))


def xml_score(path):
    root = ET.parse(path).getroot()
    if root.tag != "score-partwise":
        raise UnsupportedScore("score-partwise MusicXML required")
    score = Score()
    base = 0
    for part in root.findall("part"):
        declared_staves = [int(value.text) for value in part.findall("measure/attributes/staves")]
        note_staves = [int(value.text) for value in part.findall("measure/note/staff")]
        staff_count = max([1] + declared_staves + note_staves)
        score.parts.append(tuple(range(base + 1, base + staff_count + 1)))
        divisions = None
        meters = {}
        part_groups = []
        for mi, measure in enumerate(part.findall("measure"), 1):
            clock = Fraction()
            ends = defaultdict(Fraction)
            last = None
            for order, item in enumerate(measure):
                if item.tag == "attributes":
                    if item.findtext("divisions") is not None:
                        divisions = int(item.findtext("divisions"))
                    for time in item.findall("time"):
                        if len(time.findall("beats")) != 1 or len(time.findall("beat-type")) != 1:
                            raise UnsupportedScore("composite/senza-misura MusicXML meter is not implemented")
                        meter = (int(time.findtext("beats")), int(time.findtext("beat-type")))
                        for staff in ([int(time.attrib["number"])] if "number" in time.attrib else range(1, staff_count + 1)):
                            meters[staff] = meter
                elif item.tag in ("backup", "forward"):
                    if not divisions or divisions <= 0:
                        raise UnsupportedScore("positive MusicXML divisions required")
                    duration = fraction_text(item, "duration") / (4 * divisions)
                    clock += duration if item.tag == "forward" else -duration
                    if clock < 0:
                        raise UnsupportedScore("MusicXML negative measure position")
                    last = None
                    if item.tag == "forward":
                        for staff in ([int(item.findtext("staff"))] if item.findtext("staff") else range(1, staff_count + 1)):
                            ends[staff] = max(ends[staff], clock)
                elif item.tag == "note":
                    if not divisions or divisions <= 0:
                        raise UnsupportedScore("positive MusicXML divisions required")
                    staff = int(item.findtext("staff", "1"))
                    voice = item.findtext("voice", "1")
                    grace = item.find("grace") is not None
                    duration = Fraction() if grace else fraction_text(item, "duration") / (4 * divisions)
                    pitch = _xml_midi(item)
                    if item.find("chord") is None:
                        last = Group(base + staff, mi, clock, duration, grace,
                                     [pitch], voice, order)
                        part_groups.append(last)
                        clock += duration
                    else:
                        if last is None or (last.staff, last.voice, last.duration, last.grace) != (base + staff, voice, duration, grace):
                            raise UnsupportedScore("MusicXML chord without compatible same-staff lead note")
                        if pitch is None or None in last.pitches:
                            raise UnsupportedScore("MusicXML rest chord is not implemented")
                        last.pitches.append(pitch)
                    ends[staff] = max(ends[staff], last.onset + duration)
                    last.xml_slurs.extend((slur.attrib.get("number", "1"), slur.attrib.get("type")) for slur in item.findall("notations/slur"))
                    for kind, selector in (("sound", "tie"), ("notation", "notations/tied")):
                        for tie in item.findall(selector):
                            score.tie_endpoints.append((kind, tie.attrib.get("type"),
                                                        (last.staff, mi, str(last.onset), pitch, str(duration), grace)))
            for staff in range(1, staff_count + 1):
                score.measures.append((base + staff, mi, meters.get(staff), str(ends[staff]), measure.attrib.get("implicit", "no") == "yes"))
        _xml_slurs(score, part_groups)
        score.groups.extend(part_groups)
        base += staff_count
    return score


def metric(expected, actual):
    e, a = Counter(expected), Counter(actual)
    matched = sum((e & a).values())
    total_expected, total_actual = sum(e.values()), sum(a.values())
    precision = matched / total_actual if total_actual else None
    recall = matched / total_expected if total_expected else None
    f1 = 2 * matched / (total_expected + total_actual) if total_expected + total_actual else None
    return {"truth": total_expected, "recognized": total_actual, "matched": matched,
            "precision": precision, "recall": recall, "F1": f1,
            "equal": e == a, "qualification": "observed" if e or a else "both_empty_not_exercised",
            "missing": list((e - a).elements()), "extra": list((a - e).elements())}


def compare(truth, recognized):
    expected, actual = truth_score(truth), xml_score(recognized)
    e, a = expected.categories(), actual.categories()
    categories = {name: metric(e[name], a[name]) for name in e}
    truth_groups = Counter(group.key() for group in expected.groups)
    actual_groups = Counter(group.key() for group in actual.groups)
    shared_unique = {key for key, count in truth_groups.items() if count == actual_groups[key] == 1}
    expected_partitions, expected_successors = voice_structure(
        [group for group in expected.groups if group.key() in shared_unique])
    actual_partitions, actual_successors = voice_structure(
        [group for group in actual.groups if group.key() in shared_unique])
    categories["matched_chord_voice_partitions"] = metric(expected_partitions, actual_partitions)
    categories["matched_chord_voice_successors"] = metric(expected_successors, actual_successors)
    boundary_metric, boundary_coverage = compare_boundary_continuity(expected, actual)
    categories["across_measure_single_stream_successors"] = boundary_metric
    return {
        "schema": "audiveris-native.independent-structure-quality.v1",
        "truth_sha256": hashlib.sha256(truth.read_bytes()).hexdigest(),
        "recognized_sha256": hashlib.sha256(recognized.read_bytes()).hexdigest(),
        "categories": categories,
        "truth_relation_issues": expected.issues,
        "recognized_relation_issues": actual.issues,
        "slur_pairing_valid": not expected.issues and not actual.issues,
        "conditional_voice_coverage": {
            "shared_unique_complete_chords": len(shared_unique),
            "truth_groups_excluded": len(expected.groups) - len(shared_unique),
            "recognized_groups_excluded": len(actual.groups) - len(shared_unique),
            "qualification": "Matched-chord voice categories project each voice onto complete chord keys appearing exactly once in both scores. Unmatched or ambiguous chords are excluded, so these categories alone cannot establish recognition quality.",
        },
        "across_measure_voice_coverage": boundary_coverage,
        "scope": {
            "events": "Exact staff order, measure order, rational onset/duration, MIDI pitch/rest and grace status.",
            "chords": "Actual serialized chord grouping, compared as complete pitch multisets; simultaneous independent voices stay separate.",
            "voices": "Per-staff, per-measure ordered partitions and successive chord/rest edges; arbitrary labels excluded. Full categories include pitch/timing errors; matched-chord categories isolate voice membership over stated common anchors. Across-measure comparisons cover only jointly unambiguous single-stream boundaries, with explicit coverage and exclusions.",
            "measures": "Nominal meters, occupied/forward extent, irregular/implicit markers and part/staff membership; printed measure numbers excluded.",
            "slurs": "Reciprocal MuseScore chord locations versus paired MusicXML slur numbers. Full musical chord endpoints compared; engraving placement excluded.",
            "ties": "Tie-free truth is checked for extra XML sound/notation endpoints. Tie-bearing MuseScore truth is explicitly unsupported until a real fixture establishes its endpoint encoding.",
        },
        "not_scored": {
            "features": ["polyphonic across-measure voice identity", "cross-staff chords", "tie-bearing references",
                         "dynamics/hairpins", "articulations", "beam grouping", "lyrics", "engraving"],
            "truth_observed_unscored_spanners": dict(expected.unscored),
        },
        "quality_accepted": False,
        "qualification": "Evidence for stated categories on this page only; this utility does not decide Audiveris-equivalent recognition quality.",
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("truth", type=Path)
    parser.add_argument("recognized", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    result = compare(args.truth, args.recognized)
    with args.output.open("x") as output:
        json.dump(result, output, indent=2)
        output.write("\n")
    print(json.dumps({name: {key: values[key] for key in ("truth", "recognized", "matched", "F1", "equal")}
                      for name, values in result["categories"].items()}))


if __name__ == "__main__":
    main()
