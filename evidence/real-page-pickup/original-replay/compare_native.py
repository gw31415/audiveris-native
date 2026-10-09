#!/usr/bin/env python3
"""Compare actual native PAGE results to bounded original saved-owner replay.

Reads observations only. Output is evidence, never a native recognition input.
No direct cross-JVM/Arc identity comparison or native intermediate trace is claimed.
"""
import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()

def compare(original, native):
    required = ('PAGE_measures', 'after_PAGE', 'after_rhythm',
                'PAGE_actual_Voice_Slot_Chord_Measure_Stack_identities_retained',
                'declared_new_generation_rejects_old_PAGE_without_erasing_metadata',
                'declared_late_missing_Stack_timing_keeps_previous_PAGE_and_clocks')
    absent = [key for key in required if key not in native]
    if absent:
        raise ValueError('Native PAGE observation incomplete; missing: ' + ', '.join(absent))
    failures, rows = [], []
    def equal(label, actual, expected):
        if actual != expected:
            failures.append({'contract': label, 'native': actual, 'original_or_expected': expected})
    for key in required[3:]:
        equal(key, native[key], True)
    equal('recognition-inputs-not-source-truth', native['source_or_truth_used_as_recognition_input'], False)
    equal('quality-remains-unaccepted', native['quality_accepted'], False)
    equal('PAGE-does-not-change-actual-Graph', native['after_PAGE']['graph'] == native['after_rhythm']['graph'], True)
    equal('PAGE-does-not-change-Stack-clocks-or-Slots', native['after_PAGE']['stacks'] == native['after_rhythm']['stacks'], True)
    for replay in original['replays']:
        system, index = replay['system'], replay['stack_index']
        native_rows = [r for r in native['PAGE_measures']['stacks'] if (r['system'], r['stack_index']) == (system, index)]
        if len(native_rows) != 1:
            raise ValueError(f'Expected one native Stack row for {(system, index)}, got {len(native_rows)}')
        row = native_rows[0]
        first, final = replay['frames'][0], replay['frames'][-1]
        prefix = f'System{system}/Stack{index}'
        expected_terms = [v['termination'] for m in final['peer_measures'] for v in m['voices']]
        for key in ('expected', 'actual', 'excess'):
            equal(prefix + '/' + key, row[key], final[key])
            equal(prefix + '/before-' + key, row['before'][key], first[key])
        equal(prefix + '/ordered-Voice-terminations', row['terminations'], expected_terms)
        equal(prefix + '/common-termination', row['common_termination'], replay['common_termination'])
        if replay['selected_pickup_branch_executed']:
            for key in ('number', 'implicit'):
                equal(prefix + '/selected-original-pickup-' + key, row[key], final[key])
        peer_rows = []
        for source_measure in final['peer_measures']:
            part = source_measure['part']
            native_measures = [m for m in native['after_PAGE']['measures'] if (m['system'], m['index'], m['part']) == (system, index, part)]
            if len(native_measures) != 1:
                raise ValueError(f'Expected one actual native Measure view for {(system, index, part)}')
            nm = native_measures[0]
            for key in ('expected', 'actual', 'excess', 'abnormal'):
                equal(prefix + f'/Part{part}/' + key, nm[key], source_measure[key])
            nv = [v for v in native['after_PAGE']['voices'] if (v['system_id'], v['measure_index'], v['part_id']) == (system, index, part)]
            sv = source_measure['voices']
            equal(prefix + f'/Part{part}/ordered-Voice-state',
                  [{k: v[k] for k in ('id', 'termination', 'excess')} for v in nv],
                  [{k: v[k] for k in ('id', 'termination', 'excess')} for v in sv])
            peer_rows.append({'part': part, 'voice_count': len(sv), 'shared_excess': nm['excess']})
        rows.append({'system': system, 'stack_index': index,
                     'original_reloaded_before_terminations': [v['termination'] for m in first['peer_measures'] for v in m['voices']],
                     'native_live_before_terminations': row['before']['terminations'],
                     'compared_after_terminations': expected_terms,
                     'common_termination': replay['common_termination'], 'peer_measures': peer_rows,
                     'pickup_branch_compared': replay['selected_pickup_branch_executed']})
    return {'status': 'matched' if not failures else 'different', 'compared_stack_count': len(rows),
            'rows': rows, 'differences': failures,
            'qualification': [
                'Original is selected-method replay of two saved post-PAGE Stacks, not fresh recognition or the full PAGE step.',
                'Original before termination/kind/startingStaff are reload transients; native live post-RHYTHMS before terminations are reported separately, not required to be null.',
                'Original has before/after each Voice check and all peer views. Native report provides aggregate PAGE rows plus runtime owner-retention assertions; native intermediate callbacks are not compared by this script.',
                'Source control branches are retained evidence; this comparison does not execute declared controls on native owners.',
                'Object identity is checked within each execution. Integer object tokens are never compared across runtimes.',
                'Only the first Stack executes the selected original numbering branch; Stack3 numbering is covered by separate persisted-PAGE evidence.'
            ], 'quality_accepted': False}

def main():
    p = argparse.ArgumentParser()
    p.add_argument('--native', type=Path, required=True)
    p.add_argument('--original', type=Path, default=ROOT / 'summary.json')
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    try:
        result = compare(json.loads(a.original.read_text()), json.loads(a.native.read_text()))
        code = 0 if result['status'] == 'matched' else 1
    except (KeyError, TypeError, ValueError, OSError) as error:
        result = {'status': 'observation_incomplete', 'error': str(error), 'quality_accepted': False}
        code = 2
    result['inputs'] = [{'path': str(path), 'sha256': sha(path)} for path in (a.original, a.native) if path.exists()]
    a.output.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'status': result['status'], 'output': str(a.output), 'difference_count': len(result.get('differences', []))}))
    return code

if __name__ == '__main__':
    raise SystemExit(main())
