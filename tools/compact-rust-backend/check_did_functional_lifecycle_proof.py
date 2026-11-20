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

"""Fresh current-profile original DID functional proof and strict application.

The complete required native/CI suites remain separate. Constructor execution
is not proved. No proof, verification verdict or transaction is reused.
"""
import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tomllib

import local_parity_gate as common
import resolve_ledger_test_static as ledger_static
from check_did_functional_lifecycle import (SOURCE, FIXTURE, original_source_closure,
                                            require_clean_source)
from check_functional_effects import digest
from check_functional_effects import generated_manifest_snapshot, scheme_compiler
from did_proof_gate import dust_fixture_inventory, hashes, unchanged, require

ROOT = common.ROOT
LEGACY_PROOF_RESULT_FORMAT = "compact-did-functional-lifecycle-proof/v1"
PROOF_RESULT_FORMAT = "compact-did-functional-lifecycle-proof/v2"
V3_PROOF_RESULT_FORMAT = "compact-did-functional-lifecycle-proof/v3"
DID_PARAMETER_K = 11
CASES = {"did/valid": ("200", "--did-functional-lifecycle"),
         "did/timestamp-zero": ("0", "--did-functional-lifecycle-timestamp-zero")}


def validate_proof_summary(summary, expected_case=None, *, require_v3=False):
    """Validate the result protocol; live Rust assertions establish the behavior."""
    # Decode old case-unbound receipts only as historical protocol evidence.
    # Every current gate invocation below supplies its exact expected case.
    version = summary["format"]
    current = version in (PROOF_RESULT_FORMAT, V3_PROOF_RESULT_FORMAT)
    if require_v3:
        require(version == V3_PROOF_RESULT_FORMAT, "DID gate requires a Ledger 9 v3 proof receipt")
    if version == V3_PROOF_RESULT_FORMAT:
        require(type(summary.get("transaction_proof_version")) is int
                and summary["transaction_proof_version"] == 3
                and type(summary.get("funded_dust_spends")) is int
                and summary["funded_dust_spends"] == 1
                and type(summary.get("transaction_proof_bytes")) is int
                and summary["transaction_proof_bytes"] > 0,
                "DID v3 receipt requires a separate transaction proof and one proved Dust spend")
    if current:
        source_case = summary.get("source_case")
        require(source_case in CASES and summary.get("timestamp_witness") == CASES[source_case][0],
                "invalid DID proof case binding")
    else:
        require("source_case" not in summary and "timestamp_witness" not in summary,
                "historical DID receipt cannot claim a selected case")
    if expected_case is not None:
        require(expected_case in CASES and current and summary.get("source_case") == expected_case,
                "DID proof result does not bind the requested case")
    require((version == LEGACY_PROOF_RESULT_FORMAT or current)
            and summary["status"] == "passed"
            and summary["constructor_execution_proved"] is False
            and summary["all_twelve_exports_proved"] is False
            and summary["execution"] == "functional-recorded"
            and summary["circuit"] == "deactivate", "invalid selected DID proof scope")
    require(type(summary["proof_bytes"]) is int and summary["proof_bytes"] > 0
            and all(summary[key] is True for key in ["changed_binding_rejected",
                "changed_version_input_rejected", "changed_input_same_geometry",
                "replay_unchanged", "stale_version_rejected"])
            and summary["strict_application"] == "default"
            and summary["replay_refusal"] == "IntentAlreadyExists"
            and summary["changed_unit_result_control"] == "not applicable: Unit has zero output atoms"
            and summary["stale_version_error"] == "Controller authorization version is stale"
            and summary["standalone_call_proof_file"] == "deactivate-input-control.proof"
            and summary["applied_transaction_file"] == "did-functional-applied-transaction.bin"
            and type(summary["applied_transaction_bytes"]) is int and summary["applied_transaction_bytes"] > 0
            and summary["proof_relationship"] == "standalone mutation-control proof; transaction independently proved and default-strictly verified"
            and bool(bytes.fromhex(summary["final_state_hex"])), "incomplete DID proof/application controls")


