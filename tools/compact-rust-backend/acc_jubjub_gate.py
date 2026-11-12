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

"""Qualify the bounded ACC Jubjub Cell slice; no full-account adoption claim.

Fresh Ledger 9 TypeScript is checked against the active capture and the
independent original point oracle. Seven apply calls use fresh v3 keys and
standalone proofs, separate verification and funded default-strict application.
Raw retains behavior and refusal coverage. Constructor data is deployed
without a constructor-execution proof.
"""
import argparse
import json
import os
from pathlib import Path
import shutil

import did_proof_gate as did
import local_parity_gate as common
import did_primitive_reducer_gate as primitive
import did_reducer_proof as reducer

ROOT = common.ROOT
SOURCE = Path("examples/rust_backend/jubjub_scalar_cell.compact")
IR = Path("tools/compact-rust-backend/tests/jubjub-scalar-cell-schema20-ir.json")
FIXTURE_ROOT = Path("tests-rust-backend/jubjub-scalar-cell")
FIXTURE = FIXTURE_ROOT / "lib.rs"
REFERENCE = FIXTURE_ROOT / "oracle/cases.json"
CURRENT_CAPTURE = FIXTURE_ROOT / "oracle/cases-ledger9.json"
BOUNDARIES = FIXTURE_ROOT / "oracle/boundaries.json"
CAPTURE = Path("tools/compact-rust-backend/acc_jubjub_capture.mjs")
RUNTIME_PACKAGE = Path("runtime/package.json")
GEOMETRY = {"apply": (12, 2236), "raw": (10, 796)}
CASES = ("zero", "one", "eight", "q_minus_one", "q", "q_plus_one", "native_max")
ROW = {"kind": "acc-jubjub-cell", "proof_cases": [
    {"case": name, "operation": "apply"} for name in CASES]}
require = primitive.require
read_json = primitive.read_json
hashes = primitive.hashes


def source_inventory():
    files = [ROOT / path for path in (SOURCE, IR, FIXTURE, REFERENCE, CURRENT_CAPTURE, BOUNDARIES,
             CAPTURE, RUNTIME_PACKAGE,
             "Cargo.toml", "Cargo.lock", "tools/compact-rust-proof-smoke/Cargo.toml",
             "tools/compact-rust-backend/Cargo.toml",
             "tools/compact-rust-backend/test_acc_jubjub_gate.py",
             "tools/compact-rust-backend/test_local_parity_gate.py")]
    files += [Path(__file__), Path(common.__file__), Path(primitive.__file__),
              Path(did.__file__), Path(did.ledger_static.__file__), Path(reducer.__file__)]
    require(all(path.is_file() for path in files), "missing maintained qualification input")
    for folder in ("runtime-rs", "runtime-rs-macros", "testkit-rs",
                   "tools/compact-rust-backend/src", "tools/compact-rust-proof-smoke/src",
                   FIXTURE_ROOT):
        files.extend(path for path in (ROOT / folder).rglob("*")
                     if path.suffix in (".rs", ".json", ".toml", ".compact", ".mjs"))
    return hashes(files)


def validate_capture(value):
    require(value.get("format") == "compact-acc-jubjub-wrapper-capture/v1"
            and value.get("kind") == ROW["kind"], "wrong capture identity")
    require([(r.get("id"), r.get("operation")) for r in value.get("cases", [])]
            == [(name, "apply") for name in CASES], "capture must contain exact seven apply cases")
    require([r.get("id") for r in value.get("rawCases", [])] == ["raw-q_minus_one", "raw-q"],
            "capture must contain raw canonical success and q refusal")
    require([r.get("id") for r in value.get("refusals", [])] == [
        "raw-q-generator", "raw-q-point", "negative", "native-modulus", "number-not-bigint"],
        "capture refusal controls changed")


def check_hashes(expected, message):
    require(bool(expected) and all(Path(path).is_file() and common.sha256(Path(path)) == digest
                                  for path, digest in expected.items()), message)


NODE_SHARED_OBJECTS = "console.log(JSON.stringify({sharedObjects:process.report.getReport().sharedObjects}))"


