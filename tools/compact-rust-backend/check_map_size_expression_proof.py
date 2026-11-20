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

"""Fresh direct #610 proofs, independent verification and default-strict application.

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
from check_map_size_expression import SOURCE, digest
from check_functional_effects import generated_manifest_snapshot, scheme_compiler
from did_proof_gate import dust_fixture_inventory, hashes, unchanged

ROOT = common.ROOT
FIXTURE = ROOT / "tests-rust-backend/map-size-expression-source"


def require(value, message):
    if not value:
        raise common.GateError(message)


def run(args):
    output = args.output.absolute()
    require(not output.exists() and not output.is_symlink(), "proof destination must be absent")
    output.mkdir(parents=True)
    receipt = {"format": "compact-map-size-expression-proof-gate/v2", "status": "incomplete", "commands": [],
               "scope": "two source-bound functional-derived call proofs and default-strict funded local transactions",
               "constructor_execution_proved": False, "full_integration_acceptance": "OPEN"}
    env = dict(os.environ, RUSTUP_TOOLCHAIN="1.99.0", CARGO_INCREMENTAL="0", PYTHONDONTWRITEBYTECODE="1")
    try:
        require("MIDNIGHT_PROOF_SERVER" not in env, "Ledger 9 Map proof requires local proving")
        receipt["source_head"] = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
        receipt["source_dirty"] = bool(subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT))
        target = args.target_dir.absolute()
        env["CARGO_TARGET_DIR"] = str(target)
        require(bool(env.get("MIDNIGHT_PP")) and Path(env["MIDNIGHT_PP"]).is_absolute(), "MIDNIGHT_PP required")
        paths = [SOURCE, ROOT / "Cargo.toml", ROOT / "Cargo.lock", Path(__file__),
                 ROOT / "tools/compact-rust-backend/check_map_size_expression.py",
                 ROOT / "tools/compact-rust-backend/check_functional_effects.py",
                 Path(common.__file__), Path(ledger_static.__file__),
                 ROOT / "tools/compact-rust-backend/did_proof_gate.py",
                 ROOT / "tools/compact-rust-backend/Cargo.toml"]
        for directory in ["runtime-rs", "runtime-rs-macros", "tools/compact-rust-backend/src",
                          "tools/compact-rust-proof-smoke", "tests-rust-backend"]:
            paths += [p for p in (ROOT / directory).rglob("*") if p.suffix in {".rs", ".toml", ".json"}]
        receipt["source_hashes"] = hashes(paths)
        compiler = args.compiler.resolve(strict=True)
        scheme = scheme_compiler(compiler, env)
        selected_zkir = args.zkir or shutil.which("zkir-v3", path=env.get("PATH"))
        require(selected_zkir is not None, "official direct zkir-v3 must be selected")
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
        require(len(fresh) == 8 and fresh == retained, "complete linked functional programs differ")
        receipt["formatted_rust_sha256"] = digest(formatted)
        receipt["generated_hashes"] = hashes([generated / path for path in receipt["generated"]["assets"]] +
                                             [generated / "compiler/contract-manifest.json", formatted])
        negative = output / "changed-ir"
        (negative / "contract").mkdir(parents=True)
        changed = json.loads((generated / "contract/compact-rust-ir.json").read_text())
        changed["contractName"] = "changed-map-size-proof-source"
        (negative / "contract/compact-rust-ir.json").write_text(json.dumps(changed))
        refused = subprocess.run([runner, "--map-size-expression", str(negative)], cwd=ROOT,
                                 env=env, capture_output=True, text=True, timeout=30)
        (output / "changed-ir.stdout").write_text(refused.stdout)
        (output / "changed-ir.stderr").write_text(refused.stderr)
        require(refused.returncode != 0 and "Proof artifact IR differs from the generated source fixture" in refused.stderr,
                "changed full IR did not refuse before absent keys/state")
        require(not (negative / "keys").exists() and not list(negative.glob("map-size-*")),
                "changed IR refusal created proof/state outputs")
        receipt["changed_ir_refusal"] = {"exit_code": refused.returncode, "before_keys_or_state": True}
        negative = output / "changed-program"
        shutil.copytree(generated / "contract/checked-programs", negative / "contract/checked-programs")
        shutil.copyfile(generated / "contract/compact-rust-ir.json", negative / "contract/compact-rust-ir.json")
        program = negative / "contract/checked-programs/program-0.json"
        program.write_bytes(program.read_bytes() + b" ")
        refused = subprocess.run([runner, "--map-size-expression", str(negative)], cwd=ROOT,
                                 env=env, capture_output=True, text=True, timeout=30)
        (output / "changed-program.stdout").write_text(refused.stdout)
        (output / "changed-program.stderr").write_text(refused.stderr)
        require(refused.returncode != 0 and "checked program differs from the linked fixture" in refused.stderr
                and not (negative / "keys").exists() and not list(negative.glob("map-size-*")),
                "changed checked program did not refuse before keys/state")
        receipt["changed_program_refusal"] = {"exit_code": refused.returncode, "before_keys_or_state": True}
        zkir = components["zkir"]["snapshot"]
        version = output / "zkir-version.stdout"
        execute([zkir, "--version"], "zkir-version", version)
        require(version.read_text().strip() == "midnight-zkir 3.2.0", "expected published ZKIR v3 3.2.0")
        source_ir = generated / "zkir/put.zkir"
        execute([zkir, "mock-compile", source_ir], "model-put")
        model = re.search(r"\(k=(\d+), rows=(\d+)\)", Path(receipt["commands"][-1]["log"]).read_text())
        require(model is not None, "Map model omitted circuit geometry")
        k, rows = map(int, model.groups())
        require(k == 7 and rows == 119, "Map v3 model differs from the k7/119-row recipe")
        material = output / "proof-material.json"
        execute([runner, "--prepare-proof-material", "--verify-only", "--parameter-k", k],
                "verify-proof-material", material)
        selected = json.loads(material.read_text())
        require(selected["mode"] == "verify-only"
                and selected["verification"] == "all files checked against locked upstream SHA256 tables"
                and any(row["k"] == k for row in selected["parameters"]), "Map proof material was not verified")
        require(Path(selected["cache_directory"]).resolve() == Path(env["MIDNIGHT_PP"]).resolve(),
                "unexpected parameter directory")
        (generated / "keys").mkdir()
        execute([zkir, "compile", source_ir,
                 generated / "keys/put.prover", generated / "keys/put.verifier"], "keygen")
        keygen = re.search(r"\(k=(\d+), rows=(\d+)\)", Path(receipt["commands"][-1]["log"]).read_text())
        require(keygen is not None and keygen.groups() == model.groups(), "Map keygen model differs from preflight")
        receipt["keygen"] = {"mode": "fresh direct native v3", "circuit": "put", "k": k, "rows": rows}
        material_paths = [Path(selected["cache_directory"]) / row["name"] for row in selected["assets"] + selected["parameters"]]
        artifacts = [generated / f"keys/put.{ext}" for ext in ("prover", "verifier")]
        artifacts += [generated / f"zkir/put.{ext}" for ext in ("zkir", "bzkir")]
        require(all(p.is_file() and p.stat().st_size for p in material_paths + artifacts), "proof material missing")
        receipt["material_hashes"] = hashes(material_paths + artifacts)
        execute([runner, "--map-size-expression", generated], "prove-verify-apply")
        summary = json.loads((generated / "map-size-proof-result.json").read_text())
        require(summary["format"] == "compact-map-size-expression-proof/v2" and summary["status"] == "passed"
                and summary["constructor_execution_proved"] is False, "invalid Map size proof summary")
        require([row["key"] for row in summary["calls"]] == [1, 2], "missing replacement/new insertion proof")
        require(summary["nested_adt_proved"] is False, "unexpected nested-ADT proof claim")
        result_paths = [generated / "map-size-proof-result.json"]
        for index, row in enumerate(summary["calls"]):
            require(type(row["proof_bytes"]) is int and row["proof_bytes"] > 0 and
                    type(row["transaction_proof_bytes"]) is int and row["transaction_proof_bytes"] > 0 and
                    row["transaction_proof_version"] == 3 and row["transaction_proof_distinct"] is True and
                    row["positive_fee_dust_spends"] == 1 and
                    row["before"] == 1 and row["after"] == row["key"] and
                    row["changed_result_same_geometry"] is True and
                    row["changed_binding_rejected"] is True and row["changed_result_rejected"] is True and
                    row["strict_application"] == "default" and row["replay_refusal"] == "IntentAlreadyExists" and
                    row["replay_unchanged"] is True, "incomplete proof/application controls")
            base = generated / f"map-size-{index}"
            paths = [Path(f"{base}{suffix}") for suffix in (
                ".proof", ".preimage", ".inputs", "-transaction.proof", "-transaction.bin",
                "-state.bin", "-ledger-after.bin", "-ledger-replay.bin")]
            require(all(path.is_file() and path.stat().st_size for path in paths), "Map result artifact missing")
            proof, _, _, transaction_proof, _, _, after, replay = paths
            require(proof.stat().st_size == row["proof_bytes"]
                    and transaction_proof.stat().st_size == row["transaction_proof_bytes"]
                    and digest(proof) != digest(transaction_proof) and digest(after) == digest(replay),
                    "Map retained proof or replay bytes differ from the report")
            result_paths.extend(paths)
        receipt["proof_result"] = summary
        receipt["result_hashes"] = hashes(result_paths)
        verification = output / "verify-standalone.stdout"
        execute([runner, "--map-size-expression-verify", generated], "verify-standalone", verification)
        require(verification.read_text().splitlines() == [
            f"Map size {index}: separate v3 verification passed; corrupt proof refused" for index in range(2)
        ], "Map standalone verification coverage differs")
        unchanged(receipt["result_hashes"])
        unchanged(receipt["source_hashes"])
        unchanged(receipt["dust_fixture_hashes"])
        unchanged(receipt["material_hashes"])
        unchanged(receipt["generated_hashes"])
        for component in components.values():
            unchanged({component["source"]: component["sha256"], component["snapshot"]: component["sha256"]})
        receipt["status"] = "passed"
    except Exception as error:
        receipt["error"] = str(error)
        raise
    finally:
        (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps({"status": receipt["status"], "receipt": str(output / "receipt.json")}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler", type=Path, required=True)
    parser.add_argument("--zkir", type=Path)
    parser.add_argument("--target-dir", type=Path, default=Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")))
    parser.add_argument("--output", type=Path, required=True)
    run(parser.parse_args())
