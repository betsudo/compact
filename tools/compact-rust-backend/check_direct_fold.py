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

"""Fresh #661 source generation, four Rust routes and ordinary API refusals.

This finite local gate does not generate keys, execute proofs or claim complete
CI acceptance. Generated contracts remain unedited; consumers live separately.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import time
import tomllib

from check_functional_effects import generated_manifest_snapshot, scheme_compiler

ROOT = Path(__file__).resolve().parents[2]
FIXTURES = Path(__file__).with_name("tests") / "direct_fold"
SOURCES = ROOT / "examples/rust_backend/direct_fold"
DEFINITIONS = json.loads((FIXTURES / "sources.json").read_text())
NEGATIVES = {"wrong_capture": "E0308", "wrong_second_length": "E0308",
             "wrong_unit_item": "E0308", "bare_string": "E0308",
             "owned_reuse": "E0382", "program_reuse": "E0382", "wrong_callback": "E0053", "wrong_empty_witness": "E0053"}


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def legacy_contract(length):
    """Complete retained predecessor with explicit fallible operand controls.

    This is raw legacy IR, not new source output or TS-parity evidence. Keep
    the original fixture intact and retain each derived complete input.
    """
    contract = json.loads((FIXTURES / "named-predecessor-ir.json").read_text())
    root = contract["stateful_circuits"][0]
    root["parameters"][1]["ty"]["length"] = length
    root["parameters"].extend({"name": name, "ty": {"kind": "boolean"}}
                              for name in ("seed_ok", "source_ok"))
    fold = root["actions"][0]["bindings"][0]["value"]
    fold["length"] = length
    for operand, name in (("initial", "seed"), ("source", "source")):
        fold[operand] = {"kind": "sequence", "steps": [
            {"kind": "assert", "condition": {"kind": "parameter", "name": f"{name}_ok"},
             "message": f"{name} failed"}], "value": fold[operand]}
    reducer = contract["circuits"][0]
    reducer["body"] = {"kind": "sequence", "steps": [{
        "kind": "assert", "condition": {"kind": "not_equal",
            "left": {"kind": "parameter", "name": "item"},
            "right": {"kind": "field_literal", "value": "7"}},
        "message": "step failed"}], "value": reducer["body"]}
    return contract


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--ts-runtime", type=Path, required=True, help="Execute pinned TS parity using this existing package")
    args = parser.parse_args()
    output, report_path = args.output.absolute(), args.report.absolute()
    for path in [output, report_path]:
        if path.exists() or path.is_symlink():
            parser.error(f"destination must be absent: {path}")
    compiler = args.compiler.resolve(strict=True)
    environment = dict(os.environ)
    environment.setdefault("RUSTUP_TOOLCHAIN", "1.99.0")
    scheme = scheme_compiler(compiler, environment)
    environment["COMPACTC_SCHEME"] = str(scheme)
    premises = [Path(__file__), ROOT / "Cargo.toml", ROOT / "Cargo.lock",
                Path(__file__).with_name("check_functional_effects.py"),
                Path(__file__).with_name("check_compactc_target.py"), ROOT / "runtime-rs/Cargo.toml",
                ROOT / "runtime-rs/compatibility.json", ROOT / "runtime-rs-macros/Cargo.toml"]
    for directory in [ROOT / "runtime-rs/src", ROOT / "runtime-rs-macros/src",
                      ROOT / "tools/compact-rust-backend/src", FIXTURES]:
        premises.extend(sorted(directory.rglob("*.rs")))
    premises.extend(sorted(SOURCES.glob("*.compact")))
    premises.extend(sorted((ROOT / "compiler").glob("*.ss")))
    premises.extend(sorted(FIXTURES.glob("*.json")))
    premises.extend(sorted(FIXTURES.glob("*.compact")))
    premises.extend(sorted(FIXTURES.glob("*.mjs")))
    premises.extend([ROOT / "runtime/package.json", FIXTURES.parent / "vector_projection/report_libraries.mjs"])
    pins = {str(path.relative_to(ROOT)): digest(path) for path in premises}
    tool_pins = {str(path): digest(path) for path in [compiler, scheme]}
    output.mkdir(parents=True)
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report = {"status": "incomplete", "scope": "Focused #661 source/consumer gate; no proof or full CI acceptance",
              "premises": pins, "tools": tool_pins, "commands": []}
    started = time.monotonic()

    def run(label, command, *, success=True, env=None, stdin=None):
        command = list(map(str, command))
        began = time.monotonic()
        process = subprocess.run(command, cwd=ROOT, env=environment if env is None else env,
                                 capture_output=True, text=True, timeout=300, input=stdin)
        (output / f"{label}.stdout").write_text(process.stdout)
        (output / f"{label}.stderr").write_text(process.stderr)
        report["commands"].append({"label": label, "command": command, "exit_code": process.returncode,
                                   "seconds": time.monotonic() - began})
        if success and process.returncode:
            raise RuntimeError(f"{label} failed; retained stdout/stderr in {output}")
        return process

    try:
        generated = output / "generated"
        report["generated"] = {}
        for definition in DEFINITIONS:
            name = definition["source"]
            source = SOURCES / f"{name}.compact"
            run(f"generate-{name}", [compiler, "--target", "rust", "--target", "ts",
                 "--rust-require-recording", "--rust-functional-ledger8", "--rust-runtime-root", ROOT,
                 "--skip-zk", source.relative_to(ROOT), generated / name])
            report["generated"][name] = generated_manifest_snapshot(generated / name)
            capabilities = json.loads((generated / name / "contract/rust-capabilities.json").read_text())
            selected = next(row for row in capabilities["circuits"] if row["name"] == "run")
            if not all(selected[key] is True for key in ("recorded", "observed_call", "proof_required")):
                raise ValueError(f"{name} lacks a required execution route")
            programs = list((generated / name / "contract/checked-programs").glob("*.json"))
            if len(programs) != 1:
                raise ValueError(f"{name} did not retain its complete selected program")
        runtime = args.ts_runtime.resolve(strict=True)
        modules = output / "node_modules/@midnight-ntwrk"
        modules.mkdir(parents=True)
        (modules / "compact-runtime").symlink_to(runtime, target_is_directory=True)
        (output / "package.json").write_text('{"type":"module"}\n')
        capture = output / "ts-reference.json"
        version = json.loads((ROOT / "runtime/package.json").read_text())["version"]
        run("typescript", ["node", FIXTURES / "capture.mjs", "--generated", generated,
                           "--runtime", runtime, "--version", version, "--output", capture])
        environment["COMPACT_DIRECT_FOLD_TS_REFERENCE"] = str(capture)
        observations = json.loads(capture.read_text())
        report["typescript"] = {"capture": str(capture), "sha256": digest(capture),
                                "stateful_cases": len(observations["rows"]),
                                "successes": sum(not row["error"] for row in observations["rows"]),
                                "refusals": sum(bool(row["error"]) for row in observations["rows"]),
                                "cost_scope": "Sum of actual TS query calls; no last-query substitution"}
        # Exercise the actual capture, including its imports and hashing.
        # Injected report names test portability; they are not a Linux run.
        injected = output / "ts-kernel-library-control.json"
        names = ["linux-vdso.so.1", "linux-gate.so.1"]
        capture_command = ["node", "--import", FIXTURES.parent / "vector_projection/report_libraries.mjs",
                           FIXTURES / "capture.mjs", "--generated", generated,
                           "--runtime", runtime, "--version", version]
        run("kernel-library-names", [*capture_command, "--output", injected],
            env=dict(environment, COMPACT_VECTOR_REPORT_LIBRARIES=json.dumps(names)))
        if sorted(json.loads(injected.read_text())["node"]["kernelVirtualLibraries"]) != sorted(names):
            raise ValueError("known kernel mapping names were not retained explicitly")
        missing = output / "missing-reported-library.so"
        refused_capture = output / "must-not-capture-missing-library.json"
        if missing.exists():
            raise ValueError("missing-library control premise is not absent")
        refused = run("unknown-library-refusal", [*capture_command, "--output", refused_capture],
                      success=False,
                      env=dict(environment, COMPACT_VECTOR_REPORT_LIBRARIES=json.dumps([str(missing)])))
        if not refused.returncode or "ENOENT" not in refused.stderr or str(missing) not in refused.stderr or refused_capture.exists():
            raise ValueError("unknown missing shared library did not fail before capture publication")
        report["typescript"]["library_report_controls"] = {
            "scope": "actual host capture with injected report names; not Linux qualification",
            "known_kernel_names_retained": names, "unknown_missing_library_refused": True}
        report["source_refusals"] = {}
        for name, messages in {
            "wrong_accumulator": ["no compatible function named step is in scope", "(Boolean, Field)"],
            "wrong_length": ["mismatch in fold-argument vector lengths"],
            "pure_effect": ["circuit run is marked pure but is actually impure", "impure circuit step"],
            "missing_disclose": ["potential witness-value disclosure must be declared but is not"],
        }.items():
            for target in ("rust", "ts"):
                refused = output / f"refused-{name}-{target}"
                process = run(f"{name}-{target}", [compiler, "--target", target, "--skip-zk",
                              FIXTURES / f"{name}.compact", refused], success=False)
                if not process.returncode or not all(m in process.stderr for m in messages) or refused.exists():
                    raise ValueError(f"source {name} did not refuse at its source type/effect boundary")
                report["source_refusals"][f"{name}-{target}"] = messages
        lock = tomllib.loads((ROOT / "Cargo.lock").read_text())
        serde = next(row["version"] for row in lock["package"] if row["name"] == "serde_json")
        consumer = output / "consumer"
        (consumer / "tests").mkdir(parents=True)
        (consumer / "src/bin").mkdir(parents=True)
        (consumer / "src/lib.rs").write_text("")
        shutil.copyfile(FIXTURES / "consumer.rs", consumer / "tests/direct_fold.rs")
        shutil.copyfile(FIXTURES / "legacy_consumer.rs", consumer / "tests/legacy_fold.rs")
        lines = ['[workspace]', '[package]', 'name="direct-fold-consumer"', 'version="0.0.0"',
                 'edition="2024"', '[dependencies]',
                 f'serde_json="={serde}"']
        for definition in DEFINITIONS:
            path = generated / definition["source"] / "contract"
            package = tomllib.loads((path / "Cargo.toml").read_text())["package"]["name"]
            lines.append(f'{definition["alias"]}={{package={json.dumps(package)},path={json.dumps(str(path))}}}')
        legacy_inputs = {}
        for length, alias in ((0, "legacy_empty"), (2, "legacy_two")):
            path = output / alias
            path.mkdir()
            ir_path = path / "input.json"
            ir_path.write_text(json.dumps(legacy_contract(length), indent=2) + "\n")
            rendered = run(f"render-{alias}", ["cargo", "run", "--locked", "--offline", "--quiet",
                           "-p", "compact-rust-backend", "--bin", "compact-rust-backend"],
                           stdin=ir_path.read_text())
            (path / "lib.rs").write_text(rendered.stdout)
            (path / "Cargo.toml").write_text(
                f'[package]\nname="{alias}"\nversion="0.0.0"\nedition="2024"\n'
                '[lib]\npath="lib.rs"\n[features]\n'
                'ledger-transaction=["midnight-compact-runtime/ledger-transaction"]\n'
                f'[dependencies]\nmidnight-compact-runtime={{path={json.dumps(str(ROOT / "runtime-rs"))}}}\n')
            lines.append(f'{alias}={{path={json.dumps(str(path))}}}')
            legacy_inputs.update({str(p): digest(p) for p in path.iterdir()})
        report["legacy"] = {"scope": "Derived complete legacy IR route regression; not TS parity or proofs",
                            "pins": legacy_inputs}
        lines.append('[features]')
        lines.extend(f'{name}=[]' for name in NEGATIVES)
        for name in NEGATIVES:
            shutil.copyfile(FIXTURES / f"{name}.rs", consumer / f"src/bin/{name}.rs")
            lines.extend(['[[bin]]', f'name="{name}"', f'path="src/bin/{name}.rs"',
                          f'required-features=["{name}"]'])
        manifest = consumer / "Cargo.toml"
        manifest.write_text("\n".join(lines) + "\n")
        shutil.copyfile(ROOT / "Cargo.lock", consumer / "Cargo.lock")
        metadata = json.loads(run("resolve", ["cargo", "metadata", "--offline", "--format-version", "1",
                                               "--manifest-path", manifest]).stdout)
        allowed_local = {manifest.resolve(), (ROOT / "runtime-rs/Cargo.toml").resolve(),
                         (ROOT / "runtime-rs-macros/Cargo.toml").resolve()}
        allowed_local.update((generated / row["source"] / "contract/Cargo.toml").resolve() for row in DEFINITIONS)
        allowed_local.update((output / alias / "Cargo.toml").resolve() for alias in ("legacy_empty", "legacy_two"))
        locked = {(p["name"], p["version"], p.get("source")): p.get("checksum") for p in lock["package"]}
        for item in metadata["packages"]:
            identity = (item["name"], item["version"], item.get("source"))
            if item.get("source") is None:
                if Path(item["manifest_path"]).resolve() not in allowed_local:
                    raise ValueError(f"unexpected local consumer dependency: {identity}")
            elif identity not in locked:
                raise ValueError(f"consumer dependency differs from repository lock: {identity}")
        child_lock = consumer / "Cargo.lock"
        for item in tomllib.loads(child_lock.read_text())["package"]:
            if item.get("source"):
                identity = (item["name"], item["version"], item["source"])
                if identity not in locked or item.get("checksum") != locked[identity]:
                    raise ValueError(f"consumer lock identity/checksum differs: {identity}")
        report["consumer_lock_sha256"] = digest(child_lock)
        result = run("consumer", ["cargo", "test", "--locked", "--offline", "--manifest-path", manifest,
                                   "--test", "direct_fold"]).stdout
        if re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored", result) != [("46", "0", "0")]:
            raise ValueError("unexpected generated-consumer accounting")
        report["rust_consumer_tests"] = 46
        result = run("legacy-consumer", ["cargo", "test", "--locked", "--offline", "--manifest-path", manifest,
                                         "--test", "legacy_fold"]).stdout
        if re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored", result) != [("2", "0", "0")]:
            raise ValueError("unexpected legacy-consumer accounting")
        report["legacy"].update(tests=2, cases=16, rust_route_executions=32, successful_recorded_replays=3)
        run("consumer-clippy", ["cargo", "clippy", "--locked", "--offline", "--manifest-path", manifest,
                               "--test", "direct_fold", "--test", "legacy_fold", "--", "-D", "warnings"])
        if (report["typescript"]["stateful_cases"], report["typescript"]["successes"],
                report["typescript"]["refusals"]) != (46, 33, 13):
            raise ValueError("unexpected live TS source-family accounting")

        changed = json.loads(capture.read_text())
        next(row for row in changed["rows"] if row["id"] == "direct_1")["result"]["valueAtoms"] = [[42]]
        wrong = output / "ts-wrong-result.json"
        wrong.write_text(json.dumps(changed, indent=2) + "\n")
        environment["COMPACT_DIRECT_FOLD_TS_REFERENCE"] = str(wrong)
        rejected = run("changed-ts-result", ["cargo", "test", "--locked", "--offline",
                       "--manifest-path", manifest, "--test", "direct_fold",
                       "direct_1_four_routes_match_live_ts", "--", "--exact"],
                       success=False)
        environment["COMPACT_DIRECT_FOLD_TS_REFERENCE"] = str(capture)
        if not rejected.returncode or not re.search(r"test result: FAILED\. 0 passed; 1 failed; 0 ignored", rejected.stdout):
            raise ValueError("changed TS result did not fail the actual Rust consumer")
        report["typescript"]["changed_result_rejected"] = True
        report["compile_fail"] = {}
        for name, expected in NEGATIVES.items():
            process = run(name, ["cargo", "check", "--locked", "--offline", "--manifest-path", manifest,
                                "--bin", name, "--features", name, "--message-format=json"], success=False)
            errors = []
            for line in process.stdout.splitlines():
                try:
                    item = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if item.get("reason") == "compiler-message" and item["message"]["level"] == "error":
                    message = item["message"]
                    if not any(span["file_name"].endswith(f"src/bin/{name}.rs") for span in message["spans"]):
                        raise ValueError(f"{name} error outside intended misuse source")
                    errors.append((message.get("code") or {}).get("code"))
            if not process.returncode or not errors or set(errors) != {expected}:
                raise ValueError(f"{name} did not refuse only with {expected}: {errors}")
            report["compile_fail"][name] = errors
        for definition in DEFINITIONS:
            name = definition["source"]
            if generated_manifest_snapshot(generated / name) != report["generated"][name]:
                raise ValueError(f"generated assets changed during consumer execution: {name}")
        for path, expected in legacy_inputs.items():
            if digest(path) != expected:
                raise ValueError(f"legacy input/output changed during consumer execution: {path}")
        for relative, expected in pins.items():
            if digest(ROOT / relative) != expected:
                raise ValueError(f"source/runtime premise changed: {relative}")
        for tool, expected in tool_pins.items():
            if digest(tool) != expected:
                raise ValueError(f"compiler premise changed: {tool}")
        if digest(child_lock) != report["consumer_lock_sha256"]:
            raise ValueError("consumer lock changed after resolution")
        if digest(capture) != report["typescript"]["sha256"]:
            raise ValueError("TS reference changed during Rust comparison")
        for path, expected in observations["runtime"]["files"].items():
            if digest(path) != expected:
                raise ValueError(f"TS runtime changed during Rust comparison: {path}")
        node = observations["node"]
        for path, expected in {node["path"]: node["sha256"], **node["sharedLibraries"]}.items():
            if digest(path) != expected:
                raise ValueError(f"Node tool/library changed: {path}")
        report["status"] = "passed"
    except Exception as error:
        report["error"] = str(error)
        raise
    finally:
        report["seconds"] = time.monotonic() - started
        with report_path.open("x") as stream:
            json.dump(report, stream, indent=2)
            stream.write("\n")
    print(json.dumps({"status": report["status"], "rust_consumer_tests": report["rust_consumer_tests"],
                      "legacy": {key: report["legacy"][key] for key in ("tests", "cases", "rust_route_executions")},
                      "compile_fail": report["compile_fail"], "report": str(report_path)}))


if __name__ == "__main__":
    main()
