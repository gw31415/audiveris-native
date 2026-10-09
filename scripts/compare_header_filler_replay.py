#!/usr/bin/env python3
"""Compare observations only; never supply ownership or recognition input."""
import argparse
import hashlib
import json
from pathlib import Path


def require(test, label):
    if not test:
        raise ValueError(label)


def load(path):
    return json.loads(Path(path).read_text())


def source_contract(source):
    require(source['original_state_restored'] and not source['errors'], 'source replay/restoration failed')
    systems, additions, initial, final, exclusions = [], [], [], [], []
    for system in source['systems']:
        sid = system['system']
        selected = {header[k] for header in system['initial']['headers'] for k in ('clef', 'key') if header[k] is not None}
        descriptors = {}
        for row in system['after_direct_calls']:
            for kind in ('clefs', 'keys'):
                for inter in row[kind]:
                    if inter['id'] in selected:
                        require(inter['raw_part'] == row['measure']['part'], 'source actual Part differs')
                        descriptors[inter['id']] = dict(kind=inter['kind'], bounds=inter['raw_bounds'], staff=inter['raw_staff'], part=inter['raw_part'])
        require(set(descriptors) == selected, 'source selected Header member not present in final actual collection')
        def peers(rows):
            return [dict(system=sid, part=row['measure']['part'], measure=row['measure']['local_measure_index'], stack=row['measure']['stack_index'],
                         clefs=[descriptors[x['id']] for x in row['clefs'] if x['id'] in selected],
                         keys=[descriptors[x['id']] for x in row['keys'] if x['id'] in selected]) for row in rows]
        filling = [descriptors[x['id']] for x in system['filling_collection'] if x['id'] in selected]
        exclusions += [dict(system=sid, source_id=x['id'], kind=x['kind'], bounds=x['raw_bounds'], reason='actual nonheader Inter outside selected-header prerequisite') for x in system['filling_collection'] if x['id'] not in selected]
        stacks = []
        pending = None
        for frame in system['trace']:
            stage = frame['stage']
            if stage == 'after_declared_Header_only_membership_clear':
                initial.extend(peers(frame['all_peer_memberships']))
            if stage == 'original_Stack_filter_completed_before_additions':
                stacks.append(dict(stack=frame['stack_index'], kept=[descriptors[x['id']] for x in frame['kept'] if x['id'] in selected]))
            if stage == 'before_original_Measure_addInter' and frame['inter']['id'] in selected:
                require(pending is None, 'source overlapping add frames')
                pending = frame
            if stage == 'after_original_Measure_addInter' and frame['inter']['id'] in selected:
                require(pending is not None and pending['inter']['object'] == frame['inter']['object'], 'source add identity differs')
                require(pending['measure'] == frame['measure'], 'source add Measure identity differs')
                m = frame['measure']
                before, after = peers(pending['all_peer_memberships']), peers(frame['all_peer_memberships'])
                target_before = next(x for x in before if x['part'] == m['part'] and x['measure'] == m['local_measure_index'])
                target_after = next(x for x in after if x['part'] == m['part'] and x['measure'] == m['local_measure_index'])
                additions.append(dict(system=sid, stack=m['stack_index'], inter=descriptors[frame['inter']['id']], part=m['part'], measure_index=m['local_measure_index'],
                                      before_clefs=target_before['clefs'], after_clefs=target_after['clefs'], before_keys=target_before['keys'], after_keys=target_after['keys'],
                                      all_peer_memberships_before=before, all_peer_memberships_after=after))
                pending = None
        require(pending is None, 'source incomplete add frames')
        require(all(system[k] for k in ('all_collection_objects_retained', 'all_intermediate_SIG_manual_identity_and_clocks_unchanged', 'ordinary_process_idempotent_after_direct_source_calls')), 'source intermediate assertion failed')
        systems.append(dict(system=sid, filling=filling, stacks=stacks))
        final.extend(peers(system['after_direct_calls']))
    return dict(systems=systems, additions=additions, initial=initial, final=final, source_excluded_nonheaders=exclusions)


