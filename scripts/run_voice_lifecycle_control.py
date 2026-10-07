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
parser.add_argument("--probe", choices=("VoiceLifecycleProbe", "VoiceRemovalLifecycleProbe"),
                    default="VoiceLifecycleProbe")
args = parser.parse_args()
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
if args.probe == "VoiceRemovalLifecycleProbe":
    compile_args.append(str(oracle.ROOT / "scripts/java/VoiceLifecycleProbe.java"))
run_args = [str(lib / "runtime/bin/java"), "-Xmx512m", "-XX:ActiveProcessorCount=2",
            "-Djava.awt.headless=true", "-Duser.home=" + str(output / "home"),
            "-Djava.util.prefs.userRoot=" + str(output / "prefs"), "-cp", classpath,
            args.probe, str(args.book.resolve()), str(output / "observed.json")]
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
if args.probe == "VoiceRemovalLifecycleProbe":
    record["lifecycle_helper_sha256"] = oracle.sha(oracle.ROOT / "scripts/java/VoiceLifecycleProbe.java")
if result.returncode == 0:
    record["observed_sha256"] = oracle.sha(output / "observed.json")
(output / "provenance.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({"output": str(output), "exit_code": result.returncode}))
raise SystemExit(result.returncode)
