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

"""Emit the two upstream secp256k1 slow fixtures and measure their proof models."""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[4]
FAMILY = Path(__file__).resolve().parent
PRAGMA = b"pragma language_version >= 0.25.0;\n"
FIXTURES = {
    "prove_ethereum_address": ["storeEthereumAddress"],
    "verify_sequential": ["two", "three"],
}


def pin(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler", required=True, type=Path)
    parser.add_argument("--zkir", required=True, type=Path)
    parser.add_argument("--ts-runtime", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--allow-pragma-probe", action="store_true")
    args = parser.parse_args()
    compiler = args.compiler.resolve(strict=True)
    zkir = args.zkir.resolve(strict=True)
    runtime = args.ts_runtime.resolve(strict=True)
    out = args.output.absolute()
    out.mkdir()
    namespace = out / "node_modules/@midnight-ntwrk"
    namespace.mkdir(parents=True)
    (namespace / "compact-runtime").symlink_to(runtime, target_is_directory=True)
    command = [compiler, "--target", "ts", "--target", "rust", "--feature-zkir-v3",
               "--skip-zk", "--rust-runtime-root", ROOT]
    report = {}
    for family, exports in FIXTURES.items():
        source, = (FAMILY / family).glob("*.compact")
        work = out / family
        work.mkdir()

        def compile_to(input_path, output_path, label):
            result = subprocess.run([*command, input_path, output_path], cwd=ROOT, capture_output=True)
            (work / (label + ".stdout")).write_bytes(result.stdout)
            (work / (label + ".stderr")).write_bytes(result.stderr)
            (work / (label + ".exit")).write_text(f"{result.returncode}\n")
            return result

        selected = source
        if args.allow_pragma_probe:
            intact = work / "intact"
            result = compile_to(source, intact, "intact")
            if result.returncode != 255 or b"language version 0.23.105 mismatch" not in result.stderr or intact.exists():
                raise RuntimeError(f"{source.name} did not refuse the exact version pragma")
            original = source.read_bytes()
            if original.count(PRAGMA) != 1:
                raise RuntimeError(f"{source.name} must contain exactly one pinned language pragma")
            selected = work / source.name
            selected.write_bytes(original.replace(PRAGMA, b"", 1))

        build = work / ".build"
        result = compile_to(selected, build, "compile")
        result.check_returncode()
        info = json.loads((build / "compiler/contract-info.json").read_bytes())
        if [c["name"] for c in info["circuits"] if c["proof"]] != exports:
            raise RuntimeError(f"{source.name} proof exports differ from the pinned fixture")
        model = subprocess.run([zkir, "mock-compile-many", build / "zkir"], check=True, capture_output=True)
        (work / "models.log").write_bytes(model.stdout + model.stderr)
        print((model.stdout + model.stderr).decode(), end="")
        report[family] = {
            "original_sha256": pin(source), "selected_sha256": pin(selected),
            "files": {str(file.relative_to(build)): pin(file)
                      for file in sorted(build.rglob("*")) if file.is_file()},
        }
    (out / "artifacts.json").write_text(json.dumps(report, indent=2) + "\n")


if __name__ == "__main__":
    main()
