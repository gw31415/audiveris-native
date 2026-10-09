#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Run one original explicit-Tuplet observer using the existing pinned Linux oracle.

Each output directory must be new. The factory, stages, and lookup observers
perform independent runs; object labels are meaningful only within one run.
Use --compile-only to validate the installed observer hierarchy without running
Audiveris. This development runner never reads native recognition output.
"""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import platform
import subprocess

import linux_oracle as oracle


PROBES = {
    "factory": "RealTupletFactoryImageProbe",
    "stages": "RealTupletStagesImageProbe",
    "lookup": "RealTupletLookupImageProbe",
}
SOURCES = (
    "RealLedgerMembershipImageProbe.java",
    "RealTupletProbeSupport.java",
    "RealTupletFactoryImageProbe.java",
    "RealTupletStagesImageProbe.java",
    "RealTupletLookupImageProbe.java",
)
# These signatures let javac 21 compile observers for the newer official runtime.
# They must never enter the runtime classpath or replace original implementation.
STUBS = {
    "Book.java": "package org.audiveris.omr.sheet; public class Book {}",
    "RunClass.java": (
        "package org.audiveris.omr.step; import java.util.SortedSet; "
        "import org.audiveris.omr.sheet.Book; public abstract class RunClass {"
        "public RunClass(Book b,SortedSet<Integer> ids){} "
        "public abstract void process();}"
    ),
    "SystemInfo.java": "package org.audiveris.omr.sheet; public class SystemInfo {}",
    "Staff.java": "package org.audiveris.omr.sheet; public class Staff {}",
    "Glyph.java": "package org.audiveris.omr.glyph; public class Glyph {}",
    "Evaluation.java": "package org.audiveris.omr.classifier; public class Evaluation {}",
    "Inter.java": "package org.audiveris.omr.sig.inter; public interface Inter {}",
    "InterFactory.java": (
        "package org.audiveris.omr.sheet.symbol; public class InterFactory {"
        "public InterFactory(org.audiveris.omr.sheet.SystemInfo s){} "
        "public org.audiveris.omr.sig.inter.Inter create("
        "org.audiveris.omr.classifier.Evaluation e,"
        "org.audiveris.omr.glyph.Glyph g,org.audiveris.omr.sheet.Staff s)"
        "{return null;}}"
    ),
}


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2, allow_nan=False) + "\n")


def main() -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("--oracle-home", type=Path, required=True)
    parser.add_argument("--image", type=Path, help="Unchanged source image; required unless --compile-only")
    parser.add_argument("--output", type=Path, required=True, help="New, nonexisting evidence directory")
    parser.add_argument("--probe", choices=(*PROBES, *PROBES.values()), default="factory")
    parser.add_argument("--compile-only", action="store_true", help="Compile all three observers without running Audiveris")
    parser.add_argument("--timeout", type=int, default=180, help="Seconds allowed per compiler or original process")
    args = parser.parse_args()
    if args.timeout <= 0:
        parser.error("--timeout must be positive")
    if not args.compile_only and args.image is None:
        parser.error("--image is required when running the original observer")
    if platform.system() != "Linux" or platform.machine() not in ("x86_64", "AMD64"):
        parser.error("The pinned oracle distribution requires Linux x86_64")

    home = args.oracle_home.resolve()
    lib = home / "official/opt/audiveris/lib"
    jar = lib / "app/audiveris.jar"
    oracle.checked(jar, oracle.JAR_SHA256)
    javac = home / oracle.JDK_DIR / "bin/javac"
    java = lib / "runtime/bin/java"
    sources = [oracle.ROOT / "scripts/java" / name for name in SOURCES]
    source_hashes = {path.name: oracle.sha(path) for path in sources}
    image = args.image.resolve() if args.image is not None else None
    image_hash = oracle.sha(image) if image is not None else None
    probe = PROBES.get(args.probe, args.probe)
    out = args.output.resolve()
    out.mkdir()
    for name in ("compile-only", "classes", "tmp", "cache", "home", "prefs", "system-prefs", "config", "data", "runtime"):
        (out / name).mkdir(mode=0o700)
    stub, classes = out / "compile-only", out / "classes"
    for name, body in STUBS.items():
        (stub / name).write_text(body + "\n")

    env = dict(os.environ)
    for key, name in (
        ("XDG_CONFIG_HOME", "config"), ("XDG_DATA_HOME", "data"),
        ("XDG_CACHE_HOME", "cache"), ("XDG_RUNTIME_DIR", "runtime"), ("TMPDIR", "tmp"),
    ):
        env[key] = str(out / name)
    # Ambient Java options could add signature stubs or reuse external settings.
    for key in ("JAVA_TOOL_OPTIONS", "JDK_JAVA_OPTIONS", "_JAVA_OPTIONS", "CLASSPATH"):
        env.pop(key, None)
    java_properties = [
        "-Djava.awt.headless=true",
        "-Djava.io.tmpdir=" + str(out / "tmp"),
        "-Dorg.bytedeco.javacpp.cachedir=" + str(out / "cache"),
        "-Djava.util.prefs.userRoot=" + str(out / "prefs"),
        "-Djava.util.prefs.systemRoot=" + str(out / "system-prefs"),
        "-Duser.home=" + str(out / "home"),
    ]
    compiler = [str(javac), *["-J" + value for value in java_properties], "--release", "21"]
    commands = [
        [*compiler, "-d", str(stub), *[str(stub / name) for name in STUBS]],
        [*compiler, "-cp", str(stub), "-d", str(classes), *map(str, sources)],
    ]
    classpath = [str(lib / "app/*"), str(classes)]
    command = [
        str(java), "-Xmx512m", "-XX:ActiveProcessorCount=2", *java_properties,
        "-Dnative.probe.output=" + str(out / "observed.json"),
        "-cp", os.pathsep.join(classpath), "Audiveris", "-batch",
        "-constant", "org.audiveris.omr.Main.persistBatchCliConstants=false",
        "-constant", "org.audiveris.omr.Main.runBatchTasksInParallel=false",
        "-step", "SCALE", "-run", probe, "-output", str(out), "--", str(image),
    ] if image is not None else None
    provenance = {
        "upstream_commit": oracle.UPSTREAM_COMMIT,
        "jar_sha256": oracle.sha(jar),
        "runner_sha256": oracle.sha(Path(__file__)),
        "probe": probe,
        "observer_sha256": source_hashes[probe + ".java"],
        "source_sha256": source_hashes,
        "image": str(image) if image is not None else None,
        "image_file_sha256": image_hash,
        "compile_only": args.compile_only,
        "compile_commands": commands,
        "compile_exit_codes": [],
        "compile_only_stub_sha256": {name: oracle.sha(stub / name) for name in STUBS},
        "runtime_classpath": classpath,
        "compile_only_signatures_on_runtime_classpath": False,
        "run_argv": None,
        "planned_run_argv": command,
        "original_run_attempted": False,
        "original_run_started": False,
        "original_run_completed": False,
        "original_exit_code": None,
        "native_outputs_used": False,
        "qualification": "Installed consolidated Tuplet helpers. Each invocation is an independent original run; labels do not establish cross-run identity. Compile-only success is not an original observation.",
    }
    result_code = 1
    phase = "compile"
    try:
        with (out / "compile.log").open("w") as log:
            for compile_command in commands:
                result = subprocess.run(compile_command, env=env, stdout=log, stderr=subprocess.STDOUT, timeout=args.timeout)
                provenance["compile_exit_codes"].append(result.returncode)
                if result.returncode:
                    result_code = result.returncode
                    return result_code
        if args.compile_only:
            result_code = 0
            return result_code
        phase = "original"
        provenance["run_argv"] = command
        provenance["original_run_attempted"] = True
        with (out / "console.log").open("w") as log:
            result = subprocess.run(command, env=env, stdout=log, stderr=subprocess.STDOUT, timeout=args.timeout)
        provenance["original_run_started"] = True
        provenance["original_run_completed"] = True
        provenance["original_exit_code"] = result.returncode
        result_code = result.returncode
        if result_code == 0:
            observed = json.loads((out / "observed.json").read_text())
            if observed.get("native_outputs_used") is not False or observed.get("observer_errors"):
                raise ValueError("Original observation failed its source-only/error qualification")
        return result_code
    except subprocess.TimeoutExpired:
        provenance["original_run_started"] = phase == "original"
        provenance["error"] = "process_timeout"
        provenance["failed_phase"] = phase
        result_code = 124
        return result_code
    except (OSError, ValueError) as error:
        provenance["error"] = str(error)
        provenance["failed_phase"] = phase
        result_code = 1
        return result_code
    finally:
        for name, field in (("compile.log", "compile_log_sha256"), ("console.log", "console_sha256"), ("observed.json", "observed_sha256")):
            if (out / name).is_file():
                provenance[field] = oracle.sha(out / name)
        provenance["generated_classes"] = sorted(str(path.relative_to(classes)) for path in classes.rglob("*.class"))
        provenance["exit_code"] = result_code
        write_json(out / "provenance.json", provenance)
        print(json.dumps({"output": str(out), "probe": probe, "compile_only": args.compile_only, "exit_code": result_code}), flush=True)


if __name__ == "__main__":
    raise SystemExit(main())
