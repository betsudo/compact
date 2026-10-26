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

"""Fresh direct #661 proofs, independent verification and default-strict application.

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
from check_direct_fold import SOURCES, digest
from check_functional_effects import generated_manifest_snapshot, scheme_compiler
from did_proof_gate import dust_fixture_inventory, hashes, unchanged

ROOT = common.ROOT
CASES = ("single", "empty-ordered", "captured", "multiple-inputs", "effectful-reducer", "witness-reducer", "inline-effectful", "inline-branch")


def require(value, message):
    if not value:
        raise common.GateError(message)


def run(args):
    output = args.output.absolute()
    require(not output.exists() and not output.is_symlink(), "proof destination must be absent")
    output.mkdir(parents=True)
    receipt = {"format": "compact-direct-fold-proof-gate/v1", "status": "incomplete", "commands": [],
               "scope": "nine selected source-bound functional-derived Field call proofs and default-strict funded local transactions",
               "constructor_execution_proved": False, "full_integration_acceptance": "OPEN"}
    env = dict(os.environ, RUSTUP_TOOLCHAIN="1.99.0", CARGO_INCREMENTAL="0", PYTHONDONTWRITEBYTECODE="1")
    try:
        target = args.target_dir.absolute()
        env["CARGO_TARGET_DIR"] = str(target)
        require(bool(env.get("MIDNIGHT_PP")) and Path(env["MIDNIGHT_PP"]).is_absolute(), "MIDNIGHT_PP required")
        paths = [*(SOURCES / f"{name}.compact" for name in CASES), ROOT / "Cargo.toml", ROOT / "Cargo.lock", Path(__file__),
                 ROOT / "tools/compact-rust-backend/check_direct_fold.py",
                 ROOT / "tools/compact-rust-backend/check_functional_effects.py",
                 Path(common.__file__), Path(ledger_static.__file__),
                 ROOT / "tools/compact-rust-backend/did_proof_gate.py",
                 ROOT / "tools/compact-rust-backend/Cargo.toml"]
        for directory in ["runtime-rs", "runtime-rs-macros", "tools/compact-rust-backend/src",
                          "tools/compact-rust-proof-smoke", "tests-rust-backend"]:
            paths += [p for p in (ROOT / directory).rglob("*") if p.suffix in {".rs", ".toml", ".json"}]
        paths += list((ROOT / "compiler").glob("*.ss"))
        receipt["source_hashes"] = hashes(paths)
        receipt["source_head"] = common.git_head()
        receipt["source_dirty_paths"] = common.dirty_paths()
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
        receipt["generated"] = {}
        receipt["generated_hashes"] = {}
        receipt["changed_ir_refusals"] = {}
        for name in CASES:
            source = SOURCES / f"{name}.compact"
            destination = generated / name
            fixture = ROOT / f"tests-rust-backend/direct-fold-{name}-source"
            execute([components["compactc"]["snapshot"], "--target", "rust", "--rust-functional-ledger8",
                     "--rust-require-recording", "--rust-runtime-root", ROOT, "--skip-zk",
                     source.relative_to(ROOT), destination], f"generate-{name}")
            snapshot = receipt["generated"][name] = generated_manifest_snapshot(destination)
            formatted = output / f"formatted-{name}.rs"
            shutil.copyfile(destination / "contract/lib.rs", formatted)
            execute(["rustfmt", "--edition", "2024", formatted], f"format-{name}")
            require(formatted.read_bytes() == (fixture / "lib.rs").read_bytes(), f"{name} linked full Rust differs")
            require((destination / "contract/compact-rust-ir.json").read_bytes() == (fixture / "original-ir.json").read_bytes(), f"{name} complete linked IR differs")
            for asset in ("compact-rust-origins.raw.json", "rust-capabilities.json"):
                require((destination / "contract" / asset).read_bytes() == (fixture / asset).read_bytes(), f"{name} linked {asset} differs")
            fresh = {p.name: p.read_bytes() for p in (destination / "contract/checked-programs").glob("*.json")}
            retained = {p.name: p.read_bytes() for p in (fixture / "checked-programs").glob("*.json")}
            require(len(fresh) == 1 and fresh == retained, f"{name} complete linked programs differ")
            receipt["generated_hashes"].update(hashes([destination / path for path in snapshot["assets"]] +
                                                       [destination / "compiler/contract-manifest.json", formatted]))
            negative = output / f"changed-ir-{name}"
            (negative / "contract").mkdir(parents=True)
            changed = json.loads((destination / "contract/compact-rust-ir.json").read_text())
            changed["contractName"] = f"changed-direct-fold-{name}-proof-source"
            (negative / "contract/compact-rust-ir.json").write_text(json.dumps(changed))
            refused = subprocess.run([runner, "--direct-fold", name, str(negative)], cwd=ROOT,
                                     env=env, capture_output=True, text=True, timeout=30)
            (output / f"changed-ir-{name}.stdout").write_text(refused.stdout)
            (output / f"changed-ir-{name}.stderr").write_text(refused.stderr)
            require(refused.returncode != 0 and "Proof artifact IR differs from the generated source fixture" in refused.stderr,
                    f"{name} changed full IR did not refuse before absent keys/state")
            require(not (negative / "keys").exists() and not list(negative.glob("fold-*")),
                    f"{name} changed IR refusal created proof/state outputs")
            receipt["changed_ir_refusals"][name] = {"exit_code":refused.returncode,"before_keys_or_state":True}
        material = output / "proof-material.json"
        execute([runner, "--prepare-proof-material", "--verify-only"], "verify-builtins", material)
        prepared = json.loads(material.read_text())
        require(prepared["mode"] == "verify-only" and prepared["verification"] == "all files checked against locked upstream SHA256 tables", "built-in verification receipt differs")
        require(Path(prepared["cache_directory"]).resolve() == Path(env["MIDNIGHT_PP"]).resolve(), "unexpected parameter directory")
        execute([components["zkir"]["snapshot"], "--version"], "zkir-version")
        receipt["keygen"] = {}
        receipt["material_hashes"] = {}
        receipt["proof_results"] = {}
        receipt["result_hashes"] = {}
        selected_material = {}
        for name in CASES:
            destination = generated / name
            (destination / "keys").mkdir()
            execute([components["zkir"]["snapshot"], "compile", destination / "zkir/run.zkir",
                     destination / "keys/run.prover", destination / "keys/run.verifier"], f"keygen-{name}")
            model = re.search(r"\(k=(\d+), rows=(\d+)\)", Path(receipt["commands"][-1]["log"]).read_text())
            require(model is not None, f"{name} keygen omitted actual circuit model")
            k, rows = map(int, model.groups())
            receipt["keygen"][name] = {"mode":"fresh direct native","circuit":"run","k":k,"rows":rows}
            if k not in selected_material:
                # The SDK verifies locked tables and only fetches a missing SRS.
                # Hash verification then runs separately before this proof lane.
                execute([runner, "--prepare-proof-material", "--parameter-k", k],
                        f"prepare-selected-{k}", output / f"prepared-material-{k}.json")
                execute([runner, "--prepare-proof-material", "--verify-only", "--parameter-k", k],
                        f"verify-selected-{k}", output / f"selected-material-{k}.json")
                selected = json.loads((output / f"selected-material-{k}.json").read_text())
                require(selected["mode"] == "verify-only" and any(row["k"] == k for row in selected["parameters"]), "selected SRS not verified")
                require(Path(selected["cache_directory"]).resolve() == Path(env["MIDNIGHT_PP"]).resolve(), "unexpected selected parameter directory")
                selected_material[k] = [Path(selected["cache_directory"]) / row["name"] for row in selected["assets"] + selected["parameters"]]
            artifacts = [destination / f"keys/run.{ext}" for ext in ("prover","verifier")]
            artifacts += [destination / f"zkir/run.{ext}" for ext in ("zkir","bzkir")]
            material_paths = selected_material[k] + artifacts
            require(all(p.is_file() and p.stat().st_size for p in material_paths), f"{name} proof material missing")
            receipt["material_hashes"].update(hashes(material_paths))
            execute([runner, "--direct-fold", name, destination], f"prove-verify-apply-{name}")
            summary = json.loads((destination / "fold-proof-result.json").read_text())
            require(summary["format"] == "compact-direct-fold-proof/v1" and summary["status"] == "passed"
                    and summary["case"] == name and summary["constructor_execution_proved"] is False
                    and summary["unselected_sources_proved"] is False, f"{name} invalid proof summary")
            expected = [123,134] if name == "inline-branch" else [24] if name == "multiple-inputs" else [12] if name == "single" else [9] if name == "empty-ordered" else [123]
            require([row["expected"] for row in summary["calls"]] == expected, f"{name} selected calls differ")
            for row in summary["calls"]:
                require(type(row["proof_bytes"]) is int and row["proof_bytes"] > 0 and
                        row["private_state"] == (9 if name in {"witness-reducer", "empty-ordered"} else 7) and
                        row["witness_calls"] == (2 if name in {"witness-reducer", "empty-ordered"} else 0) and
                        row["changed_result_same_geometry"] is True and row["changed_binding_rejected"] is True and
                        row["changed_result_rejected"] is True and row["strict_application"] == "default" and
                        row["replay_refusal"] == "IntentAlreadyExists" and row["replay_unchanged"] is True,
                        f"{name} incomplete proof/application controls")
            receipt["proof_results"][name] = summary
            receipt["result_hashes"].update(hashes([p for p in destination.glob("fold-*") if p.is_file()]))
        unchanged(receipt["source_hashes"])
        unchanged(receipt["dust_fixture_hashes"])
        unchanged(receipt["material_hashes"])
        unchanged(receipt["generated_hashes"])
        for component in components.values():
            unchanged({component["source"]: component["sha256"], component["snapshot"]: component["sha256"]})
        require(common.git_head() == receipt["source_head"] and common.dirty_paths() == receipt["source_dirty_paths"],
                "repository source state changed during proof qualification")
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
