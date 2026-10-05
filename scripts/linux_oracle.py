#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Run pinned, unmodified Audiveris on Linux as a development-only oracle.

Examples (no global installation, existing fixtures are never overwritten):
  python3 scripts/linux_oracle.py --bootstrap
  python3 scripts/linux_oracle.py --probe clef-candidates
  python3 scripts/linux_oracle.py --probe clef-registration --fixtures rendered-clef-g_clef

Only test observers are compiled. Their compile-only ABI shims are explicitly
excluded from the runtime classpath. No production Rust executable calls this.
"""
from __future__ import annotations
import argparse
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import tarfile
import urllib.request
import zipfile

ROOT = Path(__file__).resolve().parents[1]
UPSTREAM_COMMIT = "9e1e55cd2746037d059345881c53e6a6754bffbd"
JAR_SHA256 = "9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5"
DEB_NAME = "Audiveris-5.11.0-ubuntu22.04-x86_64.deb"
DEB_SHA256 = "ae714594f40e54b1a4951fc3f914f08ae38fe5d07b7f2283b1a904fdb6e0a318"
DEB_URL = "https://github.com/Audiveris/audiveris/releases/download/5.11.0/" + DEB_NAME
JDK_NAME = "OpenJDK21U-jdk_x64_linux_hotspot_21.0.12.1_1.tar.gz"
JDK_SHA256 = "ce79869e1307ed8ee1e2baa86a412b1eb5b75d10a01006d788a6f968bcfaee94"
JDK_URL = "https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.12.1%2B1/" + JDK_NAME
JDK_DIR = "jdk-21.0.12.1+1"
FIXTURES = ["laura-gray", "synthetic-system-bars", "cclef-pair", "bracket-pair", "line-root",
            "rendered-clef-g_clef", "rendered-clef-f_clef", "rendered-clef-c_clef",
            "rendered-clef-empty", "rendered-clef-noise"]
PROBES = {"stem-image-head-expansion": ("StemImageHeadExpansionProbe", "HEADS"),"stem-image-head-items": ("StemImageHeadItemsProbe", "HEADS"),"stem-image-head-chunks": ("StemImageHeadChunksProbe", "HEADS"),"stem-image-head-parts": ("StemImageHeadPartsProbe", "HEADS"),"stem-image-beam-generation": ("StemImageBeamGenerationProbe", "HEADS"),"stem-image-beam-targets": ("StemImageBeamTargetsProbe", "HEADS"),"stem-image-linkers": ("StemImageLinkersProbe", "HEADS"),"stem-image-sources": ("StemImageSourcesProbe", "HEADS"),"stem-image-stumps": ("StemImageStumpsProbe", "HEADS"),"stem-image-seeds": ("StemImageSeedsProbe", "HEADS"),"stem-image-head-plan": ("StemImageHeadPlanProbe", "HEADS"),"stem-head-reference-context": ("StemHeadReferenceContextProbe", "HEADS"),"stem-head-neighborhood": ("StemHeadNeighborhoodProbe", "HEADS"),"stem-head-targets": ("StemHeadTargetsProbe", "HEADS"),"stem-parts-prefix": ("StemPartsPrefixProbe", "HEADS"),"stem-item-events": ("StemItemEventsProbe", "HEADS"),"stem-length-map": ("StemLengthMapProbe", "HEADS"),"stem-cache-head": ("StemCacheHeadProbe", "HEADS"),"stem-chunks": ("StemChunksProbe", "HEADS"),"stem-chunk-head-filter": ("StemChunkHeadFilterProbe", "HEADS"),"stem-chunk-registration": ("StemChunkRegistrationProbe", "HEADS"),"stem-beam-lookup-area": ("StemBeamLookupAreaProbe", "HEADS"),"stem-lookup-area": ("StemLookupAreaProbe", "HEADS"),"stem-parts-lookup": ("StemPartsLookupProbe", "HEADS"),"stem-parts-filter": ("StemPartsFilterProbe", "HEADS"),"stem-parts-alignment": ("StemPartsAlignmentProbe", "HEADS"),"head-stems-cleaner": ("HeadStemsCleanerControlProbe", "SCALE"),"stem-cache-sig-lifecycle": ("StemCacheSigLifecycleProbe", "HEADS"),"stem-builder-create": ("StemBuilderCreateProbe", "HEADS"),"heads-default-page": ("HeadsDefaultPageProbe", "SCALE"),"grid-cue-rest-mapping": ("GridCueRestMappingProbe", "SCALE"),"grid-cue-completion": ("GridCueCompletionProbe", "SCALE"),"grid-cue-projection": ("GridProbe", "SCALE"),"grid-second-pass": ("GridSecondPassProbe", "SCALE"),"stem-glyph-grade": ("StemGlyphGradeProbe", "SCALE"),"head-stem-stump": ("HeadStemStumpProbe", "SCALE"),"head-stem-support": ("HeadStemSupportProbe", "SCALE"),"head-stem-connection": ("HeadStemConnectionProbe", "SCALE"),"head-actual-pages": ("HeadsActualPageProbe", "SCALE"),"head-all-shapes-boundary": ("HeadAllShapesBoundaryProbe", "SCALE"),"head-all-shapes-seed": ("HeadAllShapesSeedProbe", "SCALE"),"head-all-shapes-coverage": ("HeadAllShapesCoverageProbe", "SCALE"),"head-page-catalog": ("HeadsCatalogPageProbe", "SCALE"),"head-seed-page-catalog": ("HeadSeedPageCatalogProbe", "SCALE"),"head-seed-coverage": ("HeadSeedCoverageProbe", "SCALE"),"head-drum-coverage": ("HeadDrumCoverageProbe", "SCALE"),"head-staff-catalog": ("HeadStaffCatalogProbe", "SCALE"),"head-font-coverage": ("HeadFontCoverageProbe", "SCALE"),"head-staff-dependency": ("HeadStaffDependencyProbe", "SCALE"),"heads-registration": ("HeadsRegistrationProbe", "SCALE"),"head-glyph": ("HeadGlyphProbe", "SCALE"),"head-candidate-filter": ("HeadCandidateFilterProbe", "SCALE"),"head-range-catalog": ("HeadRangeCatalogProbe", "SCALE"),"head-range-prefix": ("HeadRangePrefixProbe", "SCALE"),"beam-groups": ("BeamGroupsProbe", "SCALE"),"beam-late-hooks": ("BeamLateHooksProbe", "SCALE"),"beam-extension-loop": ("BeamExtensionLoopProbe", "SCALE"),"clef-candidates": ("ClefCandidateProbe", "SCALE"),
          "clef-registration": ("ClefRegistrationProbe", "SCALE"),
          "headers": ("HeadersFinalProbe", "HEADERS"),
          "headers-live": ("HeadersLiveProbe", "SCALE"),
          "stem-seeds": ("StemSeedsProbe", "SCALE"),
          "beams": ("BeamsProbe", "SCALE"),
          "beams-faithful": ("BeamsFaithfulProbe", "SCALE"),
          "beam-candidates": ("BeamCandidatesProbe", "SCALE"),
          "beam-filters-control": ("BeamFiltersControlProbe", "SCALE"),
          "beams-plain": ("BeamsPlainProbe", "SCALE"),
          "stem-buffer-control": ("StemBufferControlProbe", None),
          "stem-scale-control": ("StemScaleHistogramControlProbe", "SCALE"),
          "stick-factory-control": ("StickFactoryControlProbe", "SCALE"),
          "stem-seeds-plain": ("StemSeedsPlainProbe", "SCALE"),
          "grid": ("GridFinalProbe", "SCALE"),
          "clef-font-metrics": ("ClefFontMetricsProbe", None),
          "time-font-metrics": ("TimeFontMetricsProbe", None),
          "key-preparation": ("KeyPreparationProbe", "SCALE"),
          "clef-registration-control": ("ClefRegistrationControlProbe", "SCALE"),
          "key-preparation-control": ("KeyPreparationControlProbe", "SCALE"),
          "key-peaks": ("KeyPeaksProbe", "SCALE"),
          "hilo-control": ("HiLoControlProbe", None),
          "clustering-control": ("ClusteringControlProbe", None),
          "context-partition-control": ("ContextPartitionControlProbe", None),
          "key-slicing": ("KeySlicingProbe", "SCALE"),
          "key-components": ("KeyComponentsProbe", "SCALE"),
          "key-component-branches": ("KeyComponentBranchesProbe", "SCALE"),
          "key-fallback": ("KeyFallbackProbe", "SCALE"),
          "key-neighbor-control": ("KeyNeighborEraseControlProbe", "SCALE"),
          "key-clef-control": ("KeyClefControlProbe", None),
          "clef-competition-control": ("ClefCompetitionControlProbe", None),
          "key-process": ("KeyBuilderProcessProbe", "SCALE"),
          "key-column": ("KeyColumnProbe", "SCALE"),
          "time-column": ("HeaderTimeColumnProbe", "SCALE"),
          "time-selection-control": ("TimeSelectionControlProbe", "SCALE"),
          "time-selection-control-plain": ("TimeSelectionControlPlainProbe", "SCALE"),
          "time-column-plain": ("HeaderTimeColumnPlainProbe", "SCALE"),
          "key-column-plain": ("KeyColumnPlainProbe", "SCALE"),
          "key-process-plain": ("KeyBuilderProcessPlainProbe", "SCALE"),
          "key-pitch-window-control": ("KeyPitchWindowControlProbe", "SCALE"),
          "key-extract-retention-control": ("KeyExtractRetentionControlProbe", "SCALE")}


def sha(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def checked(path: Path, digest: str) -> None:
    actual = sha(path)
    if actual != digest:
        raise RuntimeError(f"SHA-256 mismatch for {path}: expected {digest}, got {actual}")


def download(url: str, dest: Path, digest: str) -> None:
    if dest.exists():
        checked(dest, digest)
        return
    dest.parent.mkdir(parents=True, exist_ok=True)
    partial = dest.with_name(dest.name + ".partial")
    print(f"Downloading {url}", flush=True)
    with urllib.request.urlopen(url, timeout=90) as response, partial.open("wb") as stream:
        shutil.copyfileobj(response, stream)
    checked(partial, digest)
    partial.rename(dest)


def bootstrap(home: Path) -> None:
    download(DEB_URL, home / "downloads" / DEB_NAME, DEB_SHA256)
    if not (home / "official/opt/audiveris/lib/app/audiveris.jar").exists():
        subprocess.run(["dpkg-deb", "-x", str(home / "downloads" / DEB_NAME),
                        str(home / "official")], check=True)
    download(JDK_URL, home / "downloads" / JDK_NAME, JDK_SHA256)
    if not (home / JDK_DIR / "bin/javac").exists():
        with tarfile.open(home / "downloads" / JDK_NAME) as archive:
            archive.extractall(home, filter="data")


def write_json(path: Path, data: object) -> None:
    path.write_text(json.dumps(data, indent=2, allow_nan=False) + "\n")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--oracle-home", type=Path, default=ROOT.parent / "linux-oracle")
    parser.add_argument("--bootstrap", action="store_true", help="Download hash-pinned official development dependencies")
    parser.add_argument("--distribution", type=Path, help="Linux distribution lib directory (contains app/ and runtime/)")
    parser.add_argument("--jdk", type=Path, help="JDK directory for compiling test observers")
    parser.add_argument("--probe", choices=PROBES, default="clef-candidates")
    parser.add_argument("--fixtures", nargs="+", default=FIXTURES, help="Names in fixtures/official-grid, or PNG paths")
    parser.add_argument("--output", type=Path, help="New, nonexisting evidence directory")
    parser.add_argument("--timeout", type=int, default=180)
    parser.add_argument("--grid-input-mode", choices=("all", "main-lines-only", "empty"), default="all", help="Only grid-second-pass observer: controlled original initial filament boundary")
    parser.add_argument("--page-processing", choices=("default", "one-line-drums", "drums", "six-string-tab"), default="default", help="Only actual-page observer: original staff processing switches before GRID")
    parser.add_argument("--head-family", choices=("Bravura", "Leland", "FinaleJazz"), help="Only HEADS page catalog probes: explicit original HEADS family after default upstream steps")
    parser.add_argument("--head-point-size", type=int, choices=(32, 40, 51, 64), help="Only HEADS page catalog probes: stored HEADS point size")
    parser.add_argument("--head-cue-first-staff", action="store_true", help="Only HEADS page catalog probes: controlled first staff specific interline 7, sheet interline unchanged")
    args = parser.parse_args()
    if args.probe not in ("head-actual-pages", "head-seed-page-catalog", "head-page-catalog") and (args.head_family is not None or args.head_point_size is not None or args.head_cue_first_staff):
        parser.error("HEADS catalog controls are only valid with a HEADS page catalog probe")
    if platform.system() != "Linux" or platform.machine() not in ("x86_64", "AMD64"):
        parser.error("This distribution is pinned for Linux x86_64 only")
    home = args.oracle_home.resolve()
    if args.bootstrap:
        bootstrap(home)
    dist = (args.distribution or home / "official/opt/audiveris/lib").resolve()
    jdk = (args.jdk or home / JDK_DIR).resolve()
    jar, java, javac = dist / "app/audiveris.jar", dist / "runtime/bin/java", jdk / "bin/javac"
    checked(jar, JAR_SHA256)
    resource_hashes = {}
    with zipfile.ZipFile(jar) as archive:
        for name, digest in {
            "res/Bravura.otf": "dca2d90c88437a701b1c2e71fa54e76f9fa41d7deee935d74dc871ea66ecfdd2",
            "res/basic-classifier.zip": "327194c47116a8ebf5771d04cea08a4e563ec93d2955e1efd990a54a9245f29e",
        }.items():
            resource_hashes[name] = hashlib.sha256(archive.read(name)).hexdigest()
            if resource_hashes[name] != digest:
                raise RuntimeError(f"Unexpected pinned JAR resource: {name}")
        if b"Implementation-Version: 5.11.0" not in archive.read("META-INF/MANIFEST.MF"):
            raise RuntimeError("Unexpected Audiveris release manifest")
    for executable in (java, javac):
        if not executable.is_file():
            raise RuntimeError(f"Missing {executable}; use --bootstrap or an explicit --jdk/--distribution")
    output = (args.output or ROOT / "evidence/generated/linux-oracle" / args.probe /
              dt.datetime.now(dt.timezone.utc).strftime("%Y%m%dT%H%M%S.%fZ")).resolve()
    output.mkdir(parents=True, exist_ok=False)
    stub, classes = output / "compile-only", output / "observer-classes"
    stub.mkdir(); classes.mkdir()
    (stub / "Book.java").write_text("package org.audiveris.omr.sheet; public class Book {}\n")
    (stub / "RunClass.java").write_text("package org.audiveris.omr.step; import java.util.SortedSet; import org.audiveris.omr.sheet.Book; public abstract class RunClass { public RunClass(Book b, SortedSet<Integer> ids){} public abstract void process(); }\n")
    if args.probe == "head-staff-catalog":
        (stub / "Staff.java").write_text("package org.audiveris.omr.sheet; import java.util.List; public class Staff { public Staff(int id,double left,double right,int interline,List lines){} public boolean isTablature(){return false;} }\n")
    if args.probe in ("key-process", "key-process-plain", "key-pitch-window-control", "key-extract-retention-control"):
        stub_packages = {
            "Staff": "org.audiveris.omr.sheet", "Shape": "org.audiveris.omr.glyph",
            "KeyRoi": "org.audiveris.omr.sheet.key", "KeyPeak": "org.audiveris.omr.sheet.key",
            "KeySlice": "org.audiveris.omr.sheet.key", "KeyAlterInter": "org.audiveris.omr.sig.inter",
        }
        for name, package in stub_packages.items():
            (stub / (name + ".java")).write_text(f"package {package}; public class {name} {{}}\n")
        (stub / "StaffHeader.java").write_text("package org.audiveris.omr.sheet.header; public class StaffHeader { public static class Range {} }\n")
        (stub / "KeyExtractor.java").write_text("package org.audiveris.omr.sheet.key; import java.util.*; import org.audiveris.omr.sheet.Staff; import org.audiveris.omr.sheet.header.StaffHeader; import org.audiveris.omr.glyph.Shape; import org.audiveris.omr.sig.inter.KeyAlterInter; public class KeyExtractor { public KeyExtractor(Staff s){} public boolean isRatherEmpty(java.awt.Rectangle r,int c,int w){return false;} public void retrieveComponents(StaffHeader.Range r,KeyRoi roi,List<KeyPeak> p,Shape sh){} public KeyAlterInter extractAlter(KeyRoi r,List<KeyPeak> p,KeySlice s,Set<Shape> sh,double g,boolean c){return null;} }\n")
    if args.grid_input_mode != "all" and args.probe != "grid-second-pass":
        parser.error("GRID input controls require grid-second-pass")
    if args.page_processing != "default" and args.probe != "head-actual-pages":
        parser.error("page processing controls require head-actual-pages")
    observer, step = PROBES[args.probe]
    sources = list(dict.fromkeys(ROOT / "scripts/java" / (name + ".java") for name in
                               ["GridProbe", "GridFinalProbe"] + (["ClefRegistrationProbe"] if args.probe == "clef-registration-control" else []) + (["KeyFallbackProbe"] if args.probe == "key-neighbor-control" else []) + (["KeyBuilderProcessProbe", "ObservedKeyExtractor"] if args.probe in ("key-process", "key-process-plain", "key-pitch-window-control", "key-extract-retention-control") else []) + (["KeyBuilderProcessProbe", "KeyColumnProbe"] if args.probe in ("key-column", "key-column-plain") else []) + (["KeyBuilderProcessProbe", "HeaderTimeColumnProbe"] if args.probe in ("time-column", "time-column-plain") else []) + (["KeyBuilderProcessProbe", "TimeSelectionControlProbe"] if args.probe in ("time-selection-control", "time-selection-control-plain") else []) + (["KeyBuilderProcessProbe", "StemSeedsProbe"] if args.probe in ("stem-seeds", "stem-seeds-plain") else []) + (["KeyBuilderProcessProbe"] if args.probe == "stem-scale-control" else []) + (["KeyBuilderProcessProbe", "StemSeedsProbe"] if args.probe == "stick-factory-control" else []) + (["KeyBuilderProcessProbe", "StemSeedsProbe", "BeamsFaithfulProbe"] if args.probe in ("beam-candidates", "beam-extension-loop", "beam-late-hooks", "beam-groups") else []) + (["KeyBuilderProcessProbe", "StemSeedsProbe", "BeamsProbe"] if args.probe in ("beams", "beams-plain", "beams-faithful", "beam-filters-control") else []) + [observer]))
    if args.probe in ("stem-chunk-head-filter", "stem-chunks"):
        sources.extend([ROOT / "scripts/java/StemChunkRegistrationProbe.java", ROOT / "scripts/java/StemPartsLookupProbe.java"])
    if args.probe in ("stem-item-events", "stem-parts-prefix"):
        sources.append(ROOT / "scripts/java/StemLengthMapProbe.java")
    if args.probe in ("stem-head-neighborhood", "stem-image-head-plan"):
        sources.append(ROOT / "scripts/java/StemHeadTargetsProbe.java")
    if args.probe in ("stem-image-head-expansion", "stem-image-head-items", "stem-image-head-chunks", "stem-image-head-parts", "stem-image-beam-generation", "stem-image-beam-targets", "stem-image-linkers", "stem-image-sources", "stem-image-stumps", "stem-image-seeds", "stem-image-head-plan", "stem-head-reference-context", "stem-head-neighborhood", "stem-head-targets", "stem-lookup-area", "stem-beam-lookup-area", "stem-chunk-registration", "stem-cache-head", "stem-length-map", "stem-item-events", "stem-parts-prefix"):
        sources.append(ROOT / "scripts/java/StemPartsLookupProbe.java")
    if args.probe in ("stem-image-head-expansion", "stem-image-head-items"):
        if args.probe == "stem-image-head-expansion":
            sources.append(ROOT / "scripts/java/StemImageHeadItemsProbe.java")
        sources.append(ROOT / "scripts/java/StemImageHeadChunksProbe.java")
    if args.probe in ("stem-image-head-expansion", "stem-image-head-items", "stem-image-head-chunks"):
        sources.append(ROOT / "scripts/java/StemImageHeadPartsProbe.java")
    if args.probe in ("stem-image-head-expansion", "stem-image-head-items", "stem-image-head-chunks", "stem-image-head-parts"):
        sources.append(ROOT / "scripts/java/StemImageBeamGenerationProbe.java")
    if args.probe in ("stem-image-head-expansion", "stem-image-head-items", "stem-image-head-chunks", "stem-image-head-parts", "stem-image-beam-generation"):
        sources.append(ROOT / "scripts/java/StemImageBeamTargetsProbe.java")
    if args.probe in ("stem-image-head-expansion", "stem-image-head-items", "stem-image-head-chunks", "stem-image-head-parts", "stem-image-beam-generation", "stem-image-beam-targets", "stem-image-linkers"):
        sources.append(ROOT / "scripts/java/StemImageSourcesProbe.java")
    if args.probe in ("head-stems-cleaner", "heads-default-page", "grid-second-pass", "stem-glyph-grade", "head-stem-stump", "head-stem-support", "head-stem-connection", "head-actual-pages", "head-all-shapes-boundary", "head-all-shapes-seed", "head-all-shapes-coverage", "head-page-catalog", "head-seed-page-catalog", "head-seed-coverage", "head-drum-coverage", "head-staff-catalog", "head-font-coverage", "head-range-prefix", "head-range-catalog", "head-candidate-filter", "head-glyph", "heads-registration", "head-staff-dependency"):
        sources = list(dict.fromkeys(sources + [ROOT / "scripts/java" / (name + ".java") for name in
            ["KeyBuilderProcessProbe", "StemSeedsProbe", "StickFactoryControlProbe", "LedgerCandidateControlProbe",
             "LedgerReferenceControlProbe", "LedgerBeamPurgeControlProbe", "LedgerRegistrationControlProbe",
             "HeadSeedSearchProbe", "HeadLedgerSearchProbe", "HeadVetoProbe", "HeadRangePrefixProbe", "HeadRangeCatalogProbe", "HeadCandidateFilterProbe", "HeadGlyphProbe", "HeadsRegistrationProbe"]]))
    if args.probe in ("head-stems-cleaner", "grid-second-pass", "stem-glyph-grade", "head-stem-stump", "head-stem-support"):
        sources.append(ROOT / "scripts/java/HeadStemConnectionProbe.java")
    if args.probe == "head-stem-stump":
        sources.append(ROOT / "scripts/java/HeadStemSupportProbe.java")
    source_snapshots = output / "observer-sources"
    source_snapshots.mkdir()
    for source in sources:
        shutil.copyfile(source, source_snapshots / source.name)
    compile_sources = [source_snapshots / source.name for source in sources]
    empty_sourcepath = output / "empty-sourcepath"
    empty_sourcepath.mkdir()
    compile_commands = [[str(javac), "--release", "21", "-d", str(stub), *map(str, sorted(stub.glob("*.java")))],
                        [str(javac), "--release", "21", "-cp", str(stub), "-sourcepath", str(empty_sourcepath), "-d", str(classes), *map(str, compile_sources)]]
    with (output / "compile.log").open("w") as log:
        for command in compile_commands:
            subprocess.run(command, check=True, stdout=log, stderr=subprocess.STDOUT)
    cp = str(dist / "app/*") + os.pathsep + str(classes)
    if str(stub) in cp:
        raise RuntimeError("Compile-only shims may not be on runtime classpath")
    provenance = {"purpose": "development-only official oracle; no native runtime dependency",
                  "upstream_commit": UPSTREAM_COMMIT, "release": "5.11.0", "jar_sha256": sha(jar),
                  "distribution_url": DEB_URL, "distribution_sha256": DEB_SHA256,
                  "compiler_url": JDK_URL, "compiler_sha256": JDK_SHA256,
                  "java_version": subprocess.run([str(java), "-version"], capture_output=True, text=True, check=True).stderr,
                  "compiler_version": subprocess.run([str(javac), "-version"], capture_output=True, text=True, check=True).stdout,
                  "runtime_executable_sha256": sha(java), "compiler_executable_sha256": sha(javac),
                  "resource_sha256": resource_hashes,
                  "dependency_jar_sha256": {p.name: sha(p) for p in sorted((dist / "app").glob("*.jar"))},
                  "platform": platform.platform(), "probe": observer, "official_cli_step": step,
                  "compile_commands": compile_commands, "runtime_classpath": cp,
                  "observer_source_sha256": {str(p.relative_to(ROOT)): sha(source_snapshots / p.name) for p in sources},
                  "observer_source_snapshots": {str(p.relative_to(ROOT)): str(source_snapshots / p.name) for p in sources},
                  "fixtures": []}
    write_json(output / "provenance.json", provenance)
    failed = False
    for item in ([args.probe] if step is None else args.fixtures):
        source = None if step is None else (Path(item).resolve() if Path(item).suffix == ".png" else ROOT / "fixtures/official-grid" / (item + ".png"))
        fixture_name = source.stem if source else args.probe
        folder = output / fixture_name
        folder.mkdir()
        for name in ("tmp", "cache", "prefs", "home", "config", "data"):
            (folder / name).mkdir()
        result_file = folder / "result.json"
        cmd = [str(java), "-Xmx512m", "-XX:ActiveProcessorCount=2", "-Djava.awt.headless=true",
               "-Djava.io.tmpdir=" + str(folder / "tmp"), "-Dorg.bytedeco.javacpp.cachedir=" + str(folder / "cache"),
               "-Djava.util.prefs.userRoot=" + str(folder / "prefs"), "-Duser.home=" + str(folder / "home"),
               "-Dnative.probe.output=" + str(result_file), "-Dnative.page.processing=" + args.page_processing, "-cp", cp, "Audiveris", "-batch",
               "-constant", "org.audiveris.omr.Main.persistBatchCliConstants=false",
               "-constant", "org.audiveris.omr.Main.runBatchTasksInParallel=false",
               "-step", step or "SCALE", "-run", observer, "-output", str(folder), "--", str(source)]
        if args.probe == "grid-second-pass":
            cmd.insert(cmd.index("-cp"), "-Dnative.grid.inputMode=" + args.grid_input_mode)
        if args.probe in ("head-actual-pages", "head-seed-page-catalog", "head-page-catalog"):
            index = cmd.index("-cp")
            cmd[index:index] = ["-Dnative.head.family=" + (args.head_family or "Bravura"),
                                "-Dnative.head.point=" + str(args.head_point_size or 51),
                                "-Dnative.head.cue=" + str(args.head_cue_first_staff).lower()]
        if step is None:
            cmd = cmd[:cmd.index("Audiveris")] + [observer, str(result_file)]
        env = dict(os.environ, HOME=str(folder / "home"), XDG_CONFIG_HOME=str(folder / "config"), XDG_DATA_HOME=str(folder / "data"), XDG_CACHE_HOME=str(folder / "cache"))
        record = {"input": str(source) if source else None, "input_sha256": sha(source) if source else None, "argv": cmd,
                  "environment_overrides": {k: env[k] for k in ("HOME", "XDG_CONFIG_HOME", "XDG_DATA_HOME", "XDG_CACHE_HOME")}}
        with (folder / "console.log").open("w") as log:
            try:
                run = subprocess.run(cmd, stdout=log, stderr=subprocess.STDOUT, env=env, timeout=args.timeout)
                record["exit_code"] = run.returncode
            except subprocess.TimeoutExpired:
                record["exit_code"] = None
                record["timeout_seconds"] = args.timeout
        record["console_sha256"] = sha(folder / "console.log")
        record["ok"] = record["exit_code"] == 0 and result_file.is_file()
        if record["ok"]:
            result = json.loads(result_file.read_text())
            record["result_sha256"] = sha(result_file)
            expected = ROOT / "fixtures/official-grid" / (fixture_name + "-clef.json")
            if args.probe == "clef-candidates" and expected.exists():
                record["preserved_fixture_sha256"] = sha(expected)
                record["matches_preserved_fixture_json"] = result == json.loads(expected.read_text())
        write_json(folder / "execution.json", record)
        provenance["fixtures"].append(record)
        write_json(output / "provenance.json", provenance)
        failed |= not record["ok"]
        print(f"{fixture_name}: {'OK' if record['ok'] else 'FAILED'}" +
              (f"; matches preserved oracle: {record['matches_preserved_fixture_json']}" if "matches_preserved_fixture_json" in record else ""), flush=True)
    print(f"Evidence: {output}", flush=True)
    return 1 if failed else 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, RuntimeError, ValueError, subprocess.CalledProcessError) as error:
        print(f"Oracle failed: {error}", file=sys.stderr)
        sys.exit(1)