def v3_artifacts(generated, summary):
    proof = generated / "deactivate-input-control.proof"
    transaction_proof = generated / "deactivate-transaction-bound.proof"
    inputs = generated / "deactivate-input-control.inputs"
    applied = generated / summary["applied_transaction_file"]
    updated = generated / "did-ledger-after.bin"
    replayed = generated / "did-ledger-replay.bin"
    for artifact, field in ((proof, "proof_bytes"),
                            (transaction_proof, "transaction_proof_bytes"),
                            (applied, "applied_transaction_bytes")):
        require(artifact.stat().st_size == summary[field], f"unexpected {artifact.name} length")
    require(digest(proof) != digest(transaction_proof), "DID call proofs must be distinct")
    require(inputs.stat().st_size > 0 and updated.stat().st_size > 0 and
            digest(updated) == digest(replayed), "DID public inputs or exact replay bytes differ")
    return [proof, transaction_proof, inputs, applied, updated, replayed]


def run(args):
    output = args.output.absolute()
    require(not output.exists() and not output.is_symlink(), "proof destination must be absent")
    output.mkdir(parents=True)
    receipt = {"format": "compact-did-functional-lifecycle-proof-gate/v2", "status": "incomplete", "commands": [],
               "scope": "one unchanged original-DID functional-recorded deactivation call proof and default-strict funded local transaction",
               "constructor_execution_proved": False, "full_integration_acceptance": "OPEN"}
    env = dict(os.environ, RUSTUP_TOOLCHAIN="1.99.0", CARGO_INCREMENTAL="0", PYTHONDONTWRITEBYTECODE="1")
    try:
        require_clean_source(receipt)
        require("MIDNIGHT_PROOF_SERVER" not in env, "DID proof gate requires local proving")
        require(args.case in CASES, "unknown DID proof case")
        runner_case = CASES[args.case][1]
        receipt["source_case"] = args.case
        receipt["timestamp_witness"] = CASES[args.case][0]
        target = args.target_dir.absolute()
        env["CARGO_TARGET_DIR"] = str(target)
        require(bool(env.get("MIDNIGHT_PP")) and Path(env["MIDNIGHT_PP"]).is_absolute(), "MIDNIGHT_PP required")
        closure = original_source_closure()
        paths = closure + [SOURCE, ROOT / "Cargo.toml", ROOT / "Cargo.lock", Path(__file__),
                 ROOT / "tools/compact-rust-backend/check_did_functional_lifecycle.py",
                 ROOT / "tools/compact-rust-backend/check_functional_effects.py",
                 ROOT / "tools/compact-rust-backend/check_compactc_target.py",
                 Path(common.__file__), Path(ledger_static.__file__),
                 ROOT / "tools/compact-rust-backend/did_proof_gate.py",
                 ROOT / "tools/compact-rust-backend/Cargo.toml"]
        for directory in ["runtime-rs", "runtime-rs-macros", "tools/compact-rust-backend/src",
                          "tools/compact-rust-proof-smoke", "tests-rust-backend"]:
            paths += [p for p in (ROOT / directory).rglob("*") if p.suffix in {".rs", ".toml", ".json"}]
        paths += sorted((ROOT / "compiler").glob("*.ss"))
        receipt["source_hashes"] = hashes(paths)
        compiler = args.compiler.resolve(strict=True)
        scheme = scheme_compiler(compiler, env)
        selected_zkir = args.zkir or shutil.which("zkir-v3", path=env.get("PATH"))
        require(selected_zkir is not None, "official direct zkir must be selected")
        (output / "bin").mkdir()
        components = receipt["components"] = {}
        for name, path in [("compactc", compiler), ("compactc-scheme", scheme), ("zkir", Path(selected_zkir).resolve(strict=True))]:
            components[name] = common.stable_copy(path, output / "bin" / name)
        env["COMPACTC_SCHEME"] = components["compactc-scheme"]["snapshot"]

        def execute(argv, label, stdout=None):
            common.run(list(map(str, argv)), label, output, receipt, env=env, stdout_path=stdout)

        execute(["cargo", "metadata", "--locked", "--offline", "--format-version", "1"],
                "cargo-metadata", output / "cargo-metadata.json")
        ledger = ledger_static.resolve(json.loads((output / "cargo-metadata.json").read_text()),
                                       tomllib.loads((ROOT / "Cargo.lock").read_text()))
        static = Path(ledger["static_directory"])
        env["MIDNIGHT_LEDGER_TEST_STATIC_DIR"] = str(static)
        receipt["ledger_static"] = ledger
        receipt["dust_fixture_hashes"] = dust_fixture_inventory(static)
        execute(["cargo", "build", "--profile", "proof-checks", "--locked", "--offline", "-p", "compact-rust-proof-smoke"], "build-runner")
        components["runner"] = common.stable_copy(target / "proof-checks/compact-rust-proof-smoke", output / "bin/proof-runner")
        runner = components["runner"]["snapshot"]
        generated = output / "generated"
        execute([components["compactc"]["snapshot"], "--target", "rust", "--rust-functional-ledger9",
                 "--rust-runtime-root", ROOT, "--skip-zk", SOURCE.relative_to(ROOT), generated], "generate")
        receipt["generated"] = generated_manifest_snapshot(generated)
        formatted = output / "formatted-lib.rs"
        shutil.copyfile(generated / "contract/lib.rs", formatted)
        execute(["rustfmt", "--edition", "2024", formatted], "format-generated")
        # Formatting is the only transformation; full IR and program bytes are exact.
        require(formatted.read_bytes() == (FIXTURE / "lib.rs").read_bytes(), "linked Rust fixture differs")
        require((generated / "contract/compact-rust-ir.json").read_bytes() == (FIXTURE / "original-ir.json").read_bytes(), "complete linked IR differs")
        fresh = {p.name: p.read_bytes() for p in (generated / "contract/checked-programs").glob("*.json")}
        retained = {p.name: p.read_bytes() for p in (FIXTURE / "checked-programs").glob("*.json")}
        require(len(fresh) == 12 and fresh == retained, "complete twelve linked functional programs differ")
        receipt["formatted_rust_sha256"] = digest(formatted)
        receipt["generated_hashes"] = hashes([generated / path for path in receipt["generated"]["assets"]] +
                                             [generated / "compiler/contract-manifest.json", formatted])
        negative = output / "changed-ir"
        (negative / "contract").mkdir(parents=True)
        changed = json.loads((generated / "contract/compact-rust-ir.json").read_text())
        changed["contractName"] = "changed-did-functional-proof-source"
        (negative / "contract/compact-rust-ir.json").write_text(json.dumps(changed))
        refused = subprocess.run([runner, runner_case, str(negative)], cwd=ROOT,
                                 env=env, capture_output=True, text=True, timeout=30)
        (output / "changed-ir.stdout").write_text(refused.stdout)
        (output / "changed-ir.stderr").write_text(refused.stderr)
        require(refused.returncode != 0 and "Proof artifact IR differs from the generated source fixture" in refused.stderr,
                "changed full IR did not refuse before absent keys/state")
        require(not (negative / "keys").exists() and not list(negative.glob("did-functional-*")),
                "changed IR refusal created proof/state outputs")
        receipt["changed_ir_refusal"] = {"exit_code": refused.returncode, "before_keys_or_state": True}
        changed_program = output / "changed-program"
        shutil.copytree(generated / "contract/checked-programs", changed_program / "contract/checked-programs")
        shutil.copyfile(generated / "contract/compact-rust-ir.json", changed_program / "contract/compact-rust-ir.json")
        selected_program = next((changed_program / "contract/checked-programs").glob("*.json"))
        selected_program.write_bytes(selected_program.read_bytes() + b" ")
        refused = subprocess.run([runner, runner_case, str(changed_program)], cwd=ROOT,
                                 env=env, capture_output=True, text=True, timeout=30)
        (output / "changed-program.stdout").write_text(refused.stdout)
        (output / "changed-program.stderr").write_text(refused.stderr)
        require(refused.returncode != 0 and "DID checked-program bytes differ" in refused.stderr
                and not (changed_program / "keys").exists(), "changed program did not refuse before keys/state")
        receipt["changed_program_refusal"] = {"exit_code": refused.returncode, "before_keys_or_state": True}
        zkir = components["zkir"]["snapshot"]
        version = output / "zkir-version.txt"
        execute([zkir, "--version"], "zkir-version", version)
        require(version.read_text().strip() == "midnight-zkir 3.2.0", "expected published ZKIR v3 3.2.0")
        source_ir = generated / "zkir/deactivate.zkir"
        execute([zkir, "mock-compile", source_ir], "model-deactivate")
        geometry = re.search(r"\(k=(\d+), rows=(\d+)\)", Path(receipt["commands"][-1]["log"]).read_text())
        require(geometry is not None, "model omitted DID circuit geometry")
        k, rows = map(int, geometry.groups())
        require(k == DID_PARAMETER_K and rows > 0, "DID proof must stay within its explicit k11 recipe")
        material = output / "proof-material.json"
        execute([runner, "--prepare-proof-material", "--verify-only", "--parameter-k", k],
                "verify-proof-material", material)
        selected = json.loads(material.read_text())
        require(selected["mode"] == "verify-only" and
                selected["verification"] == "all files checked against locked upstream SHA256 tables" and
                any(row["k"] == k for row in selected["parameters"]), "DID proof material was not verified")
        require(Path(selected["cache_directory"]).resolve() == Path(env["MIDNIGHT_PP"]).resolve(),
                "unexpected parameter directory")
        material_paths = [Path(selected["cache_directory"]) / row["name"]
                          for row in selected["assets"] + selected["parameters"]]
        (generated / "keys").mkdir()
        prover = generated / "keys/deactivate.prover"
        verifier = generated / "keys/deactivate.verifier"
        execute([zkir, "compile", source_ir, prover, verifier], "keygen-deactivate")
        keygen = re.search(r"\(k=(\d+), rows=(\d+)\)", Path(receipt["commands"][-1]["log"]).read_text())
        require(keygen is not None and keygen.groups() == geometry.groups(),
                "DID key generation geometry differs from the preflight model")
        receipt["keygen"] = [{"mode": "fresh direct native v3", "circuit": "deactivate", "k": k, "rows": rows}]
        artifacts = [prover, verifier, source_ir, generated / "zkir/deactivate.bzkir"]
        require(all(p.is_file() and p.stat().st_size for p in material_paths + artifacts), "proof material missing")
        receipt["material_hashes"] = hashes(material_paths + artifacts)
        execute([runner, runner_case, generated], "prove-verify-apply")
        result = generated / "did-functional-lifecycle-proof-result.json"
        summary = json.loads(result.read_text())
        validate_proof_summary(summary, expected_case=args.case, require_v3=True)
        receipt["proof_result"] = summary
        receipt["result_hashes"] = hashes([result, *v3_artifacts(generated, summary)])
        execute([runner, "--did-functional-lifecycle-verify", generated], "verify-standalone-separately")
        unchanged(receipt["result_hashes"])
        unchanged(receipt["source_hashes"])
        unchanged(receipt["dust_fixture_hashes"])
        unchanged(receipt["material_hashes"])
        unchanged(receipt["generated_hashes"])
        for component in components.values():
            unchanged({component["source"]: component["sha256"], component["snapshot"]: component["sha256"]})
        require_clean_source(receipt)
        receipt["status"] = "passed"
    except Exception as error:
        receipt["status"] = "failed"
        receipt["error"] = str(error)
    finally:
        (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps({"status": receipt["status"], "receipt": str(output / "receipt.json")}))
    return 0 if receipt["status"] == "passed" else 1


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler", type=Path, required=True)
    parser.add_argument("--zkir", type=Path)
    parser.add_argument("--target-dir", type=Path, default=Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")))
    parser.add_argument("--case", choices=tuple(CASES), default="did/valid")
    parser.add_argument("--output", type=Path, required=True)
    raise SystemExit(run(parser.parse_args()))
