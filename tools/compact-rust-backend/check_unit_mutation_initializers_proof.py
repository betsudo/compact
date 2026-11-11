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

"""Fresh direct #516 proofs, independent verification and default-strict application.

The complete required native/CI suites remain separate. Constructor execution
is not proved. No proof, verification verdict or transaction is reused.
"""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tomllib

import local_parity_gate as common
import resolve_ledger_test_static as ledger_static
from check_unit_mutation_initializers import SOURCE, digest, require_clean_source
from check_functional_effects import generated_manifest_snapshot, scheme_compiler
from did_proof_gate import dust_fixture_inventory, hashes, unchanged
from wrapper_v3_proof import prepare_material, verify_artifacts

ROOT = common.ROOT
FIXTURE = ROOT / "tests-rust-backend/unit-mutation-initializers-source"
PROOF_RESULT_FORMAT = "compact-unit-mutation-initializers-proof/v3"
PROOF_GATE_FORMAT = "compact-unit-mutation-initializers-proof-gate/v3"

GEOMETRY = {
    "conditional": (7, 61),
    "nested": (7, 87),
    "observed": (7, 65),
    "sequential": (7, 105),
    "shadowed": (7, 85),
    "statement_field": (7, 61),
    "statement_observed": (7, 82),
    "statement_shadowed": (8, 124),
    "statement_unit": (7, 60),
    "trailing_conditional": (7, 77),
    "trailing_unconditional": (7, 75),
    "unit_result": (7, 60),
    "unused": (7, 62),
}


def require(value, message):
    if not value:
        raise common.GateError(message)


def validate_proof_summary(summary):
    """Require the complete finite native producer protocol, never a pass flag alone."""
    require(summary["format"] == PROOF_RESULT_FORMAT and summary["status"] == "passed"
            and summary["constructor_execution_proved"] is False
            and summary["prestate_setup_proved"] is False and summary["all_exports_proved"] is False, "invalid Unit initializer proof summary")
    require([(row["circuit"], row["key"]) for row in summary["calls"]] ==
            [("conditional", 5), ("conditional", 6), ("statement_field", 5), ("statement_field", 6)],
            "missing return/statement replacement/new insertion proof")
    require(summary["unselected_branch_proved"] is False, "unexpected unselected branch proof claim")
    for index,row in enumerate(summary["calls"]):
        require(all(type(row[name]) is int for name in ("key", "before", "after", "proof_bytes", "transaction_bytes")) and
                type(row.get("transaction_proof_bytes")) is int and row["transaction_proof_bytes"] > 0 and
                type(row.get("transaction_proof_version")) is int and row["transaction_proof_version"] == 3 and
                type(row.get("positive_fee_dust_spends")) is int and row["positive_fee_dust_spends"] == 1 and
                row["proof_bytes"] > 0 and row["transaction_bytes"] > 0 and
                row["standalone_proof"] == f"unit-initializers-{index}.proof" and
                row["applied_transaction"] == f"unit-initializers-{index}-transaction.bin" and
                row["transaction_proof_relationship"] == "independently proved" and
                row["before"] == 1 and row["after"] == (1 if row["key"] == 5 else 2) and
                row["changed_result_same_geometry"] is True and
                row["changed_binding_rejected"] is True and row["changed_result_rejected"] is True and
                row["strict_application"] == "default" and row["replay_refusal"] == "IntentAlreadyExists" and
                row["replay_unchanged"] is True, "incomplete proof/application controls")


