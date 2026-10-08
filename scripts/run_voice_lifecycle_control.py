#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Observe pinned original Voice lifecycle; output must be a new directory."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

sys.path.insert(0, str(Path(__file__).parent))
import linux_oracle as oracle

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--oracle-home", type=Path, required=True)
parser.add_argument("--book", type=Path, required=True)
parser.add_argument("--output", type=Path, required=True)
parser.add_argument("--fresh-beams", type=Path, help="Nearest probe only: hash-recorded original in-memory BEAMS measurement")
parser.add_argument("--probe", choices=("VoiceLifecycleProbe", "VoiceRemovalLifecycleProbe", "VoiceRetiredNextProbe", "VoiceRetiredTieProbe", "VoiceRetiredBeamProbe", "VoiceResetRhythmProbe", "VoicePageRhythmProbe", "VoicePageNullRhythmProbe", "VoiceSelectedRhythmProbe", "BeamRestClearProbe", "BeamRestLinkedProbe", "BeamRestNearestProbe", "BeamRestAddedProbe", "BeamRestPlainProbe", "BeamRestPolygonProbe", "BeamRestCompetitionProbe", "BeamRestDetectProbe", "TupletPreparationProbe", "VoicePreparedRhythmProbe", "TupletReuseProbe", "TupletPlacementProbe", "TupletFindInputsProbe", "TupletFindProbe", "TupletForceProbe", "TupletBeatProbe", "TupletSansRetiredProbe", "TupletSansProbe", "TupletStackProbe", "TupletGettersProbe", "TupletRawBaseProbe", "ChordRawOwnersProbe", "TupletBaseFailureProbe", "StaffMidlineProbe"),
                    default="VoiceLifecycleProbe")
args = parser.parse_args()
if args.fresh_beams is not None and args.probe != "BeamRestNearestProbe":
    parser.error("--fresh-beams is only valid for BeamRestNearestProbe")
output = args.output.resolve()
output.mkdir()
for name in ("classes", "home", "cache", "data", "config", "prefs"):
    (output / name).mkdir()
lib = args.oracle_home.resolve() / "official/opt/audiveris/lib"
oracle.checked(lib / "app/audiveris.jar", oracle.JAR_SHA256)
compiler = args.oracle_home.resolve() / oracle.JDK_DIR / "bin/javac"
source = oracle.ROOT / ("scripts/java/" + args.probe + ".java")
classpath = str(lib / "app/*") + os.pathsep + str(output / "classes")
compile_args = [str(compiler), "--release", "21", "-cp", str(lib / "app/*"),
                "-d", str(output / "classes"), str(source),
                str(oracle.ROOT / "scripts/java/HeadStemPreLinkProbe.java")]
signature_sources = []
if args.probe in ("BeamRestAddedProbe", "BeamRestCompetitionProbe", "BeamRestDetectProbe", "TupletPreparationProbe", "VoicePreparedRhythmProbe"):
    # The existing compiler is 21 and the pinned runtime/JAR is 25. These
    # signatures only let javac emit an override; they are absent at runtime,
    # where the subclass invokes the untouched pinned BeamRestRelation added/removed callbacks.
    signatures = output / "compile-signatures"
    signatures.mkdir()
    for name, body in {
        "sig/inter/Inter": "public interface Inter {}",
        "sig/relation/Relation": "public abstract class Relation {}",
        "sig/relation/BeamRestRelation": "public class BeamRestRelation extends Relation { public void added(org.jgrapht.event.GraphEdgeChangeEvent<org.audiveris.omr.sig.inter.Inter,Relation> event) {} public void removed(org.jgrapht.event.GraphEdgeChangeEvent<org.audiveris.omr.sig.inter.Inter,Relation> event) {} }",
    }.items():
        source_file = signatures / ("org/audiveris/omr/" + name + ".java")
        source_file.parent.mkdir(parents=True, exist_ok=True)
        source_file.write_text("package org.audiveris.omr." + name.rsplit("/", 1)[0].replace("/", ".") + ";\n" + body + "\n")
        signature_sources.append(source_file)
    with (output / "signatures-compile.log").open("w") as log:
        subprocess.run([str(compiler), "--release", "21", "-cp", str(lib / "app/*"),
                        "-d", str(signatures), *map(str, signature_sources)],
                       stdout=log, stderr=subprocess.STDOUT, check=True)
    compile_args[compile_args.index("-cp") + 1] = str(signatures) + os.pathsep + str(lib / "app/*")
