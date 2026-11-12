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
SELECTED = [("helper", "run"), ("proof_candidate", "run_field"),
            ("proof_candidate", "run"), ("checked_helper", "run")]
SOURCES = ("helper", "proof_candidate", "checked_helper")
INPUT = [[[3, 5], 31], [[4, 6], 41]]
CASES = [("helper", "run", 0, INPUT, INPUT, 1),
         ("proof_candidate", "run_field", 0, INPUT, INPUT, 2),
         ("proof_candidate", "run", 0, INPUT, INPUT, 1),
         ("proof_candidate", "run", 1, INPUT, INPUT[::-1], 3),
         ("checked_helper", "run", 0, INPUT, INPUT[::-1], 1)]
require, digest = main_gate.require, main_gate.digest


def depfile_prerequisites(raw, base=ROOT):
    logical = raw.replace("\\\n", "")
    first_rule = next(line for line in logical.splitlines() if line.strip())
    _, separator, prerequisites = first_rule.partition(": ")
    require(separator and prerequisites, "invalid actual runner dependency rule")
    paths = {str((base / path).resolve(strict=True)) for path in shlex.split(prerequisites)}
    require(paths and all(Path(path).is_file() for path in paths), "non-file runner prerequisite")
    return paths


def validate_summary(summary, generated):
    require(summary["format"] == "compact-vector-product-elements-proof/v3" and summary["status"] == "passed"
            and summary["constructor_execution_proved"] is False and summary["all_exports_proved"] is False,
            "invalid Vector product elements proof summary")
    require([(r["source"], r["circuit"], r["initial_seen"], r["input"], r["expected"], r["seen"]) for r in summary["calls"]] == CASES,
            "selected proof scenario roster differs")
    require(summary["selected_source_exports_proved"] == list(map(list, SELECTED)) and summary["unproved_selected_source_exports"] == [],
            "selected/unproved selected source export coverage differs")
    require(summary["all_source_exports_deployed"] is False,
            "selected-call deployments must not claim full export deployment")
    for index, row in enumerate(summary["calls"]):
        source_root = generated / row["source"]
        expected_type = "Vector<2,[Vector<2,Field>,Field]>" if row["circuit"] == "run_field" else "Vector<2,[Vector<2,Uint<16>>,Uint<32>]>"
        require(row["result_type"] == expected_type, "genuine result carrier differs")
        require(row["guard"] is (True if row["source"] == "checked_helper" else None), "checked-helper guard differs")
        require(isinstance(row["expected"], list) and len(row["expected"]) == 2 and all(isinstance(pair, list) and len(pair) == 2 and isinstance(pair[0], list) and len(pair[0]) == 2 and all(type(x) is int for x in pair[0]) and type(pair[1]) is int for pair in row["expected"]) and type(row["witness_count"]) is int
                and row["witness_count"] == 0, "witness count or result differs")
        for field, path in [("proof_bytes", f"vector-product-elements-{index}.proof"),
                            ("transaction_bytes", f"vector-product-elements-{index}-transaction.bin")]:
            require(type(row[field]) is int and row[field] > 0
                    and (source_root / path).stat().st_size == row[field], "missing or truncated proof/application artifact")
        stem = f"vector-product-elements-{index}"
        require(row["standalone_proof"] == f"{row['source']}/vector-product-elements-{index}.proof"
                and row["applied_transaction"] == f"{row['source']}/vector-product-elements-{index}-transaction.bin"
                and row["transaction_proof_relationship"] == "independently proved"
                and row["transaction_proof_version"] == 3 and row["positive_fee_dust_spends"] == 1
                and row["strict_application"] == "default" and row["replay_refusal"] == "IntentAlreadyExists",
                "proof/application identity or strictness differs")
        require(all(row[name] is True for name in ["corrupt_proof_rejected", "changed_binding_rejected",
                    "changed_result_rejected", "changed_result_same_geometry", "wrong_verifier_rejected",
                    "standalone_verified_separately", "replay_unchanged"]),
                "proof/application control missing")
        require(all((source_root / f"{stem}{suffix}").stat().st_size > 0
                    for suffix in (".preimage", ".inputs", "-state.bin", "-ledger-after.bin")),
                "proof/state evidence missing")
        transaction_proof = (source_root / f"{stem}-transaction.proof").read_bytes()
        require(type(row["transaction_proof_bytes"]) is int
                and len(transaction_proof) == row["transaction_proof_bytes"] > 0
                and transaction_proof != (source_root / f"{stem}.proof").read_bytes(),
                "transaction-bound proof is missing or equals the standalone proof")
        require((source_root / f"{stem}-ledger-after.bin").read_bytes()
                == (source_root / f"{stem}-ledger-replay.bin").read_bytes(), "replayed ledger bytes differ")


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
    report = {"format": "compact-vector-product-elements-proof-gate/v3", "status": "incomplete", "commands": [],
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
        generated = output / "protocol/generated"
        env["COMPACT_VECTOR_PRODUCT_ELEMENTS_GENERATED"] = str(generated / "proof_candidate")
        env["COMPACT_VECTOR_PRODUCT_ELEMENTS_HELPER_GENERATED"] = str(generated / "helper")
        env["COMPACT_VECTOR_PRODUCT_ELEMENTS_CHECKED_GENERATED"] = str(generated / "checked_helper")
        report["generated_sdk"] = str(generated)
        report["generated_before"] = before = main_gate.inventory(generated)
        tracked_sources = subprocess.check_output(["git", "ls-files", "-z"], cwd=ROOT).split(b"\0")
        source_paths = [ROOT / os.fsdecode(path) for path in tracked_sources if path]
        require(all(p.is_file() for p in source_paths), "tracked source file is absent")
        report["source_hashes"] = hashes(source_paths)
        report["generated_input_hashes"] = hashes([p for p in (output / "protocol/generated").rglob("*") if p.is_file()])
        report["bound_api_inputs"] = admitted["build_inputs_before_use"]
        report["environment"] = {k: env.get(k) for k in ["RUSTUP_TOOLCHAIN", "RUSTC", "CARGO_INCREMENTAL", "CARGO_BUILD_JOBS", "CARGO_TARGET_DIR", "COMPACT_VECTOR_PRODUCT_ELEMENTS_GENERATED", "COMPACT_VECTOR_PRODUCT_ELEMENTS_HELPER_GENERATED", "COMPACT_VECTOR_PRODUCT_ELEMENTS_CHECKED_GENERATED", "MIDNIGHT_PP"]}
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
            for field in ["source_hashes", "generated_input_hashes", "bound_api_inputs", "dependency_source_hashes", "generated_build_inputs_before_compile", "compiler_tools"]:
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
                dirs[:] = [d for d in dirs if d != ".git"]
                for name in files:
                    path = Path(folder) / name
                    if path.is_file():
                        dependencies.add(path.resolve(strict=True))
        report["dependency_source_roots"] = sorted(map(str, dependency_roots))
        report["dependency_source_hashes"] = hashes(dependencies)
        report["metadata_before_build"] = {"path": str(output / "build-metadata.json"), "sha256": digest(output / "build-metadata.json")}
        # No SDK path is needed by the default build. The opt-in build follows it.
        selected_sdks = {key: env.pop(key) for key in (
            "COMPACT_VECTOR_PRODUCT_ELEMENTS_GENERATED",
            "COMPACT_VECTOR_PRODUCT_ELEMENTS_HELPER_GENERATED",
            "COMPACT_VECTOR_PRODUCT_ELEMENTS_CHECKED_GENERATED",
        )}
        execute("default-runner-check", ["cargo", "check", "--profile", "proof-checks", "--locked", "--offline", "-p", "compact-rust-proof-smoke", "--no-default-features"])
        execute("all-features-runner-clippy-without-sdk", ["cargo", "clippy", "--profile", "proof-checks", "--locked", "--offline", "-p", "compact-rust-proof-smoke", "--all-targets", "--all-features", "--", "-D", "warnings"])
        env.update(selected_sdks)
        # Cargo fixture build scripts produce real include! inputs outside the
        # tracked tree. Prepare and pin those physical files before qualifying
        # the runner, rather than accepting newly discovered bytes afterward.
        execute("prepare-runner-inputs", ["cargo", "check", "--profile", "proof-checks", "--locked", "--offline", "-p",
                "compact-rust-proof-smoke", "--features", "vector-product-elements", "--message-format=json"], output / "prepare-build.jsonl")
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
                                "compact-rust-proof-smoke", "--features", "vector-product-elements", "--message-format=json"], output / "build.jsonl")
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
        report["companion_sdk_artifacts"] = {}
        for source, companion in (("helper", "helper"), ("checked_helper", "checked")):
            crate = f"compact_rust_vector_product_elements_proof_{companion}"
            matches = [row for row in artifacts if row.get("reason") == "compiler-artifact"
                       and row["target"]["name"] == crate]
            require(len(matches) == 1, f"ambiguous {source} crate artifact")
            row = matches[0]
            package = package_metadata[row["package_id"]]
            manifest = Path(package["manifest_path"]).resolve(strict=True)
            libraries = [Path(path) for path in row["filenames"] if path.endswith(".rlib")]
            expected_manifest = ROOT / f"tools/compact-rust-proof-smoke/vector-product-elements-{companion}-sdk/Cargo.toml"
            require(len(libraries) == 1 and manifest == expected_manifest,
                    f"{source} SDK must compile at its own declared crate root")
            library = libraries[0]
            snapshot_path = output / f"bin/{source}-sdk.rlib"
            require(library.is_file() and not snapshot_path.exists(), "SDK library snapshot must be a new regular data artifact")
            library_sha = digest(library)
            shutil.copyfile(library, snapshot_path)
            require(digest(snapshot_path) == library_sha == digest(library), "SDK library changed during data snapshot")
            snapshot_path.chmod(0o444)
            component = {"source": str(library), "snapshot": str(snapshot_path), "sha256": library_sha,
                         "size": snapshot_path.stat().st_size, "kind": "rust-rlib-data"}
            components[f"{source}_sdk_rlib"] = component
            depfile = library.parent / (library.stem.removeprefix("lib") + ".d")
            retained = output / f"{source}-sdk.d"
            shutil.copyfile(depfile, retained)
            prerequisites = depfile_prerequisites(retained.read_text(), ROOT)
            required = [manifest.parent / "lib.rs", generated / source / "contract/lib.rs",
                        *sorted((generated / source / "contract/checked-programs").glob("*.json"))]
            require(all(str(path.resolve(strict=True)) in prerequisites for path in required),
                    f"{source} crate omitted actual complete SDK inputs")
            report["companion_sdk_artifacts"][source] = {
                "package_id": row["package_id"], "manifest": str(manifest), "crate_root": row["target"]["src_path"],
                "artifact": component, "depfile": str(retained), "depfile_sha256": digest(retained),
                "prerequisites": sorted(prerequisites),
            }
            tracked.update(prerequisites)
        required = [main_gate.FIXTURES / "sources.json"]
        for source in SOURCES:
            goldens = main_gate.FIXTURES / "ledger9-goldens" / source
            ir = goldens / "source-ir.json" if source == "checked_helper" else main_gate.FIXTURES / f"goldens/{source}/source-ir.json"
            required.extend([generated / source / "contract/lib.rs",
                             *sorted((generated / source / "contract/checked-programs").glob("*.json")),
                             ir, goldens / "artifacts.json", *sorted((goldens / "checked-programs").glob("*.json"))])
        require(all(str(path.resolve(strict=True)) in tracked for path in required), "runner omitted actual SDK/Program dependency")
        worktree_inputs = {path: report["source_hashes"].get(path) for path in tracked
                           if Path(path).is_relative_to(ROOT)}
        require(all(sha is not None and digest(path) == sha for path, sha in worktree_inputs.items()),
                "runner worktree dependency was not pinned before build")
        physical_inputs = {str(Path(p).resolve(strict=True)): sha for field in ["source_hashes", "generated_input_hashes", "dependency_source_hashes", "generated_build_inputs_before_compile"] for p, sha in report[field].items()}
        actual_files = set(tracked)
        require(actual_files and all(p in physical_inputs and digest(p) == physical_inputs[p] for p in actual_files),
                "actual runner dependency missing from physical prebuild input map")
        report["runner_physical_inputs_before_build"] = {p: physical_inputs[p] for p in sorted(actual_files)}
        report["runner_depfile"].update(membership_status="passed", all_generated_includes=True,
                worktree_inputs_pinned_before_build=worktree_inputs)
        execute("runner-clippy", ["cargo", "clippy", "--profile", "proof-checks", "--locked", "--offline", "-p",
                                  "compact-rust-proof-smoke", "--features", "vector-product-elements", "--bin", "compact-rust-proof-smoke", "--", "-D", "warnings"])
        for source in SOURCES:
            for kind, message in [("changed-ir", f"vector-product-elements {source} complete IR bytes differ"),
                                  ("changed-program", f"vector-product-elements {source} complete Program bytes differ"),
                                  ("missing-program", f"vector-product-elements {source} complete Program membership differs"),
                                  ("changed-zkir", "Ledger 9 artifact differs from the pinned fixture: zkir/run.zkir")]:
                label = source + "-" + kind
                negative = output / label
                for member in SOURCES:
                    shutil.copytree(generated / member / "contract/checked-programs", negative / member / "contract/checked-programs")
                    shutil.copyfile(generated / member / "contract/compact-rust-ir.json", negative / member / "contract/compact-rust-ir.json")
                    if kind == "changed-zkir":
                        shutil.copyfile(generated / member / "contract/lib.rs", negative / member / "contract/lib.rs")
                        shutil.copytree(generated / member / "zkir", negative / member / "zkir")
                folder = negative / source / "contract"
                if kind == "changed-ir":
                    path = folder / "compact-rust-ir.json"
                    original = path.read_bytes()
                    name = b"checked" if source == "checked_helper" else b"keep"
                    token = b'"name": "' + name + b'"'
                    position = original.index(token, original.index(b'"circuits"'))
                    offset = position + len(token) - 2
                    require(original[offset:offset+1] == name[-1:], "pure helper declaration-name mutation premise differs")
                    changed = original[:offset] + b'q' + original[offset+1:]
                    require(len(changed) == len(original) and sum(a != b for a, b in zip(changed, original)) == 1,
                            "IR control must change exactly one declared byte")
                    json.loads(changed)
                    path.write_bytes(changed)
                elif kind == "changed-program":
                    path = folder / "checked-programs/program-0.json"
                    path.write_bytes(path.read_bytes() + b" ")
                elif kind == "missing-program":
                    (folder / "checked-programs/program-0.json").unlink()
                else:
                    path = negative / source / "zkir/run.zkir"
                    path.write_bytes(path.read_bytes() + b" ")
                control_inputs = hashes([p for p in negative.rglob("*") if p.is_file()])
                result = subprocess.run([runner, "--vector-product-elements", str(negative)], cwd=ROOT, env=env,
                                        capture_output=True, text=True, timeout=30)
                (negative / "stdout").write_text(result.stdout)
                (negative / "stderr").write_text(result.stderr)
                require(result.returncode != 0 and result.stdout == "" and message in result.stderr, label + " failed outside binding guard")
                require(not any((negative / member / "keys").exists() for member in SOURCES)
                        and not list(negative.rglob("*.proof")) and not list(negative.glob("vector-product-elements-*")), "pre-key refusal produced evidence")
                unchanged(control_inputs)
                report[label] = {"exit": result.returncode, "before_keys_or_state": True, "input_hashes_before": control_inputs,
                                 "scope": "complete IR, Program or ZKIR identity refusal before key generation"}
        full_generated = output / "protocol/generated"
        full_before = main_gate.inventory(full_generated)
        execute("all-source-models", [runner, "--vector-product-elements-inventory", full_generated], output / "all-source-models.json")
        full_models = json.loads((output / "all-source-models.json").read_text())
        definitions = json.loads((main_gate.FIXTURES / "sources.json").read_text())
        require(full_models["format"] == "compact-vector-product-elements-model-inventory/v2"
                and [(r["source"], r["circuit"]) for r in full_models["circuits"]]
                == [(s["source"], entry) for s in definitions for entry in s["entries"]], "complete model roster differs")
        by_source = {s["source"]:s for s in definitions}
        require(all(r["proof_required"] is by_source[r["source"]]["stateful"]
                    and (type(r["k"]) is int and r["k"] >= 0 if r["proof_required"] else r["k"] is None)
                    and r["selected_for_proof"] == ((r["source"], r["circuit"]) in SELECTED)
                    for r in full_models["circuits"]), "model observation/applicability differs")
        expected_models = json.loads((main_gate.FIXTURES / "ledger9-goldens/model-expectations.json").read_text())
        require(expected_models["format"] == "compact-vector-product-elements-model-expectations/v2"
                and [{k: r[k] for k in ["source", "circuit", "k", "proof_required"]} for r in full_models["circuits"]] == expected_models["circuits"],
                "current full model inventory drifted; inspect source/profile and extend eligible proof coverage")
        require(main_gate.inventory(full_generated) == full_before, "model observation changed producers")
        report["all_source_models"] = full_models
        report["unproved_exports"] = [r for r in full_models["circuits"] if r["proof_required"] and not r["selected_for_proof"]]
        execute("models", [runner, "--vector-product-elements-models", generated], output / "models.json")
        models = json.loads((output / "models.json").read_text())
        require(models["format"] == "compact-vector-product-elements-models/v3" and [(r["source"], r["circuit"]) for r in models["circuits"]] == SELECTED,
                "selected model roster differs")
        require(models["selected_for_proof"] == list(map(list, SELECTED)) and models["unselected"] == [] and models["proof_execution"] is False, "selected proof scope differs")
        require(all(next(x["k"] for x in full_models["circuits"] if (x["source"], x["circuit"]) == (r["source"], r["circuit"])) == r["k"] for r in models["circuits"]), "selected model differs from complete inventory")
        selected_models = models["circuits"]
        require(len(selected_models) == len(SELECTED) and all(type(r["k"]) is int and r["k"] == 8 for r in selected_models),
                "selected circuit differs from its bound k8 model")
        report["models"] = models
        # A separate complete retained Fold source supplies a real over-cap diagnostic.
        # It never replaces the admitted complete component-Vector IR/Programs or proof inputs.
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
        shutil.copyfile(diagnostic / "generated/zkir/run_field.zkir", excessive / "proof_candidate/zkir/run_field.zkir")
        refused = subprocess.run([runner, "--vector-product-elements-models", str(excessive)],
                                 cwd=ROOT, env=env, capture_output=True, text=True, timeout=30)
        (excessive / "stdout").write_text(refused.stdout)
        (excessive / "stderr").write_text(refused.stderr)
        require(refused.returncode and "model exceeds tiny automatic k<=8" in refused.stderr,
                "selected over-cap model did not refuse")
        refused_models = json.loads(refused.stdout)
        require(next(r["k"] for r in refused_models["circuits"] if (r["source"], r["circuit"]) == ("proof_candidate", "run_field")) > 8
                and not any((excessive / source / "keys").exists() for source in SOURCES), "over-cap refusal was not before keys")
        report["selected_model_over_cap_refusal"] = {"exit": refused.returncode,
                "before_keys_or_proofs": True, "models": refused_models,
                "scope": "genuine larger compiler-produced ZKIR under selected diagnostic slot; not proof admission"}
        substituted = output / "conditional-substituted-same-k"
        shutil.copytree(generated, substituted)
        replacement = substituted / "proof_candidate/zkir/run.zkir"
        original = replacement.read_bytes()
        shutil.copyfile(generated / "helper/zkir/run.zkir", replacement)
        require(replacement.read_bytes() != original, "same-k substitution premise did not change raw ZKIR")
        substitute_inputs = hashes([path for path in substituted.rglob("*") if path.is_file()])
        refused = subprocess.run([runner, "--vector-product-elements", str(substituted)],
                                 cwd=ROOT, env=env, capture_output=True, text=True, timeout=30)
        (substituted / "stdout").write_text(refused.stdout)
        (substituted / "stderr").write_text(refused.stderr)
        require(refused.returncode != 0 and refused.stdout == ""
                and "Ledger 9 artifact differs from the pinned fixture: zkir/run.zkir" in refused.stderr,
                "substitution with another eligible model did not refuse at raw binding")
        require(not any((substituted / source / "keys").exists() for source in SOURCES)
                and not list(substituted.rglob("*.proof")), "same-k refusal produced keys or proofs")
        unchanged(substitute_inputs)
        report["conditional_same_k_substitution_refusal"] = {
            "exit": refused.returncode, "before_keys_or_proofs": True, "input_hashes_before": substitute_inputs,
        }
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
            for source in SOURCES:
                (generated / source / "keys").mkdir()
            report["keygen"] = []
            version = output / "zkir-version.txt"
            execute("zkir-version", [report["zkir"]["snapshot"], "--version"], version)
            require(version.read_text().strip() == "midnight-zkir 3.2.0", "expected the released ZKIR v3.2.0 compiler")
            for row in selected_models:
                circuit = row["circuit"]
                source_root = generated / row["source"]
                execute("keygen-" + row["source"] + "-" + circuit, [report["zkir"]["snapshot"], "compile", source_root / f"zkir/{circuit}.zkir",
                                              source_root / f"keys/{circuit}.prover", source_root / f"keys/{circuit}.verifier"])
                model = re.search(r"\(k=(\d+), rows=(\d+)\)", Path(report["commands"][-1]["log"]).read_text())
                require(model is not None and int(model[1]) == row["k"], "actual key model differs from verified pre-key model")
                report["keygen"].append({"source": row["source"], "circuit": circuit, "mode": "fresh direct native", "k": row["k"], "rows": int(model[2])})
            report["key_hashes"] = hashes([generated / source / f"keys/{c}.{ext}" for source, c in SELECTED for ext in ["prover", "verifier"]])
            report["binary_ir_hashes"] = hashes([generated / source / f"zkir/{c}.bzkir" for source, c in SELECTED])
            execute("prove-verify-apply", [runner, "--vector-product-elements", generated])
            summary = json.loads((generated / "vector-product-elements-proof-result.json").read_text())
            validate_summary(summary, generated)
            report["proof_result"] = summary
            report["proof_execution"] = True
            report["result_hashes"] = hashes([p for p in generated.rglob("vector-product-elements-*") if p.is_file()])
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
