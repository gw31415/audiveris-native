#!/usr/bin/env python3
"""Verify the committed artifact inventory, including original model/golden data."""
from pathlib import Path
import hashlib,json
root=Path(__file__).resolve().parent
manifest=json.loads((root/'ARTIFACT-MANIFEST.json').read_text())
for row in manifest['files']:
    p=root/row['path']
    assert p.stat().st_size==row['bytes'],f"size mismatch: {p}"
    assert hashlib.sha256(p.read_bytes()).hexdigest()==row['sha256'],f"hash mismatch: {p}"
print(f"Verified {len(manifest['files'])} source/data/license/evidence files")
