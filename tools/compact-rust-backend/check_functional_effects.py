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

"""Compile current opt-in programs and compare four execution routes locally.

This finite generated-consumer test does not generate keys or claim proof
coverage. Its retained output is available for a separate proof qualification.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time
import tomllib

ROOT = Path(__file__).resolve().parents[2]
FIXTURES = Path(__file__).with_name("tests") / "functional_effects"
SOURCES = {
    "let_fixture": ROOT / "examples/rust_backend/let_return_oracle.compact",
    "assert_fixture": ROOT / "examples/rust_backend/assert_witness.compact",
    "counter_fixture": FIXTURES / "counter.compact",
    "guarded_fixture": FIXTURES / "guarded_witness.compact",
    "collision_fixture": FIXTURES / "collisions.compact",
    "chunked_fixture": FIXTURES / "chunked.compact",
    "pure_fixture": FIXTURES / "pure_values.compact",
    "typed_fixture": FIXTURES / "typed_witnesses.compact",
}
EXPECTED_TESTS = 9
EXPECTED_SCENARIOS = 28  # Existing 25 + inactive, refused, successful typed-witness paths.


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()



def generated_manifest_snapshot(output):
    """Authenticate every declared asset and exact checked-program membership."""
    if not __debug__:
        raise RuntimeError("manifest validation requires Python without -O/-OO")
    from check_compactc_target import check_manifest
    check_manifest(output)
    manifest_path = output / "compiler/contract-manifest.json"
    manifest = json.loads(manifest_path.read_text())
    files = {}
    def visit(path, entries):
        for name, entry in entries.items():
            if name == "type":
                continue
            child = path / name
            if entry["type"] == "directory":
                visit(child, entry)
            else:
                files[str(child.relative_to(output))] = digest(child)
    for directory in ("compiler", "contract", "zkir"):
        visit(output / directory, manifest[directory])
    sidecars = output / "contract/checked-programs"
    actual = {str(path.relative_to(output)) for path in sidecars.rglob("*") if path.is_file()}
    declared = {name for name in files if name.startswith("contract/checked-programs/")}
    if actual != declared:
        raise ValueError("checked-program assets differ from manifest")
    return {"manifest_sha256": digest(manifest_path), "assets": files,
            "declared_asset_bytes": sum((output / name).stat().st_size for name in files),
            "checked_program_bytes": sum((output / name).stat().st_size for name in declared),
            "total_output_bytes": sum((output / name).stat().st_size for name in files)
                + (0 if "compiler/contract-manifest.json" in files else manifest_path.stat().st_size)}


def scheme_compiler(compiler, environment):
    """Resolve precisely the executable selected by compactc, relative to its cwd."""
    selected = environment.get("COMPACTC_SCHEME")
    if selected is None:
        sibling = compiler.with_name("compactc-scheme")
        selected = str(sibling) if sibling.exists() else "compactc-scheme"
    if os.sep in selected or (os.altsep and os.altsep in selected):
        candidate = Path(selected)
        if not candidate.is_absolute():
            candidate = ROOT / candidate
    else:
        search_path = os.pathsep.join(
            str(Path(entry) if Path(entry).is_absolute() else ROOT / entry)
            for entry in environment.get("PATH", os.defpath).split(os.pathsep))
        found = shutil.which(selected, path=search_path)
        if found is None:
            raise ValueError("selected Scheme compiler is not executable on PATH: " + repr(selected))
        candidate = Path(found)
        if not candidate.is_absolute():
            candidate = ROOT / candidate
    resolved = candidate.resolve(strict=True)
    if not resolved.is_file() or not os.access(resolved, os.X_OK):
        raise ValueError("selected Scheme compiler is not executable: " + str(resolved))
    return resolved


def main():
    if not __debug__:
        raise RuntimeError("qualification requires Python without -O/-OO")
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True, help="new absent retained output directory")
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    compiler = args.compiler.resolve(strict=True)
    output = args.output.resolve()
    if output.exists() or output.is_symlink():
        parser.error("output must be absent; refusing to reuse generated results")
    output.mkdir(parents=True)
    args.report.parent.mkdir(parents=True, exist_ok=True)
    started = time.monotonic()
    environment = dict(os.environ)
    environment.setdefault("RUSTUP_TOOLCHAIN", "1.99.0")
    scheme = scheme_compiler(compiler, environment)
    environment["COMPACTC_SCHEME"] = str(scheme)
    runtime_premises = [ROOT / "runtime-rs/Cargo.toml", ROOT / "runtime-rs/compatibility.json",
                        ROOT / "runtime-rs-macros/Cargo.toml"]
    runtime_premises += sorted((ROOT / "runtime-rs/src").rglob("*.rs"))
    runtime_premises += sorted((ROOT / "runtime-rs-macros/src").rglob("*.rs"))
    report = {
        "scope": "finite current generated functional/native/recorded comparison; no proofs",
        "passed": False, "compiler_sha256": digest(compiler),
        "scheme_compiler": str(scheme), "scheme_compiler_sha256": digest(scheme),
        "repository_lock_sha256": digest(ROOT / "Cargo.lock"),
        "sources": {str(path.relative_to(ROOT)): digest(path) for path in SOURCES.values()},
        "runtime_sources": {str(path.relative_to(ROOT)): digest(path) for path in runtime_premises},
        "consumer_sha256": digest(FIXTURES / "consumer.rs"),
        "phases": [], "generated": {},
        "expected_rust_tests": EXPECTED_TESTS, "expected_scenarios": EXPECTED_SCENARIOS,
        "limits": ["No failed-call state or gas returned by the existing consuming API",
                   "Both branch outcomes and exact observable witness prefixes checked",
                   "No key generation, proof, network or full-language coverage claim"],
    }

    def run(label, argv, *, cwd=ROOT):
        before = time.monotonic()
        result = subprocess.run(list(map(str, argv)), cwd=cwd, env=environment,
                                capture_output=True, text=True)
        log = output / (label + ".log")
        log.write_text(result.stdout + result.stderr)
        report["phases"].append({"name": label, "argv": list(map(str, argv)),
                                 "wall_seconds": time.monotonic() - before,
                                 "exit_code": result.returncode, "log": str(log)})
        print(label, result.returncode, flush=True)
        if result.returncode:
            print((result.stdout + result.stderr)[-8000:], file=sys.stderr)
            result.check_returncode()
        return result.stdout

    try:
        sys.path.insert(0, str(Path(__file__).parent))
        from check_compactc_target import check_manifest
        dependencies = {}
        local_manifests = {}
        local_versions = {}
        for alias, source in SOURCES.items():
            destination = output / alias
            run("compile-" + alias, [compiler, "--target", "rust", "--skip-zk",
                "--rust-functional-ledger8", "--rust-require-recording",
                "--rust-runtime-root", ROOT, source, destination])
            check_manifest(destination)
            manifest = tomllib.loads((destination / "contract/Cargo.toml").read_text())
            package = manifest["package"]["name"]
            local_manifests[package] = (destination / "contract/Cargo.toml").resolve()
            local_versions[package] = manifest["package"]["version"]
            dependencies[alias] = {"package": package, "path": str(destination / "contract")}
            library = destination / "contract/lib.rs"
            if "CHECKED_PROGRAM" not in library.read_text():
                raise ValueError("opt-in module missing: " + alias)
            report["generated"][alias] = {"library_sha256": digest(library),
                "manifest_sha256": digest(destination / "compiler/contract-manifest.json"),
                "ir_sha256": digest(destination / "contract/compact-rust-ir.json"),
                "functional_bytes": library.stat().st_size,
                "functional_lines": len(library.read_text().splitlines()),
                "authenticated_assets": generated_manifest_snapshot(destination)}
            baseline = output / (alias + "-default")
            run("compile-default-" + alias, [compiler, "--target", "rust", "--skip-zk",
                "--rust-require-recording", "--rust-runtime-root", ROOT, source, baseline])
            check_manifest(baseline)
            original = baseline / "contract/lib.rs"
            report["generated"][alias].update(default_library_sha256=digest(original),
                default_bytes=original.stat().st_size, default_lines=len(original.read_text().splitlines()),
                default_authenticated_assets=generated_manifest_snapshot(baseline),
                size_scope="opt-in adds request module and retains legacy APIs; not total LOC reduction")
        chunked = json.loads((output / "chunked_fixture/contract/compact-rust-ir.json").read_text())
        if not any(len(field.get("path", [])) > 1 for field in chunked["ledger_fields"]):
            raise ValueError("chunked source did not produce a multi-component physical path")
        consumer = output / "consumer"
        (consumer / "tests").mkdir(parents=True)
        shutil.copyfile(FIXTURES / "consumer.rs", consumer / "tests/integration.rs")
        root_lock = tomllib.loads((ROOT / "Cargo.lock").read_text())
        serde_version = next(row["version"] for row in root_lock["package"] if row["name"] == "serde_json")
        text = ['[workspace]', '[package]', 'name = "functional-effect-consumer"',
                'version = "0.0.0"', 'edition = "2024"', '[dependencies]',
                'runtime = { package = "midnight-compact-runtime", path = ' + json.dumps(str(ROOT / "runtime-rs"))
                + ', features = ["experimental-functional"] }',
                'serde_json = "=' + serde_version + '"']
        for alias, dependency in dependencies.items():
            text.append(alias + ' = { package = ' + json.dumps(dependency["package"])
                        + ', path = ' + json.dumps(dependency["path"]) + ' }')
        (consumer / "Cargo.toml").write_text("\n".join(text) + "\n")
        shutil.copyfile(ROOT / "Cargo.lock", consumer / "Cargo.lock")
        local_manifests["functional-effect-consumer"] = (consumer / "Cargo.toml").resolve()
        local_versions["functional-effect-consumer"] = "0.0.0"
        for folder in ("runtime-rs", "runtime-rs-macros"):
            path = ROOT / folder / "Cargo.toml"
            package = tomllib.loads(path.read_text())["package"]
            local_manifests[package["name"]] = path.resolve()
            local_versions[package["name"]] = package["version"]
        metadata = json.loads(run("resolve-consumer", ["cargo", "metadata", "--offline",
            "--format-version", "1", "--manifest-path", consumer / "Cargo.toml"]))
        permitted = {(row["name"], row["version"], row.get("source")) for row in root_lock["package"]}
        for package in metadata["packages"]:
            identity = (package["name"], package["version"], package.get("source"))
            if package["name"] in local_manifests:
                if package.get("source") is not None or Path(package["manifest_path"]).resolve() != local_manifests[package["name"]]:
                    raise ValueError("unexpected generated/local dependency origin: " + repr(identity))
            elif package.get("source") is None or identity not in permitted:
                raise ValueError("consumer dependency differs from repository lock: " + repr(identity))
        locked_identities = {(row["name"], row["version"], row.get("source"), row.get("checksum"))
                             for row in root_lock["package"]}
        child_lock = tomllib.loads((consumer / "Cargo.lock").read_text())
        for package in child_lock["package"]:
            identity = (package["name"], package["version"], package.get("source"), package.get("checksum"))
            if package["name"] in local_manifests:
                if package.get("source") is not None or package["version"] != local_versions[package["name"]]:
                    raise ValueError("unexpected generated/local lock origin: " + repr(identity))
            elif identity not in locked_identities:
                raise ValueError("consumer lock identity or checksum differs: " + repr(identity))
        report["consumer_lock_sha256"] = digest(consumer / "Cargo.lock")
        tests = run("consumer-tests", ["cargo", "test", "--locked", "--offline",
            "--manifest-path", consumer / "Cargo.toml", "--test", "integration", "--",
            "--nocapture", "--test-threads=1"])
        counts = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored", tests)
        if counts != [(str(EXPECTED_TESTS), "0", "0")]:
            raise ValueError("unexpected generated-consumer test accounting: " + repr(counts))
        scenarios = tests.count("FUNCTIONAL_SCENARIO_CHECKED")
        if scenarios != EXPECTED_SCENARIOS:
            raise ValueError("unexpected freshly checked scenario count: " + str(scenarios))
        report.update(rust_tests=EXPECTED_TESTS, scenarios=scenarios, execution_routes_per_scenario=4)
        samples = [json.loads(line.split("FUNCTIONAL_RUNTIME_SAMPLE ", 1)[1])
                   for line in tests.splitlines() if "FUNCTIONAL_RUNTIME_SAMPLE " in line]
        routes = ["default_native", "default_recorded", "functional_native", "functional_recorded"]
        if len(samples) != 16 or {(row["route"], row["lap"]) for row in samples} != {
                (route, lap) for route in routes for lap in range(4)}:
            raise ValueError("unexpected fresh runtime sample accounting")
        for row in samples:
            order = routes if row["lap"] % 2 == 0 else list(reversed(routes))
            if (type(row["iterations"]) is not int or row["iterations"] != 256
                    or type(row["elapsed_ns"]) is not int or row["elapsed_ns"] < 0
                    or type(row["position"]) is not int or row["position"] not in range(4)
                    or row["route"] != order[row["position"]]
                    or row["allocation_measurement"] != "not_performed"):
                raise ValueError("invalid runtime cost sample")
        report.update(runtime_samples=samples, measured_executions=sum(row["iterations"] for row in samples),
            timing_limits="tiny debug-profile local samples; no speed threshold or generalized performance claim; no allocation measurement")
        clippy = ["cargo", "clippy", "--locked", "--offline", "--manifest-path", consumer / "Cargo.toml",
                  "--all-targets", "--all-features"]
        for package in ["functional-effect-consumer", *(item["package"] for item in dependencies.values())]:
            clippy += ["--package", package]
        run("generated-clippy", [*clippy, "--", "-D", "warnings"])
        for alias in SOURCES:
            check_manifest(output / alias)
            check_manifest(output / (alias + "-default"))
            if generated_manifest_snapshot(output / alias) != report["generated"][alias]["authenticated_assets"]:
                raise ValueError("generated manifest/assets changed during execution: " + alias)
            if generated_manifest_snapshot(output / (alias + "-default")) != report["generated"][alias]["default_authenticated_assets"]:
                raise ValueError("default manifest/assets changed during execution: " + alias)
            if digest(output / alias / "contract/lib.rs") != report["generated"][alias]["library_sha256"]:
                raise ValueError("generated library changed during execution: " + alias)
            if digest(output / (alias + "-default") / "contract/lib.rs") != report["generated"][alias]["default_library_sha256"]:
                raise ValueError("default generated library changed during execution: " + alias)
        for source, expected in report["sources"].items():
            if digest(ROOT / source) != expected:
                raise ValueError("source changed during qualification: " + source)
        for source, expected in report["runtime_sources"].items():
            if digest(ROOT / source) != expected:
                raise ValueError("runtime changed during qualification: " + source)
        if scheme_compiler(compiler, environment) != scheme:
            raise ValueError("Scheme compiler selection changed during qualification")
        for path, expected in [(compiler, report["compiler_sha256"]),
                               (scheme, report["scheme_compiler_sha256"]),
                               (ROOT / "Cargo.lock", report["repository_lock_sha256"]),
                               (consumer / "Cargo.lock", report["consumer_lock_sha256"]),
                               (FIXTURES / "consumer.rs", report["consumer_sha256"]),
                               (consumer / "tests/integration.rs", report["consumer_sha256"])]:
            if digest(path) != expected:
                raise ValueError("qualification premise changed: " + str(path))
        report["passed"] = True
    except Exception as error:
        report["error"] = str(error)
        print(error, file=sys.stderr)
    finally:
        report["complete_wall_seconds"] = time.monotonic() - started
        args.report.write_text(json.dumps(report, indent=2) + "\n")
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
