# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Run the intact upstream Vector casts compile, runtime and refusal family."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tomllib


ROOT = Path(__file__).resolve().parents[2]
FAMILY = ROOT / "test-contracts/primitives/vector/casts"
POSITIVES = (
    "empty", "to_bytes_default", "to_bytes_narrow_element", "to_tuple",
    "to_vector",
)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_state(root: Path = ROOT) -> dict[str, str]:
    def git(*args: str) -> str:
        return subprocess.check_output(["git", *args], cwd=root, text=True).strip()

    if git("status", "--porcelain"):
        raise ValueError("clean committed source required for qualification")
    return {"head": git("rev-parse", "HEAD"),
            "tree": git("rev-parse", "HEAD^{tree}")}


def runtime_identity(runtime: Path) -> dict:
    runtime = runtime.resolve(strict=True)
    dist = runtime / "dist"
    if not (dist / "index.js").is_file():
        raise ValueError("TypeScript runtime dist/index.js is missing")
    files = {}
    for path in sorted(dist.rglob("*")):
        if path.is_symlink():
            raise ValueError(f"runtime dist contains an unsupported entry: {path}")
        if path.is_dir():
            continue
        if not path.is_file():
            raise ValueError(f"runtime dist contains an unsupported entry: {path}")
        files[str(path.relative_to(dist))] = sha256(path)
    return {"path": str(runtime), "package_sha256": sha256(runtime / "package.json"),
            "dist_files": files}


def compiler_recipe_identity(recipe_path: Path, source: dict[str, str],
                             compiler: Path, scheme: Path, root: Path = ROOT) -> dict:
    recipe_path = recipe_path.resolve(strict=True)
    recipe = json.loads(recipe_path.read_text())
    if recipe.get("format") != "compact-original-existing-compiler-recipe/v1" \
            or recipe.get("status") != "passed" \
            or any(recipe.get(key) != source[key] for key in ("head", "tree")):
        raise ValueError("compiler recipe does not bind this clean source head and tree")
    for name, path in (("compactc", compiler), ("compactc-scheme", scheme)):
        recorded = recipe.get(name, {})
        if recorded.get("path") != str(path) or recorded.get("sha256") != sha256(path):
            raise ValueError(f"compiler recipe {name} identity differs")
    if recipe.get("store") != str(compiler.parent.parent):
        raise ValueError("compiler recipe store differs from supplied package")
    pins = recipe.get("inputs_before_use", {})
    for name in ("flake.nix", "flake.lock", "Cargo.lock",
                 "tools/compact-rust-backend/Cargo.lock", "compiler/compiler-version.ss",
                 "tools/compact-rust-backend/record_compiler_recipe.py"):
        path = (root / name).resolve(strict=True)
        if pins.get(str(path)) != sha256(path):
            raise ValueError(f"compiler recipe source pin differs: {name}")
    return {"path": str(recipe_path), "sha256": sha256(recipe_path),
            "head": source["head"], "tree": source["tree"],
            "store": recipe["store"]}


def require_unchanged(source: dict[str, str], runtime_snapshot: dict,
                      recipe_snapshot: dict, runtime: Path, recipe_path: Path,
                      compiler: Path, scheme: Path, root: Path = ROOT) -> None:
    if source_state(root) != source:
        raise ValueError("source head or tree changed during qualification")
    if runtime_identity(runtime) != runtime_snapshot:
        raise ValueError("TypeScript runtime dist changed during qualification")
    if compiler_recipe_identity(recipe_path, source, compiler, scheme, root) != recipe_snapshot:
        raise ValueError("compiler recipe changed during qualification")


