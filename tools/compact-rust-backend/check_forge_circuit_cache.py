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

"""Sequential real proof smoke controls for the matching ledger8.1.3 Forge profile.

These two fixtures are examples, not a cache whitelist. All matching current
circuits may use Forge; --uncached-zkir retains fresh key-generation qualification.
--compiler-cache-policy exercises the actual public compiler preparation boundary.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import resource
import re
import shutil
import subprocess
import sys
import tempfile
import time
import tomllib
import unittest

from check_compactc_target import check_manifest

import forge_circuits
import forge_native
from forge_circuits import (BUILD, CONSUMER, FORGE, PROFILE, CompatibilityError,
                            digest, require, validate_export, prepare)

ROOT = Path(__file__).resolve().parents[2]
QUALIFICATIONS = Path(__file__).with_name("forge_circuit_qualification.json")


def parse_args(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler", type=Path, required=True)
    parser.add_argument("--proof-runner", type=Path, required=True)
    parser.add_argument("--forge", type=Path, default=FORGE)
    parser.add_argument("--params-dir", type=Path, required=True)
    parser.add_argument("--cache-dir", type=Path, help="otherwise Forge uses its normal default/env")
    generation = parser.add_mutually_exclusive_group()
    generation.add_argument("--compiler-uncached", action="store_true", help="large workload ordinary public compiler build; no policy and no skip-zk")
    generation.add_argument("--uncached-zkir", type=Path, help="qualify fresh legacy keys; bypass Forge")
    generation.add_argument("--compiler-cache-policy", type=Path,
                            help="qualify public compiler local-cache preparation with this independent policy")
    generation.add_argument("--compiler-fallback-policy", type=Path,
                            help="qualify explicit ordinary keygen with a deliberately absent Forge provider")
    parser.add_argument("--workload-set", choices=("smoke", "ledger8-large"), default="smoke")
    parser.add_argument("--targets", choices=("rust", "both"), default="rust",
                        help="both emits TS and Rust; add --js-runtime-root to qualify actual JS execution too")
    parser.add_argument("--js-runtime-root", type=Path, help="actual JS counter execution/proof; requires both targets")
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args(argv)
    if args.js_runtime_root is not None and args.targets != "both":
        parser.error("--js-runtime-root requires --targets both")
    if args.compiler_uncached and args.workload_set != "ledger8-large":
        parser.error("--compiler-uncached is limited to explicit large workload")
    if args.workload_set == "ledger8-large":
        if args.js_runtime_root is not None:
            parser.error("large workload does not qualify JS execution")
        if args.compiler_cache_policy is None and args.compiler_fallback_policy is None and args.uncached_zkir is None and not args.compiler_uncached:
            parser.error("large workload requires public compiler policy or explicit uncached ZKIR")
    return args


def compiler_policy(args):
    return args.compiler_cache_policy or args.compiler_fallback_policy


def compiler_command(args, qualification, output):
    flags = [] if args.compiler_uncached else ["--skip-zk"]
    policy = compiler_policy(args)
    if policy is not None:
        flags = ["--zkir-cache", "local", "--zkir-cache-policy", policy]
        if args.cache_dir is not None:
            flags += ["--zkir-cache-dir", args.cache_dir]
    targets = ["--target", "ts", "--target", "rust"] if args.targets == "both" else ["--target", "rust"]
    return [args.compiler, *targets, *flags, "--rust-runtime-root",
            ROOT, ROOT / qualification["compact_source"], output]


def acquisition_path(stderr, circuit):
    """Require exactly one helper decision for this actual circuit name."""
    name = re.escape(circuit)
    routes = []
    for line in stderr.splitlines():
        cache = re.fullmatch(r"ZKIR local cache (hit|miss): " + name + r"; fresh k=[0-9]+ model checked", line)
        ordinary = re.fullmatch(r"ZKIR ordinary key generation: " + name + r"; (automatic tiny-circuit bypass|explicit unavailable policy)", line)
        if cache:
            routes.append("forge-" + cache.group(1))
        elif ordinary:
            routes.append("auto-bypass" if ordinary.group(1) == "automatic tiny-circuit bypass" else "unavailable-keygen")
    require(len(routes) == 1, "missing or ambiguous acquisition diagnostic for circuit " + circuit)
    return routes[0]


def validate_stats_provider(forge, policy):
    """Bind metrics to the provider actually selected by the compiler policy."""
    selected = json.loads(policy.read_text())["forge"]["path"]
    require(Path(selected).resolve(strict=True) == Path(forge).resolve(strict=True),
            "stats Forge executable differs from compiler policy provider")


def validate_fallback_policy(path):
    """Missing provider is deliberate; never substitute a different stats/provider executable."""
    policy = forge_native.decode(forge_native.read_bounded(path))
    require(isinstance(policy, dict) and type(policy.get("schema")) is int and policy.get("schema") == 1
            and policy.get("selection") == "force" and policy.get("unavailable") == "keygen",
            "fallback qualification requires schema1 force selection and explicit keygen")
    require(isinstance(policy.get("forge"), dict), "fallback policy requires provider identity")
    provider = policy["forge"].get("path")
    require(isinstance(provider, str) and provider and Path(provider).is_absolute(),
            "fallback provider requires an explicit absolute missing path")
    provider = Path(provider)
    require(not provider.exists() and not provider.is_symlink(),
            "fallback provider must actually be missing and not a symlink")
    return provider


JS_MARKERS = tuple(f"JS-derived Counter branch {branch} proved, verified, and ledger-applied under shared unbalanced smoke policy" for branch in (0, 1))


def validate_js_runtime(path):
    require(path.is_absolute(), "JS runtime package requires an explicit absolute path")
    root = path.resolve(strict=True)
    package = forge_native.decode(forge_native.read_bounded(root / "package.json"))
    require(package.get("name") == "@midnight-ntwrk/compact-runtime"
            and isinstance(package.get("version"), str) and package["version"], "actual compact-runtime package identity required")
    return root


def check_js_success(result):
    require(result.returncode == 0 and all(result.stdout.splitlines().count(marker) == 1 for marker in JS_MARKERS),
            "both actual JS-derived branch proof/application markers required")


def check_js_negative(result):
    require(result.returncode == 1 and "expected header tag 'midnight:verifier-key[v6]:'" in result.stderr,
            "JS-derived negative must be exact verifier header refusal")


def js_node(environment):
    require(not any(key.startswith("NODE_") for key in environment),
            "JS qualification refuses inherited NODE_* loader/runtime overrides")
    selected = shutil.which("node", path=environment.get("PATH", ""))
    require(selected is not None, "actual Node executable required")
    return Path(selected).resolve(strict=True)


def check_js_preimage_negative(result):
    require(result.returncode == 1
            and "JS official preimage differs from replayed call proof inputs" in result.stderr,
            "swapped JS preimage must be refused by the exact call-input comparison")


def validate_js_producer_files(execution):
    files = execution.get("producer_files_sha256")
    require(isinstance(files, dict) and 0 < len(files) <= 4096,
            "original JS producer inventory required")
    for path, expected in files.items():
        require(isinstance(path, str) and Path(path).is_absolute()
                and isinstance(expected, str) and re.fullmatch(r"[0-9a-f]{64}", expected),
                "producer inventory needs absolute paths and SHA256 digests")
        require(digest(Path(path)) == expected, "JS producer component changed")
    for field in ("runtime_entry", "onchain_entry", "onchain_wasm", "node_executable"):
        require(files.get(execution.get(field)) == execution.get(field + "_sha256")
                and execution.get(field) in files, "selected producer component missing from original inventory")
    return files


LARGE_QUALIFICATIONS = Path(__file__).with_name("forge_large_qualification.json")


def large_artifacts(output, expected_circuits):
    require(len(expected_circuits) == len(set(expected_circuits)), "duplicate expected circuit")
    actual = {path.stem for path in (output / "zkir").glob("*.zkir")}
    require(actual == set(expected_circuits), "complete emitted circuit roster differs")
    return {name: {kind: digest(path) for kind, path in {
        "source": output / "zkir" / (name + ".zkir"),
        "binary_ir": output / "zkir" / (name + ".bzkir"),
        "prover": output / "keys" / (name + ".prover"),
        "verifier": output / "keys" / (name + ".verifier")}.items()} for name in sorted(actual)}


def large_publication_identity(output):
    manifest = output / "compiler/contract-manifest.json"
    contract = output / "contract"
    require(manifest.is_file() and contract.is_dir(), "actual compiler publication required")
    sources = {path.relative_to(output).as_posix(): digest(path)
               for path in sorted(contract.rglob("*")) if path.is_file()}
    require(sources, "generated contract inventory must not be empty")
    return dict(manifest_sha256=digest(manifest), generated_contract_sha256=sources)


def exact_markers(stdout, markers):
    require(markers and all(stdout.splitlines().count(marker) == 1 for marker in markers),
            "complete original proof/application markers required exactly once")


def exercise_large(args, q, output, compiled, run, row):
    policy = compiler_policy(args)
    actual = {path.stem for path in (output / "zkir").glob("*.zkir")}
    require(actual == set(q["circuits"]), "complete emitted circuit roster differs")
    routes = {}
    if policy is not None:
        for name in sorted(actual):
            routes[name] = acquisition_path(compiled.stderr, name)
            if args.compiler_fallback_policy is not None:
                require(routes[name] == "unavailable-keygen", "large fallback must be explicit unavailable keygen")
    elif args.compiler_uncached:
        routes = {name: "compiler-uncached" for name in sorted(actual)}
    else:
        (output / "keys").mkdir(exist_ok=True)
        for name in sorted(actual):
            run([args.uncached_zkir, "compile", output / "zkir" / (name + ".zkir"),
                 output / "keys" / (name + ".prover"), output / "keys" / (name + ".verifier")], "uncached_keygen_" + name)
            routes[name] = "uncached"
    publication = large_publication_identity(output)
    before = large_artifacts(output, q["circuits"])
    row.update(publication_identity=publication, artifact_hashes_by_circuit=before, acquisition_paths=routes,
               proof_scope=q["proof_scope"], expected_markers=q["success_markers"])
    proof = run([args.proof_runner, q["selector"], output], "proof_and_assertions")
    exact_markers(proof.stdout, q["success_markers"])
    negative = output.parent / "negative"
    shutil.copytree(output, negative, symlinks=False)
    (negative / "keys" / (q["negative_circuit"] + ".verifier")).write_bytes(b"invalid verifier")
    rejected = run([args.proof_runner, q["selector"], negative], "corrupt_verifier_control", expected_success=False)
    check_js_negative(rejected)  # Same exact official verifier-v6 serialization refusal.
    require(large_publication_identity(output) == publication, "compiler manifest or generated contract changed")
    require(large_artifacts(output, q["circuits"]) == before, "large artifact mutated by proof consumer")
    if policy is not None or args.compiler_uncached:
        check_manifest(output)


def large_parameter_premises(authority, directory):
    """Freeze the reviewed authority and actual parameter bytes before any compiler call."""
    raw = forge_native.read_bounded(authority)
    pins = forge_native.decode(raw)["parameters"]
    require(isinstance(pins, dict) and pins, "large parameter authority must be nonempty")
    premises = {str(authority): digest(authority)}
    for k, sha in pins.items():
        require(isinstance(k, str) and re.fullmatch(r"0|[1-9][0-9]{0,2}", k)
                and int(k) <= 255 and isinstance(sha, str) and re.fullmatch(r"[0-9a-f]{64}", sha),
                "invalid large parameter authority entry")
        path = directory / ("bls_midnight_2p" + k)
        require(digest(path) == sha, "official large-workload parameter mismatch before compile")
        premises[str(path)] = sha
    require(forge_native.read_bounded(authority) == raw, "large parameter authority changed during preparation")
    return premises


def acquisition_counts(rows, large=False):
    routes = []
    for row in rows:
        if large:
            require(isinstance(row.get("acquisition_paths"), dict), "large per-circuit routes required")
            routes.extend(row["acquisition_paths"].values())
        elif row.get("acquisition_path") is not None:
            routes.append(row["acquisition_path"])
    names = ("forge-hit", "forge-miss", "auto-bypass", "unavailable-keygen")
    if large:
        names += ("compiler-uncached", "uncached")
    require(all(route in names for route in routes), "unknown acquisition route")
    return {name: routes.count(name) for name in names}


def write_report(path, report, *, large=False, expected_acquisitions=None):
    """Keep original failed phase receipts even when no acquisition was reached."""
    if large:
        available = [row for row in report["cases"] if isinstance(row.get("acquisition_paths"), dict)]
        report["acquisition_path_counts"] = acquisition_counts(available, True)
        observed = sum(report["acquisition_path_counts"].values())
        complete = len(available) == len(report["cases"]) and observed == expected_acquisitions
        report["acquisition_accounting_complete"] = complete
        if report["passed"]:
            require(complete, "successful large run requires complete circuit acquisition accounting")
    path.write_text(json.dumps(report, indent=2) + "\n")


def main():
    args = parse_args()
    for key in ("compiler_cache_policy", "compiler_fallback_policy"):
        if getattr(args, key) is not None:
            setattr(args, key, getattr(args, key).resolve(strict=True))
    js_runtime = validate_js_runtime(args.js_runtime_root) if args.js_runtime_root is not None else None
    js_capture = Path(__file__).with_name("capture_js_counter.mjs")
    js_inputs = {str(path): digest(path) for path in (js_capture, js_runtime / "package.json")} if js_runtime else {}
    policy = compiler_policy(args)
    fallback = args.compiler_fallback_policy is not None
    if policy is not None:
        if args.cache_dir is not None:
            args.cache_dir = args.cache_dir.resolve()
    for name in ("compiler", "proof_runner", "params_dir"):
        setattr(args, name, getattr(args, name).resolve(strict=True))
    if fallback:
        missing_provider = validate_fallback_policy(policy)
        require(args.cache_dir is not None,
                "fallback qualification requires an explicit absent private cache directory")
        if args.cache_dir is not None:
            require(not args.cache_dir.exists() and not args.cache_dir.is_symlink(),
                    "fallback qualification requires an absent private cache directory")
    if not args.uncached_zkir and not args.compiler_uncached and not fallback:
        args.forge = args.forge.resolve(strict=True)
    if args.compiler_cache_policy is not None:
        validate_stats_provider(args.forge, args.compiler_cache_policy)
    start = time.perf_counter()
    child_env = dict(os.environ, MIDNIGHT_PP=str(args.params_dir),
                     COMPACT_ZKIR_FORGE_BIN=str(missing_provider if fallback else args.forge))
    if args.cache_dir is not None:
        child_env["MIDNIGHT_ZKIR_FORGE_DIR"] = str(args.cache_dir.resolve())
    node = js_node(child_env) if js_runtime else None
    if node is not None:
        js_inputs[str(node)] = digest(node)
    forge_circuits.validate_consumer()
    qualifications_path = LARGE_QUALIFICATIONS if args.workload_set == "ledger8-large" else QUALIFICATIONS
    qualifications = json.loads(qualifications_path.read_text())
    large_premises = {str(path): digest(path) for path in [qualifications_path, *[ROOT / q["compact_source"] for q in qualifications.values()]]} if args.workload_set == "ledger8-large" else {}
    if args.workload_set == "ledger8-large":
        large_premises.update(large_parameter_premises(Path(__file__).with_name("forge_native_qualification.json"), args.params_dir))
    report = {"scope": "two proof smoke examples of the matching profile; not a circuit whitelist",
              "mode": "compiler-explicit-unavailable-keygen" if fallback else ("compiler-local" if args.compiler_cache_policy else ("compiler-uncached" if args.compiler_uncached else ("uncached" if args.uncached_zkir else "forge"))), "targets": args.targets, "cases": [],
              "workload_set": args.workload_set, "source_premises": large_premises,
              "consumer": CONSUMER, "forge_profile": PROFILE,
              "proof_runner_sha256": digest(args.proof_runner),
              "compiler_sha256": digest(args.compiler)}
    if args.workload_set == "ledger8-large":
        report["scope"] = "two complete larger Compact sources; bounded original selected Rust proof scenarios, not all-export proof coverage"
    if js_runtime:
        report["js_scope"] = "counter only: fresh generated JS execution, original Rust proofs plus independent JS-derived proof/application; let_return remains Rust-only"
        report["js_source_inputs"] = js_inputs

    if policy is not None:
        report["compiler_cache_policy_sha256"] = digest(policy)
        report["preparation_scope"] = "public compiler phase includes fresh ZKIR and local artifact preparation; no second adapter/keygen"

    def stats():
        argv = [str(args.forge)]
        if args.cache_dir is not None:
            argv += ["--cache-dir", str(args.cache_dir)]
        return json.loads(subprocess.run(argv + ["stats"], check=True,
                                         capture_output=True, text=True).stdout)

    if not args.uncached_zkir and not args.compiler_uncached and not fallback:
        report["stats_before"] = stats()

    class CircuitTests(unittest.TestCase):
        def exercise(self, name):
            case_start = time.perf_counter()
            q = qualifications[name]
            row = {"circuit": name}
            report["cases"].append(row)

            def run(argv, phase, *, expected_success=True):
                t = time.perf_counter()
                result = subprocess.run([str(x) for x in argv], cwd=ROOT,
                                        env=child_env, capture_output=True, text=True)
                row[phase] = {"wall_seconds": time.perf_counter() - t,
                              "exit_code": result.returncode, "stdout": result.stdout,
                              "stderr": result.stderr}
                if expected_success:
                    result.check_returncode()
                else:
                    self.assertNotEqual(result.returncode, 0, "corrupt verifier accepted")
                return result

            with tempfile.TemporaryDirectory(prefix="compact-forge-test-") as temp:
                output = Path(temp) / "generated"
                if fallback:
                    validate_fallback_policy(policy)
                    self.assertEqual(digest(policy), report["compiler_cache_policy_sha256"])
                for path, sha in large_premises.items():
                    self.assertEqual(digest(Path(path)), sha, "large source premise changed before compile")
                compiled = run(compiler_command(args, q, output), "compile")
                check_manifest(output)
                if args.workload_set == "ledger8-large":
                    exercise_large(args, q, output, compiled, run, row)
                    if policy is not None:
                        self.assertEqual(digest(policy), report["compiler_cache_policy_sha256"])
                        if fallback:
                            validate_fallback_policy(policy)
                    for path, sha in large_premises.items():
                        self.assertEqual(digest(Path(path)), sha, "large source premise changed")
                    self.assertEqual(digest(args.compiler), report["compiler_sha256"])
                    self.assertEqual(digest(args.proof_runner), report["proof_runner_sha256"])
                    row["complete_case_wall_seconds"] = time.perf_counter() - case_start
                    return
                circuit = q["circuit"]
                source = output / "zkir" / f"{circuit}.zkir"
                self.assertEqual(digest(args.params_dir / f"bls_midnight_2p{q['k']}"),
                                 q["parameter_digest"])
                if q.get("rust_fixture"):
                    formatted = output / "contract/lib.rs"
                    if policy is not None:
                        # Normalize a private comparison copy, preserving the
                        # compiler's live output/manifest for the new mode.
                        formatted = Path(temp) / "fixture-comparison.rs"
                        shutil.copyfile(output / "contract/lib.rs", formatted)
                    run(["rustfmt", "--edition", "2024", formatted], "format")
                    self.assertEqual(formatted.read_bytes(),
                                     (ROOT / q["rust_fixture"]).read_bytes())
                if policy is not None:
                    row["acquisition_path"] = acquisition_path(compiled.stderr, circuit)
                    if fallback:
                        self.assertEqual(row["acquisition_path"], "unavailable-keygen", "explicit fallback must not be automatic bypass")
                    # The actual public compiler already materialized these.
                    # Never run a second adapter/model/keygen in this mode.
                    paths = {"source": source, "binary_ir": source.with_suffix(".bzkir"),
                             "prover": output / "keys" / f"{circuit}.prover",
                             "verifier": output / "keys" / f"{circuit}.verifier"}
                elif args.uncached_zkir:
                    (output / "keys").mkdir(exist_ok=True)
                    run([args.uncached_zkir, "compile", source,
                         output / "keys" / f"{circuit}.prover",
                         output / "keys" / f"{circuit}.verifier"], "uncached_keygen")
                    paths = {"source": source, "binary_ir": source.with_suffix(".bzkir"),
                             "prover": output / "keys" / f"{circuit}.prover",
                             "verifier": output / "keys" / f"{circuit}.verifier"}
                else:
                    actual_zkir = forge_circuits.zkir_executable(child_env)
                    require(actual_zkir is not None, "matching actual zkir CLI required for fresh model/serialization checks")
                    row["forge"] = forge_circuits.compile_circuit(source,
                        output / "keys" / f"{circuit}.prover", output / "keys" / f"{circuit}.verifier",
                        actual_zkir, child_env)
                    paths = {"source": source, "binary_ir": source.with_suffix(".bzkir"),
                             "prover": output / "keys" / f"{circuit}.prover",
                             "verifier": output / "keys" / f"{circuit}.verifier"}
                artifact_hashes = {kind: digest(path) for kind, path in paths.items()}
                row["artifact_hashes"] = artifact_hashes
                proof = run([args.proof_runner, q["selector"], output], "proof_and_assertions")
                self.assertIn(q["success_marker"], proof.stdout)
                if js_runtime is not None and name == "counter":
                    for path, expected in js_inputs.items():
                        self.assertEqual(digest(Path(path)), expected, "JS producer/runtime package changed")
                    modules = Path(temp) / "node_modules/@midnight-ntwrk"
                    modules.mkdir(parents=True)
                    (modules / "compact-runtime").symlink_to(js_runtime, target_is_directory=True)
                    js_directory = Path(temp) / "js-inputs"
                    run([node, js_capture, output / "contract", js_directory], "js_execution")
                    execution = forge_native.decode(forge_native.read_bounded(js_directory / "execution.json"))
                    self.assertEqual([case["branch"] for case in execution["cases"]], [0, 1])
                    self.assertTrue(all(case["negative_argument_refused"] is True for case in execution["cases"]))
                    self.assertEqual(digest(output / "contract/index.js"), execution["generated_index_sha256"])
                    runtime_files = validate_js_producer_files(execution)
                    self.assertEqual(Path(execution["node_executable"]).resolve(strict=True), node)
                    self.assertEqual(execution["node_executable_sha256"], js_inputs[str(node)])
                    self.assertTrue(Path(execution["runtime_entry"]).resolve(strict=True).is_relative_to(js_runtime))
                    self.assertEqual(runtime_files.get(str(js_runtime / "package.json")),
                                     js_inputs[str(js_runtime / "package.json")])
                    row["js_execution_metadata"] = execution
                    expected_files = {"execution.json", "branch-0.json", "branch-1.json",
                                      "branch-0.preimage", "branch-1.preimage"}
                    self.assertEqual({path.name for path in js_directory.iterdir()}, expected_files)
                    js_files = {str(path): digest(path) for path in js_directory.iterdir() if path.is_file()}
                    js_proof = run([args.proof_runner, "--js-counter-trace", output, js_directory], "js_proof_and_assertions")
                    check_js_success(js_proof)
                    wrong_inputs = Path(temp) / "js-swapped-preimages"
                    shutil.copytree(js_directory, wrong_inputs, symlinks=False)
                    first, second = (wrong_inputs / f"branch-{branch}.preimage" for branch in (0, 1))
                    first_bytes, second_bytes = first.read_bytes(), second.read_bytes()
                    first.write_bytes(second_bytes)
                    second.write_bytes(first_bytes)
                    mismatch = run([args.proof_runner, "--js-counter-trace", output, wrong_inputs],
                                   "js_preimage_mismatch_control", expected_success=False)
                    check_js_preimage_negative(mismatch)
                # Keep negative mutation private; never alter persistent Forge exports.
                negative = Path(temp) / "negative"
                shutil.copytree(output, negative, symlinks=False)
                (negative / "keys" / f"{circuit}.verifier").write_bytes(b"invalid verifier")
                run([args.proof_runner, q["selector"], negative], "corrupt_verifier_control",
                    expected_success=False)
                if js_runtime is not None and name == "counter":
                    js_negative = run([args.proof_runner, "--js-counter-trace", negative, js_directory],
                                      "js_corrupt_verifier_control", expected_success=False)
                    check_js_negative(js_negative)
                    for path, expected in {**js_inputs, **js_files, **runtime_files}.items():
                        self.assertEqual(digest(Path(path)), expected, "JS execution/producer input changed")
                    self.assertEqual({path.name for path in js_directory.iterdir()}, expected_files)
                for kind, path in paths.items():
                    self.assertEqual(digest(path), artifact_hashes[kind], "artifact mutated by consumer")
                if policy is not None:
                    check_manifest(output)
                    self.assertEqual(digest(policy),
                                     report["compiler_cache_policy_sha256"], "compiler cache policy changed")
                    if fallback:
                        validate_fallback_policy(policy)
            row["complete_case_wall_seconds"] = time.perf_counter() - case_start

        def test_counter_two_branches(self):
            self.exercise("schnorr" if args.workload_set == "ledger8-large" else "counter")

        def test_let_return(self):
            self.exercise("merkle" if args.workload_set == "ledger8-large" else "let_return")

    result = unittest.TextTestRunner(verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(CircuitTests))
    if not args.uncached_zkir and not args.compiler_uncached and not fallback:
        report["stats_after"] = stats()
    if policy is not None:
        if args.workload_set != "ledger8-large":
            report["acquisition_path_counts"] = acquisition_counts(report["cases"])
        report["mode_scope"] = ("explicit unavailable-keygen; missing provider is not invoked or queried for stats; original Rust proofs always execute; optional JS scope reported separately" if fallback else "compiler-local selects public preparation; per-case acquisition_path states whether Forge was used")
        if fallback:
            validate_fallback_policy(policy)
            if args.cache_dir is not None:
                require(not args.cache_dir.exists() and not args.cache_dir.is_symlink(),
                        "explicit fallback unexpectedly used a cache directory")
        require(digest(policy) == report["compiler_cache_policy_sha256"], "compiler policy changed")
    report.update(passed=result.wasSuccessful(), tests_run=result.testsRun,
                  complete_wall_seconds=time.perf_counter() - start,
                  cumulative_child_largest_process_maxrss_bytes=(
                      resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
                      * (1 if sys.platform == "darwin" else 1024)),
                  memory_scope="cumulative child largest-process highwater, not concurrent total")
    write_report(args.report, report, large=args.workload_set == "ledger8-large",
                 expected_acquisitions=sum(len(q["circuits"]) for q in qualifications.values())
                 if args.workload_set == "ledger8-large" else None)
    return 0 if result.wasSuccessful() else 1


if __name__ == "__main__":
    raise SystemExit(main())
