#!/usr/bin/env python3
# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Whole constructor Vector sources, unedited SDKs, TS/native APIs and refusals.

Requires a clean current original-compiler receipt. No keys/proofs, constructor
cryptographic admission, constructor total-cost parity or integration is implied.
"""
import argparse
import copy
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import tomllib

from check_constant_vector_projection import compile_failure
from check_stateful_product_vector_widening import (
    digest, inventory, reject_compiler_overrides, require, source_manifest_snapshot,
)

ROOT = Path(__file__).resolve().parents[2]
FIXTURES = Path(__file__).with_name("tests") / "constructor_vector_operands"


def program_assets(folder, entries):
    assets = {}
    for path in sorted(folder.iterdir()):
        require(path.is_file() and not path.is_symlink() and path.suffix == ".json", "unexpected Program asset")
        value = json.loads(path.read_text())
        root = value.get("root")
        require(isinstance(root, str) and root in entries and root not in assets, "duplicate or unexpected Program root")
        require(value.get("schema") == 1 and value.get("purpose") == "review_only_not_execution_authority"
                and value.get("inventory_semantics") == "guards_and_occurrences_not_a_linear_execution_schedule", "Program metadata policy differs")
        assets[root] = {"file": path.name, "sha256": digest(path)}
    require(set(assets) == set(entries), "complete Program root roster differs")
    return assets


def admit_oracle(path, generated, runtime, definitions):
    reference = json.loads(path.read_text())
    require(reference["status"] == "passed" and reference["format"] == "constructor-vector-operands-ts/v1", "wrong TS oracle")
    require(set(reference["wrappers"]) == {s["role"] for s in definitions}, "TS wrapper roster differs")
    for spec in definitions:
        wrapper = reference["wrappers"][spec["role"]]
        actual = (generated / spec["role"] / "contract/index.js").resolve(strict=True)
        require(Path(wrapper["path"]).resolve(strict=True) == actual and wrapper["sha256"] == digest(actual), "TS wrapper identity differs")
    expected = {str((FIXTURES / name).resolve()): digest(FIXTURES / name) for name in ["sources.json", "cases.json"]}
    require(reference["inputs"] == expected, "TS fixture identities differ")
    require(Path(reference["capture"]["path"]).resolve() == (FIXTURES / "capture.mjs").resolve()
            and reference["capture"]["sha256"] == digest(FIXTURES / "capture.mjs"), "TS capture identity differs")
    require(Path(reference["runtime"]["path"]).resolve() == runtime
            and reference["runtime"]["version"] == json.loads((ROOT / "runtime/package.json").read_text())["version"], "TS runtime identity differs")
    return reference


def check_oracle_consumption(oracle, environment):
    require(environment.get("COMPACT_CONSTRUCTOR_TS") == oracle["path"], "TS oracle path changed before consumption")
    require(digest(oracle["path"]) == oracle["sha256"], "TS oracle bytes changed before consumption")


def first_loop(node):
    if isinstance(node, dict):
        if node.get("kind") == "for_each_vector":
            return node
        for value in node.values():
            found = first_loop(value)
            if found is not None:
                return found
    elif isinstance(node, list):
        for value in node:
            found = first_loop(value)
            if found is not None:
                return found
    return None


def malformed_cases(irs):
    """Independent serialized boundary changes; keep the original complete IR."""
    for name in ["kind", "length", "element", "unknown", "leaked", "empty_body", "dead_branch", "syntax_depth"]:
        base = "empty_local" if name == "empty_body" else "conditional" if name == "dead_branch" else "parameter"
        value = copy.deepcopy(irs[base])
        loop = first_loop(value["constructor"])
        require(loop is not None, "missing constructor loop premise")
        expected = []
        if name == "kind":
            loop["source"] = {"kind": "boolean", "value": True}
            expected = ["constructor", "item"]
        elif name == "length":
            loop["length"] = 3
            expected = ["Vector", "length: 2", "length: 3"]
        elif name == "element":
            loop["binding"]["ty"] = {"kind": "boolean"}
            expected = ["Vector", "Field", "Boolean"]
        elif name == "unknown":
            loop["source"]["name"] = "not_in_scope"
            expected = ["unknown parameter", "not_in_scope"]
        elif name == "leaked":
            value["constructor"]["steps"].append({"kind": "expression", "value": {"kind": "parameter", "name": loop["binding"]["name"]}})
            expected = ["unknown parameter", "item"]
        elif name == "empty_body":
            loop["steps"].append({"kind": "expression", "value": {"kind": "parameter", "name": "empty_still_checks_body"}})
            expected = ["unknown parameter", "empty_still_checks_body"]
        elif name == "dead_branch":
            loop["source"]["condition"] = {"kind": "boolean", "value": True}
            loop["source"]["otherwise"] = {"kind": "parameter", "name": "inactive_still_checks_source"}
            expected = ["unknown parameter", "inactive_still_checks_source"]
        else:
            # Borrowed native iteration does not unroll by Vector length. Test
            # actual nested source depth against the unchanged policy instead.
            for _ in range(50):
                loop["source"] = {
                    "kind": "if",
                    "condition": {"kind": "boolean", "value": True},
                    "then": loop["source"],
                    "otherwise": {"kind": "parameter", "name": "input"},
                }
            expected = ["resource", "syntax_depth", "48"]
        yield name, value, expected


def check_lock(lock, metadata, root, allowed, local_roots):
    for item in tomllib.loads(lock.read_text())["package"]:
        key = (item["name"], item["version"], item.get("source"))
        require(key in root or key in allowed, "dependency identity drift: " + repr(key))
        if key[2] is not None:
            require(item.get("checksum") == root[key], "dependency checksum drift")
    for package in json.loads(metadata.stdout)["packages"]:
        if package["source"] is None:
            path = Path(package["manifest_path"]).resolve(strict=True)
            require(any(path.is_relative_to(p.resolve()) for p in local_roots), "unexpected local dependency: " + str(path))


def main():
    require(__debug__, "qualification requires Python without -O/-OO")
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler-receipt", type=Path, required=True)
    parser.add_argument("--ts-runtime", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    output = args.output.absolute()
    require(not output.exists() and not output.is_symlink(), "output must be absent")
    env = dict(os.environ)
    configuration = reject_compiler_overrides(env)
    env.setdefault("RUSTUP_TOOLCHAIN", "1.99.0")
    require(env["RUSTUP_TOOLCHAIN"] == "1.99.0", "use pinned Rust1.99.0")
    git = lambda *a: subprocess.check_output(["git", *a], cwd=ROOT, text=True).strip()
    head, tree = git("rev-parse", "HEAD"), git("rev-parse", "HEAD^{tree}")
    require(not git("status", "--porcelain"), "clean source HEAD required")
    build = json.loads(args.compiler_receipt.read_text())
    require(
        build["status"] == "passed" and build["head"] == head and build["tree"] == tree,
        "original compiler receipt must bind this clean HEAD/tree",
    )
    compiler, scheme = [Path(build[k]["path"]).resolve(strict=True) for k in ["compactc", "compactc-scheme"]]
    require(compiler.parent == scheme.parent, "use one original compiler package")
    for key, path in [("compactc", compiler), ("compactc-scheme", scheme)]:
        require(digest(path) == build[key]["sha256"], "original compiler changed")
    paths = [Path(__file__), ROOT / "Cargo.lock", ROOT / "Cargo.toml", ROOT / "flake.nix", ROOT / "flake.lock", ROOT / "runtime/package.json"]
    for folder in [ROOT / "compiler", ROOT / "runtime-rs", ROOT / "runtime-rs-macros", ROOT / "tools/compact-rust-backend/src", FIXTURES]:
        paths.extend(p for p in folder.rglob("*") if p.is_file() and p.suffix in {".ss", ".rs", ".json", ".toml", ".mjs", ".compact"})
    paths.extend(Path(__file__).parent.glob("*.py"))
    pins = {str(p.relative_to(ROOT)): digest(p) for p in paths}
    rustup = Path(shutil.which("rustup")).resolve(strict=True)
    tools = {
        str(p): digest(p)
        for p in [
            compiler,
            scheme,
            args.compiler_receipt.resolve(),
            rustup,
            Path(sys.executable).resolve(),
            Path(shutil.which("node")).resolve(),
        ]
    }
    for name in ["cargo", "rustc", "clippy-driver", "cargo-clippy"]:
        path = Path(subprocess.check_output([str(rustup), "which", "--toolchain", "1.99.0", name], text=True).strip()).resolve(strict=True)
        tools[str(path)] = digest(path)
        if name == "cargo":
            env["PATH"] = str(path.parent) + os.pathsep + env["PATH"]
    output.mkdir(parents=True)
    report = {
        "status": "incomplete",
        "head": head,
        "tree": tree,
        "inputs": pins,
        "tools": tools,
        "cargo_configuration_sha256": configuration,
        "commands": [],
        "sources": {},
        "scope": (
            "Whole-source constructor/native/TS/API/refusal qualification only; "
            "no keys/proofs, constructor total-cost/crypto or integration acceptance"
        ),
    }
    start = time.monotonic()

    def save():
        (output / "receipt.json").write_text(json.dumps(report, indent=2) + "\n")

    def run(label, command, *, cwd=ROOT, success=True, stdin=None, environment=None):
        command = list(map(str, command))
        selected_environment = env if environment is None else environment
        if "typescript" in report:
            check_oracle_consumption(report["typescript"], selected_environment)
        require(reject_compiler_overrides(selected_environment) == configuration, "Cargo/compiler configuration changed")
        for parent in [cwd, *cwd.parents]:
            for name in ["config", "config.toml"]:
                path = parent / ".cargo" / name
                require(not path.is_file() or str(path) in configuration, "unqualified Cargo configuration at execution cwd: " + str(path))
        began = time.monotonic()
        try:
            p = subprocess.run(command, cwd=cwd, env=selected_environment, input=stdin, text=True, capture_output=True, timeout=600)
        except subprocess.TimeoutExpired as e:
            for suffix, value in [("stdout", e.stdout), ("stderr", e.stderr)]:
                (output / f"{label}.{suffix}").write_bytes(value if isinstance(value, bytes) else (value or "").encode())
            report["commands"].append({"label": label, "command": command, "timeout": True})
            save()
            raise
        hashes = {}
        for suffix in ["stdout", "stderr"]:
            path = output / f"{label}.{suffix}"
            path.write_text(getattr(p, suffix))
            hashes[suffix + "_sha256"] = digest(path)
        report["commands"].append(
            {
                "label": label,
                "command": command,
                "cwd": str(cwd),
                "exit": p.returncode,
                "seconds": time.monotonic() - began,
                **hashes,
            }
        )
        save()
        require(not success or p.returncode == 0, label + " failed: see retained diagnostics")
        return p

    save()
    try:
        definitions = json.loads((FIXTURES / "sources.json").read_text())
        cases = json.loads((FIXTURES / "cases.json").read_text())
        require(len(definitions) == 14 and len({s["role"] for s in definitions}) == 14, "complete source roster differs")
        require(
            {s["role"] for s in definitions}
            == {p.stem for p in (FIXTURES / "sources").glob("*.compact")}
            == {c["role"] for c in cases},
            "uncovered or missing source",
        )
        require(len(cases) == len({c["id"] for c in cases}) == 24, "case roster differs")
        root_lock = (ROOT / "Cargo.lock").read_bytes()
        root = {(p["name"], p["version"], p.get("source")): p.get("checksum") for p in tomllib.loads(root_lock.decode())["package"]}
        generated, irs, snapshots, consumers = output / "generated", {}, {}, {}
        for spec in definitions:
            role = spec["role"]
            source, target = ROOT / spec["path"], generated / role
            require(digest(source) == spec["sha256"], "source bytes differ: " + role)
            run(
                role + "-generate",
                [
                    compiler,
                    "--target",
                    "rust",
                    "--target",
                    "ts",
                    "--rust-functional-ledger9",
                    "--rust-require-recording",
                    "--skip-zk",
                    source,
                    target,
                ],
            )
            independent = output / "independent" / role
            run(role + "-independent", [scheme, "--emit-rust-ir", "--feature-zkir-v3",
                                         "--skip-ts", "--skip-zk", source, independent])
            for name in ["compact-rust-ir.json", "compact-rust-origins.raw.json"]:
                require(
                    (target / "contract" / name).read_bytes()
                    == (independent / "contract" / name).read_bytes(),
                    "complete independent " + name + " differs: " + role,
                )
            irs[role] = json.loads((target / "contract/compact-rust-ir.json").read_text())
            caps = json.loads((target / "contract/rust-capabilities.json").read_text())["circuits"]
            require(
                {c["name"] for c in caps}
                == set(spec["entries"])
                == {p.stem for p in (target / "zkir").glob("*.zkir")},
                "export/model roster differs",
            )
            require(
                all(
                    c["recorded"]
                    and c["observed_call"]
                    and c["proof_required"]
                    and c["recording_status"] == "available"
                    for c in caps
                ),
                "required recorded API missing",
            )
            programs = program_assets(target / "contract/checked-programs", spec["entries"])
            require(digest(target / "zkir/advance.zkir") != digest(target / "zkir/advance_other.zkir"), "wrong-verifier relation is not distinct")
            for package in ["runtime-rs", "runtime-rs-macros"]:
                files = [ROOT / package / "Cargo.toml", *(ROOT / package / "src").rglob("*.rs")]
                for path in files:
                    require(digest(target / "contract" / path.relative_to(ROOT)) == digest(path), "bundled runtime source differs: " + str(path))
            snapshots[role] = inventory(target)
            report["sources"][role] = {"source_sha256": digest(source), "manifest": source_manifest_snapshot(target, True), "programs": programs}
        runtime = args.ts_runtime.resolve(strict=True)
        modules = output / "node_modules/@midnight-ntwrk"
        modules.mkdir(parents=True)
        (modules / "compact-runtime").symlink_to(runtime, target_is_directory=True)
        (output / "package.json").write_text('{"type":"module"}\n')
        ts = output / "typescript.json"
        run(
            "typescript",
            [
                "node",
                FIXTURES / "capture.mjs",
                "--generated",
                generated,
                "--runtime",
                runtime,
                "--version",
                json.loads((ROOT / "runtime/package.json").read_text())["version"],
                "--output",
                ts,
            ],
        )
        reference = admit_oracle(ts, generated, runtime, definitions)
        require(reference["status"] == "passed" and {c["id"] for c in reference["rows"]} == {c["id"] for c in cases}, "incomplete TS execution")
        env["COMPACT_CONSTRUCTOR_TS"] = str(ts)
        report["typescript"] = {"path": str(ts), "sha256": digest(ts), "wrappers": reference["wrappers"], "inputs": reference["inputs"]}
        for spec in definitions:
            role, sdk = spec["role"], generated / spec["role"] / "contract"
            package = tomllib.loads((sdk / "Cargo.toml").read_text())["package"]
            allowed = {(package["name"], package["version"], None), ("constructor-vector-" + role.replace("_", "-"), "0.1.0", None)}
            strict = output / "strict-sdk" / role
            shutil.copytree(sdk, strict)
            before = inventory(strict)
            (strict / "Cargo.lock").write_bytes(root_lock)
            metadata = run(role + "-sdk-resolve", ["cargo", "metadata", "--offline", "--format-version=1"], cwd=strict)
            check_lock(strict / "Cargo.lock", metadata, root, allowed, [strict])
            run(
                role + "-sdk-clippy",
                [
                    "cargo",
                    "clippy",
                    "--offline",
                    "--locked",
                    "--all-targets",
                    "--all-features",
                    "--",
                    "-D",
                    "warnings",
                ],
                cwd=strict,
            )
            require(all(digest(strict / path) == sha for path, sha in before.items()), "strict SDK source changed")
            consumer = output / "consumers" / role
            consumer.mkdir(parents=True)
            for source, name in [
                (FIXTURES / "consumer.rs", "consumer.rs"),
                (FIXTURES / "cases.json", "cases.json"),
                (
                    FIXTURES / "constructors" / (spec["constructor_consumer"] + ".rs"),
                    "constructor.rs",
                ),
            ]:
                shutil.copyfile(source, consumer / name)
            (consumer / "Cargo.lock").write_bytes(root_lock)
            manifest = (
                '[workspace]\n[package]\nname="constructor-vector-'
                + role.replace("_", "-")
                + '"\nversion="0.1.0"\nedition="2024"\n[lib]\npath="consumer.rs"\n[dependencies]\nserde_json="1"\ngenerated={package='
                + json.dumps(package["name"])
                + ',path='
                + json.dumps(str(sdk))
                + ',features=["ledger-transaction"]}\n'
            )
            (consumer / "Cargo.toml").write_text(manifest)
            metadata = run(role + "-resolve", ["cargo", "metadata", "--offline", "--format-version=1"], cwd=consumer)
            check_lock(consumer / "Cargo.lock", metadata, root, allowed, [consumer, sdk])
            env["COMPACT_CONSTRUCTOR_ROLE"] = role
            env["COMPACT_CONSTRUCTOR_PROGRAM_ROOT"] = str(sdk / "checked-programs")
            p = run(
                role + "-consumer",
                [
                    "cargo",
                    "test",
                    "--offline",
                    "--locked",
                    "--lib",
                    "constructor_cases",
                    "--",
                    "--exact",
                    "--nocapture",
                ],
                cwd=consumer,
            )
            count = sum(c["role"] == role for c in cases)
            require(
                f"constructor-cases:{count}" in p.stdout
                and "test result: ok. 1 passed; 0 failed; 0 ignored;" in p.stdout,
                "all native cases must execute",
            )
            run(
                role + "-clippy",
                [
                    "cargo",
                    "clippy",
                    "--offline",
                    "--locked",
                    "--all-targets",
                    "--all-features",
                    "--",
                    "-D",
                    "warnings",
                ],
                cwd=consumer,
            )
            consumers[role] = consumer
        report["identity_controls"] = {}
        duplicate = output / "duplicate-program-root"
        shutil.copytree(generated / "parameter/contract/checked-programs", duplicate)
        files = sorted(duplicate.glob("*.json"))
        value = json.loads(files[1].read_text())
        value["root"] = json.loads(files[0].read_text())["root"]
        files[1].write_text(json.dumps(value) + "\n")
        try:
            program_assets(duplicate, ["read", "advance", "advance_other"])
        except ValueError as e:
            require("duplicate or unexpected Program root" in str(e), "wrong duplicate-root refusal")
            report["identity_controls"]["duplicate_root"] = str(e)
        else:
            raise ValueError("duplicate Program root was accepted")
        swapped = output / "swapped-program-assets"
        shutil.copytree(generated / "parameter", swapped / "generated")
        assets = swapped / "generated/contract/checked-programs"
        files = sorted(assets.glob("*.json"))
        first, second = files[0].read_bytes(), files[1].read_bytes()
        files[0].write_bytes(second)
        files[1].write_bytes(first)
        # Only isolated metadata is corrupted. The positive generated SDKs and
        # all generated Rust remain untouched. Roots remain unique, so the real
        # public-constant consumer must catch the swapped associations.
        program_assets(assets, ["read", "advance", "advance_other"])
        swapped_consumer = swapped / "consumer"
        shutil.copytree(consumers["parameter"], swapped_consumer)
        manifest = swapped_consumer / "Cargo.toml"
        manifest.write_text(manifest.read_text().replace(str(generated / "parameter/contract"), str(swapped / "generated/contract")))
        run("swapped-program-resolve", ["cargo", "metadata", "--offline", "--format-version=1"], cwd=swapped_consumer)
        env["COMPACT_CONSTRUCTOR_ROLE"] = "parameter"
        env["COMPACT_CONSTRUCTOR_PROGRAM_ROOT"] = str(assets)
        rejected = run(
            "swapped-program-consumer",
            [
                "cargo",
                "test",
                "--offline",
                "--locked",
                "--lib",
                "constructor_cases",
                "--",
                "--exact",
            ],
            cwd=swapped_consumer,
            success=False,
        )
        require(
            rejected.returncode
            and "test result: FAILED. 0 passed; 1 failed;" in rejected.stdout
            and "exported Program root" in rejected.stdout,
            "swapped public Program mapping escaped actual consumer",
        )
        report["identity_controls"]["swapped_programs"] = inventory(assets)
        changed = output / "changed-oracle.json"
        altered = copy.deepcopy(reference)
        next(row for row in altered["rows"] if row["id"] == "parameter")["stored"] = "999"
        changed.write_text(json.dumps(altered) + "\n")
        sentinel = output / "untrusted-oracle-consumed"
        altered_environment = dict(env, COMPACT_CONSTRUCTOR_TS=str(changed))
        try:
            run(
                "changed-oracle-before-consumption",
                [
                    sys.executable,
                    "-c",
                    "from pathlib import Path; Path(" + repr(str(sentinel)) + ").touch()",
                ],
                environment=altered_environment,
            )
        except ValueError as e:
            require("TS oracle path changed before consumption" in str(e) and not sentinel.exists(), "changed oracle reached child execution")
            report["identity_controls"]["changed_oracle"] = {"refusal": str(e), "sha256": digest(changed), "child_started": False}
        else:
            raise ValueError("untrusted TS oracle was consumed")
        try:
            check_oracle_consumption(dict(report["typescript"], path=str(changed)), altered_environment)
        except ValueError as e:
            require("TS oracle bytes changed before consumption" in str(e), "wrong oracle byte-binding refusal")
            report["identity_controls"]["changed_oracle_bytes"] = str(e)
        else:
            raise ValueError("changed oracle bytes matched the retained receipt")
        report["compile_fail"] = {}
        for spec in json.loads((FIXTURES / "misuse.json").read_text()):
            name = spec["name"]
            consumer = output / "misuse" / name
            shutil.copytree(consumers[spec["role"]], consumer)
            (consumer / "src/bin").mkdir(parents=True)
            for suffix in ["", "_twin"]:
                shutil.copyfile(FIXTURES / "misuse" / (name + suffix + ".rs"), consumer / "src/bin" / (name + suffix + ".rs"))
            run(name + "-twin", ["cargo", "run", "--offline", "--locked", "--bin", name + "_twin"], cwd=consumer)
            p = run(name, ["cargo", "check", "--offline", "--locked", "--bin", name, "--message-format=json"], cwd=consumer, success=False)
            report["compile_fail"][name] = compile_failure(p, name, spec["code"], (FIXTURES / "misuse" / (name + ".rs")).read_text())
        report["semantic_controls"] = {}
        mutant = output / "changed-replay-expectation"
        shutil.copytree(consumers["parameter"], mutant)
        consumer_source = mutant / "consumer.rs"
        text = consumer_source.read_text()
        old = "Field::from(expected), // replay-state-expectation"
        # A marked replayed-state expectation leaves initial/native final-state
        # expectations unchanged; do not rely on textual occurrence order.
        require(text.count(old) == 1, "replay expectation premise changed")
        consumer_source.write_text(text.replace(old, "Field::from(expected + 1), // replay-state-expectation"))
        env["COMPACT_CONSTRUCTOR_ROLE"] = "parameter"
        env["COMPACT_CONSTRUCTOR_PROGRAM_ROOT"] = str(generated / "parameter/contract/checked-programs")
        rejected = run(
            "changed-replay-expectation",
            [
                "cargo",
                "test",
                "--offline",
                "--locked",
                "--lib",
                "constructor_cases",
                "--",
                "--exact",
            ],
            cwd=mutant,
            success=False,
        )
        require(
            rejected.returncode
            and "test result: FAILED. 0 passed; 1 failed;" in rejected.stdout
            and "assertion `left == right` failed" in rejected.stdout,
            "changed replay expectation did not execute",
        )
        report["semantic_controls"]["replay_expectation"] = digest(consumer_source)
        for role, old, new in [
                ("owned_reuse", "return input;", "return [input[1], input[0]];"),
                ("checked_helper", 'assert(ok, "iterable-helper-denied");', ""),
                ("conditional_witness", "flag ? first() : second()", "flag ? second() : first()"),
                ("nested_capture", "stored = 7;", "stored = 8;")]:
            # Regenerate complete source mutations. Never edit generated Rust,
            # trim exports, or substitute native failures for TS observations.
            spec = next(row for row in definitions if row["role"] == role)
            text = (ROOT / spec["path"]).read_text()
            require(text.count(old) == 1, "source mutation premise changed: " + role)
            folder = output / (role + "-producer-mutation")
            folder.mkdir()
            source = folder / (role + ".compact")
            source.write_text(text.replace(old, new))
            run(
                role + "-mutant-generate",
                [
                    compiler,
                    "--target",
                    "rust",
                    "--rust-functional-ledger9",
                    "--rust-require-recording",
                    "--skip-zk",
                    source,
                    folder / "generated",
                ],
            )
            consumer = folder / "consumer"
            shutil.copytree(consumers[role], consumer)
            manifest = consumer / "Cargo.toml"
            manifest.write_text(manifest.read_text().replace(str(generated / role / "contract"), str(folder / "generated/contract")))
            run(role + "-mutant-resolve", ["cargo", "metadata", "--offline", "--format-version=1"], cwd=consumer)
            env["COMPACT_CONSTRUCTOR_ROLE"] = role
            env["COMPACT_CONSTRUCTOR_PROGRAM_ROOT"] = str(folder / "generated/contract/checked-programs")
            rejected = run(
                role + "-mutant-consumer",
                [
                    "cargo",
                    "test",
                    "--offline",
                    "--locked",
                    "--lib",
                    "constructor_cases",
                    "--",
                    "--exact",
                ],
                cwd=consumer,
                success=False,
            )
            require(
                rejected.returncode
                and "test result: FAILED. 0 passed; 1 failed;" in rejected.stdout
                and "assertion `left == right` failed" in rejected.stdout,
                "same-type complete source mutation escaped behavioral assertions: " + role,
            )
            report["semantic_controls"][role] = {"source_sha256": digest(source), "generated": inventory(folder / "generated")}
        report["ir_refusals"] = {}
        backend = ["cargo", "run", "--locked", "--offline", "--quiet", "-p", "compact-rust-backend", "--bin", "compact-rust-backend"]
        for name, malformed, expected in malformed_cases(irs):
            text = json.dumps(malformed, indent=2) + "\n"
            (output / (name + "-ir.json")).write_text(text)
            p = run(name + "-ir", backend, success=False, stdin=text)
            require(p.returncode != 0 and all(piece in p.stderr for piece in expected), "wrong native IR refusal: " + name)
            report["ir_refusals"][name] = {"input_sha256": digest(output / (name + "-ir.json")), "stderr": p.stderr}
        for role, before in snapshots.items():
            require(inventory(generated / role) == before, "generated source changed: " + role)
        require(digest(ts) == report["typescript"]["sha256"], "consumed TS oracle changed")
        admit_oracle(ts, generated, runtime, definitions)
        require({p: digest(ROOT / p) for p in pins} == pins and all(digest(p) == sha for p, sha in tools.items()), "source/tool bytes changed")
        require(git("rev-parse", "HEAD") == head and not git("status", "--porcelain"), "source HEAD changed")
        report.update(
            status="passed",
            complete_sources=14,
            cases=24,
            successful_constructor_cases=17,
            constructor_error_cases=7,
            post_initialization_routes=68,
            public_replays=34,
            fresh_proofs=0,
        )
    except Exception as e:
        report.update(status="failed", error=str(e))
        raise
    finally:
        report["seconds"] = time.monotonic() - start
        save()
        print(
            json.dumps(
                {
                    "status": report["status"],
                    "seconds": report["seconds"],
                    "commands": len(report["commands"]),
                    "error": report.get("error"),
                }
            )
        )


if __name__ == "__main__":
    main()
