"""Read existing proofs and write only this /tmp evidence plan/projections."""
from pathlib import Path
import copy
import hashlib
import json

OUT = Path(__file__).resolve().parent
TMP = Path('/tmp')
PIN = '9e1e55cd2746037d059345881c53e6a6754bffbd'
assert OUT == TMP / 'slur-checkpoint-evidence-plan-v1'

def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()

def read(path):
    return json.loads(Path(path).read_text())

def write(name, value, compact=False):
    text = json.dumps(value, ensure_ascii=False, separators=(',', ':')) if compact else json.dumps(value, ensure_ascii=False, indent=2)
    (OUT / name).write_text(text + '\n')

def reference(path):
    path = Path(path)
    return {'path': str(path), 'sha256': sha(path), 'bytes': path.stat().st_size}

families = {
    'fit': 'native-slur-math-original-v1',
    'raw': 'native-slur-raw-gap-refinement-v2',
    'band': 'slur-band-routing-original-v1',
    'lookup-controls': 'slur-owned-lookup-original-v2',
    'lookup-native-curves': 'slur-native-curve-lookup-original-v1',
    'clump': 'slur-native-clump-original-v1',
    'callbacks': 'slur-fresh-callbacks-original-v1',
}

# This is a lossless encoding of all source Band input/output decisions; no native
# function is evaluated to produce expectations.
band = TMP / families['band']
inputs = read(band / 'inputs.json')
observed = read(band / 'observed.json')
queries = inputs[0]['queries']
assert len(inputs) == 8 and len(queries) == len(observed['rows']) == 1570
assert all(b['queries'] == queries for b in inputs)
band_rows = []
for index, row in enumerate(observed['rows']):
    assert row['query'] == index
    band_rows.append([queries[index], row['area_hits'], row['route'], row['reused_same_list'], row['reused_route']])
write('band-source-fixture.json', {
    'source_pin': PIN,
    'sources': [reference(band / 'inputs.json'), reference(band / 'observed.json')],
    'scope': 'All 8 generic Bands and 1570 original Java query rows. Band construction/routing proof, not image owner parity.',
    'bands': [{k: v for k, v in b.items() if k != 'queries'} for b in inputs],
    'row_schema': ['rectangle', 'area_hits_in_band_order', 'system_route', 'reused_same_list', 'reused_route'],
    'rows': band_rows,
}, compact=True)

# Keep source lookup/chord/pair semantics without copying dense geometry-query grids.
# Source IDs here are observer labels, never production selector inputs.
owner_sets, owner_lookup, lookup_rows, lookup_sources = [], {}, [], []
for family in ['lookup-controls', 'lookup-native-curves']:
    original = TMP / families[family] / 'observed.json'
    lookup_sources.append(reference(original))
    data = read(original)
    for original_row in data['rows']:
        row = copy.deepcopy(original_row)
        chords = row.pop('chords')
        canonical = json.dumps(chords, sort_keys=True, separators=(',', ':'))
        if canonical not in owner_lookup:
            owner_lookup[canonical] = len(owner_sets)
            owner_sets.append(chords)
        row['source_chords_ref'] = owner_lookup[canonical]
        row['source_family'] = family
        for area in row['areas']:
            area['omitted_dense_query_count'] = len(area.pop('queries'))
        lookup_rows.append(row)
write('lookup-source-semantic-fixture.json', {
    'source_pin': PIN, 'sources': lookup_sources,
    'scope': '43 original lookup/chord/pair rows; dense Area query grids deliberately omitted. Aggregate full-grid comparison counts are historical source proofs, not assertions executed from this projection.',
    'source_chord_sets': owner_sets, 'rows': lookup_rows,
}, compact=True)

verified = []
for family, folder in families.items():
    manifest = TMP / folder / 'MANIFEST.json'
    if not manifest.exists():
        continue
    data = read(manifest)
    rows = data if isinstance(data, list) else data['files']
    entry = {'manifest': reference(manifest), 'entries': len(rows), 'mismatches': []}
    for row in rows:
        path = Path(row['path'])
        if not path.is_absolute():
            path = manifest.parent / path
        actual = sha(path) if path.exists() else None
        if actual != row['sha256']:
            entry['mismatches'].append({'path': str(path), 'recorded_sha256': row['sha256'], 'current_sha256': actual, 'classification': 'mutable build artifact' if '/target/' in str(path) else 'requires investigation'})
    verified.append(entry)
write('original-manifest-verification.json', verified)

plan, seen = [], set()
def add(src, dest, purpose, action='copy'):
    src = Path(src)
    assert src.exists(), str(src)
    assert dest not in seen, dest
    seen.add(dest)
    plan.append({**reference(src), 'destination_relative': dest, 'action': action, 'purpose': purpose})

