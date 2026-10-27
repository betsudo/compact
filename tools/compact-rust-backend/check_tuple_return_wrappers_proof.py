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
import re
import shutil
import subprocess
import tomllib

import local_parity_gate as common
import resolve_ledger_test_static as ledger_static
from check_tuple_return_wrappers import SOURCE, digest
from check_functional_effects import generated_manifest_snapshot, scheme_compiler
from did_proof_gate import dust_fixture_inventory, hashes, unchanged

ROOT = common.ROOT
FIXTURE = ROOT / "tests-rust-backend/tuple-return-wrappers-source"
PROOF_RESULT_FORMAT = "compact-tuple-return-wrappers-proof/v2"


def require(value, message):
    if not value:
        raise common.GateError(message)


def validate_proof_summary(summary):
    """Require the complete finite native producer protocol, never a pass flag alone."""
    require(summary["format"] == PROOF_RESULT_FORMAT and summary["status"] == "passed"
            and summary["constructor_execution_proved"] is False, "invalid Tuple wrapper proof summary")
    require([(row["circuit"], row["key"]) for row in summary["calls"]] ==
            [("inferred", 5), ("inferred", 6), ("mixed", 6), ("folded", 0)],
            "missing/reordered inferred, mixed or Fold proof")
    require(summary["inferred_false_branch_proved"] is False and
            summary["mixed_false_branch_proved"] is False, "unexpected false-branch proof claim")
    for row in summary["calls"]:
        require(all(type(row[name]) is int for name in ("key", "before", "after", "proof_bytes")) and
                row["proof_bytes"] > 0 and
                row["before"] == 1 and row["after"] == (1 if row["key"] == 5 else 2) and
                row["output_type"] == ("uint8_field" if row["circuit"] == "mixed" else "field_field") and
                row["changed_result_components_rejected"] == [0, 1] and
                all(type(i) is int for i in row["changed_result_components_rejected"]) and
                row["changed_result_same_geometry"] is True and
                row["changed_binding_rejected"] is True and row["changed_result_rejected"] is True and
                row["strict_application"] == "default" and row["replay_refusal"] == "IntentAlreadyExists" and
                row["replay_unchanged"] is True, "incomplete proof/application controls")
        require(row["fold_input"] == [1, 0] and all(type(i) is int for i in row["fold_input"])
                if row["circuit"] == "folded" else row["fold_input"] is None,
                "invalid selected Fold input")


