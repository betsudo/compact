#!/usr/bin/env python3

# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Bounded Linux whole-contract comparison; genuine proofs every lap, no publication."""
import argparse
import json
from pathlib import Path
import platform
import statistics
import subprocess
import sys
import time
from types import SimpleNamespace

import check_forge_circuit_cache as harness
import qualify_linux_native as native

ROOT = native.ROOT
LIMIT = 268435456


def identities(value, expected_route):
    native.require(value.get("passed") is True and value.get("workload_set") == "ledger8-large"
                   and value.get("acquisition_accounting_complete") is True, "complete genuine large loop required")
    expected = native.read(harness.LARGE_QUALIFICATIONS)
    cases = value["cases"]
    native.require(len(cases) == len(expected) and {row["circuit"] for row in cases} == set(expected),
                   "whole contract roster changed")
    result, routes = {}, {}
    for row in cases:
        config = expected[row["circuit"]]
        artifacts = row["artifact_hashes_by_circuit"]
        native.require(set(artifacts) == set(config["circuits"]), "export roster changed")
        native.require(set(row["acquisition_paths"]) == set(config["circuits"]), "route roster changed")
        proof, negative = row["proof_and_assertions"], row["corrupt_verifier_control"]
        native.require(proof["exit_code"] == 0, "genuine proof command failed")
        harness.exact_markers(proof["stdout"], config["success_markers"])
        harness.check_js_negative(SimpleNamespace(returncode=negative["exit_code"], stderr=negative["stderr"]))
        for route in row["acquisition_paths"].values():
            routes[route] = routes.get(route, 0) + 1
        result[row["circuit"]] = dict(artifacts=artifacts, publication=row["publication_identity"])
    wanted = {"compiler-uncached": 9} if expected_route == "ordinary" else {expected_route: 6, "auto-bypass": 3}
    native.require(routes == wanted, "expected complete nine-circuit acquisition routes changed")
    if expected_route != "ordinary":
        before, after = (value[name]["totals"] for name in ("stats_before", "stats_after"))
        expected = dict(requests=6, hits=6 if expected_route == "forge-hit" else 0,
                        misses=6 if expected_route == "forge-miss" else 0, errors=0)
        native.require(all(after[key]-before[key] == count for key, count in expected.items()),
                       "provider request accounting differs from actual compiler routes")
        if expected_route == "forge-hit":
            native.require(after["generation_ms"] == before["generation_ms"], "warm loop regenerated key material")
    return result


