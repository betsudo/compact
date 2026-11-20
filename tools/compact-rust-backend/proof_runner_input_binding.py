#!/usr/bin/env python3
# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#  	http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Bind physical Cargo runner inputs before a proof runner is built."""

import json
from pathlib import Path
import shlex
import tomllib

from did_proof_gate import hashes, unchanged


def require(condition, message):
    if not condition:
        raise ValueError(message)


def depfile_prerequisites(raw, base):
    """Resolve only the physical prerequisites of Cargo's first Make rule."""
    logical = raw.replace("\\\n", "")
    first_rule = next(line for line in logical.splitlines() if line.strip())
    _, separator, prerequisites = first_rule.partition(": ")
    require(separator and prerequisites, "invalid actual runner dependency rule")
    paths = {str((base / path).resolve(strict=True)) for path in shlex.split(prerequisites)}
    require(paths and all(Path(path).is_file() for path in paths),
            "non-file runner prerequisite")
    return paths


def prepare_generated_inputs(root, output, report, execute, feature, digest):
    """Prepare OUT_DIR files with pinned build-script owners before the build."""
    execute("locked-build-metadata", ["cargo", "metadata", "--locked", "--offline",
                                       "--format-version", "1"], output / "build-metadata.json")
    metadata = json.loads((output / "build-metadata.json").read_text())
    pins = {(row["name"], row["version"], row.get("source")): row.get("checksum")
            for row in tomllib.loads((root / "Cargo.lock").read_text())["package"]}
    packages = {}
    dependency_roots = set()
    for package in metadata["packages"]:
        key = (package["name"], package["version"], package.get("source"))
        require(key in pins, "runner metadata package differs from root lock")
        packages[package["id"]] = package
        if package.get("source") is not None:
            directory = Path(package["manifest_path"]).resolve(strict=True).parent
            if package["source"].startswith("git+"):
                while directory != directory.parent and not (directory / ".git").exists():
                    directory = directory.parent
                require((directory / ".git").exists(), "Git dependency root missing")
            dependency_roots.add(directory)
    dependencies = {path.resolve(strict=True) for directory in dependency_roots
                    for path in directory.rglob("*") if path.is_file() and ".git" not in path.parts}
    report["dependency_source_hashes"] = hashes(dependencies)
    report["dependency_source_roots"] = sorted(map(str, dependency_roots))
    source_map = {str(Path(path).resolve(strict=True)): sha
                  for field in ("source_hashes", "dependency_source_hashes")
                  for path, sha in report[field].items()}
    require(all(path not in source_map or source_map[path] == sha
                for path, sha in report["generated_input_hashes"].items()),
            "generated input conflicts with source binding")
    preparation_map = {**source_map, **report["generated_input_hashes"]}
    execute("prepare-runner-inputs", ["cargo", "check", "--profile", "proof-checks",
                                      "--locked", "--offline", "-p", "compact-rust-proof-smoke",
                                      "--features", feature, "--message-format=json"],
            output / "prepare-build.jsonl")
    events = [json.loads(line) for line in (output / "prepare-build.jsonl").read_text().splitlines()
              if line.startswith("{")]
    target = Path(metadata["target_directory"]).resolve(strict=True)
    generated, owners = set(), []
    for event in events:
        if event.get("reason") != "build-script-executed":
            continue
        require(event["package_id"] in packages, "unknown generated-input package")
        package = packages[event["package_id"]]
        scripts = [Path(row["src_path"]).resolve(strict=True) for row in package["targets"]
                   if "custom-build" in row["kind"]]
        require(len(scripts) == 1 and str(scripts[0]) in source_map
                and digest(scripts[0]) == source_map[str(scripts[0])],
                "generated-input build owner was not source-bound")
        directory = Path(event["out_dir"]).resolve(strict=True)
        require(directory.is_relative_to(target), "generated-input directory outside Cargo target")
        files = {path.resolve(strict=True) for path in directory.rglob("*") if path.is_file()}
        generated.update(files)
        owner = {"package_id": event["package_id"], "source": str(scripts[0]),
                 "source_sha256": source_map[str(scripts[0])], "out_dir": str(directory),
                 "files": sorted(map(str, files))}
        script_log = directory.parent / "output"
        require(script_log.is_file(), "missing actual build-script output log")
        generated.add(script_log.resolve(strict=True))
        owner.update(output_log=str(script_log), output_log_sha256=digest(script_log))
        if package.get("source") is None:
            manifest = Path(package["manifest_path"]).resolve(strict=True)
            require(str(manifest) in source_map and digest(manifest) == source_map[str(manifest)],
                    "local build-script manifest was not bound")
            rerun_files = set()
            for line in script_log.read_text().splitlines():
                if line.startswith(("cargo:rerun-if-changed=", "cargo::rerun-if-changed=")):
                    path = (manifest.parent / line.split("=", 1)[1]).resolve(strict=True)
                    if path.is_dir():
                        rerun_files.update(p.resolve(strict=True) for p in path.rglob("*") if p.is_file())
                    else:
                        rerun_files.add(path)
            require(all(str(path) in preparation_map and digest(path) == preparation_map[str(path)]
                        for path in rerun_files),
                    "local generated-input rerun member absent from pre-preparation map")
            owner["rerun_inputs_before_preparation"] = {
                str(path): preparation_map[str(path)] for path in sorted(rerun_files)}
        owners.append(owner)
    require(owners and generated, "no actual build-generated inputs prepared")
    report["prepared_build_scripts"] = owners
    report["generated_build_inputs_before_compile"] = hashes(generated)
    unchanged(report["dependency_source_hashes"])
    unchanged(report["generated_build_inputs_before_compile"])


def qualify_dependency_inputs(prerequisites, inventories, digest):
    """Require each depfile path in a prior source or generated inventory."""
    require(bool(prerequisites), "empty actual runner dependency set")
    qualified, origins = {}, {}
    for path in sorted(prerequisites):
        physical = str(Path(path).resolve(strict=True))
        bindings = {name: files[physical] for name, files in inventories.items()
                    if physical in files}
        require(bindings and len(set(bindings.values())) == 1,
                "actual runner dependency missing or inconsistent in physical prebuild input map: " + physical)
        sha = next(iter(bindings.values()))
        require(digest(physical) == sha,
                "actual runner dependency changed after physical prebuild binding: " + physical)
        qualified[physical] = sha
        origins[physical] = sorted(bindings)
    return qualified, origins
