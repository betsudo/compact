# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
# 	http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Authenticate physical sources of this isolated constructor proof closure."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import shutil
import tarfile

import check_functional_source_sdk as sdk

GIT = str(Path(shutil.which("git")).resolve())


def require(condition, message):
    if not condition:
        raise ValueError(message)


def git(root, *arguments):
    return subprocess.check_output([GIT, "--no-replace-objects", "-C", str(root), *arguments])


def git_extras(root):
    ordinary = git(root, "ls-files", "--others", "--exclude-standard", "-z")
    ignored = git(root, "ls-files", "--others", "--ignored", "--exclude-standard", "-z")
    return {os.fsdecode(p) for p in (ordinary + ignored).split(b"\0") if p}


def git_source(root, revision):
    root = root.resolve()
    require(git(root, "rev-parse", "HEAD").decode().strip() == revision,
            "Git dependency checkout commit differs")
    require(git_extras(root) <= {".cargo-ok"}, "untracked/ignored Git dependency inputs")
    files, links = {}, {}
    for record in git(root, "ls-tree", "-rz", "--full-tree", "HEAD").split(b"\0"):
        if not record:
            continue
        description, relative = record.split(b"\t", 1)
        mode, kind, blob = description.split()
        require(kind == b"blob", "unsupported dependency Git object")
        path = root / os.fsdecode(relative)
        require(path.parent.resolve().is_relative_to(root), "dependency path escapes checkout")
        if mode == b"120000":
            require(path.is_symlink(), "tracked dependency symlink differs")
            target = os.readlink(path)
            require(path.resolve().is_relative_to(root), "dependency symlink escapes checkout")
            data = os.fsencode(target)
            links[str(path)] = target
        else:
            require(mode in (b"100644", b"100755") and path.is_file() and not path.is_symlink(),
                    "tracked dependency file kind differs")
            data = path.read_bytes()
            files[str(path)] = hashlib.sha256(data).hexdigest()
        actual = hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest()
        require(actual == blob.decode(), f"dependency Git blob bytes differ: {path}")
    marker = root / ".cargo-ok"
    if marker.exists():
        require(marker.is_file() and not marker.is_symlink(), "Cargo marker kind differs")
        files[str(marker)] = sdk.digest(marker)
    return dict(kind="git", root=str(root), revision=revision, files=files, symlinks=links)


def registry_source(package, checksum):
    root = Path(package["manifest_path"]).parent.resolve()
    archive = root.parent.parent.parent / "cache" / root.parent.name / (root.name + ".crate")
    require(checksum and archive.is_file() and sdk.digest(archive) == checksum,
            "registry archive does not match locked package checksum")
    expected = {}
    with tarfile.open(archive, "r:gz") as source:
        for member in source.getmembers():
            if member.isdir():
                continue
            require(member.isfile(), "unsupported registry archive member")
            parts = Path(member.name).parts
            require(parts and parts[0] == root.name and len(parts) > 1 and ".." not in parts,
                    "registry archive path differs")
            relative = str(Path(*parts[1:]))
            require(relative not in expected, "duplicate registry archive member")
            with source.extractfile(member) as stream:
                expected[relative] = hashlib.sha256(stream.read()).hexdigest()
    checksum_path = root / ".cargo-checksum.json"
    if checksum_path.exists():
        table = json.loads(checksum_path.read_text())
        require(table["package"] == checksum and table["files"] == expected,
                "registry source checksum table differs from locked archive")
        expected[".cargo-checksum.json"] = sdk.digest(checksum_path)
    marker = root / ".cargo-ok"
    if marker.exists():
        expected[".cargo-ok"] = sdk.digest(marker)
    sdk.equal_inventory(root, expected)
    files = {str(root / name): digest for name, digest in expected.items()}
    files[str(archive)] = checksum
    return dict(kind="registry", root=str(root), archive=str(archive), files=files,
                inventory=expected, symlinks={})


def authenticate(metadata, cohort, admitted):
    for package in metadata["packages"]:
        source = package["source"]
        if source is None:
            continue  # The caller already admits exactly five local packages.
        if source.startswith("git+"):
            root = Path(git(Path(package["manifest_path"]).parent,
                            "rev-parse", "--show-toplevel").decode().strip()).resolve()
            key = str(root)
            revision = source.rsplit("#", 1)[1]
            if key not in admitted:
                admitted[key] = git_source(root, revision)
            require(admitted[key]["revision"] == revision, "conflicting dependency revisions")
        else:
            require(source.startswith("registry+"), "unsupported source dependency")
            key = str(Path(package["manifest_path"]).parent.resolve())
            if key not in admitted:
                checksum = cohort[package["name"], package["version"], source]
                admitted[key] = registry_source(package, checksum)


def unchanged(admitted):
    for record in admitted.values():
        for name, digest in record["files"].items():
            path = Path(name)
            require(path.is_file() and not path.is_symlink() and sdk.digest(path) == digest,
                    f"physical dependency bytes changed: {path}")
        for name, target in record["symlinks"].items():
            require(Path(name).is_symlink() and os.readlink(name) == target,
                    f"dependency symlink changed: {name}")
        root = Path(record["root"])
        if record["kind"] == "git":
            require(git(root, "rev-parse", "HEAD").decode().strip() == record["revision"],
                    "dependency HEAD changed")
            require(git_extras(root) <= {".cargo-ok"}, "new dependency Git inputs")
        else:
            sdk.equal_inventory(root, record["inventory"])
