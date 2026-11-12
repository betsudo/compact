#!/usr/bin/env python3
# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0

"""Fresh source-bound Stateful Vector value proofs and strict funded local applications.

The opt-in runner includes the whole unedited current SDK. Preparation verifies
source/API/binding/parameter premises but generates no keys or proofs. Full CI,
all-export proof coverage and integration acceptance remain separate.
"""
import argparse
import json
import os
from pathlib import Path
import re
import shutil
import shlex
import subprocess
import sys
import tomllib

import check_stateful_vector_value_widening as main_gate
import local_parity_gate as common
import resolve_ledger_test_static as ledger_static
from did_proof_gate import dust_fixture_inventory, hashes, unchanged

ROOT = main_gate.ROOT
CIRCUITS = ["run", "run_field"]
ALL_SELECTED_CIRCUITS = ["run", "run_field"]
CASES = [("run", 0, [0,255], [0,255], 1),
         ("run", 1, [0,255], [255,0], 3),
         ("run_field", 0, [0,255], [0,255], 2)]
require, digest = main_gate.require, main_gate.digest


def validate_summary(summary, generated):
    require(summary["format"] == "compact-stateful-vector-value-proof/v2" and summary["status"] == "passed"
            and summary["constructor_execution_proved"] is False and summary["all_exports_proved"] is False,
            "invalid Stateful Vector value proof summary")
    require([(r["circuit"], r["initial_seen"], r["input"], r["expected"], r["seen"]) for r in summary["calls"]] == CASES,
            "selected proof scenario roster differs")
    require(summary["selected_source_exports_proved"] == CIRCUITS and summary["unproved_selected_source_exports"] == [],
            "selected/unproved selected source export coverage differs")
    for index, row in enumerate(summary["calls"]):
        require(row["result_type"] == ["Vector<2,Uint<16>>", "Vector<2,Uint<16>>", "Vector<2,Field>"][index],
                "genuine result carrier differs")
        require(isinstance(row["expected"], list) and len(row["expected"]) == 2 and all(type(x) is int for x in row["expected"]) and type(row["witness_count"]) is int
                and row["witness_count"] == 0, "witness count or result differs")
        for field, path in [("proof_bytes", f"stateful-vector-value-{index}.proof"),
                            ("transaction_bytes", f"stateful-vector-value-{index}-transaction.bin")]:
            require(type(row[field]) is int and row[field] > 0
                    and (generated / path).stat().st_size == row[field], "missing or truncated proof/application artifact")
        stem = f"stateful-vector-value-{index}"
        require(row["standalone_proof"] == f"{stem}.proof"
                and row["applied_transaction"] == f"stateful-vector-value-{index}-transaction.bin"
                and row["transaction_proof_relationship"] == "independently proved"
                and row["transaction_proof_version"] == 3 and row["positive_fee_dust_spends"] == 1
                and row["strict_application"] == "default" and row["replay_refusal"] == "IntentAlreadyExists",
                "proof/application identity or strictness differs")
        require(all(row[name] is True for name in ["corrupt_proof_rejected", "changed_binding_rejected",
                    "changed_result_rejected", "changed_result_same_geometry", "wrong_verifier_rejected",
                    "standalone_verified_separately", "replay_unchanged"]),
                "proof/application control missing")
        require(all((generated / f"{stem}{suffix}").stat().st_size > 0
                    for suffix in (".preimage", ".inputs", "-state.bin", "-ledger-after.bin")),
                "proof/state evidence missing")
        transaction_proof = (generated / f"{stem}-transaction.proof").read_bytes()
        require(type(row["transaction_proof_bytes"]) is int
                and len(transaction_proof) == row["transaction_proof_bytes"] > 0
                and transaction_proof != (generated / f"{stem}.proof").read_bytes(),
                "transaction-bound proof is missing or equals the standalone proof")
        require((generated / f"{stem}-ledger-after.bin").read_bytes()
                == (generated / f"{stem}-ledger-replay.bin").read_bytes(), "replayed ledger bytes differ")


