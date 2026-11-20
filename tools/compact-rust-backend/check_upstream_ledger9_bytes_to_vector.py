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

"""Qualify the pinned Bytes-to-Vector widening, default and refusal cases."""

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tomllib

from check_upstream_ledger9_vector_casts import (
    compiler_recipe_identity,
    require_unchanged,
    run,
    runtime_identity,
    runtime_recipe_identity,
    sha256,
    source_state,
)


ROOT = Path(__file__).resolve().parents[2]
FAMILY = ROOT / "test-contracts/primitives/bytes/casts"
POSITIVES = ("to_vector", "to_vector_default", "to_vector_wide_element")
REFUSAL = "to_vector_underflow"
DIAGNOSTIC = "cannot cast from type Bytes<4> to type Vector<4, Uint<7>>"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler", type=Path, required=True)
    parser.add_argument("--compiler-recipe", type=Path, required=True)
    parser.add_argument("--ts-runtime", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--cargo-target", type=Path,
                        help="reuse an existing local Cargo cache")
    args = parser.parse_args()
    compiler = args.compiler.resolve(strict=True)
    runtime = args.ts_runtime.resolve(strict=True)
    output = args.output.resolve()
    if args.output.is_symlink() or output.exists():
        parser.error("output must be absent")
    source = source_state()

    source_package = json.loads((ROOT / "runtime/package.json").read_text())
    runtime_package = json.loads((runtime / "package.json").read_text())
    if any(source_package[key] != runtime_package[key]
           for key in ("name", "version", "dependencies")):
        parser.error("TypeScript runtime package differs from source")
    nix_path = shutil.which("nix")
    if not nix_path:
        parser.error("Nix is required for exact-head runtime binding")
    runtime_recipe = runtime_recipe_identity(runtime, Path(nix_path).resolve(strict=True))
    initial_runtime = runtime_identity(runtime)
    scheme = compiler.with_name("compactc-scheme")
    if not scheme.is_file():
        parser.error("paired Scheme compiler is missing")
    recipe = compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme)

    manifest_path = FAMILY / "original-to-vector-sha256.json"
    manifest = json.loads(manifest_path.read_text())
    if manifest["upstream_commit"] != "672bd0f226f47a7d036c5c074ce71fbe1af1925e" \
            or len(manifest["files"]) != 11:
        parser.error("pinned original fixture inventory changed")
    for name, digest in manifest["files"].items():
        if sha256(ROOT / name) != digest:
            parser.error(f"original upstream fixture changed: {name}")

    output.mkdir(parents=True)
    link = output / "node_modules/@midnight-ntwrk/compact-runtime"
    link.parent.mkdir(parents=True)
    link.symlink_to(runtime, target_is_directory=True)
    env = os.environ.copy()
    env["COMPACTC_SCHEME"] = str(scheme)
    generated = {}
    for name in POSITIVES:
        fixture = FAMILY / name / f"bytes_casts_{name}.compact"
        cases = {}
        for mode in ("typescript", "both"):
            destination = output / "generated" / mode / name
            command = [str(compiler), "--target", "ts"]
            if mode == "both":
                command += ["--target", "rust", "--rust-runtime-root", str(ROOT)]
            command += ["--feature-zkir-v3", "--skip-zk", "--compact-path",
                        str(ROOT / "compiler"), str(fixture), str(destination)]
            run(command, output / f"{name}-{mode}.log", env=env)
            contract = destination / "contract"
            result = run(["node", str(FAMILY / "check_to_vector.mjs"),
                          str(contract / "index.js"), str(runtime), name],
                         output / f"{name}-{mode}-runtime.log", env=env)
            if f"{name}: original runtime result passed" not in result:
                raise RuntimeError(f"original TypeScript assertion missing: {name} {mode}")
            cases[mode] = {"typescript_sha256": sha256(contract / "index.js"),
                           "runtime_result": result.strip()}
            if mode == "both":
                cases[mode]["rust_sha256"] = sha256(contract / "lib.rs")
                cases[mode]["rust_ir_sha256"] = sha256(contract / "compact-rust-ir.json")
        if cases["typescript"]["typescript_sha256"] != cases["both"]["typescript_sha256"]:
            raise RuntimeError(f"TypeScript output changed in paired compile: {name}")
        generated[name] = cases

    for mode in ("typescript", "rust"):
        destination = output / "refused" / mode / REFUSAL
        command = [str(compiler), "--target", "ts" if mode == "typescript" else "rust",
                   "--skip-zk", "--compact-path", str(ROOT / "compiler"),
                   str(FAMILY / REFUSAL / f"bytes_casts_{REFUSAL}.compact"),
                   str(destination)]
        result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True)
        message = result.stdout + result.stderr
        (output / f"{REFUSAL}-{mode}-refused.log").write_text(message)
        if result.returncode == 0 or DIAGNOSTIC not in message:
            raise RuntimeError(f"original refusal changed for {mode}: {message}")
        if destination.exists() or list(destination.parent.glob(f".{destination.name}.compactc-stage-*")):
            raise RuntimeError(f"refusal published output in {mode}")

    consumer = output / "rust-consumer"
    (consumer / "src").mkdir(parents=True)
    shutil.copyfile(FAMILY / "consumer_to_vector.rs", consumer / "src/main.rs")
    cargo = ["[package]", 'name = "ledger9-bytes-to-vector-consumer"',
             'version = "0.1.0"', 'edition = "2024"', "[workspace]", "[dependencies]"]
    for name in POSITIVES:
        contract = output / "generated/both" / name / "contract"
        package = tomllib.loads((contract / "Cargo.toml").read_text())["package"]["name"]
        cargo.append(f'{name} = {{ package = {json.dumps(package)}, '
                     f'path = {json.dumps(str(contract))} }}')
    (consumer / "Cargo.toml").write_text("\n".join(cargo) + "\n")
    cargo_target = args.cargo_target.resolve() if args.cargo_target else output / "rust-target"
    cargo_env = dict(env, CARGO_TARGET_DIR=str(cargo_target))
    run(["cargo", "+1.99.0", "generate-lockfile", "--offline", "--manifest-path",
         str(consumer / "Cargo.toml")], output / "rust-lock.log", env=cargo_env)
    rust = run(["cargo", "+1.99.0", "run", "--locked", "--offline", "--manifest-path",
                str(consumer / "Cargo.toml")], output / "rust-run.log", env=cargo_env)
    if "three original Bytes-to-Vector assertions passed" not in rust:
        raise RuntimeError("generated Rust assertion result missing")
    require_unchanged(source, initial_runtime, recipe, runtime,
                      args.compiler_recipe, compiler, scheme)
    receipt = {
        **source, "source_clean": True, "platform": os.uname().sysname + " " + os.uname().machine,
        "upstream_commit": manifest["upstream_commit"], "original_files": 11,
        "manifest_sha256": sha256(manifest_path), "compiler_sha256": sha256(compiler),
        "scheme_sha256": sha256(scheme), "runtime": initial_runtime,
        "runtime_recipe": runtime_recipe, "compiler_recipe": recipe,
        "checker_sha256": sha256(Path(__file__)),
        "identity_library_sha256": sha256(ROOT / "tools/compact-rust-backend/check_upstream_ledger9_vector_casts.py"),
        "cargo_target": str(cargo_target), "generated": generated,
        "original_refusal_on_both_targets": REFUSAL,
        "rust_consumer_sha256": sha256(consumer / "src/main.rs"),
        "rust_lock_sha256": sha256(consumer / "Cargo.lock"),
        "rust_result": rust.strip(),
        "limitations": ["local Darwin arm64; native Linux qualification required",
                        "pure circuit values only; no proof or funded transaction applies",
                        "stateful Bytes-to-Vector widening remains unqualified"],
    }
    (output / "receipt.json").write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n")
    print(json.dumps({"status": "passed", "head": receipt["head"], "output": str(output)}))


if __name__ == "__main__":
    main()
