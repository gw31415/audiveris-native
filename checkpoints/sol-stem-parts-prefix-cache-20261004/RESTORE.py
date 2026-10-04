#!/usr/bin/env python3
"""Restore the exact source from this checkpoint and its verified predecessor chain.

python checkpoints/sol-stem-parts-prefix-cache-20261004/RESTORE.py \
    --repository /path/to/fresh/main/clone --destination /path/to/new/restore

Uses only the supplied archival Git checkout. Refuses an existing destination.
No downloads, credentials, provider calls, toolchain installs or configuration edits.
"""
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tarfile


def sha(data):
    return hashlib.sha256(data).hexdigest()


def restore(repo, root):
    latest = json.loads((repo / "checkpoints/sol-stem-parts-prefix-cache-20261004/MANIFEST.json").read_text())
    assert not root.exists(), "destination already exists"
    root.mkdir(parents=True)
    art = root / "artifacts"
    art.mkdir()
    source = art / "audiveris-source"
    git_dir = root / "source-git"
    env = dict(os.environ, GIT_DIR=str(git_dir), GIT_WORK_TREE=str(source))

    def run(*args):
        return subprocess.check_output(args, cwd=source, env=env,
                                       stderr=subprocess.STDOUT).decode().strip()

    def verify(path, digest):
        assert sha(path.read_bytes()) == digest, str(path)

    def extract(path):
        with tarfile.open(path) as archive:
            archive.extractall(art, filter="data")

    def check(manifest):
        assert run("git", "rev-parse", "HEAD") == manifest["local_implementation_commit"]
        assert run("git", "write-tree") == manifest["local_implementation_tree"]
        for name, digest in manifest.get("file_sha256", {}).items():
            verify(source / name, digest)
        assert len(run("git", "ls-files").splitlines()) == manifest["tracked_files"]
        assert not run("git", "status", "--porcelain")

    base = json.loads((repo / "checkpoints/sol-snapshot-loop-20261003/MANIFEST.json").read_text())
    path = repo / base["base_outer_archive_path"]
    verify(path, base["base_outer_archive_sha256"])
    extract(path)
    path = art / "verified-source-8ebbf976-20261003/source.tar.xz"
    verify(path, base["base_source_archive_sha256"])
    extract(path)
    path = repo / base["archive"]
    verify(path, base["archive_sha256"])
    extract(path)
    run("git", "init")
    run("git", "add", "-f", ".")
    assert run("git", "write-tree") == base["base_source_tree"]
    commit = run("git", "hash-object", "-t", "commit", "-w",
                 str(art / "sol-snapshot-loop-20261003/BASE-COMMIT.txt"))
    assert commit == base["reconstructed_local_base_commit"]
    run("git", "update-ref", "HEAD", commit)
    bundle = art / "sol-snapshot-loop-20261003/history.bundle"
    run("git", "bundle", "verify", str(bundle))
    run("git", "fetch", str(bundle), base["local_implementation_commit"])
    run("git", "checkout", "--detach", base["local_implementation_commit"])
    check(base)

    manifests = {}
    for path in (repo / "checkpoints").glob("*/MANIFEST.json"):
        item = json.loads(path.read_text())
        if item.get("patch") or item.get("bundle"):
            manifests[item["local_implementation_commit"]] = item
    chain = []
    commit = latest["local_implementation_commit"]
    while commit != base["local_implementation_commit"]:
        item = manifests[commit]
        assert item not in chain, "checkpoint cycle"
        chain.append(item)
        commit = item.get("parent_implementation_commit") or item["bundle"]["prerequisite"]
    for item in reversed(chain):
        if item.get("bundle"):
            description = item["bundle"]
            path = repo / description["path"]
            verify(path, description["base64_sha256"])
            data = base64.b64decode(path.read_bytes())
            assert len(data) == description["decoded_bytes"]
            assert sha(data) == description["decoded_sha256"]
            bundle = art / (item["local_implementation_commit"] + ".bundle")
            bundle.write_bytes(data)
            run("git", "bundle", "verify", str(bundle))
            run("git", "fetch", str(bundle), item["local_implementation_commit"])
            run("git", "checkout", "--detach", item["local_implementation_commit"])
        else:
            path = repo / item["patch"]["path"]
            verify(path, item["patch"]["sha256"])
            assert path.stat().st_size == item["patch"]["bytes"]
            run("git", "apply", "--check", str(path))
            run("git", "apply", "--index", str(path))
            assert run("git", "write-tree") == item["local_implementation_tree"]
            commit = run("git", "hash-object", "-t", "commit", "-w",
                         str(repo / item["implementation_commit_preimage"]))
            assert commit == item["local_implementation_commit"]
            run("git", "update-ref", "HEAD", commit)
        check(item)
        print("Verified", item["local_implementation_commit"], flush=True)
    hash_file = repo / latest["all_source_sha256"]["path"]
    verify(hash_file, latest["all_source_sha256"]["sha256"])
    hashes = json.loads(hash_file.read_text())
    assert set(hashes) == set(run("git", "ls-files").splitlines())
    for name, digest in hashes.items():
        verify(source / name, digest)
    check(latest)
    receipt = {"status": "passed", "source_commit": latest["local_implementation_commit"],
               "source_tree": latest["local_implementation_tree"],
               "all_fresh_restored_sha256_verified": len(hashes), "source_clean": True,
               "source": str(source), "source_git": str(git_dir)}
    (root / "RESTORE-VERIFIED.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps(receipt, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", type=Path, required=True)
    parser.add_argument("--destination", type=Path, required=True)
    args = parser.parse_args()
    restore(args.repository.resolve(), args.destination.resolve())
