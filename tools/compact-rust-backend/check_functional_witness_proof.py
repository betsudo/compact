#!/usr/bin/env python3
# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""One typed functional Field witness: real proofs on first/repeat/uncached laps."""
import argparse
import json
from pathlib import Path
import re
import resource
import shutil
import subprocess
import sys
import tempfile
import time
import tomllib

import forge_circuits as forge
from check_functional_proof import (ROOT, RUNNER, execution_environment,
                                   scheme_executable, snapshot, source_premises,
                                   workspace_packages)
from check_functional_effects import generated_manifest_snapshot

SOURCE = Path(__file__).parent / "tests/functional_effects/field_witness_proof.compact"
PROOF = Path(__file__).parent / "tests/functional_effects/field_witness_proof.rs"
PACKAGE = "compact-functional-field-proof-smoke"
MARKER = "functional Field witness proved, verified and ledger-applied"
CIRCUIT = "apply_offset"


def overlay(runner, generated):
    shutil.copytree(RUNNER, runner)
    main = runner / "src/main.rs"
    main.write_text(main.read_text() + "\nmod functional_field_witness;\n")
    shutil.copyfile(PROOF, runner / "src/functional_field_witness.rs")
    dispatch = runner / "src/direct_dispatch.rs"
    anchor = '    match (selector, asset) {'
    text = dispatch.read_text()
    forge.require(text.count(anchor) == 1, "proof selector insertion anchor changed")
    dispatch.write_text(text.replace(anchor, anchor + '\n        ("--functional-field-witness", _) => Some(super::functional_field_witness::arguments(arguments)),'))
    manifest = runner / "Cargo.toml"
    text = manifest.read_text()
    forge.require(tomllib.loads(text)["package"]["name"] == "compact-rust-proof-smoke", "canonical runner identity changed")
    text, count = re.subn(r'^name = "compact-rust-proof-smoke"$', f'name = "{PACKAGE}"', text, flags=re.MULTILINE)
    forge.require(count == 1, "private package rename not unique")
    def absolute(match):
        path = Path(json.loads(match[1]))
        return "path = " + json.dumps(str(path if path.is_absolute() else (RUNNER / path).resolve(strict=True)))
    text = re.sub(r'path = ("[^"\n]+")', absolute, text)
    generated_name = tomllib.loads((generated / "Cargo.toml").read_text())["package"]["name"]
    forge.require(generated_name == "compact-contract-field-witness-proof", "unexpected generated witness crate identity")
    forge.require(text.count("[dependencies]") == 1, "dependency table anchor changed")
    text = text.replace("[dependencies]", '[dependencies]\nfunctional-field-contract = { path = '
        + json.dumps(str(generated)) + ', package = ' + json.dumps(generated_name) + ', features = ["ledger-transaction"] }')
    profile = tomllib.loads((ROOT / "Cargo.toml").read_text())["profile"]["proof-checks"]
    forge.require(profile == {"inherits": "dev", "debug": 0, "debug-assertions": True,
        "overflow-checks": True, "package": {"*": {"opt-level": 3}}}, "proof profile policy changed")
    manifest.write_text(text + '\n[workspace]\n[profile.proof-checks]\ninherits = "dev"\ndebug = 0\n'
        'debug-assertions = true\noverflow-checks = true\n[profile.proof-checks.package."*"]\nopt-level = 3\n')
    shutil.copyfile(ROOT / "Cargo.lock", runner / "Cargo.lock")
    return generated_name