def run(args):
    output = args.output.absolute()
    require(not output.exists() and not output.is_symlink(), "proof output must be a new destination")
    env = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
    configuration_environment = dict(env)
    configuration = main_gate.reject_compiler_overrides(configuration_environment)
    output.mkdir(parents=True)
    report = {"format": "compact-stateful-vector-value-widening-proof-gate/v2", "status": "incomplete", "commands": [],
              "cargo_configuration_sha256": configuration, "scope": __doc__, "proof_execution": False, "full_integration_acceptance": "OPEN"}
    try:
        # This maintained gate admits the exact clean HEAD/compiler/Rust tools,
        # source/golden/Program identities, consumer behavior and negative controls.
        common.run([sys.executable, str(Path(main_gate.__file__)),
                    "--ts-runtime", str(args.ts_runtime), "--compiler-receipt", str(args.compiler_receipt),
                    "--output", str(output / "protocol")], "source-api-protocol", output, report, env=env)
        admitted = json.loads((output / "protocol/receipt.json").read_text())
        require(admitted["status"] == "passed", "source/API protocol not admitted")
        report["head"] = admitted["head"]
        report["protocol_receipt"] = {"path": str(output / "protocol/receipt.json"), "sha256": digest(output / "protocol/receipt.json")}
        report["rust_tools"] = admitted["rust_tools"]
        report["compiler_tools"] = admitted["tools"]
        unchanged(report["compiler_tools"])
        env.update({key: value for key, value in admitted["environment"].items() if value is not None})
        env["RUSTUP_TOOLCHAIN"] = "1.99.0"
        env["RUSTC"] = admitted["rust_tools"]["rustc"]["path"]
        env["CARGO_INCREMENTAL"] = "0"
        require(bool(env.get("MIDNIGHT_PP")) and Path(env["MIDNIGHT_PP"]).is_absolute(), "absolute MIDNIGHT_PP required")
        generated = output / "protocol/generated/proofs"
        env["COMPACT_STATEFUL_VECTOR_VALUE_GENERATED"] = str(generated)
        report["generated_sdk"] = str(generated)
        report["generated_before"] = before = main_gate.inventory(generated)
        source_paths = [ROOT / "Cargo.toml", ROOT / "Cargo.lock", Path(__file__),
                        Path(main_gate.__file__), Path(common.__file__), Path(ledger_static.__file__),
                        ROOT / "tools/compact-rust-backend/did_proof_gate.py"]
        for directory in ["tools/compact-rust-proof-smoke", "tests-rust-backend", "runtime-rs", "runtime-rs-macros"]:
            source_paths += [p for p in (ROOT / directory).rglob("*") if p.suffix in {".rs", ".toml", ".json"}]
        report["source_hashes"] = hashes(source_paths)
        (output / "bin").mkdir()
        zkir = args.zkir.resolve(strict=True)
        report["zkir"] = common.stable_copy(zkir, output / "bin/zkir")
        components = report["components"] = {}

        def execute(label, command, stdout=None):
            argv = list(map(str, command))
            if argv[0] in admitted["rust_tools"]:
                argv[0] = admitted["rust_tools"][argv[0]]["path"]
            common.run(argv, label, output, report, env=env, stdout_path=stdout)

        # No SDK path is needed by the default build. The opt-in build follows it.
        plain = dict(env)
        plain.pop("COMPACT_STATEFUL_VECTOR_VALUE_GENERATED", None)
        common.run([admitted["rust_tools"]["cargo"]["path"], "check", "--profile", "proof-checks", "--locked", "--offline",
                    "-p", "compact-rust-proof-smoke", "--no-default-features"], "default-runner-check", output, report, env=plain)
        common.run([admitted["rust_tools"]["cargo"]["path"], "clippy", "--profile", "proof-checks", "--locked", "--offline",
                    "-p", "compact-rust-proof-smoke", "--all-targets", "--all-features", "--", "-D", "warnings"],
                   "all-features-runner-clippy-without-sdk", output, report, env=plain)
        execute("build-runner", ["cargo", "build", "--profile", "proof-checks", "--locked", "--offline", "-p",
                                "compact-rust-proof-smoke", "--features", "stateful-vector-value-widening", "--message-format=json"], output / "build.jsonl")
        artifacts = [json.loads(line) for line in (output / "build.jsonl").read_text().splitlines() if line.startswith("{")]
        executable = {row["executable"] for row in artifacts if row.get("reason") == "compiler-artifact"
                      and row["target"]["name"] == "compact-rust-proof-smoke" and row.get("executable")}
        require(len(executable) == 1, "ambiguous proof runner artifact")
        built = Path(executable.pop())
        components["runner"] = common.stable_copy(built, output / "bin/proof-runner")
        runner = components["runner"]["snapshot"]
        depfile = built.with_suffix(".d")
        dependencies = depfile.read_text()
        # Cargo preserves lexical ../ components for relative include_bytes!, and
        # escapes spaces in Make paths. Resolve parsed paths to physical files;
        # retain the raw depfile bytes/hash and exact include membership checks.
        tracked = {str((ROOT / path).resolve()) for path in shlex.split(dependencies.replace("\\\n", ""))}
        for path in [main_gate.FIXTURES / "sources.json", generated / "contract/lib.rs",
                     *sorted((generated / "contract/checked-programs").glob("*.json")),
                     main_gate.FIXTURES / "goldens/proofs/source-ir.json",
                     main_gate.FIXTURES / "ledger9-goldens/proofs/artifacts.json",
                     *sorted((main_gate.FIXTURES / "ledger9-goldens/proofs/checked-programs").glob("*.json"))]:
            require(str(path.resolve(strict=True)) in tracked,
                    "runner omitted an actual generated SDK/Program dependency")
        report["runner_depfile"] = {"path": str(depfile), "sha256": digest(depfile), "all_generated_includes": True}
        execute("runner-clippy", ["cargo", "clippy", "--profile", "proof-checks", "--locked", "--offline", "-p",
                                  "compact-rust-proof-smoke", "--features", "stateful-vector-value-widening", "--bin", "compact-rust-proof-smoke", "--", "-D", "warnings"])
        for label, message in [("changed-ir", "stateful-vector-value complete IR bytes differ"),
                               ("changed-program", "stateful-vector-value complete Program bytes differ"),
                               ("missing-program", "stateful-vector-value complete Program membership differs"),
                               ("changed-zkir", "Ledger 9 artifact differs from the pinned fixture: zkir/run.zkir")]:
            negative = output / label
            shutil.copytree(generated / "contract/checked-programs", negative / "contract/checked-programs")
            shutil.copyfile(generated / "contract/compact-rust-ir.json", negative / "contract/compact-rust-ir.json")
            if label == "changed-ir":
                import copy
                changed = copy.deepcopy(json.loads((main_gate.FIXTURES / "goldens/proofs/source-ir.json").read_text()))
                root = next(c for c in changed["circuits"] if c["name"] == "flip")
                projection = root["body"]["elements"][0]
                require(projection["kind"] == "tuple_index" and projection["index"] == 1,
                        "complete IR helper projection mutation premise changed")
                projection["index"] = 0
                (negative / "contract/compact-rust-ir.json").write_text(json.dumps(changed, indent=2) + "\n")
            elif label == "changed-program":
                path = negative / "contract/checked-programs/program-0.json"
                path.write_bytes(path.read_bytes() + b" ")
            elif label == "missing-program":
                (negative / "contract/checked-programs/program-1.json").unlink()
            else:
                shutil.copyfile(generated / "contract/lib.rs", negative / "contract/lib.rs")
                shutil.copytree(generated / "zkir", negative / "zkir")
                path = negative / "zkir/run.zkir"
                path.write_bytes(path.read_bytes() + b" ")
            result = subprocess.run([runner, "--stateful-vector-value-widening", str(negative)], cwd=ROOT, env=env,
                                    capture_output=True, text=True, timeout=30)
            (negative / "stdout").write_text(result.stdout)
            (negative / "stderr").write_text(result.stderr)
            require(result.returncode != 0 and message in result.stderr, label + " failed outside its binding guard")
            require(not (negative / "keys").exists() and not list(negative.glob("stateful-vector-value-*")), "pre-key refusal produced runtime evidence")
            report[label] = {"exit": result.returncode, "before_keys_or_state": True}
        full_generated = output / "protocol/generated"
        full_before = main_gate.inventory(full_generated)
        execute("all-source-models", [runner, "--stateful-vector-value-widening-inventory", full_generated], output / "all-source-models.json")
        full_models = json.loads((output / "all-source-models.json").read_text())
        definitions = json.loads((main_gate.FIXTURES / "sources.json").read_text())
        require(full_models["format"] == "compact-stateful-vector-value-model-inventory/v1"
                and [(r["source"], r["circuit"]) for r in full_models["circuits"]]
                == [(s["source"], entry) for s in definitions for entry in s["entries"]], "complete model roster differs")
        by_source = {s["source"]:s for s in definitions}
        require(all(r["proof_required"] is by_source[r["source"]]["proof_required"]
                    and (type(r["k"]) is int and r["k"] >= 0 if r["proof_required"] else r["k"] is None)
                    and r["selected_for_proof"] == (r["source"] == "proofs")
                    for r in full_models["circuits"]), "model observation/applicability differs")
        require(main_gate.inventory(full_generated) == full_before, "model observation changed producers")
        report["all_source_models"] = full_models
        report["unproved_exports"] = [r for r in full_models["circuits"] if r["proof_required"] and not r["selected_for_proof"]]
        execute("models", [runner, "--stateful-vector-value-widening-models", generated], output / "models.json")
        models = json.loads((output / "models.json").read_text())
        require(models["format"] == "compact-stateful-vector-value-models/v1" and [r["circuit"] for r in models["circuits"]] == ALL_SELECTED_CIRCUITS,
                "selected model roster differs")
        require(models["selected_for_proof"] == CIRCUITS and models["unproved"] == [], "selected proof scope differs")
        selected_models = [r for r in models["circuits"] if r["circuit"] in CIRCUITS]
        require(len(selected_models) == 2 and all(type(r["k"]) is int and r["k"] == 7 for r in selected_models),
                "selected circuit differs from its bound k7 model")
        report["models"] = models
        # A separate complete retained Fold source supplies a real v3 k9 diagnostic.
        # It never replaces the admitted stateful Vector value IR/Programs or proof inputs.
        diagnostic = output / "over-cap-producer"
        diagnostic.mkdir()
        diagnostic_input = ROOT / "examples/rust_backend/fold_vector_result_widening/helper.compact"
        text = diagnostic_input.read_text()
        report["over_cap_input_source"] = {"path": str(diagnostic_input), "sha256": digest(diagnostic_input)}
        require(text.count("input:Vector<2,Uint<8>>") == 4 and text.count("run_checked") == 1,
                "retained diagnostic source premise changed")
        source = diagnostic / "helper.compact"
        source.write_text(text.replace("input:Vector<2,Uint<8>>", "input:Vector<16,Uint<8>>").replace("run_checked", "run_field"))
        unchanged(report["compiler_tools"])
        compiler = json.loads(args.compiler_receipt.read_text())["compactc"]["path"]
        execute("over-cap-producer", [compiler, "--target", "rust", "--rust-functional-ledger9", "--rust-require-recording",
                "--skip-zk", "--feature-zkir-v3", "--rust-runtime-root", ROOT, source, diagnostic / "generated"])
        report["over_cap_diagnostic_source"] = {"sha256": digest(source),
                "zkir_sha256": digest(diagnostic / "generated/zkir/run_field.zkir"),
                "scope": "complete retained Fold length16 diagnostic only, not source/proof admission"}
        excessive = output / "selected-model-over-cap"
        shutil.copytree(generated, excessive)
        shutil.copyfile(diagnostic / "generated/zkir/run_field.zkir", excessive / "zkir/run_field.zkir")
        refused = subprocess.run([runner, "--stateful-vector-value-widening-models", str(excessive)],
                                 cwd=ROOT, env=env, capture_output=True, text=True, timeout=30)
        (excessive / "stdout").write_text(refused.stdout)
        (excessive / "stderr").write_text(refused.stderr)
        require(refused.returncode and "model exceeds tiny automatic k<=8" in refused.stderr,
                "selected over-cap model did not refuse")
        refused_models = json.loads(refused.stdout)
        require(next(r["k"] for r in refused_models["circuits"] if r["circuit"] == "run_field") > 8
                and not (excessive / "keys").exists(), "over-cap refusal was not before keys")
        report["selected_model_over_cap_refusal"] = {"exit": refused.returncode,
                "before_keys_or_proofs": True, "models": refused_models,
                "scope": "genuine larger compiler-produced ZKIR under selected diagnostic slot; not proof admission"}
        material = output / "proof-material.json"
        execute("verify-proof-material-before-keygen", [runner, "--prepare-proof-material", "--verify-only",
                *[arg for k in sorted({r["k"] for r in selected_models}) for arg in ["--parameter-k", str(k)]]], material)
        verified = json.loads(material.read_text())
        require(verified["mode"] == "verify-only" and verified["verification"] == "all files checked against locked upstream SHA256 tables"
                and Path(verified["cache_directory"]).resolve() == Path(env["MIDNIGHT_PP"]).resolve(), "proof material premise differs")
        require({r["k"] for r in selected_models} <= {r["k"] for r in verified["parameters"]}, "selected SRS not verified before key generation")
        report["material_hashes"] = hashes([Path(verified["cache_directory"]) / r["name"] for r in verified["assets"] + verified["parameters"]])
        execute("ledger-metadata", ["cargo", "metadata", "--locked", "--offline", "--format-version", "1"], output / "metadata.json")
        ledger = ledger_static.resolve(json.loads((output / "metadata.json").read_text()), tomllib.loads((ROOT / "Cargo.lock").read_text()))
        env["MIDNIGHT_LEDGER_TEST_STATIC_DIR"] = ledger["static_directory"]
        report["ledger_static"] = ledger
        report["dust_fixture_hashes"] = dust_fixture_inventory(Path(ledger["static_directory"]))
        if not args.prepare_only:
            (generated / "keys").mkdir()
            report["keygen"] = []
            version = output / "zkir-version.txt"
            execute("zkir-version", [report["zkir"]["snapshot"], "--version"], version)
            require(version.read_text().strip() == "midnight-zkir 3.2.0", "expected the released ZKIR v3.2.0 compiler")
            for row in selected_models:
                circuit = row["circuit"]
                execute("keygen-" + circuit, [report["zkir"]["snapshot"], "compile", generated / f"zkir/{circuit}.zkir",
                                              generated / f"keys/{circuit}.prover", generated / f"keys/{circuit}.verifier"])
                model = re.search(r"\(k=(\d+), rows=(\d+)\)", Path(report["commands"][-1]["log"]).read_text())
                require(model is not None and int(model[1]) == row["k"], "actual key model differs from verified pre-key model")
                report["keygen"].append({"circuit": circuit, "mode": "fresh direct native", "k": row["k"], "rows": int(model[2])})
            report["key_hashes"] = hashes([generated / f"keys/{c}.{ext}" for c in CIRCUITS for ext in ["prover", "verifier"]])
            report["binary_ir_hashes"] = hashes([generated / f"zkir/{c}.bzkir" for c in CIRCUITS])
            execute("prove-verify-apply", [runner, "--stateful-vector-value-widening", generated])
            summary = json.loads((generated / "stateful-vector-value-proof-result.json").read_text())
            validate_summary(summary, generated)
            report["proof_result"] = summary
            report["proof_execution"] = True
            report["result_hashes"] = hashes([p for p in generated.glob("stateful-vector-value-*") if p.is_file()])
            unchanged(report["key_hashes"])
            unchanged(report["binary_ir_hashes"])
        for field in ["source_hashes", "material_hashes", "dust_fixture_hashes", "compiler_tools"]:
            unchanged(report[field])
        unchanged({str(generated / path): sha for path, sha in before.items()})
        for component in [report["zkir"], *components.values()]:
            # Shared Cargo target may be rebuilt elsewhere; only the private snapshot is executed.
            unchanged({component["snapshot"]: component["sha256"]})
        unchanged({str(output / "protocol/receipt.json"): report["protocol_receipt"]["sha256"]})
        require(subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip() == report["head"]
                and not subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT, text=True).strip(), "source changed during proof gate")
        require(main_gate.reject_compiler_overrides(configuration_environment) == configuration,
                "Cargo configuration membership or bytes changed during proof qualification")
        report["status"] = "prepared" if args.prepare_only else "passed"
    except Exception as error:
        report["status"] = "failed"
        report["error"] = repr(error)
        raise
    finally:
        (output / "receipt.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"status": report["status"], "receipt": str(output / "receipt.json")}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ["ts-runtime", "compiler-receipt", "zkir", "output"]:
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--prepare-only", action="store_true", help="compile and verify bindings/material only; no keys/proofs/application")
    run(parser.parse_args())

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
