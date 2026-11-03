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

"""Fresh source-bound Nested component products proofs and strict funded local applications.

The opt-in runner compiles one complete unedited13export SDK at their own
crate roots. Preparation verifies
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

import check_nested_component_products as main_gate
import local_parity_gate as common
import resolve_ledger_test_static as ledger_static
from did_proof_gate import dust_fixture_inventory, hashes, unchanged

ROOT = main_gate.ROOT
SELECTED = [("family", name) for name in ["helper", "witness_before", "witness_after", "field_identity"]]
SELECTED_SOURCES = ["family"]
require = main_gate.require
from check_passport_current_profile import digest, bounded_command, cargo_target
import time

def inventory(root):
    return {str(p.relative_to(root)): digest(p) for p in root.rglob("*") if p.is_file()}

def run_logged(command, label, directory, receipt, *, env=None, stdout_path=None):
    log = directory / "logs" / f"{len(receipt['commands']):03d}-{label}.log"
    log.parent.mkdir(parents=True, exist_ok=True)
    began = time.monotonic()
    try:
        result = bounded_command(command, env)
    except subprocess.TimeoutExpired as error:
        log.write_text((error.stderr or "") if isinstance(error.stderr, str) else (error.stderr or b"").decode(errors="replace"))
        if stdout_path:
            stdout_path.write_bytes(error.stdout if isinstance(error.stdout, bytes) else (error.stdout or "").encode())
        receipt["commands"].append(dict(label=label, argv=command, timeout=True, seconds=time.monotonic()-began, log=str(log), log_sha256=digest(log)))
        raise
    log.write_text(result.stderr if stdout_path else result.stdout + result.stderr)
    row=dict(label=label, argv=command, exit_code=result.returncode, seconds=round(time.monotonic()-began,3), log=str(log), log_sha256=digest(log))
    if stdout_path:
        stdout_path.write_text(result.stdout)
        row["stdout"]={"path":str(stdout_path),"sha256":digest(stdout_path)}
    receipt["commands"].append(row)
    if result.returncode:
        raise ValueError(f"{label} failed ({result.returncode}): {log}")



def depfile_prerequisites(raw, base=ROOT):
    logical = raw.replace("\\\n", "")
    first_rule = next(line for line in logical.splitlines() if line.strip())
    _, separator, prerequisites = first_rule.partition(": ")
    require(separator and prerequisites, "invalid actual runner dependency rule")
    paths = {str((base / path).resolve(strict=True)) for path in shlex.split(prerequisites)}
    require(paths and all(Path(path).is_file() for path in paths), "non-file runner prerequisite")
    return paths


def qualify_dependency_inputs(prerequisites, inventories):
    """Qualify every physical prerequisite by prebound identity, not location.

    A generated SDK or OUT_DIR may live below the checkout (including target).
    No path, generated directory or unknown prerequisite is exempted.
    """
    require(bool(prerequisites), "empty actual runner dependency set")
    qualified, origins = {}, {}
    for path in sorted(prerequisites):
        bindings = {name: files[path] for name, files in inventories.items() if path in files}
        require(bindings and len(set(bindings.values())) == 1,
                "actual runner dependency missing or inconsistent in physical prebuild input map: " + path)
        sha = next(iter(bindings.values()))
        require(Path(path).is_file() and digest(path) == sha,
                "actual runner dependency changed after physical prebuild binding: " + path)
        qualified[path] = sha
        origins[path] = sorted(bindings)
    return qualified, origins


def validate_summary(summary, generated):
    require(summary["format"] == "compact-nested-component-products-proof/v1" and summary["status"] == "passed"
            and summary["constructor_execution_proved"] is False and summary["all_exports_proved"] is False
            and summary["all_source_exports_deployed"] is False, "invalid proof scope")
    require(summary["selected_source_exports_proved"] == list(map(list, SELECTED))
            and summary["unproved_selected_source_exports"] == [], "selected export coverage differs")
    require([(r["source"], r["circuit"]) for r in summary["calls"]] == SELECTED, "scenario roster differs")
    for index, row in enumerate(summary["calls"]):
        root=generated / "family"
        witnessed=index in {1,2}
        require(row["initial_seen"]==0 and row["seen"]==[2,5,6,14][index]
                and row["private_state"]==(8 if witnessed else 7)
                and row["witness_count"]==(1 if witnessed else 0)
                and row["witness_seen"]==({1:0,2:6}.get(index)), "selected typed/witness state differs")
        expected=("Vector<1,[Vector<1,[[Field,Uint<16>],Uint<16>]>,Uint<32>]>" if index==3
                  else "Vector<1,[Vector<1,[[Uint<16>,Uint<32>],Uint<16>]>,Uint<32>]>")
        require(row["result_type"]==expected, "selected complete result carrier differs")
        for field,suffix in [("proof_bytes",".proof"),("transaction_bytes","-transaction.bin")]:
            require(type(row[field]) is int and row[field]>0 and (root / f"nested-component-products-{index}{suffix}").stat().st_size==row[field], "artifact missing/truncated")
        require(row["standalone_proof"]==f"family/nested-component-products-{index}.proof"
                and row["applied_transaction"]==f"family/nested-component-products-{index}-transaction.bin"
                and row["transaction_proof_relationship"]=="independently proved"
                and row["strict_application"]=="default" and row["replay_refusal"]=="IntentAlreadyExists", "proof/application identity differs")
        require(all(row[k] is True for k in ["corrupt_proof_rejected","changed_binding_rejected","changed_result_rejected","changed_result_same_geometry","changed_preimage_rejected","wrong_verifier_rejected","replay_unchanged"]), "verification control missing")
        require(all((root / f"nested-component-products-{index}{suffix}").stat().st_size>0 for suffix in [".preimage","-state.bin"]), "preimage/state missing")


def run(args):
    requested_output = args.output.absolute()
    require(not requested_output.exists() and not requested_output.is_symlink(), "proof output must be a new destination")
    output = requested_output.resolve(strict=False)
    env = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
    inherited_sdk_selectors = sorted(key for key, value in env.items()
                                     if key.startswith("COMPACT_") and key.endswith("_GENERATED") and value)
    require(not inherited_sdk_selectors, "unset inherited generated-SDK selectors: " + ", ".join(inherited_sdk_selectors))
    configuration_environment = dict(env)
    configuration = main_gate.reject_compiler_overrides(configuration_environment)
    output.mkdir(parents=True)
    report = {"format": "compact-nested-component-products-proof-gate/v1", "status": "incomplete", "commands": [],
              "cargo_configuration_sha256": configuration, "scope": __doc__, "proof_execution": False, "full_integration_acceptance": "OPEN"}
    try:
        # This maintained gate admits the exact clean HEAD/compiler/Rust tools,
        # source/golden/Program identities, consumer behavior and negative controls.
        run_logged([sys.executable, str(Path(main_gate.__file__)),
                    "--ts-runtime", str(args.ts_runtime), "--compiler-receipt", str(args.compiler_receipt),
                    "--output", str(output / "protocol")], "source-api-protocol", output, report, env=env)
        admitted = json.loads((output / "protocol/receipt.json").read_text())
        require(admitted["status"] == "passed", "source/API protocol not admitted")
        report["head"] = admitted["head"]
        report["protocol_receipt"] = {"path": str(output / "protocol/receipt.json"), "sha256": digest(output / "protocol/receipt.json")}
        report["rust_tools"] = rust_tools = {Path(path).name: {"path":path,"sha256":sha} for path,sha in admitted["tools"].items() if Path(path).name in ["cargo","rustc","clippy-driver","cargo-clippy"]}
        require(set(rust_tools)=={"cargo","rustc","clippy-driver","cargo-clippy"}, "actual Rust tool roster differs")
        report["compiler_tools"] = admitted["tools"]
        unchanged(report["compiler_tools"])
        env.setdefault("CARGO_BUILD_JOBS", "2")
        env["CARGO_TARGET_DIR"] = str(cargo_target(env, output))
        env["PATH"] = str(Path(rust_tools["cargo"]["path"]).parent) + os.pathsep + env["PATH"]
        env.pop("LD_LIBRARY_PATH", None)
        env["RUSTUP_TOOLCHAIN"] = "1.99.0"
        env["RUSTC"] = rust_tools["rustc"]["path"]
        env["CARGO_INCREMENTAL"] = "0"
        require(bool(env.get("MIDNIGHT_PP")) and Path(env["MIDNIGHT_PP"]).is_absolute(), "absolute MIDNIGHT_PP required")
        generated = output / "protocol/generated"
        env["COMPACT_NESTED_COMPONENT_PRODUCTS_GENERATED"] = str(generated)
        report["generated_sdk"] = str(generated)
        report["selected_exports"] = list(map(list, SELECTED))
        report["generated_before"] = before = inventory(generated)
        tracked_sources = subprocess.check_output(["git", "ls-files", "-z"], cwd=ROOT).split(b"\0")
        source_paths = [ROOT / os.fsdecode(path) for path in tracked_sources if path]
        require(all(p.is_file() for p in source_paths), "tracked source file is absent")
        report["source_hashes"] = hashes(source_paths)
        report["generated_input_hashes"] = hashes([p for p in (output / "protocol/generated").rglob("*") if p.is_file()])
        report["bound_api_inputs"] = admitted["bound_build_files"]
        report["environment"] = {k: env.get(k) for k in ["RUSTUP_TOOLCHAIN", "RUSTC", "CARGO_INCREMENTAL", "CARGO_BUILD_JOBS", "CARGO_TARGET_DIR", "COMPACT_NESTED_COMPONENT_PRODUCTS_GENERATED", "MIDNIGHT_PP"]}
        # Carry every already-admitted compiler/source/API input through the
        # proof epoch, including original Compact examples, not just includes.
        for relative, sha in admitted["inputs"].items():
            path = str(Path(relative).resolve(strict=True))
            require(digest(path) == sha, "admitted source changed before proof build: " + relative)
            require(path not in report["source_hashes"] or report["source_hashes"][path] == sha,
                    "prebuild source authorities disagree: " + relative)
            report["source_hashes"][path] = sha

        (output / "bin").mkdir()
        zkir = args.zkir.resolve(strict=True)
        report["zkir"] = common.stable_copy(zkir, output / "bin/zkir")
        components = report["components"] = {}

        def check_premises():
            for field in ["source_hashes", "generated_input_hashes", "bound_api_inputs", "dependency_source_hashes", "generated_build_inputs_before_compile", "compiler_tools", "material_hashes", "dust_fixture_hashes", "key_hashes", "binary_ir_hashes"]:
                if field in report:
                    unchanged(report[field])
            for component in [report["zkir"], *components.values()]:
                unchanged({component["snapshot"]: component["sha256"]})
            require(main_gate.reject_compiler_overrides(configuration_environment) == configuration,
                    "Cargo configuration changed before command")

        def execute(label, command, stdout=None):
            argv = list(map(str, command))
            if argv[0] in rust_tools:
                argv[0] = rust_tools[argv[0]]["path"]
            check_premises()
            report["current"] = label
            (output / "receipt.json").write_text(json.dumps(report, indent=2)+"\n")
            try:
                run_logged(argv, label, output, report, env=env, stdout_path=stdout)
            finally:
                if report["commands"]:
                    command = report["commands"][-1]
                    command["raw_files_after_command"] = hashes([Path(command["log"]), *([Path(command["stdout"]["path"])] if command.get("stdout") else [])])
                (output / "receipt.json").write_text(json.dumps(report, indent=2)+"\n")

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
                dirs[:] = [d for d in dirs if d != ".git"]
                for name in files:
                    path = Path(folder) / name
                    if path.is_file():
                        dependencies.add(path.resolve(strict=True))
        report["dependency_source_roots"] = sorted(map(str, dependency_roots))
        report["dependency_source_hashes"] = hashes(dependencies)
        report["metadata_before_build"] = {"path": str(output / "build-metadata.json"), "sha256": digest(output / "build-metadata.json")}
        # No SDK path is needed by the default build. The opt-in build follows it.
        selected_sdk = env.pop("COMPACT_NESTED_COMPONENT_PRODUCTS_GENERATED")
        execute("default-runner-check", ["cargo", "check", "--profile", "proof-checks", "--locked", "--offline", "-p", "compact-rust-proof-smoke", "--no-default-features"])
        execute("all-features-runner-clippy-without-sdk", ["cargo", "clippy", "--profile", "proof-checks", "--locked", "--offline", "-p", "compact-rust-proof-smoke", "--all-targets", "--all-features", "--", "-D", "warnings"])
        env["COMPACT_NESTED_COMPONENT_PRODUCTS_GENERATED"] = selected_sdk
        # Cargo fixture build scripts produce real include! inputs outside the
        # tracked tree. Prepare and pin those physical files before qualifying
        # the runner, rather than accepting newly discovered bytes afterward.
        execute("prepare-runner-inputs", ["cargo", "check", "--profile", "proof-checks", "--locked", "--offline", "-p",
                "compact-rust-proof-smoke", "--features", "nested-component-products", "--message-format=json"], output / "prepare-build.jsonl")
        package_metadata = {package["id"]: package for package in metadata["packages"]}
        source_map = {str(Path(p).resolve(strict=True)): sha for field in ["source_hashes", "dependency_source_hashes", "generated_input_hashes"] for p, sha in report[field].items()}
        events = [json.loads(line) for line in (output / "prepare-build.jsonl").read_text().splitlines() if line.startswith("{")]
        prepared_dirs, generated_inputs, owners = set(), set(), []
        target_directory = Path(metadata["target_directory"]).resolve(strict=True)
        for event in events:
            if event.get("reason") != "build-script-executed":
                continue
            require(event["package_id"] in package_metadata, "unknown generated-input package")
            package = package_metadata[event["package_id"]]
            scripts = [Path(t["src_path"]).resolve(strict=True) for t in package["targets"] if "custom-build" in t["kind"]]
            require(len(scripts) == 1 and str(scripts[0]) in source_map and digest(scripts[0]) == source_map[str(scripts[0])],
                    "generated-input build owner was not bound before preparation")
            directory = Path(event["out_dir"]).resolve(strict=True)
            require(directory.is_relative_to(target_directory), "generated-input directory outside actual Cargo target")
            prepared_dirs.add(directory)
            files = [p.resolve(strict=True) for p in directory.rglob("*") if p.is_file()]
            generated_inputs.update(files)
            owner = {"package_id": event["package_id"], "source": str(scripts[0]),
                "source_sha256": source_map[str(scripts[0])], "out_dir": str(directory), "files": sorted(map(str, files))}
            # Local fixture generation declares its tracked source inputs via
            # Cargo rerun lines; authenticate those against the pre-preparation map.
            script_log = directory.parent / "output"
            require(script_log.is_file(), "missing actual build-script output log")
            generated_inputs.add(script_log.resolve(strict=True))
            owner.update(output_log=str(script_log), output_log_sha256=digest(script_log))
            if package.get("source") is None:
                manifest = Path(package["manifest_path"]).resolve(strict=True)
                require(str(manifest) in source_map and digest(manifest) == source_map[str(manifest)],
                        "local build-script manifest was not bound")
                rerun_files = set()
                for line in script_log.read_text().splitlines():
                    if line.startswith(("cargo:rerun-if-changed=", "cargo::rerun-if-changed=")):
                        path = (manifest.parent / line.split("=", 1)[1]).resolve(strict=True)
                        rerun_files.update(p.resolve(strict=True) for p in path.rglob("*") if p.is_file()) if path.is_dir() else rerun_files.add(path)
                require(all(str(p) in source_map and digest(p) == source_map[str(p)] for p in rerun_files),
                        "local generated-input source absent from pre-preparation map")
                owner["rerun_inputs_before_preparation"] = {str(p): source_map[str(p)] for p in sorted(rerun_files)}
            owners.append(owner)
        require(owners and generated_inputs, "no actual build-generated inputs prepared")
        report["prepared_build_scripts"] = owners
        report["prepared_generated_directories"] = sorted(map(str, prepared_dirs))
        report["generated_build_inputs_before_compile"] = hashes(generated_inputs)
        report["preparation_scope"] = "Explicit preparatory Cargo check creates/caches build-script inputs; physical paths/bytes and bound source owners pinned before qualifying build. Not a proof or atomic/global supply-chain attestation."
        execute("build-runner", ["cargo", "build", "--profile", "proof-checks", "--locked", "--offline", "-p",
                                "compact-rust-proof-smoke", "--features", "nested-component-products", "--message-format=json"], output / "build.jsonl")
        artifacts = [json.loads(line) for line in (output / "build.jsonl").read_text().splitlines() if line.startswith("{")]
        executable = {row["executable"] for row in artifacts if row.get("reason") == "compiler-artifact"
                      and row["target"]["name"] == "compact-rust-proof-smoke" and row.get("executable")}
        require(len(executable) == 1, "ambiguous proof runner artifact")
        built = Path(executable.pop())
        components["runner"] = common.stable_copy(built, output / "bin/proof-runner")
        runner = components["runner"]["snapshot"]
        depfile = built.with_suffix(".d")
        retained_depfile = output / "runner.d"
        shutil.copyfile(depfile, retained_depfile)
        report["runner_depfile"] = {"path": str(retained_depfile), "build_path": str(depfile),
                "sha256": digest(retained_depfile), "membership_status": "pending"}
        dependencies = retained_depfile.read_text()
        # Cargo preserves lexical ../ components for relative include_bytes!, and
        # escapes spaces in Make paths. Resolve parsed paths to physical files;
        # retain the raw depfile bytes/hash and exact include membership checks.
        tracked = depfile_prerequisites(dependencies)
        missing = output / "dependency-that-must-be-absent.rs"
        require(not missing.exists(), "missing-dependency control path already exists")
        logical = dependencies.replace("\\\n", "")
        first_rule = next(line for line in logical.splitlines() if line.strip())
        poisoned = logical.replace(first_rule, first_rule + " " + str(missing).replace("\\", "\\\\").replace(" ", "\\ "), 1)
        poisoned_path = output / "missing-dependency.d"
        poisoned_path.write_text(poisoned)
        try:
            depfile_prerequisites(poisoned)
        except FileNotFoundError as error:
            require(error.filename == str(missing.resolve(strict=False)), "wrong missing-prerequisite locus")
        else:
            raise ValueError("missing-prerequisite control accepted")
        report["missing_dependency_control"] = {"path": str(poisoned_path), "sha256": digest(poisoned_path),
                "missing_path": str(missing), "before_keys_or_models": True}
        report["sdk_artifacts"] = {}
        required = [main_gate.FIXTURES / "sources.json"]
        for source in SELECTED_SOURCES:
            target_name = "compact_rust_nested_component_products_sdk"
            matches = [r for r in artifacts if r.get("reason") == "compiler-artifact" and r["target"]["name"] == target_name]
            require(len(matches) == 1, "ambiguous complete SDK artifact: " + source)
            row = matches[0]
            manifest = Path(package_metadata[row["package_id"]]["manifest_path"]).resolve(strict=True)
            package_root = ROOT / "tools/compact-rust-proof-smoke/nested-component-products-sdk"
            require(manifest == package_root / "Cargo.toml" and Path(row["target"]["src_path"]).resolve(strict=True) == package_root / "lib.rs", "SDK not compiled at its OWN crate root")
            libs = [Path(p) for p in row["filenames"] if p.endswith(".rlib")]
            require(len(libs) == 1 and libs[0].is_file(), "missing SDK rlib data artifact")
            lib = libs[0]
            snapshot = output / f"bin/{source}-sdk.rlib"
            require(not snapshot.exists(), "SDK data snapshot must be fresh")
            sha = digest(lib)
            shutil.copyfile(lib, snapshot)
            require(digest(snapshot) == sha == digest(lib), "SDK library changed during snapshot")
            snapshot.chmod(0o444)
            component = {"source":str(lib),"snapshot":str(snapshot),"sha256":sha,"size":snapshot.stat().st_size,"kind":"rust-rlib-data"}
            components[source + "_sdk_rlib"] = component
            raw_depfile = lib.parent / (lib.stem.removeprefix("lib") + ".d")
            retained = output / f"{source}-sdk.d"
            shutil.copyfile(raw_depfile, retained)
            sdk_tracked = depfile_prerequisites(retained.read_text(), ROOT)
            sdk_required = [package_root / "lib.rs", generated / source / "contract/lib.rs",
                            *sorted((generated / source / "contract/checked-programs").glob("*.json"))]
            require(len(list((generated / source / "contract/checked-programs").glob("*.json"))) == 13, "SDK complete Program count differs")
            require(all(str(p.resolve(strict=True)) in sdk_tracked for p in sdk_required), "SDK omitted complete raw includes")
            report["sdk_artifacts"][source] = {"package_id":row["package_id"],"manifest":str(manifest),"crate_root":row["target"]["src_path"],
                "artifact":component,"depfile":str(retained),"depfile_sha256":digest(retained),"prerequisites":sorted(sdk_tracked)}
            tracked.update(sdk_tracked)
            required.extend([*sdk_required, main_gate.FIXTURES / f"proof-goldens/{source}/source-ir.json",
                             *sorted((main_gate.FIXTURES / f"proof-goldens/{source}/checked-programs").glob("*.json"))])
        require(all(str(path.resolve(strict=True)) in tracked for path in required), "runner omitted actual paired SDK/Program dependency")
        input_maps = {field: report[field] for field in ["source_hashes", "generated_input_hashes",
                "dependency_source_hashes", "generated_build_inputs_before_compile"]}
        physical_inputs, input_origins = qualify_dependency_inputs(tracked, input_maps)
        report["runner_physical_inputs_before_build"] = physical_inputs
        report["runner_physical_input_origins"] = input_origins
        declared_sources = {p: physical_inputs[p] for p in physical_inputs if p in report["source_hashes"]}
        report["runner_depfile"].update(membership_status="passed", all_generated_includes=True,
                declared_source_inputs_pinned_before_build=declared_sources)
        execute("runner-clippy", ["cargo", "clippy", "--profile", "proof-checks", "--locked", "--offline", "-p",
                                  "compact-rust-proof-smoke", "--features", "nested-component-products", "--bin", "compact-rust-proof-smoke", "--", "-D", "warnings"])
        report["raw_binding_refusals"] = []
        for kind, indices, message in [
            ("changed-ir", [None], "complete IR bytes differ"),
            ("changed-program", range(13), "complete Program bytes differ"),
            ("missing-program", [0,12], "complete Program membership differs"),
        ]:
            for index in indices:
                label=kind+("" if index is None else "-"+str(index))
                negative=output / label
                folder=negative / "family/contract"
                shutil.copytree(generated / "family/contract/checked-programs", folder / "checked-programs")
                shutil.copyfile(generated / "family/contract/compact-rust-ir.json", folder / "compact-rust-ir.json")
                if kind=="changed-ir":
                    path=folder / "compact-rust-ir.json"
                    original = path.read_bytes()
                    changed = original + b" "
                    require(len(changed) == len(original) + 1, "raw IR byte mutation differs")
                    json.loads(changed)
                    path.write_bytes(changed)
                else:
                    path=folder / f"checked-programs/program-{index}.json"
                    if kind=="missing-program":
                        path.unlink() # Only this task-owned private malformed copy.
                    else:
                        path.write_bytes(path.read_bytes()+b" ")
                control_inputs=hashes([p for p in negative.rglob("*") if p.is_file()])
                check_premises()
                result=bounded_command([runner,"--nested-component-products",str(negative)],env)
                check_premises()
                (negative / "stdout").write_text(result.stdout);(negative / "stderr").write_text(result.stderr)
                require(result.returncode!=0 and not result.stdout and "nested-component-products family "+message in result.stderr, "wrong raw identity refusal: "+label)
                require(not (negative / "family/keys").exists() and not list(negative.rglob("*.proof")), "pre-key refusal produced proofs/keys")
                unchanged(control_inputs)
                report["raw_binding_refusals"].append(dict(label=label,exit=result.returncode,input_hashes_before=control_inputs,stderr=result.stderr,before_keys_or_state=True,scope="whole private raw IR/Program byte or membership control; not new compiler output"))
        full_generated = output / "protocol/generated"
        full_before = inventory(full_generated)
        execute("all-source-models", [runner, "--nested-component-products-inventory", full_generated], output / "all-source-models.json")
        full_models = json.loads((output / "all-source-models.json").read_text())
        definitions = json.loads((main_gate.FIXTURES / "sources.json").read_text())
        require(full_models["format"] == "compact-nested-component-products-model-inventory/v1"
                and [(r["source"], r["circuit"]) for r in full_models["circuits"]]
                == [(s["source"], entry) for s in definitions for entry in s["entries"]], "complete model roster differs")
        by_source = {s["source"]:s for s in definitions}
        require(all(r["proof_required"] is by_source[r["source"]]["stateful"]
                    and (type(r["k"]) is int and r["k"] >= 0 if r["proof_required"] else r["k"] is None)
                    and r["selected_for_proof"] == ((r["source"], r["circuit"]) in SELECTED)
                    for r in full_models["circuits"]), "model observation/applicability differs")
        expected_models = json.loads((main_gate.FIXTURES / "model-expectations.json").read_text())
        require(expected_models["format"] == "compact-nested-component-products-model-expectations/v1"
                and [{k: r[k] for k in ["source", "circuit", "k", "proof_required"]} for r in full_models["circuits"]] == expected_models["circuits"],
                "current full model inventory drifted; inspect source/profile and extend eligible proof coverage")
        require(inventory(full_generated) == full_before, "model observation changed producers")
        report["all_source_models"] = full_models
        report["unselected_required_exports"] = [r for r in full_models["circuits"] if r["proof_required"] and not r["selected_for_proof"]]
        report["unproved_exports"] = [r for r in full_models["circuits"] if r["proof_required"]]
        execute("models", [runner, "--nested-component-products-models", generated], output / "models.json")
        models = json.loads((output / "models.json").read_text())
        require(models["format"] == "compact-nested-component-products-models/v1"
                and [(r["source"],r["circuit"]) for r in models["circuits"]] == SELECTED
                and models["proof_execution"] is False, "selected model roster differs")
        require(all(next(x["k"] for x in full_models["circuits"] if (x["source"],x["circuit"]) == (r["source"],r["circuit"])) == r["k"]
                and r["selected_for_proof"] is True for r in models["circuits"]), "selected/full models disagree")
        selected_models = models["circuits"]
        require(len(selected_models) == 4 and all(type(r["k"]) is int and 0 <= r["k"] <= 8 for r in selected_models), "selected model exceeds tiny bound")
        report["models"] = models
        # Private diagnostic: an existing intact over-limit model exercises the
        # selected-model guard, without changing original source or proof inputs.
        excessive = output / "selected-model-over-cap"
        for source in SELECTED_SOURCES:
            shutil.copytree(generated / source, excessive / source)
        donor = generated / "original/zkir/run.zkir"
        require(next(r["k"] for r in full_models["circuits"] if r["source"] == "original") > 8, "over-cap control premise changed")
        shutil.copyfile(donor, excessive / "family/zkir/helper.zkir")
        control_inputs = hashes([p for p in excessive.rglob("*") if p.is_file()])
        check_premises()
        refused = bounded_command([runner,"--nested-component-products-models",str(excessive)], env)
        check_premises()
        (excessive / "stdout").write_text(refused.stdout); (excessive / "stderr").write_text(refused.stderr)
        require(refused.returncode != 0 and "model exceeds tiny automatic k<=8" in refused.stderr, "over-cap guard did not refuse")
        refused_models = json.loads(refused.stdout)
        require(refused_models["circuits"][0]["k"] > 8 and refused_models["circuits"][1:] == selected_models[1:]
                and not list(excessive.rglob("*.proof")) and not any((excessive / n / "keys").exists() for n in SELECTED_SOURCES), "over-cap refusal scope differs")
        unchanged(control_inputs)
        report["selected_model_over_cap_refusal"] = {"exit":refused.returncode,"models":refused_models,"input_hashes_before":control_inputs,
            "donor_sha256":digest(donor),"before_keys_or_proofs":True,"scope":"private intact existing original k9 ZKIR diagnostic, not admitted selected SDK semantics"}
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
            for source in SELECTED_SOURCES:
                (generated / source / "keys").mkdir()
            report["keygen"] = []
            report["key_hashes"] = {}
            report["binary_ir_hashes"] = {}
            execute("zkir-version", [report["zkir"]["snapshot"], "--version"])
            for row in selected_models:
                circuit = row["circuit"]
                source_root = generated / row["source"]
                execute("keygen-" + row["source"] + "-" + circuit, [report["zkir"]["snapshot"], "compile", source_root / f"zkir/{circuit}.zkir",
                                              source_root / f"keys/{circuit}.prover", source_root / f"keys/{circuit}.verifier"])
                model = re.search(r"\(k=(\d+), rows=(\d+)\)", Path(report["commands"][-1]["log"]).read_text())
                require(model is not None and int(model[1]) == row["k"], "actual key model differs from verified pre-key model")
                report["key_hashes"].update(hashes([source_root / f"keys/{circuit}.{ext}" for ext in ["prover", "verifier"]]))
                report["binary_ir_hashes"].update(hashes([source_root / f"zkir/{circuit}.bzkir"]))
                report["keygen"].append({"source": row["source"], "circuit": circuit, "mode": "fresh direct native", "k": row["k"], "rows": int(model[2])})
            require(all(not (generated / name / "keys").exists() for name in by_source if name not in SELECTED_SOURCES), "unselected source acquired keys")
            require({p.name for p in (generated / "family/keys").iterdir()} == {f"{c}.{ext}" for _,c in SELECTED for ext in ["prover","verifier"]}, "selected family key membership differs")
            report["key_hashes"] = hashes([generated / source / f"keys/{c}.{ext}" for source, c in SELECTED for ext in ["prover", "verifier"]])
            report["binary_ir_hashes"] = hashes([generated / source / f"zkir/{c}.bzkir" for source, c in SELECTED])
            execute("prove-verify-apply", [runner, "--nested-component-products", generated])
            summary = json.loads((generated / "nested-component-products-proof-result.json").read_text())
            validate_summary(summary, generated)
            report["proof_result"] = summary
            report["proof_execution"] = True
            report["unproved_exports"] = report["unselected_required_exports"]
            report["result_hashes"] = hashes([p for p in generated.rglob("nested-component-products-*") if p.is_file()])
            unchanged(report["key_hashes"])
            unchanged(report["binary_ir_hashes"])
        for field in ["source_hashes", "generated_input_hashes", "bound_api_inputs", "dependency_source_hashes", "generated_build_inputs_before_compile", "material_hashes", "dust_fixture_hashes", "compiler_tools"]:
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
