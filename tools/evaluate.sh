#!/bin/bash
# Evaluate the native pipeline against ground truth.
#   tools/evaluate.sh                 samples/ (3 real pages + synthetic pages)
#   tools/evaluate.sh CORPUS_DIR      a corpus of NAME.mscx + NAME-1.png, NAME-2.png, ...
# Prints, per score, the per-measure MATCH ratio and the measure-independent
# FLAT ratio (pitch/rest + duration sequences per staff).
set -e
cd "$(dirname "$0")/.."
cargo build --release -q
BIN=./target/release/audiveris-native
OUT=$(mktemp -d)
if [ -n "$1" ]; then
  for m in "$1"/*.mscx; do
    n=$(basename "$m" .mscx)
    pages=$(ls "$1/$n"-*.png | sort -V)
    $BIN -o "$OUT/$n.musicxml" $pages
    echo "$n $(python3 tools/compare.py "$m" "$OUT/$n.musicxml" | tail -2 | tr '\n' ' ')"
  done
  exit 0
fi
for f in an-laura der-winter der-brautschmuck; do
  $BIN -o "$OUT/$f.musicxml" samples/real/$f.png
  echo "$f $(python3 tools/compare.py samples/real/$f.mscx "$OUT/$f.musicxml" | tail -2 | tr '\n' ' ')"
done
for r in samples/synthetic/*.audiveris.musicxml; do
  n=$(basename "$r" .audiveris.musicxml)
  $BIN -o "$OUT/$n.musicxml" samples/synthetic/$n.png
  echo "$n $(python3 tools/compare.py "$r" "$OUT/$n.musicxml" | tail -1)"
done
