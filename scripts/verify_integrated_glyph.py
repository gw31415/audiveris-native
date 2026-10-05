#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Compare isolated binary image -> root CLI -> features/scores to Java goldens."""
import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
exe = ROOT / (sys.argv[1] if len(sys.argv) > 1 else 'target/debug/audiveris-native')
crate = ROOT / 'crates/audiveris-glyph-features'
golden_path = crate / 'tests/fixtures/java-glyph-golden.json'
golden_hash = hashlib.sha256(golden_path.read_bytes()).hexdigest()
assert golden_hash == '49fdc7c89af9df04879759b5892534afd66495e1fa46fae8279ad48a9ffafbd3'
golden = json.loads(golden_path.read_text())
model = ROOT / 'crates/audiveris-classifier/models/audiveris-5.11.0.json'
labels = json.loads((ROOT / 'crates/audiveris-classifier/tests/fixtures/java-5.11.0-golden.json').read_text())['output_labels']
work = ROOT / 'evidence/generated/glyph-cli'
work.mkdir(parents=True, exist_ok=True)
pgm = work / 'glyph.pgm'
maximum = dict.fromkeys(['raw_features', 'normalized', 'hidden', 'scores'], 0.0)
count = 0
for case in golden['cases']:
    g = case['input']
    pixels = bytes(0 if p == '#' else 255 for row in g['bitmap_rows'] for p in row)
    pgm.write_bytes(f"P5\n{g['width']} {g['height']}\n255\n".encode() + pixels)
    args = [str(exe), 'classify-glyph', str(model), str(pgm), str(g['interline']), g['orientation'].lower(), str(g['left']), str(g['top'])]
    actual = json.loads(subprocess.check_output(args))
    assert actual['schema'] == 'audiveris-native.isolated-glyph.v1'
    assert actual['origin'] == [g['left'], g['top']]
    assert actual['input_labels'] == golden['input_labels']
    assert actual['class_labels'] == labels
    for field in maximum:
        assert len(actual[field]) == len(case[field])
        for got, expected in zip(actual[field], case[field]):
            delta = abs(got - expected)
            maximum[field] = max(maximum[field], delta)
            assert delta <= 1e-12 + 1e-10 * abs(expected), (case['id'], field, got, expected)
            count += 1
    rank = sorted(range(149), key=lambda i: (-actual['scores'][i], i))[:5]
    assert rank == [v['index'] for v in case['top5']]

invalid = [(b'\x80', '8', 'vertical'), (b'\xff', '8', 'vertical'), (b'\x00', '0', 'vertical'), (b'\x00', '8', 'diagonal')]
for pixel, interline, orientation in invalid:
    pgm.write_bytes(b'P5\n1 1\n255\n' + pixel)
    result = subprocess.run([str(exe), 'classify-glyph', str(model), str(pgm), interline, orientation], capture_output=True)
    assert result.returncode == 1 and not result.stdout
report = {'cases': len(golden['cases']), 'cli_values_compared': count,
          'max_absolute_error': maximum, 'absolute_tolerance': 1e-12,
          'relative_tolerance': 1e-10, 'top5_order_match': True,
          'invalid_input_cases': len(invalid), 'root_cli_error_exit': 1,
          'source_commit': '225d081c888086e8617be920e3f3ca1bc1f9e4b2',
          'golden_sha256': golden_hash, 'profile': str(exe.relative_to(ROOT)),
          'scope': 'isolated binary glyph with explicit interline; no page segmentation or recognition accuracy claim'}
(ROOT / 'evidence/glyph-integration.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps(report, indent=2))