if args.probe in ("BeamRestCompetitionProbe", "BeamRestDetectProbe", "TupletPreparationProbe", "VoicePreparedRhythmProbe"):
    compile_args.append(str(oracle.ROOT / "scripts/java/BeamRestPolygonProbe.java"))
if args.probe in ("BeamRestDetectProbe", "TupletPreparationProbe", "VoicePreparedRhythmProbe"):
    compile_args.extend(str(oracle.ROOT / ("scripts/java/"+name+".java")) for name in ("BeamRestCompetitionProbe", "BeamRestLinkedProbe"))
if args.probe != "VoiceLifecycleProbe":
    compile_args.append(str(oracle.ROOT / "scripts/java/VoiceLifecycleProbe.java"))
if args.probe in ("BeamRestLinkedProbe", "BeamRestNearestProbe", "BeamRestAddedProbe", "BeamRestPlainProbe", "BeamRestPolygonProbe", "BeamRestCompetitionProbe", "BeamRestDetectProbe", "TupletPreparationProbe", "VoicePreparedRhythmProbe", "TupletReuseProbe", "TupletPlacementProbe", "TupletFindInputsProbe", "TupletFindProbe", "TupletForceProbe", "TupletBeatProbe", "TupletSansRetiredProbe", "TupletSansProbe", "TupletStackProbe", "TupletGettersProbe", "TupletRawBaseProbe", "ChordRawOwnersProbe", "TupletBaseFailureProbe"):
    compile_args.append(str(oracle.ROOT / "scripts/java/BeamRestClearProbe.java"))
if args.probe in ("VoiceRetiredBeamProbe", "VoiceResetRhythmProbe", "VoicePageRhythmProbe", "VoicePageNullRhythmProbe", "VoiceSelectedRhythmProbe"):
    compile_args.append(str(oracle.ROOT / "scripts/java/VoiceRetiredTieProbe.java"))
if args.probe in ("VoiceResetRhythmProbe", "VoicePageRhythmProbe", "VoicePageNullRhythmProbe", "VoiceSelectedRhythmProbe"):
    compile_args.append(str(oracle.ROOT / "scripts/java/VoiceRetiredBeamProbe.java"))
if args.probe in ("VoicePageRhythmProbe", "VoicePageNullRhythmProbe", "VoiceSelectedRhythmProbe"):
    compile_args.append(str(oracle.ROOT / "scripts/java/VoiceResetRhythmProbe.java"))
run_args = [str(lib / "runtime/bin/java"), "-Xmx512m", "-XX:ActiveProcessorCount=2",
            "-Djava.awt.headless=true", "-Duser.home=" + str(output / "home"),
            "-Djava.util.prefs.userRoot=" + str(output / "prefs"), "-cp", classpath,
            args.probe, str(args.book.resolve()), str(output / "observed.json")]
if args.fresh_beams is not None:
    run_args.append(str(args.fresh_beams.resolve()))
env = dict(os.environ)
for key, directory in (("XDG_CONFIG_HOME", "config"), ("XDG_DATA_HOME", "data"), ("XDG_CACHE_HOME", "cache")):
    env[key] = str(output / directory)
with (output / "compile.log").open("w") as log:
    subprocess.run(compile_args, stdout=log, stderr=subprocess.STDOUT, check=True)
with (output / "console.log").open("w") as log:
    result = subprocess.run(run_args, env=env, stdout=log, stderr=subprocess.STDOUT, timeout=60)
