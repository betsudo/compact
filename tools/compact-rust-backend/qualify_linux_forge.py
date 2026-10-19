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

"""Manual candidate qualification, never production fingerprint admission."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import sys
import time

import forge_circuits as adapter
import forge_native

ROOT = Path(__file__).resolve().parents[2]
PUBLIC = "d64f52db7239b0ca4891fb0316dca6adb54a5db5"
FP_PREFIX = ["Cargo.lock", "src/backend.rs", "src/engine.rs", "src/zkir3.rs", "src/worker.rs", "src/zkir31.rs", "Cargo.toml", "flake.lock"]
SOURCE_FILES = set(FP_PREFIX + ["src/domain.rs", "src/application.rs", "build.rs", "flake.nix"])


def require(value, message):
    if not value:
        raise ValueError(message)


def read(path):
    return forge_native.decode(forge_native.read_bounded(path))


def fingerprint(source, manifest, rustc_verbose):
    require(set(manifest) == {"schema", "source_commit", "files"} and manifest["schema"] == 1,
            "unsupported independent source manifest")
    require(manifest["source_commit"] == PUBLIC and set(manifest["files"]) == SOURCE_FILES,
            "source manifest must pin complete reviewed public fingerprint implementation")
    raw = {}
    for name, expected in manifest["files"].items():
        path = source / name
        require(not path.is_symlink() and adapter.digest(path) == expected, "pinned Forge source changed: " + name)
        raw[name] = forge_native.read_bounded(path)
    require("release: 1.98.1\n" in rustc_verbose and "host: x86_64-unknown-linux-gnu\n" in rustc_verbose,
            "Forge Nix Rust compiler cohort mismatch")
    # Exact order from independently pinned application.rs; build.rs replaces all newlines.
    material = b"".join(raw[name] for name in FP_PREFIX)
    material += rustc_verbose.replace("\n", ";").encode()
    material += raw["src/domain.rs"] + b"x86_64" + b"linux"
    return hashlib.sha256(material).hexdigest()


def check_derivation(value, forge, rustc_root, manifest):
    roots = value.get("derivations", value)
    require(isinstance(roots, dict), "invalid Forge derivation capture")
    matches = [row for row in roots.values() if isinstance(row, dict)
               and row.get("env", {}).get("out") == str(forge.parent.parent)]
    require(len(matches) == 1, "exact actual Forge output derivation required")
    row = matches[0]
    require(row["system"] == "x86_64-linux", "Forge derivation platform mismatch")
    require(str(rustc_root) in row["env"].get("nativeBuildInputs", "").split(),
            "selected Rust toolchain is not actual Forge derivation input")
    source = Path(row["env"]["src"])
    for name, sha in manifest["files"].items():
        if name != "flake.nix":
            require(adapter.digest(source / name) == sha, "actual Forge derivation source differs: " + name)
    return source


def checked_lap(value, route):
    require(value.get("passed") is True and type(value.get("tests_run")) is int and value.get("tests_run") == 2 and len(value.get("cases", [])) == 2,
            "qualification requires two actual passed fixture tests")
    rows = {}
    for row in value["cases"]:
        require(row["circuit"] in ("counter", "let_return") and row["circuit"] not in rows, "unexpected fixture accounting")
        require(row["proof_and_assertions"]["exit_code"] == 0 and row["corrupt_verifier_control"]["exit_code"] == 1,
                "actual proof/negative controls missing")
        negative = row["corrupt_verifier_control"].get("stderr", "")
        require("expected header tag 'midnight:verifier-key[v6]:'" in negative,
                "negative must be verifier serialization refusal")
        if route is not None:
            require(row.get("acquisition_path") == route, "unexpected actual acquisition route")
        require(set(row["artifact_hashes"]) == {"source", "binary_ir", "prover", "verifier"}
                and all(isinstance(sha, str) and re.fullmatch("[0-9a-f]{64}", sha) for sha in row["artifact_hashes"].values()),
                "incomplete artifact identity map")
        rows[row["circuit"]] = row["artifact_hashes"]
    return rows


def logical_stats(raw):
    value = adapter.read_json(raw)
    # SQLite housekeeping may change physical storage without an ensure request.
    # Keep complete physical statistics in the original command receipts.
    return {name: value[name] for name in ("totals", "files", "recent_requests")}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ["compiler", "proof-runner", "forge", "zkir", "forge-source", "rustc", "forge-derivation", "source-manifest", "params-dir", "output"]:
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args(argv)
    require(platform.system() == "Linux" and platform.machine() == "x86_64", "actual native x86_64 Linux executor required")
    selected_rustc_root = args.rustc.absolute().parent.parent
    for key, value in vars(args).items():
        setattr(args, key, value.resolve(strict=key != "output"))
    require(not args.output.exists(), "fresh absent qualification output required")
    args.output.mkdir(parents=True)
    report = {"passed": False, "scope": "candidate Linux two-fixture cold/warm/restored/auto/uncached proof qualification; no production allowlist change", "commands": []}
    env = dict(os.environ, MIDNIGHT_PP=str(args.params_dir))
    env.pop("COMPACTC_SCHEME", None)
    env.pop("COMPACT_ZKIR_SHIM", None)
    env["COMPACT_ZKIR_MODE"] = "uncached"
    env["PATH"] = str(args.zkir.parent) + os.pathsep + env.get("PATH", "")
    started = time.monotonic()
    # Tool and source premises are established before producer profile is read.

    def run(label, command):
        print("Linux Forge qualification: " + label, flush=True)
        before = time.monotonic()
        completed = subprocess.run(list(map(str, command)), env=env, cwd=ROOT, capture_output=True, text=True)
        row = {"label": label, "argv": list(map(str, command)), "exit_code": completed.returncode,
               "wall_seconds": time.monotonic() - before, "stdout": completed.stdout, "stderr": completed.stderr}
        report["commands"].append(row)
        (args.output / (label + ".command.json")).write_text(json.dumps(row, indent=2) + "\n")
        if completed.returncode:
            print(completed.stdout[-8192:], file=sys.stderr)
            print(completed.stderr[-8192:], file=sys.stderr)
        completed.check_returncode()
        return completed.stdout

    try:
        tools = {name: adapter.digest(getattr(args, name)) for name in ["compiler", "proof_runner", "forge", "zkir", "rustc"]}
        source_manifest_bytes = forge_native.read_bounded(args.source_manifest)
        source_manifest = forge_native.decode(source_manifest_bytes)
        derivation_bytes = forge_native.read_bounded(args.forge_derivation)
        check_derivation(forge_native.decode(derivation_bytes), args.forge, selected_rustc_root, source_manifest)
        verbose = run("rustc-verbose", [args.rustc, "-vV"])
        expected = fingerprint(args.forge_source, source_manifest, verbose)
        report.update(expected_build_fingerprint=expected, source_manifest_sha256=hashlib.sha256(source_manifest_bytes).hexdigest(), tools=tools, public_source=PUBLIC)
        profile = adapter.read_json(run("profile", [args.forge, "--profile", "ledger8-v2", "profile"]))
        require(profile["build_fingerprint"] == expected and profile["profile"] == adapter.PROFILE
                and profile["ledger_revision"] == adapter.REVISION, "producer differs from independently derived expected identity")
        qualification_path = Path(__file__).with_name("forge_circuit_qualification.json")
        fixtures = read(qualification_path)
        inputs = [Path(__file__), qualification_path, Path(__file__).with_name("check_forge_circuit_cache.py"),
                  Path(__file__).with_name("forge_circuits.py"), Path(__file__).with_name("forge_native.py")]
        inputs += [ROOT / row["compact_source"] for row in fixtures.values()]
        inputs += [ROOT / row["rust_fixture"] for row in fixtures.values() if row.get("rust_fixture")]
        input_hashes = {str(path): adapter.digest(path) for path in inputs}
        report["consumer_inputs"] = input_hashes
        parameters = {str(row["k"]): row["parameter_digest"] for row in fixtures.values()}
        for k, expected_sha in parameters.items():
            require(adapter.digest(args.params_dir / ("bls_midnight_2p" + k)) == expected_sha, "official fixture parameter digest mismatch")
        policy = dict(schema=1, selection="force", tiny_max_k=8,
            forge=dict(path=str(args.forge), sha256=tools["forge"]), zkir=dict(path=str(args.zkir), sha256=tools["zkir"]),
            profile=adapter.PROFILE, ledger_revision=adapter.REVISION, build_fingerprint=expected,
            serialization=adapter.SERIALIZATION, keygen_recipe="library-keygen/default-options/no-rng",
            parameter_directory=str(args.params_dir), parameters=parameters, unavailable="refuse")
        force_policy = args.output / "force-policy.json"
        force_policy.write_text(json.dumps(policy, sort_keys=True))
        store, restored = args.output / "cold-store", args.output / "restored-store"
        snapshots = args.output / "snapshot"
        base = [sys.executable, "-B", ROOT / "tools/compact-rust-backend/check_forge_circuit_cache.py", "--compiler", args.compiler,
                "--proof-runner", args.proof_runner, "--forge", args.forge, "--params-dir", args.params_dir]
        results = {}
        def lap(name, cache, selected_policy, route, uncached=False):
            destination = args.output / (name + ".json")
            extra = ["--uncached-zkir", args.zkir] if uncached else ["--compiler-cache-policy", selected_policy]
            run(name, [*base, "--cache-dir", cache, *extra, "--report", destination])
            value = read(destination)
            require(value["compiler_sha256"] == tools["compiler"] and value["proof_runner_sha256"] == tools["proof_runner"], "lap tool identity differs")
            results[name] = checked_lap(value, route)
            return value
        lap("cold", store, force_policy, "forge-miss")
        lap("warm", store, force_policy, "forge-hit")
        partition = hashlib.sha256(force_policy.read_bytes()).hexdigest()
        scope = [item for sha in sorted(set(parameters.values())) for item in ["--parameter-digest", sha]]
        snapshot = adapter.read_json(run("snapshot", [args.forge, "--profile", "ledger8-v2", "--cache-dir", store,
            "snapshot", "--output", snapshots, "--max-bytes", "268435456", "--partition", partition, *scope]))
        run("restore", [args.forge, "--profile", "ledger8-v2", "--cache-dir", restored, "restore", snapshots,
            "--expected-id", snapshot["snapshot_id"], "--partition", partition, "--max-bytes", "268435456", *scope])
        lap("restored", restored, force_policy, "forge-hit")
        auto_policy = args.output / "auto-policy.json"
        auto_policy.write_text(json.dumps({**policy, "selection": "auto"}, sort_keys=True))
        stats_before = run("stats-before-auto", [args.forge, "--cache-dir", restored, "stats"])
        lap("auto", restored, auto_policy, "auto-bypass")
        stats_after = run("stats-after-auto", [args.forge, "--cache-dir", restored, "stats"])
        require(logical_stats(stats_before) == logical_stats(stats_after), "automatic bypass changed provider requests")
        lap("uncached", restored, force_policy, None, uncached=True)
        require(all(rows == results["cold"] for rows in results.values()), "artifact identities differ across qualification laps")
        require(logical_stats(stats_after) == logical_stats(run("stats-after-uncached", [args.forge, "--cache-dir", restored, "stats"])), "uncached qualification changed provider requests")
        for path, sha in input_hashes.items():
            require(adapter.digest(Path(path)) == sha, "consumer source premise changed: " + path)
        for name, expected_sha in tools.items():
            require(adapter.digest(getattr(args, name)) == expected_sha, "tool changed during qualification")
        require(forge_native.read_bounded(args.forge_derivation) == derivation_bytes, "Forge derivation capture changed")
        check_derivation(forge_native.decode(derivation_bytes), args.forge, selected_rustc_root, source_manifest)
        require(forge_native.read_bounded(args.source_manifest) == source_manifest_bytes, "independent source manifest changed")
        require(fingerprint(args.forge_source, source_manifest, verbose) == expected, "pinned source changed")
        for k, sha in parameters.items():
            require(adapter.digest(args.params_dir / ("bls_midnight_2p" + k)) == sha, "parameter changed")
        report.update(passed=True, artifact_maps=results, snapshot=snapshot,
                      mode_scope="force and automatic actual routes; no allowlist admission, test verdict cache or hosted global speed claim")
    except BaseException as error:
        report["error"] = repr(error)
        raise
    finally:
        report["complete_wall_seconds"] = time.monotonic() - started
        (args.output / "qualification.json").write_text(json.dumps(report, indent=2) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