def run(args):
    output = args.output.absolute()
    require(not output.exists() and not output.is_symlink(), "proof destination must be absent")
    output.mkdir(parents=True)
    receipt = {"format": "compact-tuple-return-wrappers-proof-gate/v2", "status": "incomplete", "commands": [],
               "scope": "four source-bound inferred/mixed/Fold functional-derived call proofs and default-strict funded local transactions",
               "constructor_execution_proved": False, "full_integration_acceptance": "OPEN"}
    env = dict(os.environ, RUSTUP_TOOLCHAIN="1.99.0", CARGO_INCREMENTAL="0", PYTHONDONTWRITEBYTECODE="1")
    try:
        target = args.target_dir.absolute()
        env["CARGO_TARGET_DIR"] = str(target)
        require(bool(env.get("MIDNIGHT_PP")) and Path(env["MIDNIGHT_PP"]).is_absolute(), "MIDNIGHT_PP required")
        paths = [SOURCE, ROOT / "Cargo.toml", ROOT / "Cargo.lock", Path(__file__),
                 ROOT / "tools/compact-rust-backend/check_tuple_return_wrappers.py",
                 ROOT / "tools/compact-rust-backend/check_functional_effects.py",
                 Path(common.__file__), Path(ledger_static.__file__),
                 ROOT / "tools/compact-rust-backend/did_proof_gate.py",
                 ROOT / "tools/compact-rust-backend/Cargo.toml"]
        for directory in ["runtime-rs", "runtime-rs-macros", "tools/compact-rust-backend/src",
                          "tools/compact-rust-proof-smoke", "tests-rust-backend"]:
            paths += [p for p in (ROOT / directory).rglob("*") if p.suffix in {".rs", ".toml", ".json"}]
        paths += sorted((ROOT / "compiler").glob("*.ss"))
        receipt["source_head"] = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
        receipt["source_dirty"] = bool(subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT, text=True))
        receipt["source_hashes"] = hashes(paths)
        compiler = args.compiler.resolve(strict=True)
        scheme = scheme_compiler(compiler, env)
        selected_zkir = args.zkir or shutil.which("zkir", path=env.get("PATH"))
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
        execute([components["compactc"]["snapshot"], "--target", "rust", "--rust-functional-ledger8",
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
        changed["contractName"] = "changed-tuple-wrappers-proof-source"
        (negative / "contract/compact-rust-ir.json").write_text(json.dumps(changed))
        refused = subprocess.run([runner, "--tuple-return-wrappers", str(negative)], cwd=ROOT,
                                 env=env, capture_output=True, text=True, timeout=30)
        (output / "changed-ir.stdout").write_text(refused.stdout)
        (output / "changed-ir.stderr").write_text(refused.stderr)
        require(refused.returncode != 0 and "Proof artifact IR differs from the generated source fixture" in refused.stderr,
                "changed full IR did not refuse before absent keys/state")
        require(not (negative / "keys").exists() and not list(negative.glob("tuple-wrappers-*")),
                "changed IR refusal created proof/state outputs")
        receipt["changed_ir_refusal"] = {"exit_code": refused.returncode, "before_keys_or_state": True}
        material = output / "proof-material.json"
        execute([runner, "--prepare-proof-material", "--verify-only"], "verify-builtins", material)
        prepared = json.loads(material.read_text())
        require(prepared["mode"] == "verify-only" and prepared["verification"] == "all files checked against locked upstream SHA256 tables", "built-in verification receipt differs")
        require(Path(prepared["cache_directory"]).resolve() == Path(env["MIDNIGHT_PP"]).resolve(), "unexpected parameter directory")
        (generated / "keys").mkdir()
        execute([components["zkir"]["snapshot"], "--version"], "zkir-version")
        receipt["keygen"] = []
        artifacts = []
        for circuit in ("inferred", "mixed", "folded"):
            execute([components["zkir"]["snapshot"], "compile", generated / f"zkir/{circuit}.zkir",
                     generated / f"keys/{circuit}.prover", generated / f"keys/{circuit}.verifier"], f"keygen-{circuit}")
            model = re.search(r"\(k=(\d+), rows=(\d+)\)", Path(receipt["commands"][-1]["log"]).read_text())
            require(model is not None, "keygen omitted actual circuit model")
            k, rows = map(int, model.groups())
            receipt["keygen"].append({"mode": "fresh direct native", "circuit": circuit, "k": k, "rows": rows})
            artifacts += [generated / f"keys/{circuit}.{ext}" for ext in ("prover", "verifier")]
            artifacts += [generated / f"zkir/{circuit}.{ext}" for ext in ("zkir", "bzkir")]
        material_paths = []
        for k in sorted({row["k"] for row in receipt["keygen"]}):
            selected_path = output / f"selected-material-{k}.json"
            execute([runner, "--prepare-proof-material", "--verify-only", "--parameter-k", k],
                    f"verify-selected-parameters-{k}", selected_path)
            selected = json.loads(selected_path.read_text())
            require(selected["mode"] == "verify-only" and any(row["k"] == k for row in selected["parameters"]),
                    "selected SRS was not verified")
            material_paths += [Path(selected["cache_directory"]) / row["name"]
                               for row in selected["assets"] + selected["parameters"]]
        require(all(p.is_file() and p.stat().st_size for p in material_paths + artifacts), "proof material missing")
        receipt["material_hashes"] = hashes(material_paths + artifacts)
        execute([runner, "--tuple-return-wrappers", generated], "prove-verify-apply")
        summary = json.loads((generated / "tuple-wrappers-proof-result.json").read_text())
        validate_proof_summary(summary)
        receipt["proof_result"] = summary
        receipt["result_hashes"] = hashes([p for p in generated.glob("tuple-wrappers-*") if p.is_file()])
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