def main():
    if not __debug__:
        raise RuntimeError("qualification requires Python without -O/-OO")
    parser = argparse.ArgumentParser(description=__doc__)
    for flag in ("compiler", "params-dir", "target-dir", "forge", "uncached-zkir", "cache-dir", "report"):
        parser.add_argument("--" + flag, type=Path, required=True)
    args = parser.parse_args()
    for name in ("compiler", "params_dir", "forge", "uncached_zkir"):
        setattr(args, name, getattr(args, name).resolve(strict=True))
    args.cache_dir = args.cache_dir.resolve()
    forge.require(not args.cache_dir.exists(), "use a new absent private Forge cache")
    args.target_dir = args.target_dir.resolve()
    environment = execution_environment()
    args.scheme = scheme_executable(args.compiler, environment)
    environment.update(CARGO_TARGET_DIR=str(args.target_dir), MIDNIGHT_PP=str(args.params_dir),
        COMPACT_ZKIR_REAL=str(args.uncached_zkir), COMPACTC_SCHEME=str(args.scheme),
        COMPACT_ZKIR_FORGE_BIN=str(args.forge), MIDNIGHT_ZKIR_FORGE_DIR=str(args.cache_dir))
    forge.validate_consumer()
    def premises():
        values = source_premises(args)
        for path in (Path(__file__), PROOF, SOURCE, Path(__file__).with_name("check_functional_proof.py"),
                     Path(__file__).with_name("check_functional_effects.py")):
            values[str(path.resolve())] = forge.digest(path)
        return values
    initial = premises()
    runner_before = snapshot(RUNNER)
    canonical = args.target_dir / "proof-checks/compact-rust-proof-smoke"
    canonical_before = forge.digest(canonical) if canonical.exists() else None
    report = {"passed": False, "scope": "one full-source typed Field witness; functional preparation proved; ordinary observed comparison only",
        "compiler_sha256": forge.digest(args.compiler), "premises_before": initial, "laps": [],
        "cache_initially_absent": True, "profile": forge.PROFILE,
        "timing_limits": "fresh overlay build separate from proof case; no generalized speedup or peak-total-memory claim"}
    started = time.perf_counter()
    try:
        with tempfile.TemporaryDirectory(prefix="compact-functional-field-proof-") as temporary:
            temporary = Path(temporary)
            report["temporary_directory"] = str(temporary)
            for name, uncached in (("forge-first", False), ("forge-repeat", False), ("uncached", True)):
                tick = time.perf_counter()
                lap = {"name": name, "commands": []}
                report["laps"].append(lap)
                def run(argv, phase, success=True):
                    before = time.perf_counter()
                    result = subprocess.run(list(map(str, argv)), cwd=ROOT, env=environment, capture_output=True, text=True)
                    lap["commands"].append({"phase": phase, "argv": list(map(str, argv)),
                        "exit_code": result.returncode, "wall_seconds": time.perf_counter() - before,
                        "stdout": result.stdout, "stderr": result.stderr})
                    forge.require((result.returncode == 0) == success, "unexpected command outcome: " + phase)
                    return result.stdout
                def stats():
                    return json.loads(run([args.forge, "--cache-dir", args.cache_dir, "stats"], "stats"))
                directory = temporary / name
                directory.mkdir()
                generated = directory / "generated"
                run([args.compiler, "--target", "rust", "--rust-functional-ledger8", "--skip-zk",
                    "--rust-runtime-root", ROOT, SOURCE, generated], "generate")
                lap["generated_manifest"] = generated_manifest_snapshot(generated)
                before_sources = snapshot(generated)
                source = generated / f"zkir/{CIRCUIT}.zkir"
                lap["zkir_sha256"] = forge.digest(source)
                forge.require(all(old["zkir_sha256"] == lap["zkir_sha256"] for old in report["laps"][:-1]), "cross-lap current ZKIR differs")
                runner = directory / "tools/compact-rust-proof-smoke"
                runner.parent.mkdir()
                for relative, target in ((directory / "tests-rust-backend", ROOT / "tests-rust-backend"),
                    (directory / "runtime-rs", ROOT / "runtime-rs"),
                    (runner.parent / "compact-rust-backend", ROOT / "tools/compact-rust-backend")):
                    relative.symlink_to(target, target_is_directory=True)
                generated_name = overlay(runner, generated / "contract")
                metadata = json.loads(run(["cargo", "metadata", "--offline", "--format-version", "1", "--manifest-path", runner / "Cargo.toml"], "resolve"))
                root_lock = tomllib.loads((ROOT / "Cargo.lock").read_text())["package"]
                child_lock = tomllib.loads((runner / "Cargo.lock").read_text())["package"]
                identity = lambda row: (row["name"], row["version"], row.get("source"), row.get("checksum"))
                pinned = {identity(row) for row in root_lock}
                allowed = {str(path.resolve()) for path in workspace_packages().values()} | {str((runner / "Cargo.toml").resolve()), str((generated / "contract/Cargo.toml").resolve())}
                for row in child_lock:
                    forge.require(row.get("source") is None or identity(row) in pinned, "external lock identity changed")
                for row in metadata["packages"]:
                    forge.require(row.get("source") is not None or str(Path(row["manifest_path"]).resolve()) in allowed, "unexpected local package origin")
                manifest = runner / "Cargo.toml"
                owners = {row["name"] for row in child_lock if row.get("source") is None and row["name"] in workspace_packages()} | {generated_name}
                text = manifest.read_text()
                for owner in sorted(owners):
                    text += f'\n[profile.proof-checks.package.{json.dumps(owner)}]\nopt-level = 0\n'
                manifest.write_text(text)
                lap["owner_opt0_packages"] = sorted(owners)
                lap["private_manifest_sha256"] = forge.digest(manifest)
                lap["private_lock_sha256"] = forge.digest(runner / "Cargo.lock")
                run(["cargo", "build", "--locked", "--offline", "--profile", "proof-checks", "-vv", "--manifest-path", manifest], "build")
                binary = args.target_dir / "proof-checks" / PACKAGE
                binary_hash = forge.digest(binary)
                case_start = time.perf_counter()
                if not uncached:
                    lap["stats_before"] = stats()
                    lap["forge"] = forge.compile_circuit(source, generated / f"keys/{CIRCUIT}.prover",
                        generated / f"keys/{CIRCUIT}.verifier", args.uncached_zkir, environment)
                else:
                    (generated / "keys").mkdir(exist_ok=True)
                    run([args.uncached_zkir, "compile", source, generated / f"keys/{CIRCUIT}.prover",
                        generated / f"keys/{CIRCUIT}.verifier"], "uncached-keygen")
                if not uncached:
                    parameter = args.params_dir / f"bls_midnight_2p{lap['forge']['k']}"
                    parameter_hash = lap["forge"]["parameter_digest"]
                    forge.require(forge.digest(parameter) == parameter_hash, "current proving parameter differs from validated Forge profile")
                    if "parameter" not in report:
                        report["parameter"] = {"path": str(parameter), "sha256": parameter_hash}
                    forge.require(report["parameter"] == {"path": str(parameter), "sha256": parameter_hash}, "cross-lap proving parameter changed")
                forge.require(forge.digest(Path(report["parameter"]["path"])) == report["parameter"]["sha256"], "proving parameter changed")
                artifacts = [source, source.with_suffix(".bzkir"), generated / f"keys/{CIRCUIT}.prover", generated / f"keys/{CIRCUIT}.verifier"]
                lap["artifact_hashes"] = {str(p.relative_to(generated)): forge.digest(p) for p in artifacts}
                proof = run([binary, "--functional-field-witness", generated], "proof-and-assertions")
                forge.require(MARKER in proof, "functional Field proof marker absent")
                negative = directory / "corrupt-verifier"
                shutil.copytree(generated, negative)
                (negative / f"keys/{CIRCUIT}.verifier").write_bytes(b"invalid verifier")
                run([binary, "--functional-field-witness", negative], "corrupt-verifier", success=False)
                forge.require({str(p.relative_to(generated)): forge.digest(p) for p in artifacts} == lap["artifact_hashes"], "consumer changed artifacts")
                forge.require(all(forge.digest(generated / path) == digest for path, digest in before_sources.items()), "generated source changed")
                forge.require(forge.digest(binary) == binary_hash, "proof executable changed")
                forge.require(forge.digest(manifest) == lap["private_manifest_sha256"] and forge.digest(runner / "Cargo.lock") == lap["private_lock_sha256"], "private build identity changed")
                lap["proof_runner_sha256"] = binary_hash
                if not uncached:
                    lap["stats_after"] = stats()
                    before_totals = lap["stats_before"]["totals"]
                    after_totals = lap["stats_after"]["totals"]
                    expected_hit = name == "forge-repeat"
                    forge.require(lap["forge"]["cache"]["hit"] == expected_hit,
                                  "private first/repeat cache outcome changed")
                    forge.require(after_totals["hits"] - before_totals["hits"] == int(expected_hit)
                        and after_totals["misses"] - before_totals["misses"] == int(not expected_hit)
                        and after_totals["errors"] == before_totals["errors"],
                        "private cache accounting changed")
                lap["proof_case_wall_seconds"] = time.perf_counter() - case_start
                lap["complete_wall_seconds"] = time.perf_counter() - tick
            report["passed"] = True
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
    finally:
        try:
            report["premises_after"] = premises()
            report["premises_unchanged"] = report["premises_after"] == initial
        except Exception as error:
            report["premises_unchanged"] = False
            report["premises_error"] = str(error)
        report["parameter_unchanged"] = "parameter" in report and Path(report["parameter"]["path"]).is_file() and forge.digest(Path(report["parameter"]["path"])) == report["parameter"]["sha256"]
        report["canonical_runner_unchanged"] = snapshot(RUNNER) == runner_before
        report["canonical_binary_unchanged"] = (forge.digest(canonical) if canonical.exists() else None) == canonical_before
        report["temporary_cleanup"] = "temporary_directory" in report and not Path(report["temporary_directory"]).exists()
        report["passed"] = report["passed"] and all(report.get(k) for k in ("premises_unchanged", "parameter_unchanged", "canonical_runner_unchanged", "canonical_binary_unchanged", "temporary_cleanup"))
        report["complete_wall_seconds"] = time.perf_counter() - started
        report["largest_child_rss_bytes"] = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss * (1 if sys.platform == "darwin" else 1024)
        report["memory_scope"] = "largest individual child process high-water; not concurrent aggregate"
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(json.dumps(report, indent=2) + "\n")
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
