#!/usr/bin/env python3
# This file is part of Compact.
# Copyright (C) 2026 contributors to Minokawa Compact
# SPDX-License-Identifier: Apache-2.0

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
    generation.add_argument("--uncached-zkir", type=Path, help="qualify fresh legacy keys; bypass Forge")
    generation.add_argument("--compiler-cache-policy", type=Path,
                            help="qualify public compiler local-cache preparation with this independent policy")
    parser.add_argument("--report", type=Path, required=True)
    return parser.parse_args(argv)


def compiler_command(args, qualification, output):
    flags = ["--skip-zk"]
    if args.compiler_cache_policy is not None:
        flags = ["--zkir-cache", "local", "--zkir-cache-policy", args.compiler_cache_policy]
        if args.cache_dir is not None:
            flags += ["--zkir-cache-dir", args.cache_dir]
    return [args.compiler, "--target", "rust", *flags, "--rust-runtime-root",
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


def main():
    args = parse_args()
    if args.compiler_cache_policy is not None:
        args.compiler_cache_policy = args.compiler_cache_policy.resolve(strict=True)
        if args.cache_dir is not None:
            args.cache_dir = args.cache_dir.resolve()
    for name in ("compiler", "proof_runner", "params_dir"):
        setattr(args, name, getattr(args, name).resolve(strict=True))
    if not args.uncached_zkir:
        args.forge = args.forge.resolve(strict=True)
    if args.compiler_cache_policy is not None:
        validate_stats_provider(args.forge, args.compiler_cache_policy)
    start = time.perf_counter()
    child_env = dict(os.environ, MIDNIGHT_PP=str(args.params_dir), COMPACT_ZKIR_FORGE_BIN=str(args.forge))
    if args.cache_dir is not None:
        child_env["MIDNIGHT_ZKIR_FORGE_DIR"] = str(args.cache_dir.resolve())
    forge_circuits.validate_consumer()
    qualifications = json.loads(QUALIFICATIONS.read_text())
    report = {"scope": "two proof smoke examples of the matching profile; not a circuit whitelist",
              "mode": "compiler-local" if args.compiler_cache_policy else ("uncached" if args.uncached_zkir else "forge"), "cases": [],
              "consumer": CONSUMER, "forge_profile": PROFILE,
              "proof_runner_sha256": digest(args.proof_runner),
              "compiler_sha256": digest(args.compiler)}

    if args.compiler_cache_policy is not None:
        report["compiler_cache_policy_sha256"] = digest(args.compiler_cache_policy)
        report["preparation_scope"] = "public compiler phase includes fresh ZKIR and local artifact preparation; no second adapter/keygen"

    def stats():
        argv = [str(args.forge)]
        if args.cache_dir is not None:
            argv += ["--cache-dir", str(args.cache_dir)]
        return json.loads(subprocess.run(argv + ["stats"], check=True,
                                         capture_output=True, text=True).stdout)

    if not args.uncached_zkir:
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
                compiled = run(compiler_command(args, q, output), "compile")
                check_manifest(output)
                circuit = q["circuit"]
                source = output / "zkir" / f"{circuit}.zkir"
                self.assertEqual(digest(args.params_dir / f"bls_midnight_2p{q['k']}"),
                                 q["parameter_digest"])
                if q.get("rust_fixture"):
                    formatted = output / "contract/lib.rs"
                    if args.compiler_cache_policy is not None:
                        # Normalize a private comparison copy, preserving the
                        # compiler's live output/manifest for the new mode.
                        formatted = Path(temp) / "fixture-comparison.rs"
                        shutil.copyfile(output / "contract/lib.rs", formatted)
                    run(["rustfmt", "--edition", "2024", formatted], "format")
                    self.assertEqual(formatted.read_bytes(),
                                     (ROOT / q["rust_fixture"]).read_bytes())
                if args.compiler_cache_policy is not None:
                    row["acquisition_path"] = acquisition_path(compiled.stderr, circuit)
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
                # Keep negative mutation private; never alter persistent Forge exports.
                negative = Path(temp) / "negative"
                shutil.copytree(output, negative, symlinks=False)
                (negative / "keys" / f"{circuit}.verifier").write_bytes(b"invalid verifier")
                run([args.proof_runner, q["selector"], negative], "corrupt_verifier_control",
                    expected_success=False)
                for kind, path in paths.items():
                    self.assertEqual(digest(path), artifact_hashes[kind], "artifact mutated by consumer")
                if args.compiler_cache_policy is not None:
                    check_manifest(output)
                    self.assertEqual(digest(args.compiler_cache_policy),
                                     report["compiler_cache_policy_sha256"], "compiler cache policy changed")
            row["complete_case_wall_seconds"] = time.perf_counter() - case_start

        def test_counter_two_branches(self):
            self.exercise("counter")

        def test_let_return(self):
            self.exercise("let_return")

    result = unittest.TextTestRunner(verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(CircuitTests))
    if not args.uncached_zkir:
        report["stats_after"] = stats()
    if args.compiler_cache_policy is not None:
        report["acquisition_path_counts"] = {path: sum(row.get("acquisition_path") == path for row in report["cases"])
            for path in ("forge-hit", "forge-miss", "auto-bypass", "unavailable-keygen")}
        report["mode_scope"] = "compiler-local selects public preparation; per-case acquisition_path states whether Forge was used"
    report.update(passed=result.wasSuccessful(), tests_run=result.testsRun,
                  complete_wall_seconds=time.perf_counter() - start,
                  cumulative_child_largest_process_maxrss_bytes=(
                      resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
                      * (1 if sys.platform == "darwin" else 1024)),
                  memory_scope="cumulative child largest-process highwater, not concurrent total")
    args.report.write_text(json.dumps(report, indent=2) + "\n")
    return 0 if result.wasSuccessful() else 1


if __name__ == "__main__":
    raise SystemExit(main())