def native_contract(native):
    require(native['actual_Measure_Part_Stack_Arcs_retained'], 'native actual owner identity assertion failed')
    require(native['graph_data_and_cached_Chords_unchanged'], 'native graph/cache assertion failed')
    require(native['second_call_same_membership'], 'native idempotence assertion failed')
    require(len(native['systems']) == len(native['stages']), 'native missing system stage')
    additions = []
    encode = lambda x: json.dumps(x, sort_keys=True, separators=(',', ':'))
    for system, stage in zip(native['systems'], native['stages']):
        require(system['system'] == stage['system'], 'native system stage order differs')
        require(len(stage['filling_keys']) == len(system['filling']), 'native key/descriptor count differs')
        by_key = dict(zip(map(encode, stage['filling_keys']), system['filling']))
        require(len(by_key) == len(stage['filling_keys']), 'native duplicate filling identity')
        for raw in stage['additions']:
            row = dict(raw, system=stage['system'])
            row['inter'] = by_key[encode(row['inter'])]
            for field in ('before_clefs', 'after_clefs', 'before_keys', 'after_keys'):
                row[field] = [by_key[encode(key)] for key in row[field]]
            require('all_peer_memberships_before' in row and 'all_peer_memberships_after' in row, 'native actual all-peer observation missing (not reconstructed)')
            additions.append(row)
    return dict(systems=native['systems'], additions=additions, initial=native['before'], final=native['after'])


def compare(expected, actual):
    fields = ('systems', 'additions', 'initial', 'final')
    return {field: expected[field] == actual[field] for field in fields}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--source', required=True)
    parser.add_argument('--native')
    parser.add_argument('--links')
    parser.add_argument('--legacy-source-fixture')
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    source = source_contract(load(args.source))
    result = dict(scope='Header-only actual-owner observations; original saved SYMBOLS afterReload declared clear/replay, not fresh first-fill recognition. Source nonheader remains held but is outside this prerequisite. Descriptors join independent observations only; no owner/clock/recognition reconstruction.',
                  source_sha256=hashlib.sha256(Path(args.source).read_bytes()).hexdigest(), source_systems=len(source['systems']), source_selected_additions=len(source['additions']), source_measures=len(source['final']),
                  source_excluded_nonheaders=source['source_excluded_nonheaders'],
                  source_clock_scope='Actual held Chord 7/64 and shared Stack expected 1/2 are declared source controls. No nonempty Voice/Slot behavior or same-input native clock parity claimed.',
                  real_quality_accepted=False, native_comparison_performed=False)
    if args.legacy_source_fixture:
        legacy = load(args.legacy_source_fixture)
        result['source_replay_matches_legacy_source_final_membership'] = source['final'] == legacy['final_memberships']
        result['source_replay_matches_legacy_source_filling_filter_order'] = source['systems'] == legacy['systems']
        require(result['source_replay_matches_legacy_source_final_membership'] and result['source_replay_matches_legacy_source_filling_filter_order'], 'old and new source observations differ')
    passed = True
    if args.native:
        native = native_contract(load(args.native))
        result['native_comparison_performed'] = True
        result['native_sha256'] = hashlib.sha256(Path(args.native).read_bytes()).hexdigest()
        result['checks'] = compare(source, native)
        passed = all(result['checks'].values())
        if not passed:
            result['mismatches'] = {key: dict(source=source[key], native=native[key]) for key, equal in result['checks'].items() if not equal}
    if args.links:
        require(args.native is not None, '--links requires --native')
        links = load(args.links)
        result['ordinary_links_actual_owner_assertion'] = links['actual_Measure_Part_Stack_Arcs_retained']
        result['ordinary_links_header_membership_matches_source'] = links['actual_memberships'] == source['final']
        passed &= bool(result['ordinary_links_actual_owner_assertion'] and result['ordinary_links_header_membership_matches_source'])
    result['observations_match'] = passed if args.native else None
    Path(args.output).write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({k: v for k, v in result.items() if k != 'mismatches'}, indent=2))
    return 0 if passed else 1


if __name__ == '__main__':
    raise SystemExit(main())
