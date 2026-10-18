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

"""Prove one fresh functional Let program using the unchanged smoke harness.

Private overlays replace the direct recording and proof-input calls only. Counter remains the
ordinary control; all existing proof, application, comparison and negative checks
execute on each of two Forge laps and a separate uncached qualification lap.
"""
import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
import time
import tomllib

import forge_circuits

ROOT = Path(__file__).resolve().parents[2]
RUNNER = ROOT / "tools/compact-rust-proof-smoke"
MARKER = "functional Let replace proved through the shared checked program"
OLD_CALL = "contract::recorded::replace(initial.into_circuit_context(deploy.address()), next)?"
NEW_CALL = ("contract::functional::replace::Program::new(next)"
            ".execute_recorded(initial.into_circuit_context(deploy.address()))?")
OLD_PROOF = "check_transaction(root, CIRCUIT, deploy, prepared, &mut rng, |state| {"
NEW_PROOF = "check_transaction(root, CIRCUIT, deploy, manual, &mut rng, |state| {"
LET_ALIAS = "compact-rust-let-return-oracle-fixture"
OVERLAY_PACKAGE = "compact-functional-proof-smoke"


def snapshot(directory):
    return {str(p.relative_to(directory)): forge_circuits.digest(p)
            for p in sorted(directory.rglob("*")) if p.is_file()}


def execution_environment():
    environment = dict(os.environ)
    # Every child runs from ROOT, including command lookup for relative/empty
    # PATH entries. Do not resolve them against the Python caller's directory.
    environment["PATH"] = os.pathsep.join(str((ROOT / part).resolve()) if not Path(part).is_absolute() else part
        for part in environment.get("PATH", os.defpath).split(os.pathsep))
    return environment


def scheme_executable(compiler, environment):
    explicit = environment.get("COMPACTC_SCHEME")
    if explicit is not None:
        forge_circuits.require(bool(explicit), "empty explicit Scheme compiler refused")
        candidate = shutil.which(explicit, path=environment["PATH"]) if "/" not in explicit else str(ROOT / explicit)
    else:
        sibling = compiler.with_name("compactc-scheme")
        candidate = str(sibling) if sibling.is_file() else shutil.which("compactc-scheme", path=environment["PATH"])
    forge_circuits.require(candidate is not None, "actual Scheme compiler required")
    return Path(candidate).resolve(strict=True)


def workspace_packages():
    manifests = {}
    root = tomllib.loads((ROOT / "Cargo.toml").read_text())
    for member in root["workspace"]["members"]:
        for directory in ROOT.glob(member):
            manifest = directory / "Cargo.toml"
            manifests[tomllib.loads(manifest.read_text())["package"]["name"]] = manifest
    return manifests


def owner_profiles(runner, packages):
    """Preserve original workspace owner opt0 in the private single-member root."""
    owners = set(workspace_packages())
    names = {row["name"] for row in packages if "source" not in row and row["name"] in owners}
    names.add("compact-contract-let-return-oracle")
    manifest = runner / "Cargo.toml"
    source = manifest.read_text()
    for name in sorted(names):
        source += f'\n[profile.proof-checks.package.{json.dumps(name)}]\nopt-level = 0\n'
    manifest.write_text(source)
    return {"root_workspace_and_generated_packages_opt_level_0": sorted(names),
            "private_workspace_runner_opt_level": 0, "other_dependencies_opt_level": 3,
            "debug_assertions": True, "overflow_checks": True,
            "manifest_sha256": forge_circuits.digest(manifest)}


def source_premises(args):
    paths = {ROOT / "Cargo.lock", ROOT / "Cargo.toml", args.scheme, ROOT / "runtime-rs/Cargo.toml", ROOT / "runtime-rs/compatibility.json",
             ROOT / "runtime-rs-macros/Cargo.toml", Path(__file__),
             ROOT / "tools/compact-rust-backend/check_forge_circuit_cache.py",
             ROOT / "tools/compact-rust-backend/check_compactc_target.py",
             ROOT / "tools/compact-rust-backend/forge_circuits.py",
             ROOT / "tools/compact-rust-backend/forge_circuit_qualification.json",
             args.compiler, args.forge, args.uncached_zkir}
    qualifications = json.loads((ROOT / "tools/compact-rust-backend/forge_circuit_qualification.json").read_text())
    paths.update(ROOT / qualifications[name]["compact_source"] for name in ("counter", "let_return"))
    paths.update(workspace_packages().values())
    paths.update((ROOT / "runtime-rs/src").rglob("*.rs"))
    paths.update((ROOT / "runtime-rs-macros/src").rglob("*.rs"))
    for name in ("compact-rust-backend", "compact-rust-core-repl"):
        sibling = args.compiler.with_name(name)
        if sibling.exists():
            paths.add(sibling)
    # The original runner compiles external modules/fixtures through literal
    # include/path references. Preserve their bytes as well as the runner itself.
    pending = list(RUNNER.rglob("*.rs"))
    visited = set()
    references = re.compile(r'(?:include_(?:str|bytes)!\(\s*|#\[path\s*=\s*)"([^"\n]+)"')
    while pending:
        source = pending.pop().resolve(strict=True)
        if source in visited:
            continue
        visited.add(source)
        for match in references.finditer(source.read_text()):
            included = (source.parent / match[1]).resolve(strict=True)
            paths.add(included)
            if included.suffix == ".rs":
                pending.append(included)
    return {str(path): forge_circuits.digest(path) for path in sorted(paths)}


