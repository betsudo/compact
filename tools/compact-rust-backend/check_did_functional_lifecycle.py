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

"""Fresh original-DID generation, selected live-TS behavior and API refusals.

This gate does not prove or apply a transaction. All twelve generated programs
are retained, while Rust execution covers the named lifecycle cases only.
"""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import time
import tomllib

from check_functional_effects import generated_manifest_snapshot, scheme_compiler
from did_proof_gate import hashes, require, unchanged
import local_parity_gate as common

ROOT = common.ROOT
SOURCE_ROOT = ROOT / "examples/rust_backend/third_party/did_functional_lifecycle"
SOURCE = SOURCE_ROOT / "packages/contract/src/did.compact"
FIXTURE = ROOT / "tests-rust-backend/did-functional-lifecycle-source"
SOURCE_MANIFEST_SHA256 = "a5386c2d3cc8d2f3aa090599a6038efc638f36f20ede19a7067a1ba416610106"
PACKAGE = "compact-rust-did-functional-lifecycle-source-fixture"
CONTROLS = Path(__file__).with_name("tests") / "did_functional_lifecycle"
NEGATIVES = {"wrong_ordinary_version": "E0308", "wrong_signature": "E0308",
             "consumed_program": "E0382"}
TESTS = {
    "did_deactivate_recorded_lab_matches_original_typescript_and_native",
    "did_late_witness_failure_keeps_owned_lab_checkpoint_and_external_journal",
    "did_deactivate_functional_matches_retained_typescript_and_native",
    "did_functional_late_witness_failure_keeps_lab_checkpoint_and_journal",
    "did_all_captured_failure_controls_match_across_four_routes",
}


def original_source_closure():
    """Check the complete immutable upstream closure shared by both DID gates."""
    manifest_path = SOURCE_ROOT / "source-manifest.json"
    original = ROOT / "examples/rust_backend/did_adoption"
    require(common.sha256(manifest_path) == SOURCE_MANIFEST_SHA256,
            "original pinned DID source manifest differs")
    require(manifest_path.read_bytes() == (original / "source-manifest.json").read_bytes(),
            "original source authority changed")
    manifest = json.loads(manifest_path.read_text())
    require(len(manifest["files"]) == 2, "complete pinned DID closure required")
    paths = [manifest_path, original / "source-manifest.json"]
    for row in manifest["files"]:
        path = SOURCE_ROOT / row["path"]
        authority = original / row["path"]
        require(row["modification"] == "none" and common.sha256(path) == row["sha256"]
                and path.read_bytes() == authority.read_bytes(),
                "DID source differs from its original closure")
        paths += [path, authority]
    return paths


def checked_oracle_captures():
    """Bind the proof's seeded row to the complete live-TS-qualified DID oracle."""
    fixture = FIXTURE / "data/unit-composition.json"
    shared = ROOT / "runtime-rs/tests/fixtures/unit-composition.json"
    expected, capture = json.loads(fixture.read_text()), json.loads(shared.read_text())
    require(expected["runtime_version"] == json.loads((ROOT / "runtime/package.json").read_text())["version"]
            and expected["cases"] == [row for row in capture["cases"] if row["source"] == "did"],
            "proof and live-TS DID captures differ")
    return [fixture, shared]


def require_clean_source(receipt):
    head = subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    require(receipt.get("source_head", head) == head, "source HEAD changed during qualification")
    receipt["source_head"] = head
    receipt["source_dirty"] = bool(subprocess.check_output(
        ["git", "status", "--porcelain"], cwd=ROOT, text=True))
    require(not receipt["source_dirty"], "clean source checkout required")