record = {
    "upstream_commit": oracle.UPSTREAM_COMMIT,
    "jar_sha256": oracle.JAR_SHA256,
    "observer_sha256": oracle.sha(source),
    "reflection_helper_sha256": oracle.sha(oracle.ROOT / "scripts/java/HeadStemPreLinkProbe.java"),
    "book_sha256": oracle.sha(args.book),
    "native_outputs_used": False,
    "compile_argv": compile_args, "run_argv": run_args,
    "exit_code": result.returncode,
    "console_sha256": oracle.sha(output / "console.log"),
}
if signature_sources:
    record["compile_only_signatures"] = {str(path.relative_to(output)): oracle.sha(path) for path in signature_sources}
    record["compile_signatures_on_runtime_classpath"] = False
if args.probe != "VoiceLifecycleProbe":
    record["lifecycle_helper_sha256"] = oracle.sha(oracle.ROOT / "scripts/java/VoiceLifecycleProbe.java")
if args.probe in ("BeamRestLinkedProbe", "BeamRestNearestProbe", "BeamRestAddedProbe", "BeamRestPlainProbe", "BeamRestPolygonProbe", "BeamRestCompetitionProbe", "BeamRestDetectProbe", "TupletPreparationProbe", "VoicePreparedRhythmProbe", "TupletReuseProbe", "TupletPlacementProbe", "TupletFindInputsProbe", "TupletFindProbe", "TupletForceProbe", "TupletBeatProbe", "TupletSansRetiredProbe", "TupletSansProbe", "TupletStackProbe", "TupletGettersProbe", "TupletRawBaseProbe", "ChordRawOwnersProbe", "TupletBaseFailureProbe"):
    record["clear_helper_sha256"] = oracle.sha(oracle.ROOT / "scripts/java/BeamRestClearProbe.java")
if args.probe in ("VoiceRetiredBeamProbe", "VoiceResetRhythmProbe", "VoicePageRhythmProbe", "VoicePageNullRhythmProbe", "VoiceSelectedRhythmProbe"):
    record["tie_helper_sha256"] = oracle.sha(oracle.ROOT / "scripts/java/VoiceRetiredTieProbe.java")
if args.probe in ("VoiceResetRhythmProbe", "VoicePageRhythmProbe", "VoicePageNullRhythmProbe", "VoiceSelectedRhythmProbe"):
    record["beam_helper_sha256"] = oracle.sha(oracle.ROOT / "scripts/java/VoiceRetiredBeamProbe.java")
if args.probe in ("VoicePageRhythmProbe", "VoicePageNullRhythmProbe", "VoiceSelectedRhythmProbe"):
    record["reset_helper_sha256"] = oracle.sha(oracle.ROOT / "scripts/java/VoiceResetRhythmProbe.java")
if args.probe in ("BeamRestCompetitionProbe", "BeamRestDetectProbe", "TupletPreparationProbe", "VoicePreparedRhythmProbe"):
    record["polygon_helper_sha256"] = oracle.sha(oracle.ROOT / "scripts/java/BeamRestPolygonProbe.java")
    if args.probe in ("BeamRestDetectProbe", "TupletPreparationProbe", "VoicePreparedRhythmProbe"):
        record["competition_helper_sha256"] = oracle.sha(oracle.ROOT / "scripts/java/BeamRestCompetitionProbe.java")
        record["linked_helper_sha256"] = oracle.sha(oracle.ROOT / "scripts/java/BeamRestLinkedProbe.java")
    record["fresh_add_observation"] = "later GraphListener after original SigListener added callback; removed callbacks trace original super entry/exit"
if args.probe in ("TupletPreparationProbe", "VoicePreparedRhythmProbe"):
    record.pop("fresh_add_observation", None)
    record["tuplet_removal_observation"] = "later GraphListener after original ChordTuplet removed callback; actual Tuplet remains registered/removed with Measure/Stack already detached"
if result.returncode == 0:
    record["observed_sha256"] = oracle.sha(output / "observed.json")
if args.fresh_beams is not None:
    record["original_fresh_beams_sha256"] = oracle.sha(args.fresh_beams)
    record["fresh_beams_scope"] = "original independent raw BEAMS geometry as a declared real SIG group; stored timed source group untouched"
(output / "provenance.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({"output": str(output), "exit_code": result.returncode}))
raise SystemExit(result.returncode)
