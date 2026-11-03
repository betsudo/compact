#!/usr/bin/env python3
# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0

"""Fresh source-bound Vector product elements proofs and strict funded local applications.

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

import check_vector_product_elements as main_gate
import local_parity_gate as common
import resolve_ledger_test_static as ledger_static
from did_proof_gate import dust_fixture_inventory, hashes, unchanged

ROOT = main_gate.ROOT
CIRCUITS = ["run", "run_field"]
ALL_SELECTED_CIRCUITS = ["run", "run_field"]
CASES = [("run", 0, [[[3,5],31],[[4,6],41]], [[[3,5],31],[[4,6],41]], 1),
         ("run", 1, [[[3,5],31],[[4,6],41]], [[[4,6],41],[[3,5],31]], 3),
         ("run_field", 0, [[[3,5],31],[[4,6],41]], [[[3,5],31],[[4,6],41]], 2)]
require, digest = main_gate.require, main_gate.digest


def validate_summary(summary, generated):
    require(summary["format"] == "compact-vector-product-elements-proof/v1" and summary["status"] == "passed"
            and summary["constructor_execution_proved"] is False and summary["all_exports_proved"] is False,
            "invalid Vector product elements proof summary")
    require([(r["circuit"], r["initial_seen"], r["input"], r["expected"], r["seen"]) for r in summary["calls"]] == CASES,
            "selected proof scenario roster differs")
    require(summary["selected_source_exports_proved"] == CIRCUITS and summary["unproved_selected_source_exports"] == [],
            "selected/unproved selected source export coverage differs")
    for index, row in enumerate(summary["calls"]):
        require(row["result_type"] == ["Vector<2,[Vector<2,Uint<16>>,Uint<32>]>", "Vector<2,[Vector<2,Uint<16>>,Uint<32>]>", "Vector<2,[Vector<2,Field>,Field]>"][index],
                "genuine result carrier differs")
        require(isinstance(row["expected"], list) and len(row["expected"]) == 2 and all(isinstance(pair, list) and len(pair) == 2 and isinstance(pair[0], list) and len(pair[0]) == 2 and all(type(x) is int for x in pair[0]) and type(pair[1]) is int for pair in row["expected"]) and type(row["witness_count"]) is int
                and row["witness_count"] == 0, "witness count or result differs")
        for field, path in [("proof_bytes", f"vector-product-elements-{index}.proof"),
                            ("transaction_bytes", f"vector-product-elements-{index}-transaction.bin")]:
            require(type(row[field]) is int and row[field] > 0
                    and (generated / path).stat().st_size == row[field], "missing or truncated proof/application artifact")
        require(row["standalone_proof"] == f"vector-product-elements-{index}.proof"
                and row["applied_transaction"] == f"vector-product-elements-{index}-transaction.bin"
                and row["transaction_proof_relationship"] == "independently proved"
                and row["strict_application"] == "default" and row["replay_refusal"] == "IntentAlreadyExists",
                "proof/application identity or strictness differs")
        require(all(row[name] is True for name in ["corrupt_proof_rejected", "changed_binding_rejected",
                    "changed_result_rejected", "changed_result_same_geometry", "wrong_verifier_rejected", "replay_unchanged"]),
                "proof/application control missing")
        require((generated / f"vector-product-elements-{index}.preimage").stat().st_size > 0
                and (generated / f"vector-product-elements-{index}-state.bin").stat().st_size > 0, "proof/state evidence missing")


def run(args):
    output = args.output.absolute()
    require(not output.exists() and not output.is_symlink(), "proof output must be a new destination")
    env = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
    configuration_environment = dict(env)
    configuration = main_gate.reject_compiler_overrides(configuration_environment)
    output.mkdir(parents=True)
    report = {"format": "compact-vector-product-elements-proof-gate/v1", "status": "incomplete", "commands": [],
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
        generated = output / "protocol/generated/proof_candidate"
        env["COMPACT_VECTOR_PRODUCT_ELEMENTS_GENERATED"] = str(generated)
        report["generated_sdk"] = str(generated)
        report["generated_before"] = before = main_gate.inventory(generated)
        tracked_sources = subprocess.check_output(["git", "ls-files", "-z"], cwd=ROOT).split(b"\0")
        source_paths = [ROOT / os.fsdecode(path) for path in tracked_sources if path]
        require(all(p.is_file() for p in source_paths), "tracked source file is absent")
        report["source_hashes"] = hashes(source_paths)
        report["generated_input_hashes"] = hashes([p for p in (output / "protocol/generated").rglob("*") if p.is_file()])
        report["bound_api_inputs"] = admitted["build_inputs_before_use"]
        report["environment"] = {k: env.get(k) for k in ["RUSTUP_TOOLCHAIN", "RUSTC", "CARGO_INCREMENTAL", "CARGO_BUILD_JOBS", "CARGO_TARGET_DIR", "COMPACT_VECTOR_PRODUCT_ELEMENTS_GENERATED", "MIDNIGHT_PP"]}
        # Carry every already-admitted compiler/source/API input through the
        # proof epoch, including original Compact examples, not just includes.
        for relative, sha in admitted["inputs"].items():
            path = str(ROOT / relative)
            require(digest(path) == sha, "admitted source changed before proof build: " + relative)
            require(path not in report["source_hashes"] or report["source_hashes"][path] == sha,
                    "prebuild source authorities disagree: " + relative)
            report["source_hashes"][path] = sha

        (output / "bin").mkdir()
        zkir = args.zkir.resolve(strict=True)
        report["zkir"] = common.stable_copy(zkir, output / "bin/zkir")
        components = report["components"] = {}

        def execute(label, command, stdout=None):
            argv = list(map(str, command))
            if argv[0] in admitted["rust_tools"]:
                argv[0] = admitted["rust_tools"][argv[0]]["path"]
            for field in ["source_hashes", "generated_input_hashes", "bound_api_inputs", "dependency_source_hashes", "compiler_tools"]:
                if field in report:
                    unchanged(report[field])
            require(main_gate.reject_compiler_overrides(configuration_environment) == configuration,
                    "Cargo configuration changed before command")
            common.run(argv, label, output, report, env=env, stdout_path=stdout)

        execute("locked-build-metadata", ["cargo", "metadata", "--locked", "--offline", "--format-version", "1"], output / "build-metadata.json")
        metadata = json.loads((output / "build-metadata.json").read_text())
        root_packages = tomllib.loads((ROOT / "Cargo.lock").read_text())["package"]
        root_pins = {(p["name"], p["version"], p.get("source")): p.get("checksum") for p in root_packages}
        dependency_roots = set()
        report["resolved_packages"] = []
        for package in metadata["packages"]:
            source = package.get("source")
            key = (package["name"], package["version"], source)
            require(key in root_pins, "metadata package differs from root lock")
            manifest = Path(package["manifest_path"]).resolve(strict=True)
            report["resolved_packages"].append({"name": key[0], "version": key[1], "source": source,
                "checksum": root_pins[key], "manifest": str(manifest)})
            if source is not None:
                directory = manifest.parent
                if source.startswith("git+"):
                    while directory != directory.parent and not (directory / ".git").exists():
                        directory = directory.parent
                    require((directory / ".git").exists(), "Git dependency checkout root not found")
                dependency_roots.add(directory)
        dependencies = set()
        for directory in dependency_roots:
            for folder, dirs, files in os.walk(directory, followlinks=False):
                dirs[:] = [d for d in dirs if d not in {".git", "target", "node_modules", ".cache"}]
                for name in files:
                    path = Path(folder) / name
                    if path.is_file():
                        dependencies.add(path.resolve(strict=True))
        report["dependency_source_roots"] = sorted(map(str, dependency_roots))
        report["dependency_source_hashes"] = hashes(dependencies)
        report["metadata_before_build"] = {"path": str(output / "build-metadata.json"), "sha256": digest(output / "build-metadata.json")}
        # No SDK path is needed by the default build. The opt-in build follows it.
        selected_sdk = env.pop("COMPACT_VECTOR_PRODUCT_ELEMENTS_GENERATED")
        execute("default-runner-check", ["cargo", "check", "--profile", "proof-checks", "--locked", "--offline", "-p", "compact-rust-proof-smoke", "--no-default-features"])
        execute("all-features-runner-clippy-without-sdk", ["cargo", "clippy", "--profile", "proof-checks", "--locked", "--offline", "-p", "compact-rust-proof-smoke", "--all-targets", "--all-features", "--", "-D", "warnings"])
        env["COMPACT_VECTOR_PRODUCT_ELEMENTS_GENERATED"] = selected_sdk
        execute("build-runner", ["cargo", "build", "--profile", "proof-checks", "--locked", "--offline", "-p",
                                "compact-rust-proof-smoke", "--features", "vector-product-elements", "--message-format=json"], output / "build.jsonl")
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
                     main_gate.FIXTURES / "goldens/proof_candidate/source-ir.json",
                     *sorted((main_gate.FIXTURES / "goldens/proof_candidate/checked-programs").glob("*.json"))]:
            require(str(path.resolve(strict=True)) in tracked,
                    "runner omitted an actual generated SDK/Program dependency")
        worktree_inputs = {path: report["source_hashes"].get(path) for path in tracked
                           if Path(path).is_file() and Path(path).is_relative_to(ROOT)}
        require(all(sha is not None and digest(path) == sha for path, sha in worktree_inputs.items()),
                "runner worktree dependency was not pinned before build")
        physical_inputs = {str(Path(p).resolve(strict=True)): sha for field in ["source_hashes", "generated_input_hashes", "dependency_source_hashes"] for p, sha in report[field].items()}
        actual_files = {str(Path(p).resolve(strict=True)) for p in tracked if Path(p).is_file()}
        require(actual_files and all(p in physical_inputs and digest(p) == physical_inputs[p] for p in actual_files),
                "actual runner dependency missing from physical prebuild input map")
        report["runner_physical_inputs_before_build"] = {p: physical_inputs[p] for p in sorted(actual_files)}
        retained_depfile = output / "runner.d"
        shutil.copyfile(depfile, retained_depfile)
        report["runner_depfile"] = {"path": str(retained_depfile), "build_path": str(depfile),
                "sha256": digest(retained_depfile), "all_generated_includes": True,
                "worktree_inputs_pinned_before_build": worktree_inputs}
        execute("runner-clippy", ["cargo", "clippy", "--profile", "proof-checks", "--locked", "--offline", "-p",
                                  "compact-rust-proof-smoke", "--features", "vector-product-elements", "--bin", "compact-rust-proof-smoke", "--", "-D", "warnings"])
        for label, message in [("changed-ir", "vector-product-elements complete IR bytes differ"),
                               ("changed-program", "vector-product-elements complete Program bytes differ"),
                               ("missing-program", "vector-product-elements complete Program membership differs")]:
            negative = output / label
            shutil.copytree(generated / "contract/checked-programs", negative / "contract/checked-programs")
            shutil.copyfile(generated / "contract/compact-rust-ir.json", negative / "contract/compact-rust-ir.json")
            if label == "changed-ir":
                import copy
                changed = copy.deepcopy(json.loads((main_gate.FIXTURES / "goldens/proof_candidate/source-ir.json").read_text()))
                root = next(c for c in changed["circuits"] if c["name"] == "flip")
                projection = root["body"]["elements"][0]
                require(projection["kind"] == "tuple_index" and projection["index"] == 1,
                        "complete IR helper projection mutation premise changed")
                projection["index"] = 0
                (negative / "contract/compact-rust-ir.json").write_text(json.dumps(changed, indent=2) + "\n")
            elif label == "changed-program":
                path = negative / "contract/checked-programs/program-0.json"
                path.write_bytes(path.read_bytes() + b" ")
            else:
                (negative / "contract/checked-programs/program-1.json").unlink()
            result = subprocess.run([runner, "--vector-product-elements", str(negative)], cwd=ROOT, env=env,
                                    capture_output=True, text=True, timeout=30)
            (negative / "stdout").write_text(result.stdout)
            (negative / "stderr").write_text(result.stderr)
            require(result.returncode != 0 and message in result.stderr, label + " failed outside its binding guard")
            require(not (negative / "keys").exists() and not list(negative.glob("vector-product-elements-*")), "pre-key refusal produced runtime evidence")
            report[label] = {"exit": result.returncode, "before_keys_or_state": True}
        full_generated = output / "protocol/generated"
        full_before = main_gate.inventory(full_generated)
        execute("all-source-models", [runner, "--vector-product-elements-inventory", full_generated], output / "all-source-models.json")
        full_models = json.loads((output / "all-source-models.json").read_text())
        definitions = json.loads((main_gate.FIXTURES / "sources.json").read_text())
        require(full_models["format"] == "compact-vector-product-elements-model-inventory/v1"
                and [(r["source"], r["circuit"]) for r in full_models["circuits"]]
                == [(s["source"], entry) for s in definitions for entry in s["entries"]], "complete model roster differs")
        by_source = {s["source"]:s for s in definitions}
        require(all(r["proof_required"] is by_source[r["source"]]["stateful"]
                    and (type(r["k"]) is int and r["k"] >= 0 if r["proof_required"] else r["k"] is None)
                    and r["selected_for_proof"] == (r["source"] == "proof_candidate")
                    for r in full_models["circuits"]), "model observation/applicability differs")
        require(main_gate.inventory(full_generated) == full_before, "model observation changed producers")
        report["all_source_models"] = full_models
        report["unproved_exports"] = [r for r in full_models["circuits"] if r["proof_required"] and not r["selected_for_proof"]]
        execute("models", [runner, "--vector-product-elements-models", generated], output / "models.json")
        models = json.loads((output / "models.json").read_text())
        require(models["format"] == "compact-vector-product-elements-models/v1" and [r["circuit"] for r in models["circuits"]] == ALL_SELECTED_CIRCUITS,
                "selected model roster differs")
        require(models["selected_for_proof"] == CIRCUITS and models["unselected"] == [] and models["proof_execution"] is False, "selected proof scope differs")
        selected_models = [r for r in models["circuits"] if r["circuit"] in CIRCUITS]
        require(len(selected_models) == 2 and all(type(r["k"]) is int and 0 <= r["k"] <= 8 for r in selected_models),
                "selected circuit exceeds automatic tiny bound")
        report["models"] = models
        # A separate complete retained Fold source supplies a real over-cap diagnostic.
        # It never replaces the admitted stateful Vector value IR/Programs or proof inputs.
        diagnostic = output / "over-cap-producer"
        diagnostic.mkdir()
        diagnostic_input = ROOT / "examples/rust_backend/fold_vector_result_widening/helper.compact"
        text = diagnostic_input.read_text()
        report["over_cap_input_source"] = {"path": str(diagnostic_input), "sha256": digest(diagnostic_input)}
        require(text.count("input:Vector<2,Uint<8>>") == 4 and text.count("run_checked") == 1,
                "retained diagnostic source premise changed")
        source = diagnostic / "helper.compact"
        source.write_text(text.replace("input:Vector<2,Uint<8>>", "input:Vector<8,Uint<8>>").replace("run_checked", "run_field"))
        unchanged(report["compiler_tools"])
        compiler = json.loads(args.compiler_receipt.read_text())["compactc"]["path"]
        execute("over-cap-producer", [compiler, "--target", "rust", "--rust-functional-ledger8", "--rust-require-recording",
                "--skip-zk", "--rust-runtime-root", ROOT, source, diagnostic / "generated"])
        report["over_cap_diagnostic_source"] = {"sha256": digest(source),
                "zkir_sha256": digest(diagnostic / "generated/zkir/run_field.zkir"),
                "scope": "complete retained Fold length8 diagnostic only, not source/proof admission"}
        excessive = output / "selected-model-over-cap"
        shutil.copytree(generated, excessive)
        shutil.copyfile(diagnostic / "generated/zkir/run_field.zkir", excessive / "zkir/run_field.zkir")
        refused = subprocess.run([runner, "--vector-product-elements-models", str(excessive)],
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
            execute("zkir-version", [report["zkir"]["snapshot"], "--version"])
            for row in selected_models:
                circuit = row["circuit"]
                execute("keygen-" + circuit, [report["zkir"]["snapshot"], "compile", generated / f"zkir/{circuit}.zkir",
                                              generated / f"keys/{circuit}.prover", generated / f"keys/{circuit}.verifier"])
                model = re.search(r"\(k=(\d+), rows=(\d+)\)", Path(report["commands"][-1]["log"]).read_text())
                require(model is not None and int(model[1]) == row["k"], "actual key model differs from verified pre-key model")
                report["keygen"].append({"circuit": circuit, "mode": "fresh direct native", "k": row["k"], "rows": int(model[2])})
            report["key_hashes"] = hashes([generated / f"keys/{c}.{ext}" for c in CIRCUITS for ext in ["prover", "verifier"]])
            report["binary_ir_hashes"] = hashes([generated / f"zkir/{c}.bzkir" for c in CIRCUITS])
            execute("prove-verify-apply", [runner, "--vector-product-elements", generated])
            summary = json.loads((generated / "vector-product-elements-proof-result.json").read_text())
            validate_summary(summary, generated)
            report["proof_result"] = summary
            report["proof_execution"] = True
            report["result_hashes"] = hashes([p for p in generated.glob("vector-product-elements-*") if p.is_file()])
            unchanged(report["key_hashes"])
            unchanged(report["binary_ir_hashes"])
        for field in ["source_hashes", "generated_input_hashes", "bound_api_inputs", "dependency_source_hashes", "material_hashes", "dust_fixture_hashes", "compiler_tools"]:
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