def overlay(destination, generated):
    """Copy source privately and authenticate each deliberately changed slot."""
    shutil.copytree(RUNNER, destination)
    source = destination / "src/let_return.rs"
    original = source.read_text()
    forge_circuits.require(original.count(OLD_CALL) == 1, "direct Let call patch does not match exactly once")
    marker = '    println!("root-Let replace proved and applied through ledger-8");'
    forge_circuits.require(original.count(marker) == 1, "original success marker changed")
    forge_circuits.require(original.count(OLD_PROOF) == 1, "default proof-input patch does not match exactly once")
    source.write_text(original.replace(OLD_CALL, NEW_CALL).replace(OLD_PROOF, NEW_PROOF)
                      .replace(marker, marker + f'\n    println!("{MARKER}");'))
    manifest = destination / "Cargo.toml"
    original_manifest = manifest.read_text()
    parsed = tomllib.loads(original_manifest)
    forge_circuits.require(parsed["dependencies"][LET_ALIAS]["path"] == "../../tests-rust-backend/let-return-oracle",
                           "original Let dependency identity changed")
    generated_manifest = tomllib.loads((generated / "Cargo.toml").read_text())
    forge_circuits.require(generated_manifest["package"]["name"] == "compact-contract-let-return-oracle",
                           "unexpected generated Let package")
    replacement = (f'{LET_ALIAS} = {{ path = {json.dumps(str(generated))}, '
                   'package = "compact-contract-let-return-oracle", features = ["ledger-transaction"] }')
    pattern = re.compile(r"^" + re.escape(LET_ALIAS) + r" = \{[^\n]+\}$", re.MULTILINE)
    forge_circuits.require(parsed["package"]["name"] == "compact-rust-proof-smoke", "canonical runner package changed")
    rewritten, count = pattern.subn(lambda _: replacement, original_manifest)
    forge_circuits.require(count == 1, "Let alias patch does not match exactly once")
    rewritten, package_count = re.subn(r'^name = "compact-rust-proof-smoke"$',
        f'name = "{OVERLAY_PACKAGE}"', rewritten, flags=re.MULTILINE)
    forge_circuits.require(package_count == 1, "overlay package rename did not match exactly once")
    # Remaining dependency paths retain their exact original absolute targets.
    def absolute(match):
        target = Path(json.loads(match[1]))
        return "path = " + json.dumps(str(target if target.is_absolute() else (RUNNER / target).resolve(strict=True)))
    rewritten = re.sub(r'path = ("[^"\n]+")', absolute, rewritten)
    expected_profile = {"inherits": "dev", "debug": 0, "debug-assertions": True,
                        "overflow-checks": True, "package": {"*": {"opt-level": 3}}}
    forge_circuits.require(tomllib.loads((ROOT / "Cargo.toml").read_text())["profile"]["proof-checks"] == expected_profile,
                           "checked proof profile changed; review overlay profile")
    manifest.write_text(rewritten + '\n[workspace]\n\n[profile.proof-checks]\ninherits = "dev"\ndebug = 0\n'
        'debug-assertions = true\noverflow-checks = true\n\n[profile.proof-checks.package."*"]\nopt-level = 3\n')
    shutil.copyfile(ROOT / "Cargo.lock", destination / "Cargo.lock")
    return {"original_source_sha256": forge_circuits.digest(RUNNER / "src/let_return.rs"),
            "overlay_source_sha256": forge_circuits.digest(source),
            "overlay_manifest_sha256": forge_circuits.digest(manifest),
            "patch": {"from": OLD_CALL, "to": NEW_CALL, "proof_from": OLD_PROOF, "proof_to": NEW_PROOF,
                      "success_marker": MARKER},
            "generated_crate_sha256": forge_circuits.digest(generated / "lib.rs"),
            "private_package": OVERLAY_PACKAGE}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler", type=Path, required=True)
    parser.add_argument("--params-dir", type=Path, required=True)
    parser.add_argument("--target-dir", type=Path, required=True)
    parser.add_argument("--forge", type=Path, required=True, help="explicit qualified Forge executable")
    parser.add_argument("--uncached-zkir", type=Path, required=True)
    parser.add_argument("--cache-dir", type=Path)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args(argv)
    for field in ("compiler", "params_dir", "forge", "uncached_zkir"):
        setattr(args, field, getattr(args, field).resolve(strict=True))
    environment = execution_environment()
    args.scheme = scheme_executable(args.compiler, environment)
    args.target_dir = args.target_dir.resolve()
    args.report = args.report.resolve()
    args.report.parent.mkdir(parents=True, exist_ok=True)
    environment.update(CARGO_TARGET_DIR=str(args.target_dir), MIDNIGHT_PP=str(args.params_dir),
                       COMPACT_ZKIR_REAL=str(args.uncached_zkir), COMPACTC_SCHEME=str(args.scheme))
    canonical_binary = args.target_dir / "proof-checks/compact-rust-proof-smoke"
    canonical_binary_before = forge_circuits.digest(canonical_binary) if canonical_binary.exists() else None
    premises = source_premises(args)
    before = snapshot(RUNNER)
    canonical_fixture = snapshot(ROOT / "tests-rust-backend/let-return-oracle")
    report = {"scope": "Let recording and proved manual preparation execute functional Program; Counter and typed observed/prepared comparison remain default",
              "passed": False, "laps": [], "compiler_sha256": forge_circuits.digest(args.compiler),
              "canonical_runner_before": before, "canonical_fixture_before": canonical_fixture,
              "source_premises_before": premises, "cache_measurement_scope": "first/repeat; cold/warm only when observed cache statistics establish it",
              "cached_artifacts_only": "keys/binary IR; never proof, assertion or test result",
              "timing_limits": "fresh private runner paths may cause recompilation/binary identity changes; build/generation and existing circuit-test wall times are separate; Forge ensure timing is not complete test acceleration",
              "started_wall": time.time()}
    start = time.perf_counter()
    try:
        with tempfile.TemporaryDirectory(prefix="compact-functional-proof-") as private:
            private = Path(private)
            report["temporary_directory"] = str(private)
            for name, uncached in (("forge-first", False), ("forge-repeat", False), ("uncached", True)):
                lap_started = time.perf_counter()
                lap = {"name": name, "commands": []}
                report["laps"].append(lap)
                def run(command):
                    tick = time.perf_counter()
                    completed = subprocess.run([str(v) for v in command], cwd=ROOT, env=environment,
                                               capture_output=True, text=True)
                    lap["commands"].append({"argv": [str(v) for v in command], "wall_seconds": time.perf_counter() - tick,
                                             "exit_code": completed.returncode, "stdout": completed.stdout, "stderr": completed.stderr})
                    completed.check_returncode()
                    return completed
                directory = private / name
                directory.mkdir()
                generated = directory / "generated"
                run([args.compiler, "--target", "rust", "--rust-functional-ledger8", "--skip-zk",
                     "--rust-runtime-root", ROOT, ROOT / "examples/rust_backend/let_return_oracle.compact", generated])
                lap["functional_generated_before"] = snapshot(generated)
                generated_source = generated / "zkir/replace.zkir"
                functional_hash = forge_circuits.digest(generated_source)
                lap["functional_current_zkir_sha256"] = functional_hash
                previous_hashes = [row["functional_current_zkir_sha256"] for row in report["laps"][:-1]]
                forge_circuits.require(all(value == functional_hash for value in previous_hashes),
                                       "functional Let ZKIR changed across laps; timing comparison refused")
                # Preserve original relative include/path layout without changing
                # source assertions. These source-only links are never mutated.
                runner = directory / "tools/compact-rust-proof-smoke"
                runner.parent.mkdir()
                (directory / "tests-rust-backend").symlink_to(ROOT / "tests-rust-backend", target_is_directory=True)
                (directory / "runtime-rs").symlink_to(ROOT / "runtime-rs", target_is_directory=True)
                (runner.parent / "compact-rust-backend").symlink_to(ROOT / "tools/compact-rust-backend", target_is_directory=True)
                lap["overlay"] = overlay(runner, generated / "contract")
                initial_lock = forge_circuits.digest(runner / "Cargo.lock")
                run(["cargo", "metadata", "--offline", "--format-version", "1", "--manifest-path", runner / "Cargo.toml"])
                resolved_lock = runner / "Cargo.lock"
                root_packages = tomllib.loads((ROOT / "Cargo.lock").read_text())["package"]
                resolved_packages = tomllib.loads(resolved_lock.read_text())["package"]
                external = lambda packages: {(v["name"], v["version"], v["source"], v.get("checksum")) for v in packages if "source" in v}
                forge_circuits.require(external(resolved_packages) <= external(root_packages),
                                       "private lock resolution introduced unpinned external dependency")
                local = lambda packages: {(v["name"], v["version"]) for v in packages if "source" not in v}
                added_local = local(resolved_packages) - local(root_packages)
                forge_circuits.require({name for name, _ in added_local} <= {OVERLAY_PACKAGE, "compact-contract-let-return-oracle"},
                                       "unexpected private local package addition")
                lap["lock"] = {"seed_sha256": initial_lock, "added_local_packages": sorted(added_local), "resolved_sha256": forge_circuits.digest(resolved_lock),
                               "resolution": "explicit initial offline lock resolution; external names/versions/sources/checksums retained from root lock"}
                lap["effective_profile"] = owner_profiles(runner, resolved_packages)
                lap["overlay"]["initial_manifest_sha256"] = lap["overlay"]["overlay_manifest_sha256"]
                lap["overlay"]["overlay_manifest_sha256"] = lap["effective_profile"]["manifest_sha256"]
                run(["cargo", "build", "--locked", "--offline", "--profile", "proof-checks", "-vv", "--manifest-path", runner / "Cargo.toml"])
                lap["build_wall_seconds"] = lap["commands"][-1]["wall_seconds"]
                binary = args.target_dir / "proof-checks" / OVERLAY_PACKAGE
                lap["proof_runner_sha256"] = forge_circuits.digest(binary)
                harness_report = directory / "proof-report.json"
                command = [sys.executable, ROOT / "tools/compact-rust-backend/check_forge_circuit_cache.py",
                           "--compiler", args.compiler, "--proof-runner", binary, "--params-dir", args.params_dir,
                           "--forge", args.forge, "--report", harness_report]
                if args.cache_dir is not None:
                    command += ["--cache-dir", args.cache_dir.resolve()]
                if uncached:
                    command += ["--uncached-zkir", args.uncached_zkir]
                run(command)
                forge_circuits.require(forge_circuits.digest(binary) == lap["proof_runner_sha256"],
                                       "overlay proof runner changed during proof execution")
                lap["proof_runner_after_sha256"] = forge_circuits.digest(binary)
                outcome = json.loads(harness_report.read_text())
                lap["existing_harness"] = outcome
                lap["existing_harness_wall_seconds"] = outcome["complete_wall_seconds"]
                forge_circuits.require(outcome["passed"] is True and outcome["tests_run"] == 2,
                                       "original two-case proof harness failed or accounting changed")
                cases = outcome["cases"]
                forge_circuits.require([row["circuit"] for row in cases] == ["counter", "let_return"],
                                       "original proof case identities changed")
                let_case = cases[1]
                forge_circuits.require(let_case["artifact_hashes"]["source"] == functional_hash,
                                       "functional program ZKIR differs from fresh proof input")
                forge_circuits.require(MARKER in let_case["proof_and_assertions"]["stdout"],
                                       "functional proof execution marker absent")
                forge_circuits.require(forge_circuits.digest(generated_source) == functional_hash,
                                       "functional input changed during proof execution")
                lap["functional_generated_after"] = snapshot(generated)
                forge_circuits.require(lap["functional_generated_after"] == lap["functional_generated_before"],
                                       "functional generated source/manifest/IR changed during build or proof")
                lap["complete_wall_seconds"] = time.perf_counter() - lap_started
        report["temporary_cleanup"] = not private.exists()
        report["passed"] = True
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
    finally:
        report["temporary_cleanup"] = "temporary_directory" in report and not Path(report["temporary_directory"]).exists()
        try:
            after_premises = source_premises(args)
        except Exception as error:
            after_premises = None
            report["source_premises_error"] = f"{type(error).__name__}: {error}"
        report["source_premises_after"] = after_premises
        report["source_premises_unchanged"] = after_premises == premises
        report["canonical_binary_before_sha256"] = canonical_binary_before
        report["canonical_binary_after_sha256"] = forge_circuits.digest(canonical_binary) if canonical_binary.exists() else None
        report["canonical_binary_unchanged"] = report["canonical_binary_after_sha256"] == canonical_binary_before
        report["canonical_runner_unchanged"] = snapshot(RUNNER) == before
        report["canonical_fixture_unchanged"] = snapshot(ROOT / "tests-rust-backend/let-return-oracle") == canonical_fixture
        report["passed"] = report["passed"] and report["canonical_runner_unchanged"] and report["canonical_fixture_unchanged"] and report["temporary_cleanup"] and report["canonical_binary_unchanged"] and report["source_premises_unchanged"]
        report["complete_wall_seconds"] = time.perf_counter() - start
        args.report.write_text(json.dumps(report, indent=2) + "\n")
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    if "--core-five" in sys.argv:
        sys.argv.remove("--core-five")
        from check_functional_core_case import lifecycle
        raise SystemExit(lifecycle.main())
    raise SystemExit(main())
