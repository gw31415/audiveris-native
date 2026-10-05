#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Render independent key-header test rasters using pinned official Bravura.

Renders only synthetic test artwork, with no native recognition output as input.
Dependencies are shared with linux_oracle.py --bootstrap; no global installation.
"""
import argparse
import datetime as dt
from pathlib import Path
import subprocess
from linux_oracle import ROOT, JAR_SHA256, JDK_DIR, checked, sha, write_json


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--oracle-home', type=Path, default=ROOT.parent / 'linux-oracle')
    p.add_argument('--output', type=Path)
    p.add_argument('--kind', choices=['header', 'key-column', 'time-column', 'stem-seeds'], default='header')
    args = p.parse_args()
    home = args.oracle_home.resolve()
    dist = home / 'official/opt/audiveris/lib'
    checked(dist / 'app/audiveris.jar', JAR_SHA256)
    output = (args.output or ROOT / 'evidence/generated/linux-oracle/header-fixtures' /
              (args.kind + '-' + dt.datetime.now(dt.timezone.utc).strftime('%Y%m%dT%H%M%S.%fZ'))).resolve()
    output.mkdir(parents=True, exist_ok=False)
    classes = output / 'classes'; classes.mkdir()
    renderer = {'header': 'MakeHeaderFixtures', 'key-column': 'MakeKeyColumnFixtures', 'time-column': 'MakeTimeColumnFixtures', 'stem-seeds': 'MakeStemSeedsFixtures'}[args.kind]
    source = ROOT / 'scripts/java' / (renderer + '.java')
    snapshot = output / source.name
    snapshot.write_bytes(source.read_bytes())
    commands = [[str(home / JDK_DIR / 'bin/javac'), '--release', '21', '-d', str(classes), str(snapshot)],
                [str(dist / 'runtime/bin/java'), '-Djava.awt.headless=true', '-Duser.home=' + str(output),
                 '-cp', str(dist / 'app/*') + ':' + str(classes), renderer, str(output)]]
    with (output / 'render.log').open('w') as log:
        for command in commands:
            subprocess.run(command, stdout=log, stderr=subprocess.STDOUT, check=True, timeout=90)
    fixtures = {path.name: sha(path) for path in sorted(output.glob('*.png'))}
    expected = {'header': 6, 'key-column': 15, 'time-column': 20, 'stem-seeds': 8}[args.kind]
    if len(fixtures) != expected:
        raise RuntimeError(f'Expected {expected} fixtures, got {len(fixtures)}')
    write_json(output / 'manifest.json', {'kind': 'synthetic independent input raster', 'jar_sha256': JAR_SHA256,
        'font_resource': 'res/Bravura.otf', 'font_sha256': 'dca2d90c88437a701b1c2e71fa54e76f9fa41d7deee935d74dc871ea66ecfdd2',
        'source': str(source.relative_to(ROOT)), 'source_sha256': sha(snapshot), 'source_snapshot': str(snapshot), 'commands': commands,
        'fixtures': fixtures, 'render_log_sha256': sha(output / 'render.log')})
    print(output)


if __name__ == '__main__':
    main()
