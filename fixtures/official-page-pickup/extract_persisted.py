#!/usr/bin/env python3
"""Extract original persisted PAGE state; never run recognition or alter a book."""
import argparse
import hashlib
import json
import xml.etree.ElementTree as ET
import zipfile
from pathlib import Path


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--book", type=Path, required=True)
    parser.add_argument("--xml", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    before = digest(args.book)
    with zipfile.ZipFile(args.book) as archive:
        book = ET.fromstring(archive.read("book.xml"))
        sheet = ET.fromstring(archive.read("sheet#1/sheet#1.xml"))
    pages = sheet.findall("page")
    refs = book.findall("./sheet/page")
    if len(pages) != 1 or len(refs) != 1:
        raise ValueError("single original page required")
    rows = []
    for system in pages[0].findall("system"):
        stacks = system.findall("stack")
        parts = system.findall("part")
        if any(len(p.findall("measure")) != len(stacks) for p in parts):
            raise ValueError("actual Part/Stack membership required")
        for index, stack in enumerate(stacks):
            if any(p.findall("measure")[index].get("id") != stack.get("id") for p in parts):
                raise ValueError("serialized Measure/Stack identity differs")
            rows.append({
                "system": int(system.attrib["id"]),
                "stack_index": index + 1,
                "number": int(stack.attrib["id"]),
                "implicit": stack.get("special") == "PICKUP",
                "special": stack.get("special"),
                "expected": stack.get("expected"),
                "actual": stack.get("duration"),
                "excess": stack.get("excess"),
                "parts": [int(p.attrib["id"]) for p in parts],
            })
    for part in ET.parse(args.xml).getroot().findall("part"):
        measures = part.findall("measure")
        if len(measures) != len(rows):
            raise ValueError("exported measure count differs")
        for measure, row in zip(measures, rows):
            if (measure.get("number"), measure.get("implicit", "no") == "yes") != (
                str(row["number"]), row["implicit"]
            ):
                raise ValueError("exported PAGE measure identity differs")
    result = {
        "source_commit": book.attrib["software-build"],
        "original_book_sha256": before,
        "original_XML_sha256": digest(args.xml),
        "saved_PAGE_book_state": True,
        "runtime_termination_callback_observed": False,
        "native_input": False,
        "qualification": "Book attributes are original persisted PAGE owner state. Voice termination is not serialized; XML endpoints are separately derived diagnostics, not original runtime getter observations and not native inputs.",
        "page_measure_count": int(pages[0].attrib["measure-count"]),
        "delta_measure_id": int(refs[0].attrib["delta-measure-id"]),
        "stacks": rows,
    }
    if digest(args.book) != before:
        raise ValueError("original book changed")
    with args.output.open("x") as out:
        out.write(json.dumps(result, indent=2) + "\n")


if __name__ == "__main__":
    main()
