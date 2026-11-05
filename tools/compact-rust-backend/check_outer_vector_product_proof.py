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

"""Fresh whole-source component-product Vector proofs and strict funded local applications.

The opt-in runner compiles two complete unedited15/1export SDKs at their own
crate roots. Preparation verifies
source/API/binding/parameter premises but generates no keys or proofs. Full CI,
all-export proof coverage and integration acceptance remain separate.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import shlex
import subprocess
import sys
import tomllib

import check_outer_vector_product_semantics as main_gate
import local_parity_gate as common
import resolve_ledger_test_static as ledger_static
from did_proof_gate import dust_fixture_inventory, hashes, unchanged

ROOT = main_gate.ROOT
SELECTED = [("original", "run")] + [("family", name) for name in [
    "parameter", "helper", "local", "checked_helper", "witness_before", "witness_after", "deep", "boolean_field",
    "empty", "tuple_empty", "tuple_single", "field_identity", "empty_leaf", "last_child"]]
SELECTED_SOURCES = ["family", "original"]
SDK_PACKAGES = {
    "family": ("outer-vector-product-family-sdk", "compact_rust_outer_vector_product_family_sdk", 15),
    "original": ("outer-vector-product-original-sdk", "compact_rust_outer_vector_product_original_sdk", 1),
}


def expected_refusal_roster():
    """Complete exact binding/member roster, followed by the actual over-cap guard."""
    labels = []
    for source, (_, _, count) in SDK_PACKAGES.items():
        labels.append(source + "-changed-ir")
        labels.extend(source + "-changed-program-" + str(index) for index in range(count))
        labels.extend(source + "-missing-program-" + str(index) for index in sorted({0, count - 1}))
    return labels + ["selected-model-over-cap"]


def validate_pre_key_refusals(report, runner_sha256):
    expected = expected_refusal_roster()
    actual = report.get("refusal_invocations", [])
    require(len(expected) == len(set(expected))
            and [row["label"] for row in actual] == expected,
            "complete pre-key refusal roster differs")
    require([row["label"] for row in report.get("raw_binding_refusals", [])] == expected[:-1],
            "raw binding refusal membership differs")
    require(all(row["runner_sha256"] == runner_sha256
                and row["ended_monotonic"] >= row["started_monotonic"]
                for row in actual), "pre-key refusal runner or timing differs")
    return expected

require = main_gate.require
from check_passport_current_profile import digest, bounded_command, cargo_target
import time

def inventory(root):
    return {str(p.relative_to(root)): digest(p) for p in root.rglob("*") if p.is_file()}

def captured(command, directory, label, receipt, env, stdout_path=None):
    directory.mkdir(parents=True, exist_ok=True)
    prefix = directory / f"{len(receipt.get('commands', [])) + len(receipt.get('refusal_invocations', [])):03d}-{label}"
    stdout = stdout_path if stdout_path is not None else prefix.with_suffix(".stdout")
    stderr = prefix.with_suffix(".stderr")
    log = prefix.with_suffix(".log")
    row = dict(label=label, argv=list(map(str, command)), started_monotonic=time.monotonic(), started_unix_ns=time.time_ns())
    try:
        result = main_gate.capture_command(row["argv"], env, ROOT, stdout, stderr)
        row.update(exit_code=result.returncode, exit=result.returncode)
        # Text is parsed only after immutable raw stdout/stderr have been retained.
        return subprocess.CompletedProcess(row["argv"], result.returncode,
                                           result.stdout.decode("utf-8"), result.stderr.decode("utf-8")), row
    except subprocess.TimeoutExpired:
        row["timeout"] = True
        raise
    finally:
        row.update(ended_monotonic=time.monotonic(), seconds=time.monotonic()-row["started_monotonic"])
        for name, path in [("stdout",stdout),("stderr",stderr)]:
            if path.is_file(): row[name] = dict(path=str(path), sha256=digest(path))
        log.write_bytes((stdout.read_bytes() if stdout.exists() else b"") + (stderr.read_bytes() if stderr.exists() else b""))
        row.update(log=str(log),log_sha256=digest(log))
        receipt.setdefault("captured_commands", []).append(row)

def run_logged(command, label, directory, receipt, *, env=None, stdout_path=None):
    result, row = captured(command, directory / "logs", label, receipt, env, stdout_path)
    receipt["commands"].append(row)
    require(result.returncode == 0, f"{label} failed ({result.returncode}): {row['log']}")

def run_refusal(command, label, receipt, env):
    result,row = captured(command, Path(receipt["output"]) / "refusal-logs", label, receipt, env)
    row["runner_sha256"] = digest(command[0])
    receipt.setdefault("refusal_invocations", []).append(row)
    return result, row

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
    require(summary["format"] == "compact-outer-vector-product-proof/v1" and summary["status"] == "passed"
            and summary["constructor_execution_proved"] is False and summary["all_exports_proved"] is False
            and summary["all_source_exports_deployed"] is False, "invalid proof scope")
    require(summary["selected_source_exports_proved"] == list(map(list, SELECTED))
            and summary["unproved_selected_source_exports"] == [], "selected export coverage differs")
    require([(r["source"], r["circuit"]) for r in summary["calls"]] == SELECTED, "scenario roster differs")
    seen = [1,1,2,3,4,5,6,9,10,11,12,13,14,15,16]
    for index, row in enumerate(summary["calls"]):
        source, name = SELECTED[index]
        root = generated / source
        witnessed = name in {"witness_before", "witness_after"}
        require(row["initial_seen"] == 0 and row["seen"] == seen[index]
                and row["private_state"] == (8 if witnessed else 7)
                and row["witness_count"] == (1 if witnessed else 0)
                and row["witness_seen"] == (0 if name == "witness_before" else 6 if name == "witness_after" else None), "typed/witness state differs")
        ir = json.loads((root / "contract/compact-rust-ir.json").read_text())
        result_type = next(c["result"] for c in ir["stateful_circuits"] if c["name"] == name)
        require(row["result_type"] == result_type, "complete original result carrier differs")
        for field,suffix in [("proof_bytes",".proof"),("transaction_bytes","-transaction.bin")]:
            require(type(row[field]) is int and row[field] > 0 and (root / f"outer-vector-product-{index}{suffix}").stat().st_size == row[field], "artifact missing/truncated")
        require(row["standalone_proof"] == f"{source}/outer-vector-product-{index}.proof"
                and row["applied_transaction"] == f"{source}/outer-vector-product-{index}-transaction.bin"
                and row["transaction_proof_relationship"] == "independently proved"
                and row["strict_application"] == "default" and row["replay_refusal"] == "IntentAlreadyExists", "proof/application identity differs")
        require(all(row[k] is True for k in ["corrupt_proof_rejected","changed_binding_rejected","changed_cell_write_preimage_rejected","wrong_verifier_rejected","replay_unchanged"]), "verification control missing")
        if name == "empty":
            require(result_type["kind"] == "vector" and result_type["length"] == 0
                    and row["changed_preimage_rejected"] is None
                    and row["changed_result_rejected"] is None and row["changed_result_same_geometry"] is None
                    and row["result_negative_applicability"] == "inapplicable_unique_empty_carrier", "unique-empty result control fabricated")
        else:
            require(row["changed_preimage_rejected"] is True
                    and row["changed_result_rejected"] is True and row["changed_result_same_geometry"] is True
                    and row["result_negative_applicability"] == "applicable_nonempty_result", "same-geometry result refusal missing")
        require(all((root / f"outer-vector-product-{index}{suffix}").stat().st_size > 0 for suffix in [".preimage","-state.bin"]), "preimage/state missing")


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
    report = {"format": "compact-outer-vector-product-proof-gate/v1", "status": "incomplete", "commands": [],
              "cargo_configuration_sha256": configuration, "scope": __doc__, "proof_execution": False, "full_integration_acceptance": "OPEN", "output": str(output)}
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
        report["rust_tools"] = rust_tools = {Path(path).name: {"path":path,"sha256":sha} for path,sha in admitted["inputs"].items() if Path(path).name in ["cargo","rustc","clippy-driver","cargo-clippy"]}
        require(set(rust_tools)=={"cargo","rustc","clippy-driver","cargo-clippy"}, "actual Rust tool roster differs")
        report["compiler_tools"] = admitted["inputs"]
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
        env["COMPACT_OUTER_VECTOR_PRODUCT_GENERATED"] = str(generated)
        report["generated_sdk"] = str(generated)
        report["selected_exports"] = list(map(list, SELECTED))
        report["generated_before"] = before = inventory(generated)
        tracked_sources = subprocess.check_output(["git", "ls-files", "-z"], cwd=ROOT).split(b"\0")
        source_paths = [ROOT / os.fsdecode(path) for path in tracked_sources if path]
        require(all(p.is_file() for p in source_paths), "tracked source file is absent")
        report["source_hashes"] = hashes(source_paths)
        report["generated_input_hashes"] = hashes([p for p in (output / "protocol/generated").rglob("*") if p.is_file()])
        report["bound_api_inputs"] = admitted["bound_build_inputs"]
        report["bound_api_inputs"][str(output / "protocol/ts-results.json")] = digest(output / "protocol/ts-results.json")
        report["environment"] = {k: env.get(k) for k in ["RUSTUP_TOOLCHAIN", "RUSTC", "CARGO_INCREMENTAL", "CARGO_BUILD_JOBS", "CARGO_TARGET_DIR", "COMPACT_OUTER_VECTOR_PRODUCT_GENERATED", "MIDNIGHT_PP"]}
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
        selected_sdk = env.pop("COMPACT_OUTER_VECTOR_PRODUCT_GENERATED")
        execute("default-runner-check", ["cargo", "check", "--profile", "proof-checks", "--locked", "--offline", "-p", "compact-rust-proof-smoke", "--no-default-features"])
        execute("all-features-runner-clippy-without-sdk", ["cargo", "clippy", "--profile", "proof-checks", "--locked", "--offline", "-p", "compact-rust-proof-smoke", "--all-targets", "--all-features", "--", "-D", "warnings"])
        env["COMPACT_OUTER_VECTOR_PRODUCT_GENERATED"] = selected_sdk
        # Cargo fixture build scripts produce real include! inputs outside the
        # tracked tree. Prepare and pin those physical files before qualifying
        # the runner, rather than accepting newly discovered bytes afterward.
        execute("prepare-runner-inputs", ["cargo", "check", "--profile", "proof-checks", "--locked", "--offline", "-p",
                "compact-rust-proof-smoke", "--features", "outer-vector-product-proofs", "--message-format=json"], output / "prepare-build.jsonl")
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
                                "compact-rust-proof-smoke", "--features", "outer-vector-product-proofs", "--message-format=json"], output / "build.jsonl")
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
            package_name, target_name, program_count = SDK_PACKAGES[source]
            matches = [r for r in artifacts if r.get("reason") == "compiler-artifact" and r["target"]["name"] == target_name]
            require(len(matches) == 1, "ambiguous complete SDK artifact: " + source)
            row = matches[0]
            manifest = Path(package_metadata[row["package_id"]]["manifest_path"]).resolve(strict=True)
            package_root = ROOT / "tools/compact-rust-proof-smoke" / package_name
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
            require(len(list((generated / source / "contract/checked-programs").glob("*.json"))) == program_count, "SDK complete Program count differs")
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
                                  "compact-rust-proof-smoke", "--features", "outer-vector-product-proofs", "--bin", "compact-rust-proof-smoke", "--", "-D", "warnings"])
        report["raw_binding_refusals"] = []
        for source, (_, _, count) in SDK_PACKAGES.items():
            for kind, indices, message in [
                ("changed-ir", [None], "complete IR bytes differ"),
                ("changed-program", range(count), "complete Program bytes differ"),
                ("missing-program", sorted({0,count-1}), "complete Program membership differs"),
            ]:
                for index in indices:
                    label=source+"-"+kind+("" if index is None else "-"+str(index))
                    negative=output / label
                    for other in SELECTED_SOURCES:
                        target=negative / other / "contract"
                        shutil.copytree(generated / other / "contract/checked-programs", target / "checked-programs")
                        shutil.copyfile(generated / other / "contract/compact-rust-ir.json", target / "compact-rust-ir.json")
                    folder=negative / source / "contract"
                    if kind=="changed-ir":
                        path=folder / "compact-rust-ir.json"
                        original = path.read_bytes(); changed = original + b" "
                        require(len(changed) == len(original) + 1, "raw IR byte mutation differs")
                        json.loads(changed); path.write_bytes(changed)
                    else:
                        path=folder / f"checked-programs/program-{index}.json"
                        if kind=="missing-program": path.unlink() # Only this private malformed copy.
                        else: path.write_bytes(path.read_bytes()+b" ")
                    control_inputs=hashes([p for p in negative.rglob("*") if p.is_file()])
                    check_premises(); result, invocation=run_refusal([runner,"--outer-vector-product-models",str(negative)],label,report,env); check_premises()
                    (negative / "stdout").write_text(result.stdout);(negative / "stderr").write_text(result.stderr)
                    require(result.returncode!=0 and not result.stdout and "outer-vector-product "+source+" "+message in result.stderr, "wrong raw identity refusal: "+label)
                    require(not any((negative / n / "keys").exists() for n in SELECTED_SOURCES) and not list(negative.rglob("*.proof")), "pre-key refusal produced keys/proofs")
                    unchanged(control_inputs)
                    report["raw_binding_refusals"].append(dict(label=label,invocation=invocation,exit=result.returncode,input_hashes_before=control_inputs,stderr=result.stderr,before_keys_or_state=True,scope="whole private raw IR/Program bytes or membership control, not new compiler output or execution metadata authority"))
        full_generated = output / "protocol/generated"
        full_before = inventory(full_generated)
        execute("all-source-models", [runner, "--outer-vector-product-inventory", full_generated], output / "all-source-models.json")
        full_models = json.loads((output / "all-source-models.json").read_text())
        definitions = json.loads((main_gate.FIXTURES / "sources.json").read_text())
        require(full_models["format"] == "compact-outer-vector-product-model-inventory/v1"
                and [(r["source"], r["circuit"]) for r in full_models["circuits"]]
                == [(s["role"], entry) for s in definitions for entry in s["entries"]], "complete model roster differs")
        by_source = {s["role"]:s for s in definitions}
        require(all(r["proof_required"] is True
                    and (type(r["k"]) is int and r["k"] >= 0 if r["proof_required"] else r["k"] is None)
                    and r["selected_for_proof"] is False
                    for r in full_models["circuits"]), "model observation/applicability differs")
        expected_models = json.loads((main_gate.FIXTURES / "model-expectations.json").read_text())
        require(expected_models["format"] == "compact-outer-vector-product-model-expectations/v1"
                and [{k: r[k] for k in ["source", "circuit", "k", "proof_required"]} for r in full_models["circuits"]] == expected_models["circuits"],
                "current full model inventory drifted; inspect source/profile and extend eligible proof coverage")
        require(inventory(full_generated) == full_before, "model observation changed producers")
        require(len(SELECTED) == len(set(SELECTED)) == 15 and set(SELECTED) == {
            (r["source"], r["circuit"]) for r in full_models["circuits"] if r["proof_required"] and r["k"] <= 8
        }, "selected cohort must retain every actual in-cap export")
        report["all_source_models"] = full_models
        report["unselected_required_exports"] = [r for r in full_models["circuits"] if r["proof_required"] and (r["source"], r["circuit"]) not in SELECTED]
        report["unproved_exports"] = [r for r in full_models["circuits"] if r["proof_required"]]
        execute("models", [runner, "--outer-vector-product-models", generated], output / "models.json")
        models = json.loads((output / "models.json").read_text())
        require(models["format"] == "compact-outer-vector-product-models/v1"
                and [(r["source"],r["circuit"]) for r in models["circuits"]] == SELECTED
                and models["proof_execution"] is False, "selected model roster differs")
        require(all(next(x["k"] for x in full_models["circuits"] if (x["source"],x["circuit"]) == (r["source"],r["circuit"])) == r["k"]
                and r["selected_for_proof"] is True for r in models["circuits"]), "selected/full models disagree")
        selected_models = models["circuits"]
        require(len(selected_models) == 15 and all(type(r["k"]) is int and 0 <= r["k"] <= 8 for r in selected_models), "selected model exceeds tiny bound")
        report["models"] = models
        # Private diagnostic: an existing intact over-limit model exercises the
        # selected-model guard, without changing original source or proof inputs.
        excessive = output / "selected-model-over-cap"
        for source in SELECTED_SOURCES:
            shutil.copytree(generated / source, excessive / source)
        donor = generated / "family/zkir/branch.zkir"
        require(next(r["k"] for r in full_models["circuits"] if r["source"] == "family" and r["circuit"] == "branch") > 8, "over-cap control premise changed")
        shutil.copyfile(donor, excessive / "original/zkir/run.zkir")
        control_inputs = hashes([p for p in excessive.rglob("*") if p.is_file()])
        check_premises()
        refused, invocation = run_refusal([runner,"--outer-vector-product-models",str(excessive)], "selected-model-over-cap", report, env)
        check_premises()
        (excessive / "stdout").write_text(refused.stdout); (excessive / "stderr").write_text(refused.stderr)
        require(refused.returncode != 0 and "model exceeds tiny automatic k<=8" in refused.stderr, "over-cap guard did not refuse")
        refused_models = json.loads(refused.stdout)
        require(refused_models["circuits"][0]["k"] > 8 and refused_models["circuits"][1:] == selected_models[1:]
                and not list(excessive.rglob("*.proof")) and not any((excessive / n / "keys").exists() for n in SELECTED_SOURCES), "over-cap refusal scope differs")
        unchanged(control_inputs)
        report["selected_model_over_cap_refusal"] = {"invocation":invocation,"exit":refused.returncode,"models":refused_models,"input_hashes_before":control_inputs,
            "donor_sha256":digest(donor),"before_keys_or_proofs":True,"scope":"private intact existing family branch k9 ZKIR diagnostic, not admitted selected SDK semantics"}
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
        report["pre_key_refusal_roster"] = validate_pre_key_refusals(report, components["runner"]["sha256"])
        require(not list(generated.rglob("*.proof"))
                and not any((generated / source / "keys").exists() for source in SELECTED_SOURCES),
                "refusal qualification must complete before keys or proofs exist")
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
            first_key = next(c["started_monotonic"] for c in report["commands"] if c["label"].startswith("keygen-"))
            require([row["label"] for row in report["refusal_invocations"]] == expected_refusal_roster()
                    and all(r["ended_monotonic"] <= first_key and r["runner_sha256"] == components["runner"]["sha256"]
                            for r in report["refusal_invocations"]), "refusal invocations did not precede fresh keys with the same immutable runner")
            require(all(not (generated / name / "keys").exists() for name in by_source if name not in SELECTED_SOURCES), "unselected source acquired keys")
            for source in SELECTED_SOURCES:
                require({p.name for p in (generated / source / "keys").iterdir()} == {f"{c}.{ext}" for s,c in SELECTED if s == source for ext in ["prover","verifier"]}, "selected source key membership differs: " + source)
            report["key_hashes"] = hashes([generated / source / f"keys/{c}.{ext}" for source, c in SELECTED for ext in ["prover", "verifier"]])
            report["binary_ir_hashes"] = hashes([generated / source / f"zkir/{c}.bzkir" for source, c in SELECTED])
            execute("prove-verify-apply", [runner, "--outer-vector-product", generated, output / "protocol/ts-results.json"])
            summary = json.loads((generated / "outer-vector-product-proof-result.json").read_text())
            validate_summary(summary, generated)
            report["proof_result"] = summary
            report["proof_execution"] = True
            report["unproved_exports"] = report["unselected_required_exports"]
            report["result_hashes"] = hashes([p for p in generated.rglob("outer-vector-product-*") if p.is_file()])
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
        if "generated_sdk" in report:
            actual_root = Path(report["generated_sdk"])
            actual_proofs = list(actual_root.rglob("outer-vector-product-*.proof"))
            actual_transactions = list(actual_root.rglob("outer-vector-product-*-transaction.bin"))
            reports = {stage:[] for stage in ["proof-started", "proof-verified", "application-started", "application-applied"]}
            invalid = []
            paths = []
            for index,(source,name) in enumerate(SELECTED):
                root = actual_root / source
                for stage in reports:
                    path = root / f"outer-vector-product-{index}-{stage}.json"
                    if not path.exists(): continue
                    paths.append(path)
                    try:
                        row = json.loads(path.read_text())
                        require(row["format"] == "compact-outer-vector-product-progress/v1"
                                and row["source"] == source and row["circuit"] == name
                                and row["index"] == index and row["stage"] == stage
                                and type(row["bytes"]) is int and row["bytes"] >= 0, "runtime progress identity differs")
                        if stage in {"proof-verified", "application-applied"}:
                            suffix = ".proof" if stage == "proof-verified" else "-transaction.bin"
                            other = ".preimage" if stage == "proof-verified" else "-state.bin"
                            require(row["bytes"] > 0 and (root / f"outer-vector-product-{index}{suffix}").stat().st_size == row["bytes"]
                                    and (root / f"outer-vector-product-{index}{other}").stat().st_size > 0, "completed runtime artifact missing or partial")
                        reports[stage].append(row)
                    except Exception as error:
                        invalid.append(dict(path=str(path),error=repr(error)))
            report["runtime_progress"] = reports
            report["invalid_runtime_progress"] = invalid
            report["standalone_proofs_completed"] = len(reports["proof-verified"])
            report["strict_applications_completed"] = len(reports["application-applied"])
            report["proof_artifacts_present"] = len(actual_proofs)
            report["transaction_artifacts_present"] = len(actual_transactions)
            report["completed_proof_artifact_hashes"] = hashes(actual_proofs + actual_transactions + paths)
            report["proof_execution"] = bool(reports["proof-started"]) or report.get("proof_execution", False)
            if report.get("status") == "passed" and (invalid or any(len(rows) != 15 for rows in reports.values())):
                report["status"] = "failed"
                report["error"] = "full result lacks completed actual runtime stages"
        (output / "receipt.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"status": report["status"], "receipt": str(output / "receipt.json")}))
    require(report["status"] in {"prepared", "passed"}, "proof gate failed runtime-stage validation")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ["ts-runtime", "compiler-receipt", "zkir", "output"]:
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--prepare-only", action="store_true", help="compile and verify bindings/material only; no keys/proofs/application")
    run(parser.parse_args())
