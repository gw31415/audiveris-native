#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Prepare the CC0 Der Winter image with an existing Linux MuseScore 3.2.3 tree.

This development fixture renderer is not part of native recognition. --raw-png
rechecks normalization of an existing render without invoking MuseScore.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import tempfile
import time
import zlib

from PIL import Image, __version__ as pillow_version

ROOT = Path(__file__).resolve().parents[1]
SCORE_SHA = "776b60aa1488636fe8d2e83c79391aa80d15981ce17ecc2ca30ca3a3fac6ccd6"
BINARY_SHA = "9decd483ac1c421a3a9cb9920c097e05fcba8c01407fc28a7645f27007b5b16b"
PIXEL_SHA = "d2b9a3209383e23e3a226081a9f996f9115506bd5dcb2e85ca55a2ea559ebf23"
PNG_SHA = "10f61bf0d44be190df7177b57c1cd26a31bfbcc68d9ad8f85a76813d51ae59e0"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=ROOT / "fixtures/openscore-lieder/lc6053913.mscx")
    parser.add_argument("--renderer-home", type=Path, default=Path("/workspace/native-build-tools/fixture-renderer-ms3"))
    parser.add_argument("--output", type=Path, default=Path(tempfile.gettempdir()) / "real-rest-fixture")
    parser.add_argument("--raw-png", type=Path, help="normalize an existing RGBA page; do not render")
    args = parser.parse_args()
    source = args.source.resolve()
    source_sha = sha(source)
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    record = {"source": str(source), "source_score_sha256": source_sha,
              "native_outputs_used": False, "source_score_modified": False,
              "renderer_invoked": args.raw_png is None}
    if args.raw_png is None:
        if platform.system() != "Linux":
            raise SystemExit("This fixture preparation requires the existing Linux renderer")
        renderer = args.renderer_home.resolve()
        binary = renderer / "root/usr/bin/musescore3"
        if sha(binary) != BINARY_SHA:
            raise SystemExit("Expected the recorded MuseScore 3.2.3 binary")
        download = json.loads((renderer / "DOWNLOAD.json").read_text())
        for package in download["packages"]:
            if sha(renderer / Path(package["file"]).name) != package["sha256"]:
                raise SystemExit("Renderer package hash mismatch")
        with tempfile.TemporaryDirectory(prefix="real-rest-render-") as name:
            private = Path(name)
            env = dict(os.environ)
            for key, subdir in (("XDG_CONFIG_HOME", "config"), ("XDG_CACHE_HOME", "cache"),
                                ("XDG_DATA_HOME", "data"), ("XDG_RUNTIME_DIR", "runtime"),
                                ("TMPDIR", "tmp")):
                path = private / subdir
                path.mkdir(mode=0o700)
                env[key] = str(path)
            env["QT_QPA_PLATFORM"] = "offscreen"
            env["LD_LIBRARY_PATH"] = str(renderer / "root/usr/lib/x86_64-linux-gnu")
            argv = [str(binary), "-s", "-m", "-c", str(private / "prefs"), "-r", "180",
                    "-o", str(out / "der-winter.png"), str(source)]
            start = time.monotonic()
            with (out / "render.log").open("w") as log:
                result = subprocess.run(argv, env=env, stdout=log, stderr=subprocess.STDOUT,
                                        timeout=120, check=False)
            record["render"] = {"argv": argv, "exit_code": result.returncode,
                                "elapsed_seconds": time.monotonic() - start,
                                "binary_sha256": sha(binary), "package_version": "3.2.3+dfsg2-19",
                                "private_environment": {k: env[k] for k in (
                                    "QT_QPA_PLATFORM", "LD_LIBRARY_PATH", "XDG_CONFIG_HOME",
                                    "XDG_CACHE_HOME", "XDG_DATA_HOME", "XDG_RUNTIME_DIR", "TMPDIR")},
                                "package_download_manifest_sha256": sha(renderer / "DOWNLOAD.json")}
            if result.returncode:
                (out / "RESULT.json").write_text(json.dumps(record, indent=2) + "\n")
                raise SystemExit(result.returncode)
        pages = sorted(out.glob("der-winter-*.png"))
        if len(pages) != 1:
            raise SystemExit("Expected one full page; no silent page selection")
        raw = pages[0]
    else:
        raw = args.raw_png.resolve()
    with Image.open(raw) as image:
        if image.mode != "RGBA":
            raise SystemExit("Expected the original RGBA renderer output")
        rgba = image.copy()
    # Transparent pixels have black RGB in the old renderer. Dropping alpha
    # would turn the background black and change the recognition input.
    gray = Image.alpha_composite(Image.new("RGBA", rgba.size, "white"), rgba).convert("L")
    output = out / "der-winter-gray.png"
    gray.save(output, format="PNG", compress_level=6)
    pixel_sha = hashlib.sha256(gray.tobytes()).hexdigest()
    record["normalization"] = {
        "operation": "RGBA alpha-composite over opaque white, then Pillow convert('L')",
        "raw_png": str(raw), "raw_png_sha256": sha(raw),
        "raw_rgba_pixels_sha256": hashlib.sha256(rgba.tobytes()).hexdigest(),
        "output": str(output), "output_png_sha256": sha(output),
        "opaque_grayscale_pixels_sha256": pixel_sha, "dimensions": list(gray.size),
        "pillow_version": pillow_version, "zlib_version": zlib.ZLIB_RUNTIME_VERSION,
        "png_metadata": "none", "png_compress_level": 6,
        "reference_pixel_match": pixel_sha == PIXEL_SHA,
        "reference_png_byte_match": sha(output) == PNG_SHA}
    record["source_score_unchanged_after_preparation"] = sha(source) == source_sha
    (out / "RESULT.json").write_text(json.dumps(record, indent=2) + "\n")
    if not record["source_score_unchanged_after_preparation"]:
        raise SystemExit("Source score changed during fixture preparation")
    if source_sha == SCORE_SHA and (gray.size != (1489, 2104) or pixel_sha != PIXEL_SHA):
        raise SystemExit("Rendered pixels differ from the recorded fixture; inspect RESULT.json")
    print(json.dumps({"output": str(output), "png_sha256": sha(output),
                      "reference_png_byte_match": sha(output) == PNG_SHA}))


if __name__ == "__main__":
    main()
