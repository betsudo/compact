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

"""Fresh #674 source generation, four Rust routes and ordinary API refusals.

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
FIXTURES = Path(__file__).with_name("tests") / "vector_projection"
SOURCE = ROOT / "examples/rust_backend/constant_vector_projection.compact"
NEGATIVES = {"wrong_length": "E0308", "wrong_type": "E0308", "double_execute": "E0382",
             "double_context": "E0382"}


def compile_failure(process, name, expected, source):
    """Accept only errors at the marked ordinary misuse in this consumer binary."""
    markers = [i for i, line in enumerate(source.splitlines(), 1) if "// expected-error" in line]
    if len(markers) != 1:
        raise ValueError(f"{name} needs exactly one intended-error marker")
    errors = []
    for line in process.stdout.splitlines():
        try:
            item = json.loads(line)
        except json.JSONDecodeError:
            continue
        if item.get("reason") != "compiler-message" or item["message"]["level"] != "error":
            continue
        message = item["message"]
        primary = [span for span in message.get("spans", []) if span.get("is_primary")]
        if (message.get("code") or {}).get("code") != expected or not primary or not all(
            span["file_name"] == f"src/bin/{name}.rs"
            and span["line_start"] <= markers[0] <= span["line_end"]
            and any(row["text"] == source.splitlines()[markers[0] - 1] for row in span["text"])
            for span in primary
        ):
            raise ValueError(f"{name} error is not {expected} at its intended source expression")
        errors.append({"code": expected, "primary_spans": primary})
    if not process.returncode or not errors:
        raise ValueError(f"{name} did not refuse at its intended source expression")
    return errors


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--ts-runtime", type=Path, help="Execute pinned TS parity using this existing package")
    parser.add_argument("--source", type=Path, default=SOURCE,
                        help="Complete source with the maintained projection consumer API (default: #674)")
    args = parser.parse_args()
    source = args.source.resolve(strict=True)
    source.relative_to(ROOT)  # All admitted complete sources belong to this checkout.
    output, report_path = args.output.absolute(), args.report.absolute()
    for path in [output, report_path]:
        if path.exists() or path.is_symlink():
            parser.error(f"destination must be absent: {path}")
    compiler = args.compiler.resolve(strict=True)
    environment = dict(os.environ)
    environment.setdefault("RUSTUP_TOOLCHAIN", "1.99.0")
    scheme = scheme_compiler(compiler, environment)
    environment["COMPACTC_SCHEME"] = str(scheme)
    premises = [source, Path(__file__), ROOT / "Cargo.lock", ROOT / "runtime-rs/Cargo.toml",
                ROOT / "runtime-rs/compatibility.json", ROOT / "runtime-rs-macros/Cargo.toml"]
    for directory in [ROOT / "runtime-rs/src", ROOT / "runtime-rs-macros/src",
                      ROOT / "tools/compact-rust-backend/src", FIXTURES]:
        premises.extend(sorted(directory.rglob("*.rs")))
    premises.extend(sorted(FIXTURES.glob("*.compact")))
    premises.extend(sorted(FIXTURES.glob("*.mjs")))
    premises.append(ROOT / "runtime/package.json")
    pins = {str(path.relative_to(ROOT)): digest(path) for path in premises}
    tool_pins = {str(path): digest(path) for path in [compiler, scheme]}
    output.mkdir(parents=True)
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report = {"status": "incomplete", "scope": "Focused Vector projection source/consumer gate; no proof or full CI acceptance",
              "source": str(source.relative_to(ROOT)),
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
        generated = output / "generated"
        run("generate", [compiler, "--target", "rust", "--target", "ts", "--rust-functional-ledger8",
                         "--rust-runtime-root", ROOT, "--skip-zk", source.relative_to(ROOT), generated])
        report["generated"] = generated_manifest_snapshot(generated)
        if args.ts_runtime:
            runtime = args.ts_runtime.resolve(strict=True)
            modules = output / "node_modules/@midnight-ntwrk"
            modules.mkdir(parents=True)
            (modules / "compact-runtime").symlink_to(runtime, target_is_directory=True)
            (output / "package.json").write_text('{"type":"module"}\n')
            capture = output / "ts-reference.json"
            version = json.loads((ROOT / "runtime/package.json").read_text())["version"]
            run("typescript", ["node", FIXTURES / "capture.mjs", "--generated", generated / "contract/index.js",
                               "--runtime", runtime, "--version", version, "--output", capture])
            environment["COMPACT_VECTOR_TS_REFERENCE"] = str(capture)
            observations = json.loads(capture.read_text())
            report["typescript"] = {"capture": str(capture), "sha256": digest(capture),
                                    "pure_cases": len(observations["pure"]),
                                    "stateful_cases": len(observations["cases"]),
                                    "refusals": len(observations["refusals"]),
                                    "cost_scope": "Sum of actual TS query calls, including witness-view reads; raw last-query cost retained"}
            # Exercise the actual capture, including its imports and hashing.
            # Injected report names test portability; they are not a Linux run.
            injected = output / "ts-kernel-library-control.json"
            names = ["linux-vdso.so.1", "linux-gate.so.1"]
            capture_command = ["node", "--import", FIXTURES / "report_libraries.mjs",
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
        else:
            environment.pop("COMPACT_VECTOR_TS_REFERENCE", None)
            report["typescript"] = "NOT RUN; no --ts-runtime selected"
        report["source_refusals"] = {}
        for name, message in {
            "out_of_bounds": "index 2 is out-of-bounds for a vector of length 2",
            "empty": "index 0 is out-of-bounds for a vector of length 0",
        }.items():
            refused = output / f"refused-{name}"
            process = run(name, [compiler, "--target", "rust", "--skip-zk", FIXTURES / f"{name}.compact", refused],
                          success=False)
            if not process.returncode or message not in process.stderr or refused.exists():
                raise ValueError(f"source {name} did not refuse at its static vector bound")
            report["source_refusals"][name] = message
        # Exercise the numeric source owners retained by constant bindings:
        # aliases, zero multiplication, genuine checked casts and first failure.
        numeric_generated = output / "numeric-generated"
        run("numeric-generate", [compiler, "--target", "rust",
                                *(["--target", "ts"] if args.ts_runtime else []),
                                "--rust-runtime-root", ROOT, "--skip-zk",
                                FIXTURES / "numeric_aliases.compact", numeric_generated])
        from check_compactc_target import check_manifest
        check_manifest(numeric_generated, require_zkir=False)
        numeric_snapshot = {str(path.relative_to(numeric_generated)): digest(path)
                            for path in numeric_generated.rglob("*") if path.is_file()}
        report["numeric_aliases"] = {"generated": numeric_snapshot, "rust_tests": 5}
        numeric_package = tomllib.loads((numeric_generated / "contract/Cargo.toml").read_text())["package"]["name"]
        if args.ts_runtime:
            numeric_capture = output / "numeric-ts-reference.json"
            run("numeric-typescript", ["node", FIXTURES / "capture_numeric_aliases.mjs",
                                       numeric_generated / "contract/index.js", numeric_capture])
            environment["COMPACT_VECTOR_NUMERIC_TS_REFERENCE"] = str(numeric_capture)
            report["numeric_aliases"]["typescript_capture_sha256"] = digest(numeric_capture)
        else:
            environment.pop("COMPACT_VECTOR_NUMERIC_TS_REFERENCE", None)
            report["numeric_aliases"]["typescript"] = "NOT RUN; no --ts-runtime selected"
        package = tomllib.loads((generated / "contract/Cargo.toml").read_text())["package"]["name"]
        lock = tomllib.loads((ROOT / "Cargo.lock").read_text())
        serde = next(row["version"] for row in lock["package"] if row["name"] == "serde_json")
        consumer = output / "consumer"
        (consumer / "tests").mkdir(parents=True)
        (consumer / "src/bin").mkdir(parents=True)
        (consumer / "src/lib.rs").write_text("")
        shutil.copyfile(FIXTURES / "consumer.rs", consumer / "tests/vector.rs")
        shutil.copyfile(FIXTURES / "numeric_aliases.rs", consumer / "tests/numeric.rs")
        lines = ['[workspace]', '[package]', 'name="constant-vector-consumer"', 'version="0.0.0"',
                 'edition="2024"', '[dependencies]',
                 f'vector_fixture={{package={json.dumps(package)},path={json.dumps(str(generated / "contract"))}}}',
                 f'numeric_fixture={{package={json.dumps(numeric_package)},path={json.dumps(str(numeric_generated / "contract"))}}}',
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
                         (ROOT / "runtime-rs/Cargo.toml").resolve(), (ROOT / "runtime-rs-macros/Cargo.toml").resolve(),
                         (numeric_generated / "contract/Cargo.toml").resolve()}
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
                                   "--test", "vector"]).stdout
        if re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored", result) != [("8", "0", "0")]:
            raise ValueError("unexpected generated-consumer accounting")
        report["rust_consumer_tests"] = 8
        numeric_result = run("numeric-consumer", ["cargo", "test", "--locked", "--offline",
                             "--manifest-path", manifest, "--test", "numeric"]).stdout
        if re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored", numeric_result) != [("5", "0", "0")]:
            raise ValueError("unexpected numeric alias/boundary control accounting")

        if args.ts_runtime:
            changed = json.loads(capture.read_text())
            next(row for row in changed["cases"] if row["id"] == "choose-first")["output"]["valueAtoms"] = [[42]]
            wrong = output / "ts-wrong-result.json"
            wrong.write_text(json.dumps(changed, indent=2) + "\n")
            environment["COMPACT_VECTOR_TS_REFERENCE"] = str(wrong)
            rejected = run("changed-ts-result", ["cargo", "test", "--locked", "--offline",
                           "--manifest-path", manifest, "--test", "vector",
                           "source_selection_updates_the_cell_and_replays_across_four_routes", "--", "--exact"],
                           success=False)
            environment["COMPACT_VECTOR_TS_REFERENCE"] = str(capture)
            if not rejected.returncode or not re.search(r"test result: FAILED\. 0 passed; 1 failed; 0 ignored", rejected.stdout):
                raise ValueError("changed TS result did not fail the actual Rust consumer")
            report["typescript"]["changed_result_rejected"] = True
        report["compile_fail"] = {}
        for name, expected in NEGATIVES.items():
            process = run(name, ["cargo", "check", "--locked", "--offline", "--manifest-path", manifest,
                                "--bin", name, "--features", name, "--message-format=json"], success=False)
            report["compile_fail"][name] = compile_failure(
                process, name, expected, (consumer / f"src/bin/{name}.rs").read_text())
        if {str(path.relative_to(numeric_generated)): digest(path)
            for path in numeric_generated.rglob("*") if path.is_file()} != numeric_snapshot:
            raise ValueError("numeric generated assets changed during execution")
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
        if args.ts_runtime and digest(capture) != report["typescript"]["sha256"]:
            raise ValueError("TS reference changed during Rust comparison")
        if args.ts_runtime:
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
                      "compile_fail": report["compile_fail"], "report": str(report_path)}))


if __name__ == "__main__":
    main()