def run(command: list[str], log: Path, *, env: dict[str, str]) -> str:
    result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True)
    log.write_text(result.stdout + result.stderr)
    if result.returncode:
        raise RuntimeError(f"{command[0]} exited {result.returncode}: {log}\n"
                           f"{result.stderr[-2000:]}")
    return result.stdout


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler", type=Path, required=True)
    parser.add_argument("--compiler-recipe", type=Path, required=True)
    parser.add_argument("--ts-runtime", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--cargo-target", type=Path,
                        help="reuse an existing Cargo build cache when local disk is constrained")
    args = parser.parse_args()
    compiler = args.compiler.resolve(strict=True)
    runtime = args.ts_runtime.resolve(strict=True)
    output = args.output.resolve()
    if args.output.is_symlink() or output.exists():
        parser.error("output must be absent")
    initial_source = source_state()
    source_package = json.loads((ROOT / "runtime/package.json").read_text())
    runtime_package = json.loads((runtime / "package.json").read_text())
    if any(source_package[key] != runtime_package[key]
           for key in ("name", "version", "dependencies")):
        parser.error("TypeScript runtime package does not match this source tree")
    initial_runtime = runtime_identity(runtime)
    scheme = compiler.with_name("compactc-scheme")
    if not scheme.is_file():
        parser.error("paired Scheme compiler is missing")
    recipe_identity = compiler_recipe_identity(args.compiler_recipe, initial_source,
                                               compiler, scheme)
    manifest_path = FAMILY / "original-source-sha256.json"
    manifest = json.loads(manifest_path.read_text())
    if manifest["upstream_commit"] != "672bd0f226f47a7d036c5c074ce71fbe1af1925e" \
            or len(manifest["files"]) != 21:
        parser.error("pinned original fixture inventory changed")
    for source, digest in manifest["files"].items():
        if sha256(ROOT / source) != digest:
            parser.error(f"original upstream fixture changed: {source}")

    output.mkdir(parents=True)
    link = output / "node_modules/@midnight-ntwrk/compact-runtime"
    link.parent.mkdir(parents=True)
    link.symlink_to(runtime, target_is_directory=True)
    env = os.environ.copy()
    env["COMPACTC_SCHEME"] = str(scheme)
    generated = {}
    for name in POSITIVES:
        source = FAMILY / name / f"vector_casts_{name}.compact"
        cases = {}
        for mode in ("typescript", "both"):
            destination = output / "generated" / mode / name
            command = [str(compiler), "--target", "ts"]
            if mode == "both":
                command += ["--target", "rust", "--rust-runtime-root", str(ROOT)]
            command += ["--feature-zkir-v3", "--skip-zk", "--compact-path",
                        str(ROOT / "compiler"), str(source), str(destination)]
            log = output / f"{name}-{mode}.log"
            run(command, log, env=env)
            contract = destination / "contract"
            result = run(["node", str(FAMILY / "check.mjs"),
                          str(contract / "index.js"), str(runtime), name],
                         output / f"{name}-{mode}-runtime.log", env=env)
            if f"{name}: original runtime result passed" not in result:
                raise RuntimeError(f"original runtime assertion missing for {name} {mode}")
            cases[mode] = {
                "typescript_sha256": sha256(contract / "index.js"),
                "runtime_result": result.strip(),
            }
            if mode == "both":
                cases[mode]["rust_sha256"] = sha256(contract / "lib.rs")
                cases[mode]["rust_ir_sha256"] = sha256(contract / "compact-rust-ir.json")
        if cases["typescript"]["typescript_sha256"] != cases["both"]["typescript_sha256"]:
            raise RuntimeError(f"TypeScript output changed in paired compile: {name}")
        generated[name] = cases

    refusals = {
        "to_bytes_field_element": "cannot cast from type Vector<1, Field> to type Bytes<1>",
        "to_bytes_overflow": "cannot cast from type Vector<4, Uint<16>> to type Bytes<4>",
        "to_vector_length_mismatch": "cannot cast from type [Boolean, Boolean, Boolean, Boolean, Boolean, Boolean]",
    }
    for name, diagnostic in refusals.items():
        refusal = FAMILY / name / f"vector_casts_{name}.compact"
        for mode in ("typescript", "rust"):
            destination = output / "refused" / mode / name
            command = [str(compiler), "--target", "ts" if mode == "typescript" else "rust",
                       "--skip-zk", "--compact-path", str(ROOT / "compiler"),
                       str(refusal), str(destination)]
            result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True)
            message = result.stdout + result.stderr
            (output / f"{name}-{mode}-refused.log").write_text(message)
            if result.returncode == 0 or diagnostic not in message:
                raise RuntimeError(f"original Vector cast refusal changed for {name} {mode}: {message}")
            if destination.exists() or list(destination.parent.glob(f".{destination.name}.compactc-stage-*")):
                raise RuntimeError(f"Vector cast refusal published output for {name} {mode}")

    consumer = output / "rust-consumer"
    (consumer / "src").mkdir(parents=True)
    shutil.copyfile(FAMILY / "consumer.rs", consumer / "src/main.rs")
    lines = ["[package]", 'name = "ledger9-vector-casts-consumer"',
             'version = "0.1.0"', 'edition = "2024"', "[workspace]", "[dependencies]"]
    for name in POSITIVES:
        contract = output / "generated/both" / name / "contract"
        package = tomllib.loads((contract / "Cargo.toml").read_text())["package"]["name"]
        lines.append(f'{name} = {{ package = {json.dumps(package)}, '
                     f'path = {json.dumps(str(contract))} }}')
    (consumer / "Cargo.toml").write_text("\n".join(lines) + "\n")
    cargo_target = args.cargo_target.resolve() if args.cargo_target else output / "rust-target"
    cargo_env = dict(env, CARGO_TARGET_DIR=str(cargo_target))
    run(["cargo", "+1.99.0", "generate-lockfile", "--offline", "--manifest-path",
         str(consumer / "Cargo.toml")], output / "rust-lock.log", env=cargo_env)
    rust_output = run(["cargo", "+1.99.0", "run", "--locked", "--offline",
                       "--manifest-path", str(consumer / "Cargo.toml")],
                      output / "rust-run.log", env=cargo_env)
    if "five original Vector cast assertions passed" not in rust_output:
        raise RuntimeError("generated Rust assertion result missing")
    require_unchanged(initial_source, initial_runtime, recipe_identity,
                      runtime, args.compiler_recipe, compiler, scheme)
    receipt = {
        **initial_source,
        "source_clean": True,
        "platform": os.uname().sysname + " " + os.uname().machine,
        "upstream_commit": manifest["upstream_commit"],
        "original_files": 21,
        "manifest_sha256": sha256(manifest_path),
        "compiler_sha256": sha256(compiler),
        "scheme_sha256": sha256(scheme),
        "runtime": initial_runtime,
        "compiler_recipe": recipe_identity,
        "cargo_target": str(cargo_target),
        "generated": generated,
        "original_refusals_on_both_targets": sorted(refusals),
        "rust_consumer_sha256": sha256(consumer / "src/main.rs"),
        "rust_lock_sha256": sha256(consumer / "Cargo.lock"),
        "rust_result": rust_output.strip(),
        "limitations": ["local Darwin arm64; native Linux qualification required",
                        "pure circuit values only; no proof or funded transaction applies",
                        "The upstream to_bytes case depends on separately owned Vector slice lowering and is excluded from this independent base-head slice"],
    }
    (output / "receipt.json").write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n")
    print(json.dumps({"status": "passed", "head": receipt["head"], "output": str(output)}))


if __name__ == "__main__":
    main()
