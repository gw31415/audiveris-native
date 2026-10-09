"""Read the already-generated native MusicXML; no image or expected-data input."""
import json
import sys
import xml.etree.ElementTree as ET

root = ET.fromstring(sys.stdin.buffer.read())
assert root.tag == "score-partwise", root.tag
notes = []
for part in root.findall("part"):
    identifier = part.attrib["id"]
    assert identifier.startswith("P"), identifier
    part_id = int(identifier[1:])
    for physical_measure, measure in enumerate(part.findall("measure"), 1):
        for index, note in enumerate(measure.findall("note")):
            pitch = note.find("pitch")
            notes.append({
                "part": part_id,
                "physical_measure": physical_measure,
                "measure_score_id": measure.attrib["number"],
                "note_index": index,
                "voice": int(note.findtext("voice")),
                "staff_number": int(note.findtext("staff") or 1),
                "pitch": {child.tag: child.text for child in pitch} if pitch is not None else None,
                "slurs": [dict(slur.attrib) for slur in note.findall("notations/slur")],
            })
assert len(notes) == len(list(root.iter("note")))
assert sum(len(note["slurs"]) for note in notes) == len(list(root.iter("slur")))
json.dump(notes, sys.stdout)
