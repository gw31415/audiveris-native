# SPDX-License-Identifier: AGPL-3.0-or-later
"""Compare saved declared-control observations only; never runs recognition/tests.

Run only after the enclosing native test runner reports success. Native marker
creation by itself is not a pass report for the enclosing test command.
"""
import argparse
import hashlib
import json
from pathlib import Path

COMMON_FIELDS = (
    'rest.removed', 'rest.registered', 'rest.manual',
    'chord.removed', 'chord.registered', 'chord.manual',
    'chord.time_offset', 'chord.member_count', 'chord.same_identity',
    'chord.same_bounds', 'chord.same_voice', 'chord.same_slot',
    'staff_contains_rest', 'measure_contains_chord', 'measure_contains_voice',
    'expected_duration',
    'voice.same_identity', 'voice.id', 'voice.chord_count', 'voice.slot_count',
    'voice.slot_entries_unchanged', 'voice.termination', 'voice.excess',
    'slot.same_identity', 'slot.time_offset', 'slot.incoming_same_chord',
    'containment.registered', 'containment.manual', 'containment.same_identity',
)
MAPPED_FIELDS = (
    ('scalar_measure_excess', 'stack_excess'),
    ('slot.retained_in_measure_timing', 'slot.retained_in_stack'),
)
EXPECTED_EVENTS = (
    ('StaffRemoved', 'RestInter'),
    ('MeasureRemoved', 'RestChordInter'),
    ('EdgeRemoved', 'RestChordInter'),
    ('VertexRemoved', 'RestChordInter'),
    ('VertexRemoved', 'RestInter'),
)
QUALIFICATIONS = [
    'Two declared extensive singleton quarter-rest removal controls, not ordinary fresh-image LINKS clock evidence.',
    'Source has actual reloaded Staff/Part/Measure/Stack owners and null cached Rest/Chord Part fields. Native scalar Measure.geometry=None uses its declared measure index and part_id. Actual Part/Measure/Stack Arc or raw Part-cache parity is not compared.',
    'Absolute source/native glyph bounds and cached Graph bounds differ. Compare preservation within each run; native Graph bounds remain null while its retained typed Chord has declared scalar bounds.',
    'Native Voice/Slot/Staff assertions retain actual Arcs. Native Chord identity means its retained key/id and typed Chord record; it does not claim Java Chord allocation identity.',
    'Containment identity compares preservation independently: source actual Java object, native retained edge id/key/body. Source/native relation IDs and addresses are never joined.',
    'Source SlotVoice and Rational objects are retained. Native SlotVoice and Fraction are values: compare slot chord/status and exact rational values, not Java object identity.',
    'Source has21 method/listener observations per control; native has five callback aftermath observations. previous_observed_state is the preceding recorded boundary, not a method-entry observation.',
    'Native scalar Measure excess is compared only as a value to source Stack excess. Native timing-slot retention is a conceptual container contract, not proof of a native actual Stack owner.',
    'Java rollback is not claimed. Native late-callback transaction rollback is a separate native test contract.',
    'This comparison does not run source/native code or use source observations as native recognition inputs.',
]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def get(value, dotted):
    for key in dotted.split('.'):
        value = value[key]
    return value


def compare(native, source):
    mismatches, controls = [], []
    source_controls = {control['name']: control for control in source['controls']}
    native_controls = {control['name']: control for control in native['controls']}
    if len(source_controls) != len(source['controls']) or len(native_controls) != len(native['controls']):
        raise ValueError('duplicate control names')
    if set(source_controls) != {'underfill', 'excess'} or set(native_controls) != set(source_controls):
        raise ValueError('exact underfill/excess controls required')
    for name in ('underfill', 'excess'):
        n, s = native_controls[name], source_controls[name]
        count = 0
        def check(path, actual, expected):
            nonlocal count
            count += 1
            if type(actual) is not type(expected) or actual != expected:
                mismatches.append({'control': name, 'path': path, 'native': actual, 'source': expected})
        check('declared_expected', n['declared_expected'], s['declared_expected'])
        for label, c in (('native', n), ('source', s)):
            actual = [(e['phase'], e['subject_kind']) for e in c['events']]
            if actual != list(EXPECTED_EVENTS):
                raise ValueError(f'{name} {label} callback order differs: {actual!r}')
        paired = [('prepared', n['prepared'], s['prepared'])]
        paired += [(f'events[{i}]/{sn[0]}/{sn[1]}', ne['state'], se['state'])
                   for i, (sn, ne, se) in enumerate(zip(EXPECTED_EVENTS, n['events'], s['events'], strict=True))]
        paired.append(('after', n['after'], s['after']))
        previous = n['prepared']
        for i, event in enumerate(n['events']):
            check(f'events[{i}].previous_observed_state_continuity', event['previous_observed_state'], previous)
            previous = event['state']
        for phase, ns, ss in paired:
            for field in COMMON_FIELDS:
                check(f'{phase}/{field}', get(ns, field), get(ss, field))
            for nf, sf in MAPPED_FIELDS:
                check(f'{phase}/{nf}~{sf}', get(ns, nf), get(ss, sf))
            # Representation-specific invariants are independently asserted, not
            # turned into a cross-language physical-identity equivalence claim.
            for field in ('measure_geometry_is_none', 'rest.same_actual_Staff',
                          'chord.raw_bounds_unchanged', 'chord.same_raw_staff_id',
                          'chord.same_raw_part_id', 'chord.same_measure_index',
                          'voice.same_measure_index', 'containment.same_body'):
                check(f'{phase}/native_invariant/{field}', get(ns, field), True)
            check(f'{phase}/native_invariant/chord.raw_bounds', ns['chord']['raw_bounds'], None)
            check(f'{phase}/native_slot_status', ns['voice']['slot_entries'],
                  [{'slot_id': 1, 'chord_is_held': True, 'status': 'BEGIN'}])
        check('later_queued_Chord_removal_idempotent', n['later_queued_Chord_removal_idempotent'], True)
        controls.append({'name': name, 'states_compared': len(paired), 'callbacks_compared': len(EXPECTED_EVENTS),
                         'common_fields_per_state': len(COMMON_FIELDS), 'mapped_fields_per_state': len(MAPPED_FIELDS),
                         'checks_including_native_invariants': count})
    return {'controls': controls, 'mismatches': mismatches, 'all_supported_contracts_equal': not mismatches,
            'common_fields': COMMON_FIELDS, 'mapped_fields': MAPPED_FIELDS,
            'representation_qualifications': QUALIFICATIONS}


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--native', type=Path, required=True)
    ap.add_argument('--source', type=Path, default=Path(__file__).with_name('summary.json'))
    ap.add_argument('--output', type=Path, required=True)
    args = ap.parse_args()
    result = compare(json.loads(args.native.read_text()), json.loads(args.source.read_text()))
    result['inputs'] = {role: {'path': str(path), 'sha256': sha(path)}
                        for role, path in (('source', args.source), ('native', args.native))}
    result['comparator_sha256'] = sha(Path(__file__))
    result['native_enclosing_test_success'] = 'Parent must separately verify the enclosing runner RESULT; marker presence alone is insufficient.'
    args.output.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'all_supported_contracts_equal': result['all_supported_contracts_equal'],
                      'mismatches': len(result['mismatches']), 'output': str(args.output)}))
    raise SystemExit(0 if result['all_supported_contracts_equal'] else 1)


if __name__ == '__main__':
    main()
