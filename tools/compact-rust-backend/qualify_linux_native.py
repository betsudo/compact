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

"""Explicit Linux full-native cold/restored qualification; never cache publication."""
import argparse
from collections import Counter
import json
import os
from pathlib import Path
import platform
import re
import resource
import subprocess
import sys
import time

import forge_circuits as adapter
import forge_native
import qualify_linux_forge as finite

ROOT = Path(__file__).resolve().parents[2]
FINGERPRINT = "9dbd59f1254a3f432d43158113a38efa833692f68293f22ab083b2fadf58c292"


def require(value, message):
    if not value:
        raise ValueError(message)


def read(path):
    return forge_native.decode(forge_native.read_bounded(path))


def configuration(value):
    require(set(value) == {"schema", "purpose", "tiny_max_k", "max_bytes", "expected_acquisitions", "expected_cached_k", "parameters"}, "unknown or missing qualification configuration")
    require(type(value["schema"]) is int and value["schema"] == 1 and isinstance(value["purpose"], str) and value["purpose"], "invalid configuration identity")
    require(type(value["tiny_max_k"]) is int and value["tiny_max_k"] == 16
            and type(value["max_bytes"]) is int and value["max_bytes"] == 268435456
            and type(value["expected_acquisitions"]) is int and value["expected_acquisitions"] == 290
            and value["expected_cached_k"] == {"17": 1}, "unreviewed experiment dimensions")
    require(set(value["parameters"]) == {str(k) for k in range(5, 18)}, "complete reviewed parameter cohort required")
    require(all(isinstance(sha, str) and re.fullmatch("[0-9a-f]{64}", sha) for sha in value["parameters"].values()), "invalid parameter digest")
    return value


def acquisitions(rows, config, route):
    require(len(rows) == config["expected_acquisitions"], "acquisition count changed")
    identities, admitted = Counter(), Counter()
    for row in rows:
        k, count = row["k"], row["rows"]
        require(type(k) is int and type(count) is int and 0 <= count <= 2**64 - 1, "invalid current model")
        require(row["parameter_digest"] == config["parameters"].get(str(k)), "current parameter differs from reviewed pin")
        require(re.fullmatch("[0-9a-f]{64}", row["source_sha256"]) is not None, "invalid current source identity")
        expected = route if k > config["tiny_max_k"] else "auto-bypass"
        require(row["acquisition_path"] == expected, "unexpected actual acquisition route")
        if k > config["tiny_max_k"]:
            admitted[str(k)] += 1
        artifacts = row["artifacts"]
        require(len(artifacts) == 3 and {a["kind"] for a in artifacts} == {"binary_ir", "prover", "verifier"}, "missing or duplicate artifact")
        for artifact in artifacts:
            require(type(artifact["size"]) is int and artifact["size"] > 0
                    and re.fullmatch("[0-9a-f]{64}", artifact["digest"]) is not None, "invalid artifact identity")
        identity = (row["source_sha256"], k, count, row["parameter_digest"],
                    tuple(sorted((a["kind"], a["digest"], a["size"]) for a in artifacts)))
        identities[identity] += 1
    require(dict(admitted) == config["expected_cached_k"], "cached circuit cohort changed")
    return identities


def printed_acquisitions(command):
    """One canonical owner: actual full-script stdout/stderr, never nested event directories."""
    rows = []
    for line in (command["stdout"] + "\n" + command["stderr"]).splitlines():
        if line.startswith("COMPACT_FORGE "):
            row = forge_native.decode(line[len("COMPACT_FORGE "):].encode())
            require(isinstance(row, dict), "printed acquisition must be an object")
            rows.append(row)
    return rows


def rust_cohorts(text):
    summaries = re.findall(r"test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out", text)
    require(summaries and all(row[0] == "ok" and int(row[2]) == 0 for row in summaries), "missing or failed printed Rust cohorts")
    names = re.findall(r"^test (.+?) \.\.\. (ok|ignored(?:,.*)?)$", text, re.MULTILINE)
    require(names, "missing printed Rust test names")
    return {"summaries": sorted(summaries), "names": sorted(names),
            "passed": sum(int(row[1]) for row in summaries), "ignored": sum(int(row[3]) for row in summaries),
            "scope": "printed Rust cohorts and names only; not inferred proof coverage"}


