#!/usr/bin/env python3
"""Project existing actual Java observations; never execute Java or native kernels."""
import hashlib
import json
from pathlib import Path

SOURCE = Path('/tmp/native-slur-math-original-v1')
OUT = Path(__file__).resolve().parent
PIN = '9e1e55cd2746037d059345881c53e6a6754bffbd'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
original = json.loads((SOURCE / 'observed.json').read_text())
provenance = json.loads((SOURCE / 'provenance.json').read_text())
assert provenance['source_pin'] == PIN
assert sha(SOURCE / 'observed.json') == provenance['observed_sha256']
assert sha(SOURCE / 'sequences.tsv') == provenance['sequences_sha256']
assert provenance['exit_code'] == 0
assert not provenance['image_recognition_invoked']
assert not provenance['ordinary_lifecycle_claimed']
sequences = {}
for line in (SOURCE / 'sequences.tsv').read_text().splitlines():
    index, kind, encoded = line.split('\t')
    sequences[int(index)] = (kind, [[int(v) for v in p.split(',')] for p in encoded.split(';')])
assert len(sequences) == len(original['rows']) == 157
circle_keys = ('center', 'radius', 'distance', 'ccw', 'above', 'first_angle', 'last_angle')
def scalar_text(value):
    # Preserve original double values independently of serde_json's float-parser features.
    return value if isinstance(value, str) else repr(value)

def project_circle(circle):
    if circle is None:
        return None
    return {
        key: [scalar_text(v) for v in circle[key]] if key == 'center'
        else circle[key] if key in ('ccw', 'above')
        else scalar_text(circle[key])
        for key in circle_keys
    }

rows = []
for index, row in enumerate(original['rows']):
    assert row['candidate_index'] == index
    assert (row['kind'], row['points']) == sequences[index]
    circle = row.get('selected_after_impacts')
    assert (circle is not None) == (row['compute_model_return'] == 'CircleModel')
    assert ('grade' in row) == (row['compute_impacts_return'] == 'Impacts')
    rows.append({
        'id': index,
        'kind': row['kind'],
        'points': row['points'],
        'circle': project_circle(circle),
        'grade': None if 'grade' not in row else scalar_text(row['grade']),
        'bezier': None if row['bezier'] is None else [[scalar_text(v) for v in point] for point in row['bezier']],
        'accepted': row['math_accepted_before_glyph_and_linkage'],
    })
fixture = {
    'schema': 'audiveris.short-slur-fit.java-observation.v1',
    'source_pin': PIN,
    'interline': original['interline'],
    'native_sequences': 153,
    'controls': 4,
    'tolerances': {'center_radius_distance': 0.0, 'angles_bezier_grade': 1e-12},
    'rows': rows,
}
header = json.dumps({k: v for k, v in fixture.items() if k != 'rows'}, indent=2, allow_nan=False)
text = header[:-2] + ',\n  "rows": [\n'
text += ',\n'.join('    ' + json.dumps(row, separators=(',', ':'), allow_nan=False) for row in rows)
text += '\n  ]\n}\n'
(OUT / 'source-observations.json').write_text(text)
assert json.loads(text) == fixture
source_meta = {
    'scope': 'Projection of existing actual Java observations onto the currently exposed Rust Fit fields. No original/native execution or numerical regeneration.',
    'source_pin': PIN,
    'original_observed_sha256': sha(SOURCE / 'observed.json'),
    'original_sequences_sha256': sha(SOURCE / 'sequences.tsv'),
    'original_provenance_sha256': sha(SOURCE / 'provenance.json'),
    'original_jar_sha256': provenance['jar_sha256'],
    'original_observer_sha256': provenance['observer_sha256'],
    'original_runner_sha256': provenance['runner_sha256'],
    'original_book_sha256': provenance['original_book_sha256'],
    'original_native_exploration_sha256': provenance['native_exploration_sha256'],
    'source_observation_scope': original['scope'],
    'source_builder_instrumentation': original['builder_instrumentation'],
    'source_kernel_reviewed': 'src/music_slurs/fit.rs',
    'source_kernel_reviewed_sha256': sha(Path('/workspace/audiveris-native/src/music_slurs/fit.rs')),
    'projected_fixture_sha256': sha(OUT / 'source-observations.json'),
    'row_count': len(rows),
    'single_count': sum(r['kind'] == 'single' for r in rows),
    'pair_count': sum(r['kind'] == 'pair' for r in rows),
    'control_count': sum(r['kind'].startswith('control_') for r in rows),
    'selected_model_count': sum(r['circle'] is not None for r in rows),
    'null_model_count': sum(r['circle'] is None for r in rows),
    'impacts_count': sum(r['grade'] is not None for r in rows),
    'bezier_count': sum(r['bezier'] is not None for r in rows),
    'accepted_count': sum(r['accepted'] for r in rows),
    'accepted_native_count': sum(r['accepted'] for r in rows if not r['kind'].startswith('control_')),
    'special_numeric_encoding': 'Original NaN/Infinity/-Infinity strings are retained unchanged; null remains absence.',
    'finite_numeric_encoding': 'Roundtrip decimal strings of existing original JSON doubles, parsed with Rust str::parse::<f64>; no math recomputation and no dependence on serde_json float-parser features.',
    'expectations_from_numpy_or_rust': False,
    'all_inputs_match_existing_sequences_tsv': True,
    'rust_test_executed_in_this_unit': False,
}
(OUT / 'SOURCE.json').write_text(json.dumps(source_meta, indent=2, allow_nan=False) + '\n')
print(json.dumps({
    'rows': len(rows),
    'fixture_bytes': (OUT / 'source-observations.json').stat().st_size,
    'fixture_sha256': sha(OUT / 'source-observations.json'),
    'model_count': source_meta['selected_model_count'],
    'accepted_count': source_meta['accepted_count'],
}))
