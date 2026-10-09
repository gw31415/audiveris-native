# SPDX-License-Identifier: AGPL-3.0-or-later
from pathlib import Path
import sys
import tempfile
import unittest
import xml.etree.ElementTree as ET

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from compare_score_structure_quality import UnsupportedScore, compare
from compare_image_score_events import compare as compare_events


def chord(pitches, kind="quarter", extra=""):
    return (f"<Chord><durationType>{kind}</durationType>{extra}"
            + "".join(f"<Note><pitch>{pitch}</pitch></Note>" for pitch in pitches)
            + "</Chord>")


def slur_location(direction, fraction):
    return (f'<Spanner type="Slur"><{direction}><location>'
            f"<fractions>{fraction}</fractions></location></{direction}></Spanner>")


def truth(voices, measure_attributes="", meter=(4, 4)):
    time = f"<TimeSig><sigN>{meter[0]}</sigN><sigD>{meter[1]}</sigD></TimeSig>"
    content = "".join(f"<voice>{time if index == 0 else ''}{voice}</voice>"
                      for index, voice in enumerate(voices))
    return (f'<museScore version="3.01"><Score><Part><Staff id="42"/></Part>'
            f'<Staff id="42"><Measure {measure_attributes}>{content}'
            '</Measure></Staff></Score></museScore>')


def note(step="C", duration=4, voice="7", extra="", slur=None, grace=False):
    pitch = "<rest/>" if step is None else f"<pitch><step>{step}</step><octave>4</octave></pitch>"
    rhythm = "<grace/>" if grace else f"<duration>{duration}</duration>"
    notation = "" if slur is None else f'<notations><slur type="{slur[0]}" number="{slur[1]}"/></notations>'
    return f"<note>{extra}{rhythm}{pitch}<voice>{voice}</voice>{notation}</note>"


def xml(notes, measure_attributes="", meter=(4, 4)):
    return (f'<score-partwise><part id="Arbitrary"><measure {measure_attributes}>'
            f'<attributes><divisions>4</divisions><time><beats>{meter[0]}</beats>'
            f'<beat-type>{meter[1]}</beat-type></time></attributes>{notes}'
            '</measure></part></score-partwise>')


def joined_truth(*scores):
    root = ET.fromstring(scores[0])
    staff = root.find("Score/Staff")
    for text in scores[1:]:
        staff.append(ET.fromstring(text).find("Score/Staff/Measure"))
    return ET.tostring(root, encoding="unicode")


def joined_xml(*scores):
    root = ET.fromstring(scores[0])
    part = root.find("part")
    for text in scores[1:]:
        part.append(ET.fromstring(text).find("part/measure"))
    return ET.tostring(root, encoding="unicode")