def prepare_native(args):
    """Shared current finite/tool/source admission; never executes a full native lap."""
    qdir = args.qualification_dir.resolve(strict=True)
    config = configuration(read(args.config))
    qualified, force = read(qdir / "qualification.json"), read(qdir / "force-policy.json")
    require(qualified["passed"] is True and qualified["public_source"] == finite.PUBLIC
            and qualified["expected_build_fingerprint"] == FINGERPRINT == force["build_fingerprint"], "current finite Linux identity qualification required")
    require(force["selection"] == "force" and force["profile"] == adapter.PROFILE
            and force["ledger_revision"] == adapter.REVISION and force["serialization"] == adapter.SERIALIZATION
            and force["keygen_recipe"] == "library-keygen/default-options/no-rng" and force["unavailable"] == "refuse", "finite policy identity mismatch")
    require(qualified["source_manifest_sha256"] == adapter.digest(Path(__file__).with_name("forge_linux_source.json")), "independent source manifest changed since finite qualification")
    finite_maps = []
    for label, route in (("cold", "forge-miss"), ("warm", "forge-hit"), ("restored", "forge-hit"), ("auto", "auto-bypass"), ("uncached", None)):
        lap = read(qdir / (label + ".json"))
        require(lap["compiler_sha256"] == qualified["tools"]["compiler"] and lap["proof_runner_sha256"] == qualified["tools"]["proof_runner"], "finite lap tool mismatch")
        finite_maps.append(finite.checked_lap(lap, route))
    require(all(row == finite_maps[0] for row in finite_maps), "finite qualification artifact identities differ")
    command = read(qdir / "cold.command.json")
    require(command["exit_code"] == 0, "finite cold command failed")
    def option(name):
        require(command["argv"].count(name) == 1, "ambiguous finite tool option")
        return Path(command["argv"][command["argv"].index(name) + 1]).resolve(strict=True)
    compiler, proof = option("--compiler"), option("--proof-runner")
    forge, zkir = Path(force["forge"]["path"]), Path(force["zkir"]["path"])
    params = Path(force["parameter_directory"])
    native_helper = compiler.with_name("compact-zkir-cache")
    for protected in (ROOT, params.resolve(strict=True), qdir):
        require(not args.output.resolve().is_relative_to(protected.resolve()), "output must not overlap source, finite qualification or parameters")
    for name, path in (("compiler", compiler), ("proof_runner", proof), ("forge", forge), ("zkir", zkir)):
        require(adapter.digest(path) == qualified["tools"][name], "current tool differs from finite qualification: " + name)
    require(force["forge"]["sha256"] == qualified["tools"]["forge"] and force["zkir"]["sha256"] == qualified["tools"]["zkir"], "finite policy tool mismatch")
    for k, sha in force["parameters"].items():
        require(config["parameters"].get(k) == sha, "finite parameter differs from full cohort")
    for k, sha in config["parameters"].items():
        require(adapter.digest(params / ("bls_midnight_2p" + k)) == sha, "full parameter bytes mismatch")
    policy = {**force, "selection": "auto", "tiny_max_k": config["tiny_max_k"], "parameters": config["parameters"]}
    policy_path = args.output / "auto-policy.json"
    policy_path.write_text(json.dumps(policy, sort_keys=True) + "\n")
    premises = {str(Path(path).resolve(strict=True)): sha for path, sha in qualified["consumer_inputs"].items()}
    for label in ("cold", "warm", "restored", "auto", "uncached"):
        path = qdir / (label + ".json")
        premises[str(path.resolve(strict=True))] = adapter.digest(path)
    for path in [args.config, args.helper, native_helper, policy_path, qdir / "qualification.json", qdir / "force-policy.json", Path(__file__), *[compiler, proof, forge, zkir], *[params / ("bls_midnight_2p" + k) for k in config["parameters"]]]:
        premises[str(path.resolve(strict=True))] = adapter.digest(path)
    def live():
        for path, sha in premises.items():
            require(adapter.digest(Path(path)) == sha, "live qualification premise changed: " + path)
    live()
    env = dict(os.environ, MIDNIGHT_PP=str(params))
    env.pop("COMPACTC_SCHEME", None)
    env.pop("COMPACT_ZKIR_SHIM", None)
    env["PATH"] = str(zkir.parent) + os.pathsep + env.get("PATH", "")
    return dict(config=config, compiler=compiler, proof=proof, forge=forge, zkir=zkir, params=params, native_helper=native_helper, policy=policy, policy_path=policy_path, premises=premises, live=live, env=env)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("qualification-dir", "config", "helper", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args(argv)
    require(platform.system() == "Linux" and platform.machine() == "x86_64", "native x86_64 Linux required")
    args.output = args.output.absolute()
    require(not args.output.exists(), "new absent output required")
    args.output.mkdir(parents=True)
    report = {"passed": False, "commands": [], "scope": "two full original native laps plus independent uncached two-fixture keygen/proof smokes; no cache publication or allowlist change"}
    start = time.monotonic()
    def run(label, command):
        print("Full Linux Forge qualification: " + label, flush=True)
        before = time.monotonic()
        # Keep full child output on disk without accumulating the full native log in RAM.
        stdout, stderr = args.output / (label + ".stdout"), args.output / (label + ".stderr")
        with stdout.open("w") as out, stderr.open("w") as err:
            result = subprocess.run(list(map(str, command)), cwd=ROOT, env=env, stdout=out, stderr=err)
        receipt = {"argv": list(map(str, command)), "exit_code": result.returncode,
                   "wall_seconds": time.monotonic() - before, "stdout_path": str(stdout), "stderr_path": str(stderr),
                   "child_highwater_bytes": resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss * 1024,
                   "memory_scope": "Linux cumulative child largest-process highwater through this command; not concurrent process-tree peak or isolated warm peak"}
        report["commands"].append(receipt)
        (args.output / (label + ".command.json")).write_text(json.dumps(receipt, indent=2) + "\n")
        if result.returncode:
            for path in (stdout, stderr):
                with path.open("rb") as stream:
                    stream.seek(max(0, path.stat().st_size - 8192))
                    print(stream.read().decode("utf-8", errors="replace"), file=sys.stderr)
        require(result.returncode == 0, "command failed: " + label)
    try:
        prepared = prepare_native(args)
        config, compiler, proof, forge, zkir, params, native_helper, policy, policy_path, premises, live, env = (
            prepared[name] for name in ("config", "compiler", "proof", "forge", "zkir", "params", "native_helper", "policy", "policy_path", "premises", "live", "env"))
        snapshot = args.output / "snapshot"
        identities, cohorts = {}, {}
        for label in ("cold", "restored"):
            live()
            plan = args.output / label
            run(label + "-plan", [sys.executable, "-B", ROOT / "tools/compact-rust-backend/forge_ci.py", "plan",
                "--compiler", compiler, "--forge", forge, "--helper", args.helper, "--proof-runner", proof,
                "--params-dir", params, "--output", plan, "--generation", "linux-native-" + label,
                "--native-script", ROOT / "tools/compact-rust-backend/check_native_ci.sh", "--native-policy", policy_path, "--native-helper", native_helper])
            require(not (plan / "cache").exists(), "fresh plan cache unexpectedly exists")
            extra = ["--snapshot-out", snapshot] if label == "cold" else []
            run(label + "-run", [sys.executable, "-B", ROOT / "tools/compact-rust-backend/forge_ci.py", "run",
                "--plan", plan, "--snapshot", args.output / "absent-snapshot" if label == "cold" else snapshot,
                "--uncached-zkir", zkir, "--max-bytes", config["max_bytes"], *extra])
            require(read(plan / "run.json")["passed"] is True and read(plan / "run.json")["mode"] == "native-script", "full native lap incomplete")
            delta = read(plan / "cache-report.json")["delta"]
            expected_cached = sum(config["expected_cached_k"].values())
            require(delta["requests"] == expected_cached and delta["errors"] == 0
                    and delta["misses"] == (expected_cached if label == "cold" else 0)
                    and delta["hits"] == (expected_cached if label == "restored" else 0),
                    "provider statistics differ from actual acquisition accounting")
            require(delta["generation_ms"] > 0 if label == "cold" else delta["generation_ms"] == 0,
                    "unexpected provider key generation")
            finite.checked_lap(read(plan / "uncached-report.json"), None)
            original = read(plan / "native-script.command.json")
            require(original["exit_code"] == 0, "original full script failed")
            rows = printed_acquisitions(original)
            identities[label] = acquisitions(rows, config, "forge-miss" if label == "cold" else "forge-hit")
            cohorts[label] = rust_cohorts(original["stdout"] + "\n" + original["stderr"])
            (args.output / (label + "-acquisitions.json")).write_text(json.dumps(rows, indent=2) + "\n")
            live()
        require(identities["cold"] == identities["restored"], "complete source/model/artifact multiset changed")
        require(cohorts["cold"] == cohorts["restored"], "printed Rust cohorts or names changed")
        snapshot_receipt = read(args.output / "cold/snapshot.command.json")
        snapshot_value = adapter.read_json(snapshot_receipt["stdout"])
        require(snapshot_receipt["exit_code"] == 0 and snapshot_value["bytes"] <= config["max_bytes"], "official snapshot exceeds unchanged budget")
        live()
        report.update(passed=True, policy=policy, premises=premises, printed_rust=cohorts, snapshot=snapshot_value,
                      acquisitions_per_lap=config["expected_acquisitions"], cached_k=config["expected_cached_k"],
                      routes={"cold": {"forge-miss": 1, "auto-bypass": 289}, "restored": {"forge-hit": 1, "auto-bypass": 289}},
                      memory_scope="per-command Linux cumulative child largest-process highwater; not concurrent process-tree peak or isolated warm peak; no cross-platform or whole-lap causal savings claim")
    except BaseException as error:
        report["error"] = repr(error)
        raise
    finally:
        report["complete_wall_seconds"] = time.monotonic() - start
        (args.output / "root.json").write_text(json.dumps(report, indent=2) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