def run(args):
    output = args.output.absolute()
    require(not output.exists() and not output.is_symlink(), "output must be absent")
    output.mkdir(parents=True)
    env = dict(os.environ, RUSTUP_TOOLCHAIN="1.99.0", CARGO_INCREMENTAL="0",
               PYTHONDONTWRITEBYTECODE="1")
    env["CARGO_TARGET_DIR"] = str(args.target_dir.absolute())
    receipt = {"format": "compact-did-functional-lifecycle/v1", "status": "incomplete",
               "commands": [], "scope": "selected deactivation and late-witness-failure lifecycle across four APIs against live TS",
               "proof_and_application": "NOT RUN", "constructor_execution_proved": False}
    start = time.monotonic()
    try:
        compiler = args.compiler.resolve(strict=True)
        scheme = scheme_compiler(compiler, env)
        env["COMPACTC_SCHEME"] = str(scheme)
        runtime = args.ts_runtime.resolve(strict=True)
        version = json.loads((ROOT / "runtime/package.json").read_text())["version"]
        require_clean_source(receipt)
        closure = original_source_closure() + checked_oracle_captures()
        paths = [ROOT / "Cargo.toml", ROOT / "Cargo.lock", Path(__file__),
                 ROOT / "runtime/package.json", ROOT / "runtime-rs/compatibility.json",
                 ROOT / "examples/rust_backend/unit_composition_crypto.compact",
                 ROOT / "examples/rust_backend/did_adoption/source-manifest.json"]
        for directory in [SOURCE_ROOT, FIXTURE, CONTROLS, ROOT / "runtime-rs",
                          ROOT / "runtime-rs-macros", ROOT / "testkit-rs",
                          ROOT / "tools/compact-rust-backend/src", ROOT / "compiler"]:
            paths += [p for p in directory.rglob("*") if p.is_file() and
                      p.suffix in {".compact", ".rs", ".toml", ".json", ".mjs", ".ss"}]
        paths += [Path(__file__).with_name(name) for name in [
            "local_parity_gate.py", "did_proof_gate.py", "check_functional_effects.py",
            "check_compactc_target.py"]]
        receipt["source_hashes"] = hashes(paths + closure)
        receipt["compiler_hashes"] = hashes([compiler, scheme])
        rust_tools = [Path(subprocess.check_output(
            ["rustup", "which", "--toolchain", "1.99.0", name], env=env, text=True).strip())
            for name in ("cargo", "rustc", "rustfmt")]
        receipt["rust_tool_hashes"] = hashes(rust_tools)

        def execute(argv, name, stdout=None):
            common.run(list(map(str, argv)), name, output, receipt, env=env, stdout_path=stdout)

        generated = output / "generated"
        execute([compiler, "--target", "rust", "--target", "ts", "--rust-functional-ledger8",
                 "--rust-runtime-root", ROOT, "--skip-zk", SOURCE.relative_to(ROOT), generated], "generate-did")
        receipt["generated"] = generated_manifest_snapshot(generated)
        for name, retained in [("compact-rust-ir.json", "original-ir.json"),
                               ("compact-rust-origins.raw.json", "compact-rust-origins.raw.json"),
                               ("rust-capabilities.json", "rust-capabilities.json")]:
            require((generated / "contract" / name).read_bytes() == (FIXTURE / retained).read_bytes(),
                    f"fresh complete {name} differs from fixture")
        programs = {p.name: p.read_bytes() for p in (generated / "contract/checked-programs").glob("*.json")}
        retained = {p.name: p.read_bytes() for p in (FIXTURE / "checked-programs").glob("*.json")}
        require(len(programs) == 12 and programs == retained, "complete twelve programs differ")
        formatted = output / "formatted.rs"
        shutil.copyfile(generated / "contract/lib.rs", formatted)
        execute(["rustfmt", "--edition", "2024", formatted], "format-generated")
        require(formatted.read_bytes() == (FIXTURE / "lib.rs").read_bytes(), "fresh complete generated Rust differs")
        capabilities = json.loads((generated / "contract/rust-capabilities.json").read_text())
        common.validate_report(capabilities, SOURCE)
        receipt["proof_applicability_metadata"] = common.proof_cross_tab(
            capabilities, json.loads((generated / "compiler/contract-info.json").read_text()), SOURCE)
        require(capabilities["schema_version"] == 3 and len(capabilities["circuits"]) == 12 and
                all(row["recorded"] and row["observed_call"] and row["proof_required"]
                    for row in capabilities["circuits"]), "DID capability metadata changed")

        crypto = output / "crypto"
        execute([compiler, "--target", "ts", "--skip-zk",
                 "examples/rust_backend/unit_composition_crypto.compact", crypto], "generate-crypto-oracle-control")
        receipt["oracle_wrapper_hashes"] = hashes([
            generated / "contract/index.js", crypto / "contract/index.js"])
        for directory in [generated, crypto]:
            link = directory / "contract/node_modules/@midnight-ntwrk/compact-runtime"
            link.parent.mkdir(parents=True)
            link.symlink_to(runtime, target_is_directory=True)
        capture = output / "live-typescript.json"
        execute(["node", CONTROLS / "capture.mjs", crypto / "contract/index.js",
                 generated / "contract/index.js", runtime, version], "live-typescript", capture)
        observations = json.loads(capture.read_text())
        did_rows = [row for row in observations["cases"] if row["source"] == "did"]
        expected = json.loads((FIXTURE / "data/unit-composition.json").read_text())
        require(expected["runtime_version"] == version and len(did_rows) == 12 and
                did_rows == expected["cases"], "complete live DID observations differ")
        receipt["typescript"] = {"capture_sha256": common.sha256(capture),
                                  "did_rows": 12, "all_rows": len(observations["cases"]),
                                  "rust_selected_ids": ["did/valid"] + [row["id"] for row in did_rows if "error" in row],
                                  "failure_route_executions": 40}
        tests = output / "tests.log"
        execute(["cargo", "test", "--locked", "--offline", "-p", PACKAGE, "--tests"], "four-route-tests", tests)
        text = tests.read_text()
        counts = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored", text)
        require(counts and sum(int(row[0]) for row in counts) == len(TESTS) and
                all(row[1:] == ("0", "0") for row in counts) and
                all(f"test {name} ... ok" in text for name in TESTS), "selected lifecycle accounting changed")
        execute(["cargo", "clippy", "--locked", "--offline", "-p", PACKAGE, "--all-targets",
                 "--", "-D", "warnings"], "consumer-clippy")

        consumer = output / "api-refusals"
        consumer.mkdir()
        manifest_path = consumer / "Cargo.toml"
        lines = ["[workspace]", "[package]", 'name="did-functional-api-refusals"',
                 'version="0.0.0"', 'edition="2024"', "[dependencies]",
                 f'did_contract = {{ package={json.dumps(PACKAGE)}, path={json.dumps(str(FIXTURE))} }}',
                 "[features]", *[f"{name}=[]" for name in NEGATIVES]]
        for name in NEGATIVES:
            shutil.copyfile(CONTROLS / f"{name}.rs", consumer / f"{name}.rs")
            lines += ["[[bin]]", f'name="{name}"', f'path="{name}.rs"', f'required-features=["{name}"]']
        manifest_path.write_text("\n".join(lines) + "\n")
        shutil.copyfile(ROOT / "Cargo.lock", consumer / "Cargo.lock")
        execute(["cargo", "metadata", "--offline", "--format-version", "1", "--manifest-path", manifest_path],
                "refusal-consumer-lock", output / "consumer-metadata.json")
        metadata = json.loads((output / "consumer-metadata.json").read_text())
        runtimes = [row for row in metadata["packages"] if row["name"] == "midnight-compact-runtime"]
        require(len(runtimes) == 1 and Path(runtimes[0]["manifest_path"]).resolve() ==
                (ROOT / "runtime-rs/Cargo.toml").resolve(), "refusal consumer has a foreign runtime")
        receipt["refusal_consumer_hashes"] = hashes([
            manifest_path, consumer / "Cargo.lock", *consumer.glob("*.rs")])
        original_lock = {(p["name"], p["version"], p.get("source")): p.get("checksum")
                         for p in tomllib.loads((ROOT / "Cargo.lock").read_text())["package"]}
        for item in tomllib.loads((consumer / "Cargo.lock").read_text())["package"]:
            if item.get("source", "").startswith("registry+"):
                key = (item["name"], item["version"], item["source"])
                require(key in original_lock and original_lock[key] == item.get("checksum"),
                        "refusal consumer registry identity differs")
        receipt["compile_fail"] = {}
        for name, expected_code in NEGATIVES.items():
            argv = ["cargo", "check", "--locked", "--offline", "--manifest-path", str(manifest_path),
                    "--bin", name, "--features", name, "--message-format=json"]
            process = subprocess.run(argv, cwd=ROOT, env=env, capture_output=True, text=True, timeout=300)
            (output / f"{name}.stdout").write_text(process.stdout)
            (output / f"{name}.stderr").write_text(process.stderr)
            errors = []
            for line in process.stdout.splitlines():
                try:
                    item = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if item.get("reason") == "compiler-message" and item["message"]["level"] == "error":
                    errors.append((item["message"].get("code") or {}).get("code"))
            require(process.returncode != 0 and set(errors) == {expected_code}, f"{name} refused for the wrong reason")
            receipt["compile_fail"][name] = errors
            receipt["commands"].append({"label": name, "argv": argv, "exit_code": process.returncode,
                                        "stdout_sha256": common.sha256(output / f"{name}.stdout"),
                                        "stderr_sha256": common.sha256(output / f"{name}.stderr")})

        require(generated_manifest_snapshot(generated) == receipt["generated"], "generated output changed")
        unchanged(receipt["source_hashes"])
        unchanged(receipt["compiler_hashes"])
        unchanged(receipt["rust_tool_hashes"])
        unchanged(receipt["oracle_wrapper_hashes"])
        unchanged(receipt["refusal_consumer_hashes"])
        require(common.sha256(capture) == receipt["typescript"]["capture_sha256"], "TS capture changed")
        identity = observations["qualification"]
        unchanged(identity["runtime"]["files"])
        unchanged({identity["node"]["path"]: identity["node"]["sha256"], **identity["node"]["sharedLibraries"]})
        receipt["rust_tests"] = len(TESTS)
        require_clean_source(receipt)
        receipt["status"] = "passed"
    except Exception as error:
        receipt["status"] = "failed"
        receipt["error"] = str(error)
    finally:
        receipt["seconds"] = time.monotonic() - start
        (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps({"status": receipt["status"], "receipt": str(output / "receipt.json")}))
    return 0 if receipt["status"] == "passed" else 1


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler", type=Path, required=True)
    parser.add_argument("--ts-runtime", type=Path, required=True)
    parser.add_argument("--target-dir", type=Path, default=Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")))
    parser.add_argument("--output", type=Path, required=True)
    raise SystemExit(run(parser.parse_args()))
