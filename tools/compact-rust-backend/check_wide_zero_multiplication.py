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

"""Fresh wide-zero API/TypeScript/compile-fail gate; proofs run separately.

Consumers use the complete unedited compiler output. Inputs, emitted assets,
compiler/runtime identities and resolved Cargo checksums are retained and checked.
"""
import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import time
import tomllib

from check_constant_vector_projection import compile_failure, digest
from check_functional_effects import generated_manifest_snapshot, scheme_compiler

ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "examples/rust_backend/wide_zero_multiplication.compact"
FIXTURES = Path(__file__).with_name("tests") / "wide_zero_multiplication"
NEGATIVES = ("wrong_wide", "wrong_zero")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--ts-runtime", type=Path, required=True)
    args = parser.parse_args()
    output, report_path = args.output.absolute(), args.report.absolute()
    for path in (output, report_path):
        if path.exists() or path.is_symlink():
            parser.error(f"destination must be absent: {path}")
    compiler = args.compiler.resolve(strict=True)
    environment = dict(os.environ)
    environment.setdefault("RUSTUP_TOOLCHAIN", "1.99.0")
    scheme = scheme_compiler(compiler, environment)
    environment["COMPACTC_SCHEME"] = str(scheme)
    premises = [SOURCE, Path(__file__), ROOT / "Cargo.lock", ROOT / "runtime/package.json",
                Path(__file__).with_name("check_constant_vector_projection.py"),
                Path(__file__).with_name("check_functional_effects.py")]
    for folder in (ROOT / "runtime-rs", ROOT / "runtime-rs-macros",
                   ROOT / "tools/compact-rust-backend/src", FIXTURES):
        premises.extend(p for p in folder.rglob("*") if p.is_file() and p.suffix in (".rs", ".toml", ".mjs", ".json"))
    pins = {str(path.relative_to(ROOT)): digest(path) for path in premises}
    tools = {str(path): digest(path) for path in (compiler, scheme)}
    output.mkdir(parents=True)
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report = {"status": "incomplete", "scope": "wide-zero source/consumer controls; no proof/fullCI verdict",
              "premises": pins, "tools": tools, "commands": []}
    started = time.monotonic()

    def run(label, command, success=True, env=None):
        command = list(map(str, command))
        began = time.monotonic()
        process = subprocess.run(command, cwd=ROOT, env=environment if env is None else env,
                                 capture_output=True, text=True, timeout=300)
        (output / f"{label}.stdout").write_text(process.stdout)
        (output / f"{label}.stderr").write_text(process.stderr)
        report["commands"].append({"label": label, "command": command, "exit": process.returncode,
                                   "seconds": time.monotonic() - began})
        if success and process.returncode:
            raise RuntimeError(f"{label} failed; see retained stdout/stderr")
        return process

    try:
        generated = output / "generated"
        run("generate", [compiler, "--target", "rust", "--target", "ts", "--rust-functional-ledger8",
                         "--rust-require-recording", "--rust-runtime-root", ROOT,
                         "--skip-zk", SOURCE.relative_to(ROOT), generated])
        report["generated"] = generated_manifest_snapshot(generated)
        runtime = args.ts_runtime.resolve(strict=True)
        modules = output / "node_modules/@midnight-ntwrk"
        modules.mkdir(parents=True)
        (modules / "compact-runtime").symlink_to(runtime, target_is_directory=True)
        (output / "package.json").write_text('{"type":"module"}\n')
        capture = output / "typescript.json"
        run("typescript", ["node", FIXTURES / "capture.mjs", "--generated", generated / "contract/index.js",
                           "--runtime", runtime, "--version",
                           json.loads((ROOT / "runtime/package.json").read_text())["version"],
                           "--output", capture])
        observations = json.loads(capture.read_text())
        report["typescript_sha256"] = digest(capture)
        environment["COMPACT_WIDE_ZERO_TS_REFERENCE"] = str(capture)
        consumer = output / "consumer"
        (consumer / "tests").mkdir(parents=True)
        (consumer / "src/bin").mkdir(parents=True)
        (consumer / "src/lib.rs").write_text("")
        shutil.copyfile(FIXTURES / "consumer.rs", consumer / "tests/behavior.rs")
        package = tomllib.loads((generated / "contract/Cargo.toml").read_text())["package"]["name"]
        lock = tomllib.loads((ROOT / "Cargo.lock").read_text())
        serde = next(row["version"] for row in lock["package"] if row["name"] == "serde_json")
        lines = ['[workspace]', '[package]', 'name="wide-zero-source-consumer"', 'version="0.0.0"',
                 'edition="2024"', '[dependencies]',
                 f'wide_zero_fixture={{package={json.dumps(package)},path={json.dumps(str(generated / "contract"))},features=["ledger-transaction"]}}',
                 f'serde_json="={serde}"', '[features]']
        lines.extend(f'{name}=[]' for name in NEGATIVES)
        for name in (*NEGATIVES, "positive"):
            shutil.copyfile(FIXTURES / f"{name}.rs", consumer / f"src/bin/{name}.rs")
            lines.extend(['[[bin]]', f'name="{name}"', f'path="src/bin/{name}.rs"'])
            if name in NEGATIVES:
                lines.append(f'required-features=["{name}"]')
        manifest = consumer / "Cargo.toml"
        manifest.write_text("\n".join(lines) + "\n")
        shutil.copyfile(ROOT / "Cargo.lock", consumer / "Cargo.lock")
        metadata = json.loads(run("resolve", ["cargo", "metadata", "--offline", "--format-version", "1",
                                               "--manifest-path", manifest]).stdout)
        allowed = {p.resolve() for p in (manifest, generated / "contract/Cargo.toml",
                   ROOT / "runtime-rs/Cargo.toml", ROOT / "runtime-rs-macros/Cargo.toml")}
        locked = {(p["name"], p["version"], p.get("source")): p.get("checksum") for p in lock["package"]}
        for item in metadata["packages"]:
            identity = (item["name"], item["version"], item.get("source"))
            if item.get("source") is None:
                if Path(item["manifest_path"]).resolve() not in allowed:
                    raise ValueError(f"unexpected local consumer dependency: {identity}")
            elif identity not in locked:
                raise ValueError(f"dependency differs from repository lock: {identity}")
        child_lock = consumer / "Cargo.lock"
        for item in tomllib.loads(child_lock.read_text())["package"]:
            if item.get("source"):
                identity = (item["name"], item["version"], item["source"])
                if identity not in locked or item.get("checksum") != locked[identity]:
                    raise ValueError(f"consumer lock identity/checksum differs: {identity}")
        report["consumer_lock_sha256"] = digest(child_lock)
        result = run("consumer", ["cargo", "test", "--locked", "--offline", "--manifest-path", manifest,
                                   "--test", "behavior"]).stdout
        if re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored", result) != [("6", "0", "0")]:
            raise ValueError("unexpected generated-consumer accounting")
        run("positive", ["cargo", "run", "--locked", "--offline", "--manifest-path", manifest, "--bin", "positive"])
        report["compile_fail"] = {}
        for name in NEGATIVES:
            process = run(name, ["cargo", "check", "--locked", "--offline", "--manifest-path", manifest,
                                "--bin", name, "--features", name, "--message-format=json"], success=False)
            report["compile_fail"][name] = compile_failure(
                process, name, "E0308", (consumer / f"src/bin/{name}.rs").read_text())
        changed = json.loads(capture.read_text())
        changed["rows"]["pure"][0]["result"] = "1"
        wrong = output / "typescript-changed-result.json"
        wrong.write_text(json.dumps(changed))
        refused = run("changed-ts-result", ["cargo", "test", "--locked", "--offline", "--manifest-path", manifest,
                      "--test", "behavior", "genuine_typescript_capture_agrees_with_wide_zero_source_results", "--", "--exact"],
                      success=False, env=dict(environment, COMPACT_WIDE_ZERO_TS_REFERENCE=str(wrong)))
        if not refused.returncode or not re.search(r"test result: FAILED\. 0 passed; 1 failed; 0 ignored", refused.stdout):
            raise ValueError("changed TS result did not fail the actual Rust consumer")
        report["changed_ts_result_rejected"] = True
        if generated_manifest_snapshot(generated) != report["generated"]:
            raise ValueError("generated assets changed during consumer execution")
        if digest(child_lock) != report["consumer_lock_sha256"] or digest(capture) != report["typescript_sha256"]:
            raise ValueError("lock or TS reference changed")
        for relative, expected in pins.items():
            if digest(ROOT / relative) != expected:
                raise ValueError(f"source premise changed: {relative}")
        for path, expected in tools.items():
            if digest(path) != expected:
                raise ValueError(f"compiler changed: {path}")
        node = observations["node"]
        for path, expected in {**observations["runtime"]["files"], node["path"]: node["sha256"], **node["sharedLibraries"]}.items():
            if digest(path) != expected:
                raise ValueError(f"TS runtime or tool changed: {path}")
        report["status"] = "passed"
    except Exception as error:
        report["error"] = str(error)
        raise
    finally:
        report["seconds"] = time.monotonic() - started
        with report_path.open("x") as stream:
            json.dump(report, stream, indent=2)
            stream.write("\n")
    print(json.dumps({"status": report["status"], "report": str(report_path)}))


if __name__ == "__main__":
    main()