def run(args):
    output = args.output.absolute()
    require(not output.exists() and not output.is_symlink(), "proof destination must be absent")
    require("MIDNIGHT_PROOF_SERVER" not in os.environ, "wrapper proof requires local proving")
    output.mkdir(parents=True)
    receipt = {"format": PROOF_GATE_FORMAT, "status": "incomplete", "commands": [],
               "scope": "four source-bound return/statement call proofs and independently proved default-strict funded local transactions",
               "constructor_execution_proved": False, "full_integration_acceptance": "OPEN"}
    env = dict(os.environ, RUSTUP_TOOLCHAIN="1.99.0", CARGO_INCREMENTAL="0", PYTHONDONTWRITEBYTECODE="1")
    try:
        require_clean_source(receipt)
        target = args.target_dir.absolute()
        env["CARGO_TARGET_DIR"] = str(target)
        require(bool(env.get("MIDNIGHT_PP")) and Path(env["MIDNIGHT_PP"]).is_absolute(), "MIDNIGHT_PP required")
        paths = [SOURCE, ROOT / "Cargo.toml", ROOT / "Cargo.lock", Path(__file__),
                 ROOT / "tools/compact-rust-backend/check_unit_mutation_initializers.py",
                 ROOT / "tools/compact-rust-backend/check_functional_effects.py",
                 Path(common.__file__), Path(ledger_static.__file__),
                 ROOT / "tools/compact-rust-backend/did_proof_gate.py", Path(__file__).with_name("wrapper_v3_proof.py"),
                 ROOT / "tools/compact-rust-backend/Cargo.toml"]
        for directory in ["runtime-rs", "runtime-rs-macros", "tools/compact-rust-backend/src",
                          "tools/compact-rust-proof-smoke", "tests-rust-backend"]:
            paths += [p for p in (ROOT / directory).rglob("*") if p.suffix in {".rs", ".toml", ".json"}]
        paths += sorted((ROOT / "compiler").glob("*.ss"))
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
        require(len(fresh) == 15 and fresh == retained, "complete linked functional programs differ")
        receipt["formatted_rust_sha256"] = digest(formatted)
        receipt["generated_hashes"] = hashes([generated / path for path in receipt["generated"]["assets"]] +
                                             [generated / "compiler/contract-manifest.json", formatted])
        negative = output / "changed-ir"
        (negative / "contract").mkdir(parents=True)
        changed = json.loads((generated / "contract/compact-rust-ir.json").read_text())
        changed["contractName"] = "changed-unit-initializers-proof-source"
        (negative / "contract/compact-rust-ir.json").write_text(json.dumps(changed))
        refused = subprocess.run([runner, "--unit-mutation-initializers", str(negative)], cwd=ROOT,
                                 env=env, capture_output=True, text=True, timeout=30)
        (output / "changed-ir.stdout").write_text(refused.stdout)
        (output / "changed-ir.stderr").write_text(refused.stderr)
        require(refused.returncode != 0 and "Proof artifact IR differs from the generated source fixture" in refused.stderr,
                "changed full IR did not refuse before absent keys/state")
        require(not (negative / "keys").exists() and not list(negative.glob("unit-initializers-*")),
                "changed IR refusal created proof/state outputs")
        receipt["changed_ir_refusal"] = {"exit_code": refused.returncode, "before_keys_or_state": True}
        program_negative = output / "changed-program"
        (program_negative / "contract").mkdir(parents=True)
        shutil.copyfile(generated / "contract/compact-rust-ir.json", program_negative / "contract/compact-rust-ir.json")
        shutil.copytree(generated / "contract/checked-programs", program_negative / "contract/checked-programs")
        bad_program = program_negative / "contract/checked-programs/program-0.json"
        bad_program.write_bytes(bad_program.read_bytes() + b" ")
        refused = subprocess.run([runner, "--unit-mutation-initializers", str(program_negative)], cwd=ROOT,
                                 env=env, capture_output=True, text=True, timeout=30)
        (output / "changed-program.stdout").write_text(refused.stdout)
        (output / "changed-program.stderr").write_text(refused.stderr)
        require(refused.returncode != 0 and "Unit initializer checked-program bytes differ" in refused.stderr,
                "changed checked program did not refuse before absent keys/state")
        require(not (program_negative / "keys").exists() and not list(program_negative.glob("unit-initializers-*")),
                "changed program refusal created proof/state outputs")
        receipt["changed_program_refusal"] = {"exit_code": refused.returncode, "before_keys_or_state": True}
        receipt["material_hashes"] = prepare_material(
            execute, receipt, generated, output, components["zkir"]["snapshot"], runner,
            env["MIDNIGHT_PP"], GEOMETRY, ('conditional', 'statement_field'),
        )
        execute([runner, "--unit-mutation-initializers", generated], "prove-verify-apply")
        summary = json.loads((generated / "unit-initializers-proof-result.json").read_text())
        validate_proof_summary(summary)
        receipt["proof_result"] = summary
        verify_artifacts(execute, receipt, generated, runner, "--unit-mutation-initializers-verify",
                         "unit-initializers", summary["calls"])
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