def node_library_identity(report):
    """Bind loaded Node libraries; macOS system-cache objects have no loose files."""
    paths = report.get("sharedObjects")
    require(isinstance(paths, list) and paths and all(isinstance(p, str) and Path(p).is_absolute() for p in paths),
            "invalid Node shared-library report")
    system = sorted(p for p in paths if p.startswith(("/System/Library/", "/usr/lib/")))
    external = sorted(set(paths) - set(system))
    require(external and all(Path(path).is_file() for path in external), "missing Node dynamic library")
    return {"file_hashes": hashes([Path(path) for path in external]),
            "system_libraries": system, "system_scope": "host OS libraries; names retained, not byte qualified"}


def run_gate(directory, compiler, scheme, target, environment=None, *, command=common.run,
             snapshot=common.stable_copy):
    directory = directory.resolve()
    directory.mkdir(parents=True, exist_ok=False, mode=0o700)
    receipt = {"format": "compact-acc-jubjub-gate/v1", "status": "failed", "commands": [],
               "scope": __doc__, "expected_proof_cases": ROW["proof_cases"],
               "oracle_mode": "retained-original-reference"}
    env = dict(os.environ if environment is None else environment)
    try:
        require("MIDNIGHT_PROOF_SERVER" not in env, "ACC Jubjub requires local proving")
        prereq = receipt["prerequisites"] = did.prerequisites(env, zkir_name="zkir-v3")
        discovered = {name: shutil.which(name, path=env.get("PATH")) for name in ("node", "rustup")}
        require(all(discovered.values()), "ACC Jubjub gate requires node and rustup")
        discovered["zkir"] = prereq["zkir"]
        receipt["dust_fixture_hashes"] = did.dust_fixture_inventory(Path(prereq["MIDNIGHT_LEDGER_TEST_STATIC_DIR"]))
        runtime_root = Path(env.get("COMPACT_TS_RUNTIME_DIR", ROOT / "runtime")).resolve(strict=True)
        runtime_manifest = reducer.checked_file(runtime_root / "package.json")
        runtime_package = read_json(runtime_manifest)
        source_package = read_json(ROOT / RUNTIME_PACKAGE)
        require(all(runtime_package.get(key) == source_package[key] for key in ("name", "version")),
                "ACC Jubjub requires the current Compact TypeScript runtime")
        receipt["typescript_runtime"] = {"directory": str(runtime_root), "version": runtime_package["version"]}
        receipt["source_hashes"] = source_inventory()
        receipt["git_head"] = common.git_head()
        (directory / "bin").mkdir()
        components = receipt["components"] = {}
        formatter_selection = directory / "rustfmt-path.txt"
        command([discovered["rustup"], "which", "--toolchain", "1.99.0", "rustfmt"],
                "select-rustfmt", directory, receipt, env=env, stdout_path=formatter_selection)
        formatter = Path(formatter_selection.read_text().strip())
        require(formatter.is_absolute() and formatter.is_file() and os.access(formatter, os.X_OK),
                "rustup did not resolve pinned rustfmt")
        # rustfmt loads toolchain-relative shared libraries, so moving its binary
        # breaks macOS. Bind the selected executable in place and recheck it.
        components["rustfmt"] = {"source": str(formatter), "snapshot": str(formatter),
                                  "sha256": common.sha256(formatter), "size": formatter.stat().st_size,
                                  "execution": "pinned toolchain executable in place"}
        del discovered["rustup"]
        node_path = Path(discovered.pop("node")).resolve()
        components["node"] = {"source": str(node_path), "snapshot": str(node_path),
                              "sha256": common.sha256(node_path), "size": node_path.stat().st_size,
                              "execution": "in place with loaded shared-library identities"}
        for name, path in (("compactc", compiler), ("compactc-scheme", scheme),
                           *((name, Path(path)) for name, path in discovered.items())):
            components[name] = snapshot(path, directory / "bin" / name)
        env.update({"COMPACTC_SCHEME": components["compactc-scheme"]["snapshot"],
                    "COMPACT_RUST_RUNTIME_DIR": str(ROOT), "CARGO_TARGET_DIR": str(target.resolve()),
                    "CARGO_INCREMENTAL": "0", "RUSTUP_TOOLCHAIN": "1.99.0",
                    "PYTHONDONTWRITEBYTECODE": "1"})

        zkir = components["zkir"]["snapshot"]
        receipt["key_generation"] = {"mode": "direct-native-v3", "actual_zkir": zkir}
        def execute(argv, label, stdout_path=None):
            command([str(arg) for arg in argv], label, directory, receipt, env=env,
                    stdout_path=stdout_path)

        node = components["node"]["snapshot"]
        version = directory / "zkir-version.txt"
        reducer.require_zkir_version(execute, zkir, version)
        node_libraries = directory / "node-libraries.json"
        execute([node, "-e", NODE_SHARED_OBJECTS], "freeze-node-runtime", node_libraries)
        receipt["node_libraries"] = node_library_identity(read_json(node_libraries))
        runtime_inventory = directory / "runtime-inventory.json"
        execute([node, ROOT / CAPTURE, "--runtime-root", runtime_root, "--runtime-inventory"],
                "freeze-ts-runtime", runtime_inventory)
        receipt["ts_runtime_hashes"] = read_json(runtime_inventory)
        check_hashes(receipt["ts_runtime_hashes"], "invalid initial TS runtime inventory")
        require(str(runtime_manifest) in receipt["ts_runtime_hashes"],
                "TypeScript runtime inventory omitted its package manifest")
        reference = read_json(ROOT / REFERENCE)
        validate_capture(reference)
        current = read_json(ROOT / CURRENT_CAPTURE)
        validate_capture(current)
        output = directory / ROW["kind"]
        execute([components["compactc"]["snapshot"], "--target", "ts", "--target", "rust",
                 "--feature-zkir-v3", "--skip-zk",
                 "--rust-runtime-root", ROOT, ROOT / SOURCE, output], "compile-jubjub")
        # Keep generated source untouched; the maintained fixture uses pinned rustfmt.
        generated_source_hash = common.sha256(output / "contract/lib.rs")
        formatted = directory / "formatted-fixture.rs"
        formatted.write_bytes((output / "contract/lib.rs").read_bytes())
        execute([components["rustfmt"]["snapshot"], "--edition", "2024", formatted],
                "compare-formatted-fixture")
        require(common.sha256(output / "contract/lib.rs") == generated_source_hash,
                "formatter changed compiler-generated source")
        require(formatted.read_bytes() == (ROOT / FIXTURE).read_bytes(), "generated Jubjub fixture differs")
        require(read_json(output / "contract/compact-rust-ir.json") == read_json(ROOT / IR),
                "maintained Jubjub IR differs")
        report = read_json(output / "contract/rust-capabilities.json")
        common.validate_report(report, ROOT / SOURCE)
        cap = common.proof_cross_tab(report, read_json(output / "compiler/contract-info.json"), ROOT / SOURCE)
        require([row["name"] for row in cap] == ["apply", "raw"]
                and all(all(row.get(k) is True for k in ("proof", "recorded", "observed_call")) for row in cap),
                "Jubjub apply/raw proof/recording capabilities changed")
        receipt["capabilities"] = report
        generated = [reducer.checked_file(path) for folder in ("contract", "compiler", "zkir")
                     for path in (output / folder).rglob("*") if path.is_file()]
        receipt["generated_hashes"] = hashes(generated)
        receipt["generated_hashes"].update(reducer.generated_hashes(output))
        geometry = receipt["geometry"] = reducer.model(execute, receipt, zkir, output, GEOMETRY)
        capture = directory / "ts-capture.json"
        provenance = directory / "capture-provenance.json"
        execute([node, ROOT / CAPTURE, "--ledger9", "--runtime-root", runtime_root,
                 "--generated", output / "contract/index.js", "--reference", ROOT / REFERENCE,
                 "--boundaries", ROOT / BOUNDARIES, "--source", ROOT / SOURCE,
                 "--provenance", provenance], "capture-ts-reference", capture)
        actual = read_json(capture)
        validate_capture(actual)
        require(capture.read_bytes() == (ROOT / CURRENT_CAPTURE).read_bytes(),
                "fresh Ledger 9 TypeScript differs from the current capture")
        provenance_data = read_json(provenance)
        require(provenance_data.get("format") == "compact-acc-jubjub-wrapper-provenance/v1"
                and provenance_data.get("oracleMode") == "retained-original-reference"
                and provenance_data.get("runtimeHashes") == receipt["ts_runtime_hashes"],
                "capture provenance or runtime selection changed")
        input_hashes = {str(Path(row["path"]).resolve()): row["sha256"] for row in provenance_data["inputs"]}
        runtime_entry = Path(provenance_data["runtime"]["path"]).resolve(strict=True)
        require(runtime_entry.is_relative_to(runtime_root) and str(runtime_entry) in receipt["ts_runtime_hashes"],
                "capture loaded a different TypeScript runtime")
        required_inputs = {str(path.resolve()) for path in (ROOT / CAPTURE, ROOT / SOURCE, ROOT / REFERENCE,
                           ROOT / BOUNDARIES, output / "contract/index.js", runtime_entry)}
        require(required_inputs <= input_hashes.keys(), "capture provenance omitted required input")
        check_hashes(input_hashes, "capture input identity mismatch")
        capture_hashes = hashes([capture, provenance, runtime_inventory, formatted, formatter_selection,
                                 node_libraries, version])
        execute(["cargo", "+1.99.0", "test", "--locked", "--offline", "-j4", "-p",
                 "compact-rust-jubjub-scalar-cell-fixture", "--all-features"], "test-jubjub-behavior")
        execute(["cargo", "+1.99.0", "build", "--profile", "proof-checks", "--locked", "--offline", "-j4", "-p",
                 "compact-rust-proof-smoke"], "build-proof-runner")
        components["proof-runner"] = snapshot(target.resolve() / "proof-checks/compact-rust-proof-smoke",
                                               directory / "bin/compact-rust-proof-smoke")
        runner = components["proof-runner"]["snapshot"]
        material = directory / "proof-material.json"
        k, rows = geometry["apply"]
        receipt["parameter_hashes"] = reducer.prepare_parameters(
            execute, receipt, runner, material, Path(prereq["MIDNIGHT_PP"]), {k})
        reducer.keygen(execute, receipt, zkir, output, "apply", (k, rows), "keygen-apply")
        receipt["keygen"] = {"operation": "apply", "k": k, "rows": rows}
        artifacts = [output / f"{folder}/apply.{ext}" for folder, ext in (
            ("keys", "prover"), ("keys", "verifier"), ("zkir", "zkir"), ("zkir", "bzkir"))]
        receipt["artifact_hashes"] = hashes(reducer.checked_file(path) for path in artifacts)
        execute([runner, "--did-primitive-reducer", ROW["kind"], output], "prove-apply")
        summary_path = reducer.checked_file(output / "proof-result.json")
        summary = read_json(summary_path)
        primitive.validate_summary(summary, ROW, require_v3=True)
        results = [summary_path, reducer.checked_file(output / "reducer-deployment.bin"),
                   reducer.checked_file(output / "reducer-ledger-after-deploy.bin")]
        for call in summary["calls"]:
            results.extend(reducer.proof_artifacts(output, call["case"], call, call["state_bytes"]))
        receipt["summary"] = summary
        receipt["result_hashes"] = hashes([*results, material]) | capture_hashes
        node_recheck = directory / "node-libraries-final.json"
        execute([node, "-e", NODE_SHARED_OBJECTS], "recheck-node-runtime", node_recheck)
        require(node_library_identity(read_json(node_recheck)) == receipt["node_libraries"],
                "Node shared-library identities changed during gate")
        receipt["result_hashes"].update(hashes([node_recheck]))
        require(source_inventory() == receipt["source_hashes"] and common.git_head() == receipt["git_head"],
                "source inventory or HEAD changed during gate")
        check_hashes(input_hashes, "capture inputs changed during gate")
        for field in ("generated_hashes", "ts_runtime_hashes", "dust_fixture_hashes",
                      "parameter_hashes", "artifact_hashes", "result_hashes"):
            check_hashes(receipt[field], f"{field} changed during gate")
        require(all(Path(row["snapshot"]).is_file() and common.sha256(Path(row["snapshot"])) == row["sha256"]
                    for row in components.values()), "frozen tool changed during gate")
        receipt["status"] = "passed"
    except (common.GateError, OSError, ValueError, KeyError, IndexError, TypeError) as error:
        receipt["error"] = str(error)
    finally:
        (directory / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    return receipt


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("compiler", "scheme", "target", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args()
    receipt = run_gate(args.output, args.compiler, args.scheme, args.target)
    print(f"ACC Jubjub gate {receipt['status']}; receipt {args.output / 'receipt.json'}")
    return 0 if receipt["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