for family, folder in families.items():
    for name in ['README.md', 'PROVENANCE.json', 'provenance.json', 'RUST-PROVENANCE.json', 'MANIFEST.json']:
        src = TMP / folder / name
        if src.exists():
            add(src, 'source/' + family + '/' + name, 'Preserve original provenance and historical scope unchanged.')
    name = 'rust-comparison.json' if family in ['fit', 'raw'] else 'summary.json' if family == 'callbacks' else 'comparison.json'
    add(TMP / folder / name, 'source/' + family + '/' + name, 'Actual recorded comparison result, not a new execution.')

for name in ['band-source-fixture.json', 'lookup-source-semantic-fixture.json']:
    add(OUT / name, 'source/projected/' + name, 'Deterministic original-only projection; full originals remain referenced by hash.')
for family, filename in [('clump', 'observed.json'), ('clump', 'selection-inputs.tsv'), ('clump', 'component-inputs.tsv'), ('clump', 'groups.tsv'), ('raw', 'observed.json'), ('raw', 'sequences.tsv')]:
    add(TMP / families[family] / filename, 'source/' + family + '/' + filename, 'Small exact original observation or actual candidate input.')
for name in ['observed.json', 'gap-area-observed.json', 'provenance.json', 'input-provenance.json']:
    add(TMP / 'native-slur-binary-support-v1' / name, 'source/raw-baseline/' + name, 'Strict original BINARY/gap/run evidence; raw image buffers are excluded.')

scripts = ['NativeSlurCandidateMathProbe.java', 'NativeSlurBinarySupportProbe.java', 'NativeSlurRawGapSupportProbe.java', 'SlurBandRoutingProbe.java', 'SlurOwnedLookupProbeV2.java', 'SlurNativeCurveLookupProbe.java', 'SlurNativeClumpProbe.java', 'run-native-slur-math.py', 'run-slur-raw-gap-support-v2.py', 'run-slur-band-routing.py', 'run-slur-owned-lookup-v2.py', 'run-slur-native-curve-lookup.py', 'run-slur-native-clump.py', 'compare-rust-slur-math.py', 'compare_slur_raw_support_actual.py', 'compare_slur_owned_lookup.py', 'compare_slur_native_curve_lookup.py', 'compare_slur_clump_selection.py']
for name in scripts:
    add(TMP / name, 'source/harness/' + name, 'Reproduction source; historical absolute paths require deliberate adaptation, never auto-run during copy.')
for name in ['SlurLifecycleObserver.java', 'SlurLifecycleLoader.java', 'SlurLifecycleRewriter.java', 'VerifyObservationOverlay.java', 'run.py', 'check.py', 'instrumentation.json', 'instrumentation-audit.json', 'setup-attempts.json']:
    add(TMP / families['callbacks'] / name, 'source/callbacks/' + name, 'Original resumed-CURVES observation and audited instruction-only hooks.')
for name in ['observed.json', 'PROVENANCE.json']:
    add(TMP / 'slur-native-integration-v2/source-fixture' / name, 'fixtures/official-slur-fresh-lifecycle/' + name, 'Already adopted callback fixture; verify existing bytes, do not duplicate under evidence.', 'verify_existing')
for name in ['slur-fit-source-hashes.json', 'slur-jama-operation-order-hashes.json', 'short-slur-run-gap-source-hashes.json', 'slur-point-extraction-source-hashes.json', 'slur-linker-proposal-hashes.json', 'short-slur-proposal-hashes.json', 'short-slur-raw-support-hashes.json']:
    add(TMP / name, 'source/references/' + name, 'Exact frozen source/kernel hashes; do not replace them with current build hashes.')
for name in ['prepare.py', 'original-manifest-verification.json']:
    add(OUT / name, name, 'Plan/projection reproduction or explicit historical-manifest qualification.')
write('copy-plan.json', {'source_pin': PIN, 'destination_policy': 'All destination_relative except fixtures are under a root-chosen evidence directory. No files copied to checkout by this plan.', 'copy_count': sum(r['action'] == 'copy' for r in plan), 'copy_bytes': sum(r['bytes'] for r in plan if r['action'] == 'copy'), 'entries': plan})

retained = []
for folder in families.values():
    for name in ['observed.json', 'inputs.json', 'rust-observed.json', 'console.log']:
        f = TMP / folder / name
        if f.exists():
            retained.append(reference(f))
write('retained-full-artifacts.json', {'policy': 'Remain at original paths unchanged. No archives, raw image buffers, build binaries, or repeated dense native outputs included in copy plan.', 'artifacts': retained})
print(json.dumps({'copy_files': sum(r['action'] == 'copy' for r in plan), 'copy_bytes': sum(r['bytes'] for r in plan if r['action'] == 'copy'), 'band_fixture_bytes': (OUT/'band-source-fixture.json').stat().st_size, 'lookup_fixture_bytes': (OUT/'lookup-source-semantic-fixture.json').stat().st_size, 'manifest_mismatch_count': sum(len(v['mismatches']) for v in verified)}))
