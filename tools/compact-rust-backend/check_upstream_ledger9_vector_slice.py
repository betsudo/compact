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

"""Run the intact upstream Vector slice compile, runtime and refusal family."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tomllib


ROOT = Path(__file__).resolve().parents[2]
FAMILY = ROOT / "test-contracts/primitives/vector/slice"
POSITIVES = (
    "computed_index", "default", "empty", "from_parameter",
    "mixed_types", "nested", "radix_bounds", "semantics",
)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


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
    parser.add_argument("--ts-runtime", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    compiler = args.compiler.resolve(strict=True)
    runtime = args.ts_runtime.resolve(strict=True)
    output = args.output.resolve()
    if args.output.is_symlink() or output.exists():
        parser.error("output must be absent")
    if subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT):
        parser.error("clean committed source required for qualification")
    source_package = json.loads((ROOT / "runtime/package.json").read_text())
    runtime_package = json.loads((runtime / "package.json").read_text())
    if any(source_package[key] != runtime_package[key]
           for key in ("name", "version", "dependencies")):
        parser.error("TypeScript runtime package does not match this source tree")
    manifest_path = FAMILY / "original-source-sha256.json"
    manifest = json.loads(manifest_path.read_text())
    if manifest["upstream_commit"] != "672bd0f226f47a7d036c5c074ce71fbe1af1925e" \
            or len(manifest["files"]) != 30:
        parser.error("pinned original fixture inventory changed")
    for source, digest in manifest["files"].items():
        if sha256(ROOT / source) != digest:
            parser.error(f"original upstream fixture changed: {source}")

    output.mkdir(parents=True)
    link = output / "node_modules/@midnight-ntwrk/compact-runtime"
    link.parent.mkdir(parents=True)
    link.symlink_to(runtime, target_is_directory=True)
    env = os.environ.copy()
    scheme = compiler.with_name("compactc-scheme")
    if not scheme.is_file():
        parser.error("paired Scheme compiler is missing")
    env["COMPACTC_SCHEME"] = str(scheme)
    generated = {}
    for name in POSITIVES:
        source = FAMILY / name / f"vector_slice_{name}.compact"
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
        "index_out_of_bounds": "slice index 100 plus length 4 is out-of-bounds for a vector of length 8",
        "non_constant_index": "slice index did not reduce to a constant nonnegative value at compile time",
        "tuple_incompatible_elements": "tuple slice with a non-constant index should be a vector but has a tuple type",
    }
    for name, diagnostic in refusals.items():
        refusal = FAMILY / name / f"vector_slice_{name}.compact"
        for mode in ("typescript", "rust"):
            destination = output / "refused" / mode / name
            command = [str(compiler), "--target", "ts" if mode == "typescript" else "rust",
                       "--skip-zk", "--compact-path", str(ROOT / "compiler"),
                       str(refusal), str(destination)]
            result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True)
            message = result.stdout + result.stderr
            (output / f"{name}-{mode}-refused.log").write_text(message)
            if result.returncode == 0 or diagnostic not in message:
                raise RuntimeError(f"original slice refusal changed for {name} {mode}: {message}")
            if destination.exists() or list(destination.parent.glob(f".{destination.name}.compactc-stage-*")):
                raise RuntimeError(f"slice refusal published output for {name} {mode}")

    # The circuit evaluator accepts more constant expressions than the Rust
    # value backend can currently represent. Keep those limits as source
    # diagnostics; an internal compiler error is never an admissible refusal.
    limited_indices = {
        "field_to_uint": (
            "(1 as Field) as Uint<8>",
            "Rust backend does not yet support Field-to-Uint downcasts"),
        "constant_call": (
            "one()",
            "Rust backend cannot preserve evaluation of this constant Vector index"),
        "tuple_projection": (
            "pair[1]",
            "Rust backend cannot preserve evaluation of this constant Vector index"),
    }
    for name, (index, diagnostic) in limited_indices.items():
        prelude = "pure circuit one(): Uint<8> { return 1; } " if name == "constant_call" else ""
        binding = "const pair = [0 as Uint<8>, 1 as Uint<8>]; " if name == "tuple_projection" else ""
        source = ("import CompactStandardLibrary; " + prelude
                  + "export circuit f(v: Vector<3, Field>): Vector<1, Field> { "
                  + binding + f"return slice<1>(v, {index}); }}\n")
        path = output / f"limited-{name}.compact"
        path.write_text(source)
        run([str(compiler), "--target", "ts", "--skip-zk", "--compact-path",
             str(ROOT / "compiler"), str(path), str(output / "limited-ts" / name)],
            output / f"limited-{name}-ts.log", env=env)
        destination = output / "limited-rust" / name
        result = subprocess.run([str(compiler), "--target", "rust", "--skip-zk",
                                 "--compact-path", str(ROOT / "compiler"), str(path),
                                 str(destination)], cwd=ROOT, env=env, capture_output=True, text=True)
        message = result.stdout + result.stderr
        (output / f"limited-{name}-rust.log").write_text(message)
        if result.returncode == 0 or diagnostic not in message or destination.exists():
            raise RuntimeError(f"limited constant index needs a source diagnostic: {name}: {message}")

    consumer = output / "rust-consumer"
    (consumer / "src").mkdir(parents=True)
    shutil.copyfile(FAMILY / "consumer.rs", consumer / "src/main.rs")
    lines = ["[package]", 'name = "ledger9-vector-slice-consumer"',
             'version = "0.1.0"', 'edition = "2024"', "[workspace]", "[dependencies]"]
    for name in POSITIVES:
        contract = output / "generated/both" / name / "contract"
        package = tomllib.loads((contract / "Cargo.toml").read_text())["package"]["name"]
        lines.append(f'{name} = {{ package = {json.dumps(package)}, '
                     f'path = {json.dumps(str(contract))} }}')
    (consumer / "Cargo.toml").write_text("\n".join(lines) + "\n")
    cargo_env = dict(env, CARGO_TARGET_DIR=str(output / "rust-target"))
    run(["cargo", "+1.99.0", "generate-lockfile", "--offline", "--manifest-path",
         str(consumer / "Cargo.toml")], output / "rust-lock.log", env=cargo_env)
    rust_output = run(["cargo", "+1.99.0", "run", "--locked", "--offline",
                       "--manifest-path", str(consumer / "Cargo.toml")],
                      output / "rust-run.log", env=cargo_env)
    if "eight original vector slice assertions passed" not in rust_output:
        raise RuntimeError("generated Rust assertion result missing")
    receipt = {
        "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "tree": subprocess.check_output(["git", "rev-parse", "HEAD^{tree}"], cwd=ROOT, text=True).strip(),
        "source_clean": not subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT),
        "platform": os.uname().sysname + " " + os.uname().machine,
        "upstream_commit": manifest["upstream_commit"],
        "original_files": 30,
        "manifest_sha256": sha256(manifest_path),
        "compiler_sha256": sha256(compiler),
        "scheme_sha256": sha256(scheme),
        "runtime_package_sha256": sha256(runtime / "package.json"),
        "generated": generated,
        "original_refusals_on_both_targets": sorted(refusals),
        "additional_constant_index_diagnostics": sorted(limited_indices),
        "rust_consumer_sha256": sha256(consumer / "src/main.rs"),
        "rust_lock_sha256": sha256(consumer / "Cargo.lock"),
        "rust_result": rust_output.strip(),
        "limitations": ["local Darwin arm64; native Linux qualification required",
                        "pure circuit values only; no proof or funded transaction applies",
                        "Nonconstant slice indices retain upstream compile refusal",
                        "Field-to-Uint casts, constant-returning calls and tuple projections as slice indices have explicit Rust source diagnostics"],
    }
    (output / "receipt.json").write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n")
    print(json.dumps({"status": "passed", "head": receipt["head"], "output": str(output)}))


if __name__ == "__main__":
    main()
