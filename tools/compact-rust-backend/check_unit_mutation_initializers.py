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

"""Fresh #516 source generation, four Rust routes and ordinary API refusals.

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
FIXTURES = Path(__file__).with_name("tests") / "unit_mutation_initializers"
SOURCE = ROOT / "examples/rust_backend/unit_mutation_initializers.compact"
NEGATIVES = {"wrong_type": "E0308", "wrong_result": "E0308", "double_execute": "E0382",
             "wrong_statement_type": "E0308", "wrong_statement_result": "E0308",
             "double_execute_statement": "E0382"}


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()



def require_clean_source(receipt):
    head = subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    if receipt.get("source_head", head) != head:
        raise RuntimeError("source HEAD changed during qualification")
    receipt["source_head"] = head
    receipt["source_dirty"] = bool(subprocess.check_output(
        ["git", "status", "--porcelain"], cwd=ROOT, text=True))
    if receipt["source_dirty"]:
        raise RuntimeError("clean source checkout required")


def validate_consumer_lock(metadata, child_lock, allowed_local, locked):
    """Keep every isolated consumer within the selected repository dependency cohort."""
    for item in metadata["packages"]:
        identity = (item["name"], item["version"], item.get("source"))
        if item.get("source") is None:
            if Path(item["manifest_path"]).resolve() not in allowed_local:
                raise ValueError(f"unexpected local consumer dependency: {identity}")
        elif identity not in locked:
            raise ValueError(f"consumer dependency differs from repository lock: {identity}")
    for item in tomllib.loads(child_lock.read_text())["package"]:
        if item.get("source"):
            identity = (item["name"], item["version"], item["source"])
            if identity not in locked or item.get("checksum") != locked[identity]:
                raise ValueError(f"consumer lock identity/checksum differs: {identity}")
    return digest(child_lock)


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
    premises = [SOURCE, Path(__file__), ROOT / "Cargo.lock", ROOT / "runtime-rs/Cargo.toml",
                ROOT / "runtime-rs/compatibility.json", ROOT / "runtime-rs-macros/Cargo.toml"]
    for directory in [ROOT / "runtime-rs/src", ROOT / "runtime-rs-macros/src",
                      ROOT / "tools/compact-rust-backend/src", FIXTURES]:
        premises.extend(sorted(directory.rglob("*.rs")))
    premises.extend(sorted(FIXTURES.glob("*.compact")))
    premises.extend(sorted(FIXTURES.glob("*.mjs")))
    premises.extend(sorted(FIXTURES.glob("*.json")))
    premises.extend([ROOT / "runtime/package.json", FIXTURES.parent / "vector_projection/report_libraries.mjs"])
    # Preserve the complete local Scheme source premise during development,
    # including the frontend owner changed by this slice. The selected compiler
    # and its own entry point remain separately pinned below.
    premises.extend(sorted((ROOT / "compiler").rglob("*.ss")))
    premises.extend(sorted((ROOT / "compiler").rglob("*.sls")))
    pins = {str(path.relative_to(ROOT)): digest(path) for path in premises}
    tool_pins = {str(path): digest(path) for path in [compiler, scheme]}
    output.mkdir(parents=True)
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report = {"status": "incomplete", "scope": "Ordered Unit mutation initializer source/consumer gate; no proof or full CI acceptance",
              "premises": pins, "tools": tool_pins, "commands": []}
    started = time.monotonic()

    def run(label, command, *, success=True, env=None):
        command = list(map(str, command))
        began = time.monotonic()
        process = subprocess.run(command, cwd=ROOT, env=environment if env is None else env,
                                 capture_output=True, text=True, timeout=300)
        (output / f"{label}.stdout").write_text(process.stdout)
        (output / f"{label}.stderr").write_text(process.stderr)
        report["commands"].append({"label": label, "command": command, "exit_code": process.returncode,
                                   "seconds": time.monotonic() - began})
        if success and process.returncode:
            raise RuntimeError(f"{label} failed; retained stdout/stderr in {output}")
        return process

    try:
        require_clean_source(report)
        generated = output / "generated"
        run("generate", [compiler, "--target", "rust", "--target", "ts", "--rust-functional-ledger8",
                         "--rust-runtime-root", ROOT, "--skip-zk", SOURCE.relative_to(ROOT), generated])
        report["generated"] = generated_manifest_snapshot(generated)
        runtime = args.ts_runtime.resolve(strict=True)
        modules = output / "node_modules/@midnight-ntwrk"
        modules.mkdir(parents=True)
        (modules / "compact-runtime").symlink_to(runtime, target_is_directory=True)
        (output / "package.json").write_text('{"type":"module"}\n')
        capture = output / "ts-reference.json"
        version = json.loads((ROOT / "runtime/package.json").read_text())["version"]
        run("typescript", ["node", FIXTURES / "capture.mjs", "--generated", generated / "contract/index.js",
                           "--runtime", runtime, "--version", version, "--output", capture])
        environment["COMPACT_UNIT_INITIALIZERS_TS_REFERENCE"] = str(capture)
        environment["COMPACT_UNIT_INITIALIZERS_CASES"] = str(FIXTURES / "cases.json")
        observations = json.loads(capture.read_text())
        report["typescript"] = {"capture": str(capture), "sha256": digest(capture),
                                "cases": len(observations["rows"]),
                                "cost_scope": "Sum of actual TS query calls, including witness-view reads; raw last-query cost retained"}
        # Exercise the actual capture, including its imports and hashing.
        # Injected report names test portability; they are not a Linux run.
        injected = output / "ts-kernel-library-control.json"
        names = ["linux-vdso.so.1", "linux-gate.so.1"]
        capture_command = ["node", "--import", FIXTURES.parent / "vector_projection/report_libraries.mjs",
                           FIXTURES / "capture.mjs", "--generated", generated / "contract/index.js",
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
        package = tomllib.loads((generated / "contract/Cargo.toml").read_text())["package"]["name"]
        report["source_refusals"] = {}
        for name, messages in {
            "pure_effect": ["marked pure but is actually impure"],
            "missing_disclose": ["potential witness-value disclosure must be declared but is not"],
        }.items():
            for target in ("rust", "ts"):
                refused = output / f"refused-{name}-{target}"
                process = run(f"{name}-{target}", [compiler, "--target", target, "--skip-zk",
                              FIXTURES / f"{name}.compact", refused], success=False)
                if not process.returncode or not all(m in process.stderr for m in messages) or refused.exists():
                    raise ValueError(f"source {name} did not refuse at its source type/effect boundary")
                report["source_refusals"][f"{name}-{target}"] = messages
        report["statement_source_admissions"] = {}
        for name in ("statement_block", "unit_trailing_conditional"):
            # Retained source boundaries formerly refused by compiler 0.31.139.
            source = FIXTURES / f"{name}_admission.compact"
            admitted = {}
            for target in ("ts", "rust"):
                destination = output / f"{name}-{target}-source"
                command = [compiler, "--target", target, "--skip-zk"]
                if target == "rust":
                    command += ["--rust-functional-ledger8", "--rust-runtime-root", ROOT]
                run(f"{name}-{target}-source", [*command, source, destination])
                entry = destination / "contract" / ("index.js" if target == "ts" else "lib.rs")
                if not entry.is_file() or not entry.stat().st_size:
                    raise ValueError(f"source admission {name}/{target} omitted its generated entry point")
                admitted[target] = {"entry": str(entry), "sha256": digest(entry), "bytes": entry.stat().st_size}
            report["statement_source_admissions"][name] = {
                "scope": "Exact boundary source admission only; the separate main cohort executes its own cases, not these boundary crates",
                "generated_entries": admitted}
        lock = tomllib.loads((ROOT / "Cargo.lock").read_text())
        serde = next(row["version"] for row in lock["package"] if row["name"] == "serde_json")
        consumer = output / "consumer"
        (consumer / "tests").mkdir(parents=True)
        (consumer / "src/bin").mkdir(parents=True)
        (consumer / "src/lib.rs").write_text("")
        shutil.copyfile(FIXTURES / "consumer.rs", consumer / "tests/unit_initializers.rs")
        lines = ['[workspace]', '[package]', 'name="unit-mutation-initializers-consumer"', 'version="0.0.0"',
                 'edition="2024"', '[dependencies]',
                 f'fixture={{package={json.dumps(package)},path={json.dumps(str(generated / "contract"))}}}',
                 f'serde_json="={serde}"', '[features]']
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
        allowed_local = {manifest.resolve(), (generated / "contract/Cargo.toml").resolve(),
                         (ROOT / "runtime-rs/Cargo.toml").resolve(), (ROOT / "runtime-rs-macros/Cargo.toml").resolve()}
        locked = {(p["name"], p["version"], p.get("source")): p.get("checksum") for p in lock["package"]}
        child_lock = consumer / "Cargo.lock"
        report["consumer_lock_sha256"] = validate_consumer_lock(metadata, child_lock, allowed_local, locked)
        result = run("consumer", ["cargo", "test", "--locked", "--offline", "--manifest-path", manifest,
                                   "--test", "unit_initializers", "--", "--show-output"]).stdout
        if re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored", result) != [("1", "0", "0")]:
            raise ValueError("unexpected generated-consumer accounting")
        run("consumer-clippy", ["cargo", "clippy", "--locked", "--offline", "--manifest-path", manifest,
                               "--test", "unit_initializers", "--", "-D", "warnings"])
        generated_manifest = generated / "contract/Cargo.toml"
        shutil.copyfile(ROOT / "Cargo.lock", generated / "contract/Cargo.lock")
        run("generated-resolve", ["cargo", "metadata", "--offline", "--format-version", "1",
                                 "--manifest-path", generated_manifest])
        run("generated-clippy", ["cargo", "clippy", "--locked", "--offline", "--manifest-path",
                                 generated_manifest, "--all-targets", "--", "-D", "warnings"])
        for name, admission in report["statement_source_admissions"].items():
            library = Path(admission["generated_entries"]["rust"]["entry"])
            boundary_manifest = library.with_name("Cargo.toml")
            boundary_lock = library.with_name("Cargo.lock")
            shutil.copyfile(ROOT / "Cargo.lock", boundary_lock)
            metadata = json.loads(run(f"{name}-resolve", ["cargo", "metadata", "--offline", "--format-version", "1",
                                  "--manifest-path", boundary_manifest]).stdout)
            allowed = {boundary_manifest.resolve(), (ROOT / "runtime-rs/Cargo.toml").resolve(),
                       (ROOT / "runtime-rs-macros/Cargo.toml").resolve()}
            pin = validate_consumer_lock(metadata, boundary_lock, allowed, locked)
            run(f"{name}-cargo-check", ["cargo", "check", "--locked", "--offline",
                                      "--manifest-path", boundary_manifest, "--all-targets"])
            run(f"{name}-clippy", ["cargo", "clippy", "--locked", "--offline",
                                  "--manifest-path", boundary_manifest, "--all-targets", "--", "-D", "warnings"])
            if digest(boundary_lock) != pin:
                raise ValueError(f"source admission {name} lock changed after resolution")
            admission["rust_crate_compilation"] = {"manifest": str(boundary_manifest), "lock_sha256": pin,
                                                    "status": "passed", "strict_clippy": "passed"}
            admission["scope"] = "TS/Rust source admission and isolated Rust crate compilation/strict Clippy; no exact boundary execution or proof claim"
            for entry in admission["generated_entries"].values():
                if digest(entry["entry"]) != entry["sha256"]:
                    raise ValueError(f"source admission {name} generated entry changed during compilation")
        report["rust_consumer_tests"] = 1
        measured = re.findall(r"COMPACT_UNIT_INITIALIZERS (\d+) (\d+) (\d+)", result)
        if measured != [("95", "344", "126")]:
            raise ValueError(f"unexpected measured route accounting: {measured}")
        report["measured"] = {"cases": 95, "routes": 344, "replays": 126}
        changed = json.loads(capture.read_text())
        next(row for row in changed["rows"] if row["id"] == "conditional-0")["output"]["valueAtoms"] = [[42]]
        wrong = output / "ts-wrong-result.json"
        wrong.write_text(json.dumps(changed, indent=2) + "\n")
        environment["COMPACT_UNIT_INITIALIZERS_TS_REFERENCE"] = str(wrong)
        rejected = run("changed-ts-result", ["cargo", "test", "--locked", "--offline",
                       "--manifest-path", manifest, "--test", "unit_initializers",
                       "live_typescript_four_routes_and_pure_error_order", "--", "--exact"],
                       success=False)
        environment["COMPACT_UNIT_INITIALIZERS_TS_REFERENCE"] = str(capture)
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
                    errors.append((item["message"].get("code") or {}).get("code"))
            if not process.returncode or errors != [expected]:
                raise ValueError(f"{name} did not refuse only with {expected}: {errors}")
            report["compile_fail"][name] = errors
        if generated_manifest_snapshot(generated) != report["generated"]:
            raise ValueError("generated assets changed during consumer execution")
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
        require_clean_source(report)
        report["status"] = "passed"
    except Exception as error:
        report["status"] = "failed"
        report["error"] = str(error)
        raise
    finally:
        report["seconds"] = time.monotonic() - started
        with report_path.open("x") as stream:
            json.dump(report, stream, indent=2)
            stream.write("\n")
    print(json.dumps({"status": report["status"], "rust_consumer_tests": 1,
                      "compile_fail": report["compile_fail"], "report": str(report_path)}))


if __name__ == "__main__":
    main()