def comparison(rows):
    direct = statistics.median(rows[name]["whole_command_seconds"] for name in ("ordinary-1", "ordinary-2"))
    warm = statistics.median(rows[name]["whole_command_seconds"] for name in ("warm-1", "warm-2"))
    native.require(direct > 0 and warm > 0, "positive whole-command durations required")
    return dict(median_ordinary_seconds=direct, median_warm_seconds=warm,
                absolute_savings_seconds=direct-warm, relative_savings=(direct-warm)/direct)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("compiler", "proof-runner", "forge", "params-dir", "qualification-dir", "helper", "output"):
        parser.add_argument("--"+name, type=Path, required=True)
    args = parser.parse_args(argv)
    native.require(platform.system() == "Linux" and platform.machine() == "x86_64", "native Linux required")
    output = args.output.resolve()
    native.require(not output.exists() and not output.is_relative_to(ROOT), "fresh external value-loop output required")
    output.mkdir(parents=True)
    prepared = native.prepare_native(SimpleNamespace(qualification_dir=args.qualification_dir,
        config=ROOT/"tools/compact-rust-backend/forge_native_qualification.json", helper=args.helper, output=output))
    for name, actual in (("compiler", args.compiler), ("proof", args.proof_runner), ("forge", args.forge), ("params", args.params_dir)):
        native.require(actual.resolve(strict=True) == prepared[name].resolve(strict=True), "value loop tool differs: "+name)
    # This experiment measures the existing default cutoff, not the native capacity experiment's 16.
    policy = output/"auto8-policy.json"
    policy.write_text(json.dumps({**prepared["policy"], "tiny_max_k": 8}, sort_keys=True)+"\n")
    qualifications = native.read(harness.LARGE_QUALIFICATIONS)
    inputs = [Path(__file__), Path(harness.__file__), harness.LARGE_QUALIFICATIONS, policy,
              *[ROOT/row["compact_source"] for row in qualifications.values()]]
    pins = {str(path.resolve(strict=True)): native.adapter.digest(path) for path in inputs}

    def live():
        prepared["live"]()
        for path, sha in pins.items():
            native.require(native.adapter.digest(Path(path)) == sha, "value-loop premise changed: "+path)

    env = {**prepared["env"]}
    env.pop("COMPACT_ZKIR_MODE", None)
    env.pop("COMPACT_ZKIR_FORGE_BIN", None)

    def command(name, argv):
        live()
        start = time.monotonic()
        completed = subprocess.run(list(map(str, argv)), cwd=ROOT, env=env, capture_output=True, text=True)
        receipt = dict(argv=list(map(str, argv)), exit_code=completed.returncode,
                       whole_command_seconds=time.monotonic()-start, stdout=completed.stdout, stderr=completed.stderr)
        (output/(name+".command.json")).write_text(json.dumps(receipt, indent=2)+"\n")
        live()
        native.require(completed.returncode == 0, "value-loop command failed: "+name)
        return receipt

    common = [sys.executable, "-B", Path(harness.__file__), "--workload-set", "ledger8-large",
              "--compiler", prepared["compiler"], "--proof-runner", prepared["proof"],
              "--forge", prepared["forge"], "--params-dir", prepared["params"]]
    rows, maps = {}, []
    store = output/"measured-store"

    def lap(name, route, cache):
        report = output/(name+".json")
        flags = ["--compiler-uncached"] if route == "ordinary" else ["--compiler-cache-policy", policy]
        receipt = command(name, [*common, "--cache-dir", cache, *flags, "--report", report])
        value = native.read(report)
        native.require(value["compiler_sha256"] == native.adapter.digest(prepared["compiler"])
                       and value["proof_runner_sha256"] == native.adapter.digest(prepared["proof"]), "lap tool identity changed")
        if route != "ordinary":
            native.require(value["compiler_cache_policy_sha256"] == pins[str(policy)], "lap policy differs")
        maps.append(identities(value, route))
        native.require(all(item == maps[0] for item in maps), "artifact/manifest identity changed across laps")
        rows[name] = dict(whole_command_seconds=receipt["whole_command_seconds"], acquisition_paths=value["acquisition_path_counts"])
        if route != "ordinary":
            rows[name]["provider_delta"] = {key: value["stats_after"]["totals"][key]-value["stats_before"]["totals"][key]
                for key in ("requests", "hits", "misses", "errors", "generation_ms", "total_request_ms")}

    for name, route in (("ordinary-1", "ordinary"), ("cold", "forge-miss"), ("warm-1", "forge-hit"),
                        ("warm-2", "forge-hit"), ("ordinary-2", "ordinary")):
        lap(name, route, store)
    scope = [flag for sha in sorted(set(prepared["policy"]["parameters"].values())) for flag in ("--parameter-digest", sha)]
    partition = native.adapter.digest(policy)
    snapshot = output/"stopped-snapshot"
    saved = command("snapshot", [prepared["forge"], "--profile", "ledger8-v2", "--cache-dir", store,
        "snapshot", "--output", snapshot, "--max-bytes", LIMIT, "--partition", partition, *scope])
    saved_value = native.adapter.read_json(saved["stdout"])
    native.require(saved_value["bytes"] <= LIMIT, "snapshot exceeds unchanged private budget")
    restored = output/"restored-store"
    command("restore", [prepared["forge"], "--profile", "ledger8-v2", "--cache-dir", restored, "restore", snapshot,
        "--expected-id", saved_value["snapshot_id"], "--partition", partition, "--max-bytes", LIMIT, *scope])
    lap("restored", "forge-hit", restored)
    summary = dict(passed=True, rows=rows, **comparison(rows), pins=pins, snapshot=saved_value,
                   artifact_and_manifest_equality=True,
                   scope="bounded two-contract Linux loop, settled tools; not full-CI causal savings or remote transfer measurement")
    (output/"comparison.json").write_text(json.dumps(summary, indent=2)+"\n")
    print(json.dumps(summary))


if __name__ == "__main__":
    main()