class StructureQualityContracts(unittest.TestCase):
    def compare(self, expected, recognized):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary)
            source, output = path / "truth.mscx", path / "actual.musicxml"
            source.write_text(expected)
            output.write_text(recognized)
            return compare(source, output)

    def test_voice_and_relation_labels_are_not_musical_identity(self):
        expected = truth([chord([60], extra=slur_location("next", "1/4"))
                          + chord([62], extra=slur_location("prev", "-1/4"))])
        recognized = xml(note("C", slur=("start", "93")) + note("D", slur=("stop", "93")))
        first = self.compare(expected, recognized)
        second = self.compare(expected, recognized.replace("<voice>7</voice>", "<voice>alto</voice>").replace('number="93"', 'number="417"'))
        self.assertEqual(first["categories"], second["categories"])
        self.assertTrue(all(value["equal"] for value in first["categories"].values()))
        self.assertEqual(first["categories"]["slur_relations"]["matched"], 1)
        self.assertFalse(first["quality_accepted"])

    def test_simultaneous_voices_cannot_masquerade_as_one_chord(self):
        expected = truth([chord([60, 64])])
        grouped = self.compare(expected, xml(note("C") + note("E", extra="<chord/>")))
        separate = self.compare(expected, xml(note("C") + "<backup><duration>4</duration></backup>" + note("E", voice="91")))
        self.assertTrue(grouped["categories"]["chord_groups"]["equal"])
        self.assertTrue(separate["categories"]["events"]["equal"])
        self.assertFalse(separate["categories"]["chord_groups"]["equal"])
        self.assertEqual(separate["categories"]["multiple_note_chords"]["matched"], 0)

    def test_voice_split_changes_continuity_even_when_notes_are_identical(self):
        expected = truth([chord([60]) + chord([62]) + chord([64])])
        result = self.compare(expected, xml(note("C") + note("D", voice="8") + note("E", voice="8")))
        self.assertTrue(result["categories"]["events"]["equal"])
        self.assertTrue(result["categories"]["chord_groups"]["equal"])
        successors = result["categories"]["within_measure_voice_successors"]
        self.assertEqual((successors["truth"], successors["recognized"], successors["matched"]), (2, 1, 1))
        self.assertFalse(result["categories"]["within_measure_voice_partitions"]["equal"])
        self.assertFalse(result["categories"]["matched_chord_voice_partitions"]["equal"])

    def test_conditional_voice_metric_distinguishes_pitch_error_from_voice_split(self):
        expected = truth([chord([60]) + chord([62]) + chord([64])])
        result = self.compare(expected, xml(note("C") + note("F") + note("E")))
        self.assertFalse(result["categories"]["events"]["equal"])
        self.assertFalse(result["categories"]["within_measure_voice_successors"]["equal"])
        self.assertTrue(result["categories"]["matched_chord_voice_successors"]["equal"])
        self.assertTrue(result["categories"]["matched_chord_voice_partitions"]["equal"])
        self.assertEqual(result["conditional_voice_coverage"]["shared_unique_complete_chords"], 2)
        self.assertEqual(result["conditional_voice_coverage"]["truth_groups_excluded"], 1)

    def test_pickup_whole_measure_rest_uses_actual_short_measure(self):
        expected = truth(["<Rest><durationType>measure</durationType></Rest>"], 'len="1/8"', (2, 4))
        result = self.compare(expected, xml(note(None, duration=2), 'implicit="yes" number="0"', (2, 4)))
        self.assertTrue(all(value["equal"] for value in result["categories"].values()))
        self.assertEqual(result["categories"]["rests"]["matched"], 1)

    def test_different_slur_connection_is_detected_with_unchanged_note_count(self):
        expected = truth([chord([60], extra=slur_location("next", "1/4"))
                          + chord([62], extra=slur_location("prev", "-1/4")) + chord([64])])
        result = self.compare(expected, xml(note("C", slur=("start", "1"))
                                          + note("D") + note("E", slur=("stop", "1"))))
        self.assertTrue(result["categories"]["events"]["equal"])
        self.assertTrue(result["slur_pairing_valid"])
        self.assertEqual(result["categories"]["slur_endpoints"]["matched"], 1)
        self.assertEqual(result["categories"]["slur_relations"]["matched"], 0)

    def test_unclosed_xml_slur_reports_endpoint_without_fabricating_link(self):
        result = self.compare(truth([chord([60])]), xml(note("C", slur=("start", "1"))))
        self.assertFalse(result["slur_pairing_valid"])
        self.assertEqual(result["recognized_relation_issues"][0][0], "unclosed_slur_start")
        self.assertEqual(result["categories"]["slur_relations"]["recognized"], 0)

    def test_missing_truth_reciprocal_endpoint_is_an_issue(self):
        expected = truth([chord([60], extra=slur_location("next", "1/4")) + chord([62])])
        result = self.compare(expected, xml(note("C") + note("D")))
        self.assertFalse(result["slur_pairing_valid"])
        self.assertEqual(result["truth_relation_issues"][0][0], "missing_reciprocal_slur_stop")
        self.assertEqual(result["categories"]["slur_relations"]["truth"], 0)

    def test_new_tie_is_observed_but_empty_tie_category_is_not_success_evidence(self):
        expected = truth([chord([60])])
        empty = self.compare(expected, xml(note("C")))
        self.assertEqual(empty["categories"]["tie_endpoints"]["qualification"], "both_empty_not_exercised")
        self.assertIsNone(empty["categories"]["tie_endpoints"]["F1"])
        added = self.compare(expected, xml(note("C", extra='<tie type="start"/>')))
        self.assertFalse(added["categories"]["tie_endpoints"]["equal"])
        self.assertEqual(added["categories"]["tie_endpoints"]["recognized"], 1)

    def test_unsupported_timing_and_tie_truth_fail_instead_of_scoring_omission(self):
        for extra, message in (("<Tuplet>1</Tuplet>", "tuplet"),
                               ('<Spanner type="Tie"/>', "tie endpoint"),
                               ("<staffMove>1</staffMove>", "cross-staff")):
            with self.subTest(extra=extra), self.assertRaisesRegex(UnsupportedScore, message):
                self.compare(truth([chord([60], extra=extra)]), xml(note("C")))

    def test_grace_is_ordered_before_main_note_at_same_onset_without_duration(self):
        expected = truth([chord([60], "eighth", "<appoggiatura/>") + chord([62])])
        result = self.compare(expected, xml(note("C", grace=True) + note("D")))
        self.assertTrue(all(value["equal"] for value in result["categories"].values()))
        self.assertEqual(result["categories"]["within_measure_voice_successors"]["matched"], 1)

    def test_rest_duration_and_meter_changes_are_separate_observations(self):
        expected = truth(["<Rest><durationType>quarter</durationType></Rest>"])
        result = self.compare(expected, xml(note(None, duration=2), meter=(3, 4)))
        self.assertFalse(result["categories"]["rests"]["equal"])
        self.assertFalse(result["categories"]["meters"]["equal"])
        self.assertFalse(result["categories"]["measure_extents"]["equal"])

    def test_single_stream_boundary_ignores_per_measure_voice_renumbering(self):
        expected = joined_truth(truth([chord([60])]), truth([chord([62])]))
        result = self.compare(expected, joined_xml(xml(note("C", voice="99")), xml(note("D", voice="soprano"))))
        metric = result["categories"]["across_measure_single_stream_successors"]
        self.assertEqual((metric["truth"], metric["recognized"], metric["matched"]), (1, 1, 1))
        self.assertEqual(result["across_measure_voice_coverage"]["jointly_scored_boundaries"], 1)

    def test_reused_polyphonic_labels_do_not_supply_cross_measure_identity(self):
        expected = joined_truth(truth([chord([60]), chord([64])]), truth([chord([62]), chord([65])]))
        before = note("C", voice="1") + "<backup><duration>4</duration></backup>" + note("E", voice="2")
        after = note("D", voice="1") + "<backup><duration>4</duration></backup>" + note("F", voice="2")
        result = self.compare(expected, joined_xml(xml(before), xml(after)))
        self.assertTrue(result["categories"]["events"]["equal"])
        metric = result["categories"]["across_measure_single_stream_successors"]
        self.assertEqual(metric["truth"], 0)
        self.assertIsNone(metric["F1"])
        coverage = result["across_measure_voice_coverage"]
        self.assertEqual(coverage["unscored_boundary_union"], 1)
        self.assertIn("multiple_local_streams_without_cross_measure_identity_evidence",
                      coverage["observations"][0]["recognized"]["left"]["reasons"])

    def test_one_reused_voice_with_overlapping_notes_is_not_single_stream_evidence(self):
        expected = joined_truth(truth([chord([60, 64])]), truth([chord([62])]))
        before = note("C") + "<backup><duration>4</duration></backup>" + note("E")
        result = self.compare(expected, joined_xml(xml(before), xml(note("D"))))
        self.assertTrue(result["categories"]["events"]["equal"])
        coverage = result["across_measure_voice_coverage"]
        self.assertEqual(coverage["jointly_scored_boundaries"], 0)
        self.assertEqual(coverage["observations"][0]["recognized"]["left"]["reasons"],
                         ["overlapping_groups_in_local_stream"])

    def test_empty_measure_is_not_bridged_to_manufacture_continuity(self):
        expected = joined_truth(truth([chord([60])]), truth([""]), truth([chord([62])]))
        result = self.compare(expected, joined_xml(xml(note("C")), xml(""), xml(note("D"))))
        coverage = result["across_measure_voice_coverage"]
        self.assertEqual(coverage["truth_boundaries"], 2)
        self.assertEqual(coverage["jointly_scored_boundaries"], 0)
        self.assertEqual(coverage["unscored_boundary_union"], 2)
        self.assertTrue(all(item["measures"][1] - item["measures"][0] == 1 for item in coverage["observations"]))

    def test_boundary_wrong_pitch_is_scored_and_rest_endpoint_is_retained(self):
        expected = joined_truth(truth([chord([60])]), truth([chord([62])]),
                                truth(["<Rest><durationType>quarter</durationType></Rest>"]))
        result = self.compare(expected, joined_xml(xml(note("F")), xml(note("D")), xml(note(None))))
        metric = result["categories"]["across_measure_single_stream_successors"]
        self.assertEqual((metric["truth"], metric["recognized"], metric["matched"]), (2, 2, 1))
        self.assertEqual(result["across_measure_voice_coverage"]["jointly_scored_boundaries"], 2)
        self.assertEqual(result["across_measure_voice_coverage"]["observations"][1]["truth"]["successor"][1][-1], (None,))

    def test_real_pages_retain_prior_event_baselines_and_exercise_six_slurs(self):
        root = Path(__file__).resolve().parents[2]
        pages = (("lc6019054", "real-image-quality-baseline/laura-original.musicxml", 101, 92, 73, 0),
                 ("lc6053913", "real-rest-quality-baseline/der-winter-original.musicxml", 124, 124, 102, 6))
        for name, output, truth_count, recognized_count, matches, slurs in pages:
            with self.subTest(page=name):
                source = root / "fixtures/openscore-lieder" / (name + ".mscx")
                recognized = root / "evidence" / output
                result = compare(source, recognized)
                old = compare_events(source, recognized)
                notes = result["categories"]["events"]
                self.assertEqual((notes["truth"], notes["recognized"], notes["matched"]),
                                 (truth_count, recognized_count, matches))
                self.assertEqual(notes["F1"], old["F1"])
                self.assertEqual(result["categories"]["slur_relations"]["matched"], slurs)
                self.assertTrue(result["slur_pairing_valid"])
                self.assertTrue(result["categories"]["pickup_markers"]["equal"])
                self.assertFalse(result["quality_accepted"])
                coverage = result["across_measure_voice_coverage"]
                boundaries, scored, matched = (9, 4, 3) if name == "lc6019054" else (24, 24, 19)
                self.assertEqual(coverage["truth_boundaries"], boundaries)
                self.assertEqual(coverage["recognized_boundaries"], boundaries)
                self.assertEqual(coverage["jointly_scored_boundaries"], scored)
                self.assertEqual(result["categories"]["across_measure_single_stream_successors"]["matched"], matched)
                relations = coverage["available_relation_observations"]
                self.assertEqual(relations["truth_cross_measure_slurs"], 0)
                self.assertEqual(relations["recognized_cross_measure_slurs"], 0)
                self.assertEqual(relations["truth_tie_endpoints"], 0)
                self.assertEqual(relations["recognized_tie_endpoints"], 0)


if __name__ == "__main__":
    unittest.main()
