#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Regenerate independent seeded BeamStructure and Math.hypot controls locally.

The expected values come exclusively from the hash-pinned official JAR/runtime.
No downloads, source fetches, Mac access, or fixture overwrites are performed.
"""
import argparse
import hashlib
import itertools
import json
from pathlib import Path
import random
import shutil
import struct
import subprocess

ROOT = Path(__file__).resolve().parents[1]
JAR_SHA = '9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5'

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def generate_geometry():
    rng = random.Random(20261002)
    cases = []
    for i in range(1000):
        width = rng.randrange(13, 150)
        origin = rng.choice([0, 100, 8000, 16000])
        slope = rng.choice([.05, .1, .2, .3, .7, 1., -.1, -.3, -.7, -1.])
        noise = rng.randrange(3)
        thickness = rng.randrange(2, 20)
        runs = [[x, int(150 + slope * (x - width / 2)) + rng.randrange(-noise, noise + 1), thickness]
                for x in range(width)]
        cases.append({'name': f'review_{i:04}', 'glyph': {
            'bounds': [origin, origin, width, 300], 'orientation': 'VERTICAL', 'runs': runs}})
    # Both wrong branch directions were found by contiguous cropping of case 483.
    for start in [34, 42]:
        runs = [[x - start, y, n] for x, y, n in cases[483]['glyph']['runs'] if start <= x < start + 18]
        low = min(y for _, y, _ in runs)
        runs = [[x, y - low, n] for x, y, n in runs]
        cases.append({'name': f'minimized_sub_{start}_18', 'glyph': {
            'bounds': [100, 100 + low, 18, max(y + n for _, y, n in runs)],
            'orientation': 'VERTICAL', 'runs': runs}})
    return cases

def generate_hypot():
    rng = random.Random(51102026)
    values = [0, 1, 0x8000000000000000, 0x8000000000000001, 0xfffffffffffff,
              0x10000000000000, 0x1ff0000000000000, 0x3fe0000000000000,
              0x3ff0000000000000, 0x4000000000000000, 0x5f300000ffffffff,
              0x5f30000100000000, 0x7fefffffffffffff, 0x7ff0000000000000,
              0x7ff8000000000000]
    pairs = list(itertools.product(values, repeat=2))
    pairs += [(rng.getrandbits(64), rng.getrandbits(64)) for _ in range(4096)]
    pairs += [(struct.unpack('>Q', struct.pack('>d', rng.uniform(-1, 1)))[0], 0xbff0000000000000)
              for _ in range(1000)]
    return ''.join(f'{a:016x} {b:016x}\n' for a, b in pairs)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--oracle-home', type=Path, default=ROOT.parent / 'linux-oracle')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    home = args.oracle_home.resolve()
    output = args.output.resolve()
    distribution = home / 'official/opt/audiveris/lib'
    jar = distribution / 'app/audiveris.jar'
    java = distribution / 'runtime/bin/java'
    javac = home / 'jdk-21.0.12.1+1/bin/javac'
    if not all(p.is_file() for p in [jar, java, javac]) or sha(jar) != JAR_SHA:
        raise RuntimeError('Cached official distribution/JDK missing or JAR hash mismatch')
    output.mkdir(parents=True, exist_ok=False)
    sources = output / 'sources'
    classes = output / 'classes'
    sources.mkdir()
    classes.mkdir()
    for name in ['BeamStructureControlProbe.java', 'HypotProbe.java']:
        shutil.copyfile(ROOT / 'scripts/java' / name, sources / name)
    shutil.copyfile(Path(__file__), sources / Path(__file__).name)
    (output / 'geometry-input.json').write_text(json.dumps(generate_geometry(), separators=(',', ':')) + '\n')
    (output / 'hypot-input.txt').write_text(generate_hypot())
    commands = [
        [str(javac), '--release', '21', '-cp', str(distribution / 'app/*'), '-d', str(classes),
         str(sources / 'BeamStructureControlProbe.java'), str(sources / 'HypotProbe.java')],
        [str(java), '-Djava.awt.headless=true', '-cp', str(classes) + ':' + str(distribution / 'app/*'),
         'BeamStructureControlProbe', str(output / 'geometry-input.json'), str(output / 'geometry-original.json')],
        [str(java), '-cp', str(classes), 'HypotProbe', str(output / 'hypot-input.txt'), str(output / 'hypot-original.txt')],
    ]
    executions = []
    for i, command in enumerate(commands):
        with (output / f'{i:02}-console.log').open('w') as log:
            result = subprocess.run(command, stdout=log, stderr=subprocess.STDOUT, timeout=180, check=False)
        executions.append({'command': command, 'exit_code': result.returncode})
        if result.returncode:
            raise RuntimeError(f'Command {i} failed; inspect its local console log')
    actual = json.loads((output / 'geometry-original.json').read_text())
    expected = json.loads((ROOT / 'fixtures/beam-structure-review-controls.json').read_text())
    assert actual == expected, 'Regenerated geometry differs from the retained fixture'
    assert (output / 'hypot-original.txt').read_bytes() == (ROOT / 'crates/audiveris-area/upstream/hypot-vectors.txt').read_bytes()
    provenance = {
        'scope': 'Direct original geometry and numerical primitives only; no production builder acceptance decisions',
        'geometry_seed': 20261002, 'hypot_seed': 51102026,
        'geometry_cases': 1002, 'hypot_cases': 5321,
        'original_jar_sha256': sha(jar), 'java_sha256': sha(java), 'javac_sha256': sha(javac),
        'sources': {p.name: sha(p) for p in sources.iterdir()},
        'outputs': {name: sha(output / name) for name in ['geometry-input.json', 'geometry-original.json', 'hypot-input.txt', 'hypot-original.txt']},
        'executions': executions,
        'retained_fixture_replay': 'matched parsed geometry JSON and exact hypot vector bytes',
    }
    (output / 'provenance.json').write_text(json.dumps(provenance, indent=2) + '\n')
    print(json.dumps({'geometry_cases': 1002, 'hypot_cases': 5321, 'replay': 'matched'}, indent=2))

if __name__ == '__main__':
    main()
