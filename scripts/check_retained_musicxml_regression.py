#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Check retained user CLI XML bytes and MusicXML 4 XSD in a fresh output directory."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
from lxml import etree

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--native-bin", type=Path, required=True)
parser.add_argument("--schema-dir", type=Path, required=True)
parser.add_argument("--output", type=Path, required=True)
args = parser.parse_args()
output = args.output.resolve()
output.mkdir()
schema_dir = args.schema_dir.resolve()


class LocalImports(etree.Resolver):
    def resolve(self, url, public, context):
        name = url.rsplit("/", 1)[-1]
        if name in ("xml.xsd", "xlink.xsd"):
            return self.resolve_filename(str(schema_dir / name), context)


xml_parser = etree.XMLParser(no_network=True)
xml_parser.resolvers.add(LocalImports())
schema = etree.XMLSchema(etree.parse(str(schema_dir / "musicxml.xsd"), xml_parser))
saved = json.loads((ROOT / "evidence/musicxml-headstem-prelink-existing/VALIDATION.json").read_text())["cli_regression"]
results = []
cases = saved["automatic_inputs"] + [dict(saved["declared_correction"],
    input="fixtures/official-image-musicxml-augmentation/double-augmentation.png")]
for index, case in enumerate(cases):
    xml = output / f"case{index}.musicxml"
    report = output / f"case{index}.json"
    argv = [str(args.native_bin.resolve())]
    if index == 6:
        argv += ["--link-dot-support", "72", "6", "manual", "--edit-dot", "72", "3", "0"]
    argv += [str(ROOT / case["input"]), str(xml), str(report)]
    run = subprocess.run(argv, capture_output=True, text=True)
    (output / f"case{index}.log").write_text(run.stdout + run.stderr)
    assert run.returncode == 0, f"CLI case {index} failed; see local log"
    sha = hashlib.sha256(xml.read_bytes()).hexdigest()
    assert sha == case["XML_sha256"], f"XML bytes changed in case {index}"
    observed = json.loads(report.read_text())
    assert len(observed["events"]) == case["events"]
    assert observed["musicxml_complete"] is False
    assert observed["automatic_raster_pipeline_complete"] is False
    schema.assertValid(etree.parse(str(xml)))
    results.append(dict(case, XML_sha256=sha, previous_XML_bytes="identical", XSD="passed"))
record = {"automatic_inputs": results[:6], "automatic_events": sum(r["events"] for r in results[:6]),
          "declared_correction": results[6], "XSD_sha256": {
              p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in schema_dir.glob("*.xsd")},
          "checker_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
(output / "RESULTS.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({"automatic_events": record["automatic_events"], "declared_events": results[6]["events"],
                  "XML_bytes": "identical", "XSD": "passed"}))
